import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_canvas_endpoint.dart';
import 'package:bitflip_server_server/src/colour/colour_canvas_reducer.dart';
import 'package:bitflip_server_server/src/colour/colour_canvas_store.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:bs58/bs58.dart';
import 'package:serverpod/serverpod.dart';
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

void main() {
  withServerpod('Indexed colour canvas', (sessionBuilder, endpoints) {
    setUp(() => _clearCanvas(sessionBuilder));
    tearDown(() => _clearCanvas(sessionBuilder));

    final signature = base58.encoder.convert(
      Uint8List(64)..fillRange(0, 64, 1),
    );

    test(
      'signature replays across endpoint instances only read indexed state',
      () async {
        await _apply(sessionBuilder, [
          _event(revision: 12, colour: 4, pixels: [0, 4095]),
          _event(revision: 11, colour: 2, pixels: [0, 1]),
        ]);
        final before = await _rowVersion(sessionBuilder);

        // No RPC source is configured. Independent endpoint instances model
        // replicas: neither the first submission nor concurrent replays may
        // depend on a process-local cache to avoid chain reads and writes.
        final first = await endpoints.colourCanvas.recordSignature(
          sessionBuilder,
          transactionSignature: signature,
          gameIndex: _gameIndex,
          sectionIndex: _sectionIndex,
        );
        final replicas = [ColourCanvasEndpoint(), ColourCanvasEndpoint()];
        final replays = await Future.wait([
          for (final replica in replicas)
            _run(
              sessionBuilder,
              (session) => replica.recordSignature(
                session,
                transactionSignature: signature,
                gameIndex: _gameIndex,
                sectionIndex: _sectionIndex,
              ),
            ),
        ]);

        for (final view in [first, ...replays]) {
          expect(view.policyVersion, 1);
          expect(view.highestRevision, 12);
          expect(_bytes(view.colours!)[0], 4);
          expect(_bytes(view.colours!)[1], 2);
          expect(_bytes(view.colours!)[4095], 4);
        }
        expect(await _rowVersion(sessionBuilder), before);
      },
    );

    test('does not fetch or create state for an unindexed signature', () async {
      final view = await endpoints.colourCanvas.recordSignature(
        sessionBuilder,
        transactionSignature: signature,
        gameIndex: _gameIndex,
        sectionIndex: _sectionIndex,
      );
      expect(view.policyVersion, 0);
      expect(view.highestRevision, 0);
      expect(_bytes(view.colours!), everyElement(noPixelColour));
      expect(await _rowVersion(sessionBuilder), isEmpty);
    });

    test('rejects malformed signatures and invalid section indices', () async {
      await expectLater(
        endpoints.colourCanvas.recordSignature(
          sessionBuilder,
          transactionSignature: 'invalid',
          gameIndex: _gameIndex,
          sectionIndex: _sectionIndex,
        ),
        throwsFormatException,
      );
      await expectLater(
        endpoints.colourCanvas.recordSignature(
          sessionBuilder,
          transactionSignature: signature,
          gameIndex: _gameIndex,
          sectionIndex: -1,
        ),
        throwsRangeError,
      );
      expect(await _rowVersion(sessionBuilder), isEmpty);
    });

    test(
      'replayed and stale indexer events do not rewrite unchanged rows',
      () async {
        final event = _event(revision: 12, colour: 4, pixels: [0, 4095]);
        await _apply(sessionBuilder, [event]);
        final before = await _rowVersion(sessionBuilder);
        await _apply(sessionBuilder, [
          event,
          _event(revision: 10, colour: 6, pixels: [0]),
          _event(policyVersion: 0, revision: 99, colour: 7, pixels: [0]),
        ]);
        expect(await _rowVersion(sessionBuilder), before);

        // An older event can still fill an unseen pixel. A newer event with the
        // same colour must persist its revision to reject later stale colours.
        await _apply(sessionBuilder, [
          event,
          _event(revision: 10, colour: 2, pixels: [1]),
          _event(revision: 13, colour: 4, pixels: [0]),
        ]);
        expect(await _rowVersion(sessionBuilder), isNot(before));
        await _apply(sessionBuilder, [
          _event(revision: 12, colour: 7, pixels: [0]),
        ]);
        final view = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: _gameIndex,
          sectionIndex: _sectionIndex,
        );
        expect(view.highestRevision, 13);
        expect(_bytes(view.colours!)[0], 4);
        expect(_bytes(view.colours!)[1], 2);
      },
    );

    test(
      'confirms an unchanged canvas without resending the colours',
      () async {
        final recorded = await _apply(sessionBuilder, [
          _event(revision: 5, colour: 3, pixels: [9]),
        ]);
        final unchanged = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: _gameIndex,
          sectionIndex: _sectionIndex,
          knownPolicyVersion: recorded.policyVersion,
          knownRevision: recorded.highestRevision,
        );
        expect(unchanged.colours, isNull);
        expect(unchanged.highestRevision, 5);
        expect(unchanged.policyVersion, 1);

        final staleRevision = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: _gameIndex,
          sectionIndex: _sectionIndex,
          knownPolicyVersion: 1,
          knownRevision: 4,
        );
        expect(_bytes(staleRevision.colours!)[9], 3);

        final otherPolicy = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: _gameIndex,
          sectionIndex: _sectionIndex,
          knownPolicyVersion: 0,
          knownRevision: 5,
        );
        expect(otherPolicy.colours, isNotNull);

        final knownEmpty = await endpoints.colourCanvas.load(
          sessionBuilder,
          gameIndex: 3,
          sectionIndex: 1,
          knownPolicyVersion: 0,
          knownRevision: 0,
        );
        expect(knownEmpty.colours, isNull);
      },
    );
  }, rollbackDatabase: RollbackDatabase.disabled);
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

// Use a dedicated section because concurrent replica calls need independent,
// committed sessions instead of the test harness's shared rollback transaction.
const _gameIndex = 3;
const _sectionIndex = 250;

Future<void> _clearCanvas(TestSessionBuilder builder) async {
  await _run(
    builder,
    (session) => ColourCanvasState.db.deleteWhere(
      session,
      where: (table) =>
          table.gameIndex.equals(_gameIndex) &
          table.sectionIndex.equals(_sectionIndex),
    ),
  );
}

Future<ColourCanvasState> _apply(
  TestSessionBuilder builder,
  List<ColourPixelsFlipped> events,
) => _run(
  builder,
  (session) => ColourCanvasStore.applySectionEvents(
    session,
    gameIndex: _gameIndex,
    sectionIndex: _sectionIndex,
    events: events,
  ),
);

// PostgreSQL gives every UPDATE a new tuple location, even when the values
// and timestamps are identical. This catches unconditional no-op rewrites.
Future<List<List<Object?>>> _rowVersion(TestSessionBuilder builder) => _run(
  builder,
  (session) => session.db.unsafeQuery(
    'SELECT ctid::text FROM bitflip_colour_canvas_state '
    'WHERE "gameIndex" = $_gameIndex AND "sectionIndex" = $_sectionIndex;',
  ),
);

ColourPixelsFlipped _event({
  int policyVersion = 1,
  required int revision,
  required int colour,
  required List<int> pixels,
}) => ColourPixelsFlipped(
  player: 'player',
  policyVersion: policyVersion,
  revision: revision,
  gameIndex: _gameIndex,
  sectionIndex: _sectionIndex,
  colour: colour,
  coordinates: pixels
      .map(
        (pixel) => ColourPixelCoordinate(
          pixel % colourCanvasSide,
          pixel ~/ colourCanvasSide,
        ),
      )
      .toList(),
);

Uint8List _bytes(ByteData data) =>
    data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes);
