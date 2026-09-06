import 'dart:convert';
import 'dart:math';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/colour/colour_canvas_store.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event_source.dart';
import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:serverpod/serverpod.dart';

final class ColourIndexerConfiguration {
  const ColourIndexerConfiguration({
    required this.enabled,
    required this.cluster,
    required this.startSignature,
    this.interval = const Duration(seconds: 2),
    this.pageSize = 128,
    this.maximumConcurrentTransactions = 16,
  });

  factory ColourIndexerConfiguration.fromEnvironment(
    Map<String, String> environment, {
    required bool requireEnabled,
  }) {
    final enabledValue = environment['BITFLIP_COLOUR_INDEXER_ENABLED']?.trim();
    final enabled = switch (enabledValue) {
      null || '' => false,
      'true' => true,
      'false' => false,
      _ => throw StateError(
        'BITFLIP_COLOUR_INDEXER_ENABLED must be true or false.',
      ),
    };
    if (requireEnabled && !enabled) {
      throw StateError(
        'BITFLIP_COLOUR_INDEXER_ENABLED must be true for release deployments.',
      );
    }
    final cluster = environment['BITFLIP_CLUSTER']?.trim();
    final startSignature = environment['BITFLIP_COLOUR_INDEXER_START_SIGNATURE']
        ?.trim();
    if (enabled && (cluster == null || cluster.isEmpty)) {
      throw StateError(
        'BITFLIP_CLUSTER is required when the colour indexer is enabled.',
      );
    }
    if (enabled && (startSignature == null || startSignature.isEmpty)) {
      throw StateError(
        'BITFLIP_COLOUR_INDEXER_START_SIGNATURE is required when the colour '
        'indexer is enabled.',
      );
    }
    if (startSignature != null && startSignature.isNotEmpty) {
      validateTransactionSignature(startSignature);
    }
    return ColourIndexerConfiguration(
      enabled: enabled,
      cluster: cluster ?? 'development',
      startSignature: startSignature,
      interval: Duration(
        seconds: _environmentInteger(
          environment,
          'BITFLIP_COLOUR_INDEXER_INTERVAL_SECONDS',
          defaultValue: 2,
          minimum: 2,
          maximum: 60,
        ),
      ),
      pageSize: _environmentInteger(
        environment,
        'BITFLIP_COLOUR_INDEXER_PAGE_SIZE',
        defaultValue: 128,
        minimum: 1,
        maximum: 256,
      ),
      maximumConcurrentTransactions: _environmentInteger(
        environment,
        'BITFLIP_COLOUR_INDEXER_CONCURRENCY',
        defaultValue: 16,
        minimum: 1,
        maximum: 32,
      ),
    );
  }

  final bool enabled;
  final String cluster;
  final String? startSignature;
  final Duration interval;
  final int pageSize;
  final int maximumConcurrentTransactions;

  String get requiredStartSignature =>
      startSignature ??
      (throw StateError('The colour indexer start signature is unavailable.'));
}

final class ColourEventIndexer {
  ColourEventIndexer({
    required this.configuration,
    required this.signatureSource,
    required this.eventSource,
    DateTime Function()? clock,
    String Function()? leaseToken,
    this.transactionDeadline = const Duration(seconds: 10),
    Duration? leaseDuration,
  }) : _clock = clock ?? DateTime.now,
       _leaseToken = leaseToken ?? _secureLeaseToken,
       leaseDuration =
           leaseDuration ??
           transactionDeadline *
               (((configuration.pageSize - 1) ~/
                       configuration.maximumConcurrentTransactions) +
                   3);

  final ColourIndexerConfiguration configuration;
  final ColourProgramSignatureSource signatureSource;
  final ColourFlipEventSource eventSource;
  final Duration transactionDeadline;
  final Duration leaseDuration;
  final DateTime Function() _clock;
  final String Function() _leaseToken;

  Future<ColourIndexerBatchResult> runOnce(Session session) async {
    if (!configuration.enabled) {
      return const ColourIndexerBatchResult.disabled();
    }
    final claim = await _claim(session);
    if (claim == null) return const ColourIndexerBatchResult.leased();

    try {
      final page = await signatureSource
          .signatures(
            limit: configuration.pageSize,
            before: claim.cursor.beforeSignature,
            until: claim.cursor.completedHeadSignature,
          )
          .timeout(transactionDeadline);
      final events = await _loadEvents(page.entries);
      await ColourCanvasStore.applyEvents(session, events);

      final hasAnotherPage = page.entries.length == configuration.pageSize;
      final batchHead =
          claim.cursor.catchUpHeadSignature ?? page.newestSignature;
      final completesSweep = !hasAnotherPage;
      await _complete(
        session,
        claim,
        completedHeadSignature: completesSweep && batchHead != null
            ? batchHead
            : claim.cursor.completedHeadSignature,
        catchUpHeadSignature: completesSweep ? null : batchHead,
        beforeSignature: completesSweep ? null : page.oldestSignature,
      );
      return ColourIndexerBatchResult.completed(
        signaturesScanned: page.entries.length,
        successfulTransactions: page.entries
            .where((entry) => entry.succeeded)
            .length,
        eventsApplied: events.length,
        sweepCompleted: completesSweep,
      );
    } on Object catch (error, stackTrace) {
      await _releaseAfterFailure(session, claim);
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  Future<List<ColourPixelsFlipped>> _loadEvents(
    List<ColourProgramSignature> entries,
  ) async {
    final successful = entries.where((entry) => entry.succeeded).toList();
    final events = <ColourPixelsFlipped>[];
    for (
      var offset = 0;
      offset < successful.length;
      offset += configuration.maximumConcurrentTransactions
    ) {
      final end = min(
        successful.length,
        offset + configuration.maximumConcurrentTransactions,
      );
      final batches = await Future.wait(
        successful
            .sublist(offset, end)
            .map(
              (entry) => eventSource
                  .eventsForSignature(entry.signature)
                  .timeout(transactionDeadline),
            ),
      );
      for (final batch in batches) {
        events.addAll(batch);
      }
    }
    return events;
  }

  Future<_ColourIndexerClaim?> _claim(Session session) async {
    final now = _clock().toUtc();
    final token = _leaseToken();
    _ColourIndexerClaim? claim;
    await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
      transaction,
    ) async {
      await session.db.unsafeQuery(
        'SELECT pg_advisory_xact_lock(1112100428, 1129270866);',
        transaction: transaction,
      );
      var cursor = await ColourIndexerCursor.db.findFirstRow(
        session,
        where: (table) =>
            table.cluster.equals(configuration.cluster) &
            table.programAddress.equals(bitflipProgramProgramAddress.value),
        transaction: transaction,
        lockMode: LockMode.forUpdate,
      );
      cursor ??= await ColourIndexerCursor.db.insertRow(
        session,
        ColourIndexerCursor(
          cluster: configuration.cluster,
          programAddress: bitflipProgramProgramAddress.value,
          startSignature: configuration.requiredStartSignature,
          completedHeadSignature: configuration.requiredStartSignature,
          updatedAt: now,
        ),
        transaction: transaction,
      );
      if (cursor.startSignature != configuration.requiredStartSignature) {
        throw StateError(
          'The configured colour indexer start signature does not match the '
          'durable cursor.',
        );
      }
      if (cursor.leasedUntil case final leasedUntil?
          when leasedUntil.isAfter(now)) {
        return;
      }
      final leased = cursor.copyWith(
        leaseToken: token,
        leasedUntil: now.add(leaseDuration),
        updatedAt: now,
      );
      final stored = await ColourIndexerCursor.db.updateRow(
        session,
        leased,
        transaction: transaction,
      );
      claim = _ColourIndexerClaim(stored, token);
    });
    return claim;
  }

  Future<void> _complete(
    Session session,
    _ColourIndexerClaim claim, {
    required String completedHeadSignature,
    required String? catchUpHeadSignature,
    required String? beforeSignature,
  }) async {
    final now = _clock().toUtc();
    await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
      transaction,
    ) async {
      final cursor = await ColourIndexerCursor.db.findById(
        session,
        claim.cursor.id!,
        transaction: transaction,
        lockMode: LockMode.forUpdate,
      );
      if (cursor?.leaseToken != claim.token) {
        throw StateError('The colour indexer lease expired during a batch.');
      }
      await ColourIndexerCursor.db.updateRow(
        session,
        cursor!.copyWith(
          completedHeadSignature: completedHeadSignature,
          catchUpHeadSignature: catchUpHeadSignature,
          beforeSignature: beforeSignature,
          leaseToken: null,
          leasedUntil: null,
          lastSuccessAt: now,
          updatedAt: now,
        ),
        transaction: transaction,
      );
    });
  }

  Future<void> _releaseAfterFailure(
    Session session,
    _ColourIndexerClaim claim,
  ) async {
    try {
      await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
        transaction,
      ) async {
        final cursor = await ColourIndexerCursor.db.findById(
          session,
          claim.cursor.id!,
          transaction: transaction,
          lockMode: LockMode.forUpdate,
        );
        if (cursor?.leaseToken != claim.token) return;
        await ColourIndexerCursor.db.updateRow(
          session,
          cursor!.copyWith(
            leaseToken: null,
            leasedUntil: null,
            updatedAt: _clock().toUtc(),
          ),
          transaction: transaction,
        );
      });
    } on Object {
      // The durable lease expires on its own if recovery cannot reach the DB.
    }
  }
}

final class ColourIndexerBatchResult {
  const ColourIndexerBatchResult._({
    required this.outcome,
    required this.signaturesScanned,
    required this.successfulTransactions,
    required this.eventsApplied,
    required this.sweepCompleted,
  });

  const ColourIndexerBatchResult.disabled()
    : this._(
        outcome: 'disabled',
        signaturesScanned: 0,
        successfulTransactions: 0,
        eventsApplied: 0,
        sweepCompleted: false,
      );

  const ColourIndexerBatchResult.leased()
    : this._(
        outcome: 'leased',
        signaturesScanned: 0,
        successfulTransactions: 0,
        eventsApplied: 0,
        sweepCompleted: false,
      );

  const ColourIndexerBatchResult.completed({
    required int signaturesScanned,
    required int successfulTransactions,
    required int eventsApplied,
    required bool sweepCompleted,
  }) : this._(
         outcome: 'completed',
         signaturesScanned: signaturesScanned,
         successfulTransactions: successfulTransactions,
         eventsApplied: eventsApplied,
         sweepCompleted: sweepCompleted,
       );

  final String outcome;
  final int signaturesScanned;
  final int successfulTransactions;
  final int eventsApplied;
  final bool sweepCompleted;
}

final class _ColourIndexerClaim {
  const _ColourIndexerClaim(this.cursor, this.token);

  final ColourIndexerCursor cursor;
  final String token;
}

abstract final class ColourEventIndexerRegistry {
  static ColourEventIndexer? _indexer;

  static ColourEventIndexer get indexer =>
      _indexer ??
      (throw StateError('The Bitflip colour event indexer is unavailable.'));

  static void configure(ColourEventIndexer indexer) {
    _indexer = indexer;
  }
}

String _secureLeaseToken() {
  final random = Random.secure();
  return base64Url.encode(List.generate(24, (_) => random.nextInt(256)));
}

int _environmentInteger(
  Map<String, String> environment,
  String name, {
  required int defaultValue,
  required int minimum,
  required int maximum,
}) {
  final value = environment[name]?.trim();
  final parsed = value == null || value.isEmpty
      ? defaultValue
      : int.tryParse(value);
  if (parsed == null || parsed < minimum || parsed > maximum) {
    throw StateError('$name must be between $minimum and $maximum.');
  }
  return parsed;
}
