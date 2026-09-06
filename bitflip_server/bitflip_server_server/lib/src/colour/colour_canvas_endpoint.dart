import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_canvas_reducer.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event_source.dart';
import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:serverpod/serverpod.dart';

final class ColourCanvasEndpoint extends Endpoint {
  static const _chainReadDeadline = Duration(seconds: 10);
  static final _submissionLimiter = ColourSubmissionRateLimiter();

  @override
  bool get requireLogin => false;

  Future<ColourCanvasView> load(
    Session session, {
    required int gameIndex,
    required int sectionIndex,
  }) async {
    _validateIndices(gameIndex, sectionIndex);
    final stored = await ColourCanvasState.db.findFirstRow(
      session,
      where: (table) =>
          table.gameIndex.equals(gameIndex) &
          table.sectionIndex.equals(sectionIndex),
    );
    return _viewFor(gameIndex, sectionIndex, stored);
  }

  Future<ColourCanvasView> recordSignature(
    Session session, {
    required String transactionSignature,
    required int gameIndex,
    required int sectionIndex,
  }) async {
    _validateIndices(gameIndex, sectionIndex);
    _submissionLimiter.record(
      session.request?.connectionInfo.remote.address.toString() ?? 'internal',
    );
    final events = await ColourFlipEventSourceRegistry.source
        .eventsForSignature(transactionSignature)
        .timeout(_chainReadDeadline);
    final relevantEvents =
        events
            .where(
              (event) =>
                  event.gameIndex == gameIndex &&
                  event.sectionIndex == sectionIndex,
            )
            .toList()
          ..sort((left, right) => left.revision.compareTo(right.revision));
    if (relevantEvents.isEmpty) {
      throw StateError(
        'The transaction has no verified colour event for this section.',
      );
    }

    late ColourCanvasState result;
    await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
      transaction,
    ) async {
      await session.db.unsafeQuery(
        'SELECT pg_advisory_xact_lock(@game, @section);',
        transaction: transaction,
        parameters: QueryParameters.named({
          'game': gameIndex,
          'section': sectionIndex,
        }),
      );
      final stored = await ColourCanvasState.db.findFirstRow(
        session,
        where: (table) =>
            table.gameIndex.equals(gameIndex) &
            table.sectionIndex.equals(sectionIndex),
        transaction: transaction,
        lockMode: LockMode.forUpdate,
      );
      final canvas = stored == null
          ? ColourCanvasBuffer.empty()
          : ColourCanvasBuffer.fromBytes(
              policyVersion: stored.policyVersion,
              highestRevision: stored.highestRevision,
              colours: _bytes(stored.colours),
              pixelRevisions: _bytes(stored.pixelRevisions),
            );
      for (final event in relevantEvents) {
        canvas.apply(event);
      }
      final next = ColourCanvasState(
        id: stored?.id,
        gameIndex: gameIndex,
        sectionIndex: sectionIndex,
        policyVersion: canvas.policyVersion,
        highestRevision: canvas.highestRevision,
        colours: ByteData.sublistView(canvas.colours),
        pixelRevisions: ByteData.sublistView(canvas.pixelRevisions),
        updatedAt: DateTime.now().toUtc(),
      );
      result = stored == null
          ? await ColourCanvasState.db.insertRow(
              session,
              next,
              transaction: transaction,
            )
          : await ColourCanvasState.db.updateRow(
              session,
              next,
              transaction: transaction,
            );
    });
    return _viewFor(gameIndex, sectionIndex, result);
  }
}

ColourCanvasView _viewFor(
  int gameIndex,
  int sectionIndex,
  ColourCanvasState? state,
) {
  final canvas = state == null
      ? ColourCanvasBuffer.empty()
      : ColourCanvasBuffer.fromBytes(
          policyVersion: state.policyVersion,
          highestRevision: state.highestRevision,
          colours: _bytes(state.colours),
          pixelRevisions: _bytes(state.pixelRevisions),
        );
  return ColourCanvasView(
    gameIndex: gameIndex,
    sectionIndex: sectionIndex,
    policyVersion: canvas.policyVersion,
    highestRevision: canvas.highestRevision,
    colours: ByteData.sublistView(canvas.colours),
  );
}

Uint8List _bytes(ByteData data) => data.buffer.asUint8List(
  data.offsetInBytes,
  data.lengthInBytes,
);

void _validateIndices(int gameIndex, int sectionIndex) {
  if (gameIndex < 0 || gameIndex >= 4) {
    throw RangeError.range(gameIndex, 0, 3, 'gameIndex');
  }
  if (sectionIndex < 0 || sectionIndex > 255) {
    throw RangeError.range(sectionIndex, 0, 255, 'sectionIndex');
  }
}

/// Process-local protection for signature submissions. Production deployments
/// must also apply a distributed edge limit.
final class ColourSubmissionRateLimiter {
  ColourSubmissionRateLimiter({
    this.window = const Duration(minutes: 1),
    this.maximumPerSource = 60,
    this.maximumGlobal = 480,
    DateTime Function()? clock,
  }) : _clock = clock ?? DateTime.now;

  final Duration window;
  final int maximumPerSource;
  final int maximumGlobal;
  final DateTime Function() _clock;
  final Map<String, List<DateTime>> _attemptsBySource = {};
  final List<DateTime> _globalAttempts = [];

  void record(String source) {
    final now = _clock().toUtc();
    final cutoff = now.subtract(window);
    _globalAttempts.removeWhere((attempt) => attempt.isBefore(cutoff));
    _attemptsBySource.removeWhere((_, attempts) {
      attempts.removeWhere((attempt) => attempt.isBefore(cutoff));
      return attempts.isEmpty;
    });
    final attempts = _attemptsBySource.putIfAbsent(source, () => []);
    if (attempts.length >= maximumPerSource ||
        _globalAttempts.length >= maximumGlobal) {
      throw StateError('Too many colour submissions. Wait before retrying.');
    }
    attempts.add(now);
    _globalAttempts.add(now);
  }
}
