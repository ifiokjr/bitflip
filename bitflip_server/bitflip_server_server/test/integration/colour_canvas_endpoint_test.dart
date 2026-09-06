import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_canvas_reducer.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event_source.dart';
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

void main() {
  withServerpod('Verified colour canvas', (sessionBuilder, endpoints) {
    late _FakeColourEventSource source;

    setUp(() {
      source = _FakeColourEventSource();
      ColourFlipEventSourceRegistry.configure(source);
    });

    test('persists only matching verified events and handles replay', () async {
      source.events = [
        _event(revision: 12, colour: 4, pixels: [0, 4095]),
        _event(revision: 11, colour: 2, pixels: [0, 1]),
        _event(revision: 99, colour: 7, pixels: [2], sectionIndex: 8),
      ];
      final first = await endpoints.colourCanvas.recordSignature(
        sessionBuilder,
        transactionSignature: 'verified-by-the-fake-source',
        gameIndex: 0,
        sectionIndex: 7,
      );
      final firstColours = _bytes(first.colours);
      expect(first.policyVersion, 1);
      expect(first.highestRevision, 12);
      expect(firstColours[0], 4);
      expect(firstColours[1], 2);
      expect(firstColours[4095], 4);

      source.events = [
        _event(revision: 10, colour: 6, pixels: [0]),
      ];
      final replay = await endpoints.colourCanvas.recordSignature(
        sessionBuilder,
        transactionSignature: 'replayed',
        gameIndex: 0,
        sectionIndex: 7,
      );
      expect(_bytes(replay.colours)[0], 4);
      expect(replay.highestRevision, 12);
    });

    test('returns an empty canvas and rejects unrelated signatures', () async {
      final empty = await endpoints.colourCanvas.load(
        sessionBuilder,
        gameIndex: 2,
        sectionIndex: 90,
      );
      expect(empty.policyVersion, 0);
      expect(empty.highestRevision, 0);
      expect(_bytes(empty.colours), everyElement(noPixelColour));

      source.events = [
        _event(revision: 1, colour: 0, pixels: [1]),
      ];
      await expectLater(
        endpoints.colourCanvas.recordSignature(
          sessionBuilder,
          transactionSignature: 'wrong-section',
          gameIndex: 0,
          sectionIndex: 8,
        ),
        throwsA(isA<StateError>()),
      );
    });
  });
}

final class _FakeColourEventSource implements ColourFlipEventSource {
  List<ColourPixelsFlipped> events = const [];

  @override
  Future<List<ColourPixelsFlipped>> eventsForSignature(String signature) async {
    return events;
  }
}

ColourPixelsFlipped _event({
  required int revision,
  required int colour,
  required List<int> pixels,
  int sectionIndex = 7,
}) {
  return ColourPixelsFlipped(
    player: 'player',
    policyVersion: 1,
    revision: revision,
    gameIndex: 0,
    sectionIndex: sectionIndex,
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
}

Uint8List _bytes(ByteData data) => data.buffer.asUint8List(
  data.offsetInBytes,
  data.lengthInBytes,
);
