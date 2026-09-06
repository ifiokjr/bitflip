import 'dart:async';
import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/colour/colour_event_indexer.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event_source.dart';
import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:serverpod/serverpod.dart';
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

void main() {
  withServerpod('Durable colour event indexer', (sessionBuilder, endpoints) {
    test(
      'paginates to the durable head and ignores failed transactions',
      () async {
        final source = _FakeIndexerSource()
          ..pages.addAll([
            ColourProgramSignaturePage([
              const ColourProgramSignature(signature: 'new', succeeded: true),
              const ColourProgramSignature(
                signature: 'failed',
                succeeded: false,
              ),
            ]),
            ColourProgramSignaturePage([
              const ColourProgramSignature(signature: 'old', succeeded: true),
            ]),
          ])
          ..events['new'] = [_event(revision: 3, colour: 5, pixel: 9)]
          ..events['old'] = [_event(revision: 2, colour: 1, pixel: 9)];
        final indexer = _indexer(source);

        final first = await _run(sessionBuilder, indexer.runOnce);
        expect(first.signaturesScanned, 2);
        expect(first.successfulTransactions, 1);
        expect(first.sweepCompleted, isFalse);
        expect(source.eventRequests, ['new']);

        final second = await _run(sessionBuilder, indexer.runOnce);
        expect(second.signaturesScanned, 1);
        expect(second.sweepCompleted, isTrue);
        expect(source.eventRequests, ['new', 'old']);
        expect(source.requests, [
          const _SignatureRequest(before: null, until: 'start'),
          const _SignatureRequest(before: 'failed', until: 'start'),
        ]);

        final canvas = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: 0,
          sectionIndex: 7,
        );
        expect(_bytes(canvas.colours)[9], 5);
        expect(canvas.highestRevision, 3);

        final cursor = await _cursor(sessionBuilder);
        expect(cursor.completedHeadSignature, 'new');
        expect(cursor.catchUpHeadSignature, isNull);
        expect(cursor.beforeSignature, isNull);
        expect(cursor.leaseToken, isNull);
        expect(cursor.leasedUntil, isNull);
        expect(cursor.lastSuccessAt, isNotNull);
      },
    );

    test(
      'releases its lease after failure and safely retries the page',
      () async {
        final source = _FakeIndexerSource()
          ..pages.addAll([
            ColourProgramSignaturePage([
              const ColourProgramSignature(signature: 'retry', succeeded: true),
            ]),
            ColourProgramSignaturePage([
              const ColourProgramSignature(signature: 'retry', succeeded: true),
            ]),
          ])
          ..failOnceFor = 'retry'
          ..events['retry'] = [_event(revision: 4, colour: 2, pixel: 100)];
        final indexer = _indexer(source);

        await expectLater(
          _run(sessionBuilder, indexer.runOnce),
          throwsA(isA<StateError>()),
        );
        var cursor = await _cursor(sessionBuilder);
        expect(cursor.completedHeadSignature, 'start');
        expect(cursor.leaseToken, isNull);

        final retry = await _run(sessionBuilder, indexer.runOnce);
        expect(retry.eventsApplied, 1);
        expect(retry.sweepCompleted, isTrue);
        cursor = await _cursor(sessionBuilder);
        expect(cursor.completedHeadSignature, 'retry');
        final canvas = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: 0,
          sectionIndex: 7,
        );
        expect(_bytes(canvas.colours)[100], 2);
      },
    );

    test('does no RPC work while another worker holds the lease', () async {
      final now = DateTime.utc(2026, 9, 6, 17);
      await _run(sessionBuilder, (session) async {
        await ColourIndexerCursor.db.insertRow(
          session,
          ColourIndexerCursor(
            cluster: 'devnet',
            programAddress: bitflipProgramProgramAddress.value,
            startSignature: 'start',
            completedHeadSignature: 'start',
            leaseToken: 'other-worker',
            leasedUntil: now.add(const Duration(minutes: 1)),
            updatedAt: now,
          ),
        );
      });
      final source = _FakeIndexerSource();
      final result = await _run(
        sessionBuilder,
        _indexer(source, clock: () => now).runOnce,
      );

      expect(result.outcome, 'leased');
      expect(source.requests, isEmpty);
    });
  });
}

ColourEventIndexer _indexer(
  _FakeIndexerSource source, {
  DateTime Function()? clock,
}) {
  return ColourEventIndexer(
    configuration: const ColourIndexerConfiguration(
      enabled: true,
      cluster: 'devnet',
      startSignature: 'start',
      pageSize: 2,
      maximumConcurrentTransactions: 2,
    ),
    signatureSource: source,
    eventSource: source,
    clock: clock ?? () => DateTime.utc(2026, 9, 6, 17),
    leaseToken: () => 'test-lease',
  );
}

Future<T> _run<T>(
  TestSessionBuilder builder,
  Future<T> Function(Session session) operation,
) async {
  final session = builder.build();
  try {
    return await operation(session);
  } finally {
    await session.close();
  }
}

Future<ColourIndexerCursor> _cursor(TestSessionBuilder builder) async {
  return _run(builder, (session) async {
    final cursor = await ColourIndexerCursor.db.findFirstRow(session);
    return cursor!;
  });
}

final class _FakeIndexerSource
    implements ColourProgramSignatureSource, ColourFlipEventSource {
  final pages = <ColourProgramSignaturePage>[];
  final events = <String, List<ColourPixelsFlipped>>{};
  final requests = <_SignatureRequest>[];
  final eventRequests = <String>[];
  String? failOnceFor;

  @override
  Future<ColourProgramSignaturePage> signatures({
    required int limit,
    String? before,
    String? until,
  }) async {
    requests.add(_SignatureRequest(before: before, until: until));
    return pages.removeAt(0);
  }

  @override
  Future<List<ColourPixelsFlipped>> eventsForSignature(String signature) async {
    eventRequests.add(signature);
    if (failOnceFor == signature) {
      failOnceFor = null;
      throw StateError('temporary RPC failure');
    }
    return events[signature] ?? const [];
  }
}

final class _SignatureRequest {
  const _SignatureRequest({required this.before, required this.until});

  final String? before;
  final String? until;

  @override
  bool operator ==(Object other) =>
      other is _SignatureRequest &&
      other.before == before &&
      other.until == until;

  @override
  int get hashCode => Object.hash(before, until);

  @override
  String toString() => '(before: $before, until: $until)';
}

ColourPixelsFlipped _event({
  required int revision,
  required int colour,
  required int pixel,
}) {
  return ColourPixelsFlipped(
    player: 'player',
    policyVersion: 1,
    revision: revision,
    gameIndex: 0,
    sectionIndex: 7,
    colour: colour,
    coordinates: [
      ColourPixelCoordinate(
        pixel % colourCanvasSide,
        pixel ~/ colourCanvasSide,
      ),
    ],
  );
}

Uint8List _bytes(ByteData data) => data.buffer.asUint8List(
  data.offsetInBytes,
  data.lengthInBytes,
);
