import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_canvas_reducer.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:test/test.dart';

void main() {
  group('ColourCanvasBuffer', () {
    test('uses per-pixel revisions for replay and out-of-order safety', () {
      final canvas = ColourCanvasBuffer.empty();
      expect(
        canvas.apply(_event(revision: 10, colour: 2, pixels: [0, 1])),
        isTrue,
      );
      expect(
        canvas.apply(_event(revision: 9, colour: 5, pixels: [0, 2])),
        isTrue,
      );
      expect(
        canvas.apply(_event(revision: 10, colour: 7, pixels: [0])),
        isFalse,
      );

      expect(canvas.colours[0], 2);
      expect(canvas.colours[1], 2);
      expect(canvas.colours[2], 5);
      expect(canvas.highestRevision, 10);
      expect(_revision(canvas, 0), 10);
      expect(_revision(canvas, 2), 9);
    });

    test('resets colours for a new policy and ignores stale policies', () {
      final canvas = ColourCanvasBuffer.empty();
      canvas.apply(
        _event(policyVersion: 3, revision: 20, colour: 1, pixels: [9]),
      );
      canvas.apply(
        _event(policyVersion: 4, revision: 21, colour: 6, pixels: [10]),
      );
      expect(canvas.colours[9], noPixelColour);
      expect(canvas.colours[10], 6);
      expect(canvas.policyVersion, 4);

      expect(
        canvas.apply(
          _event(policyVersion: 3, revision: 99, colour: 7, pixels: [10]),
        ),
        isFalse,
      );
      expect(canvas.colours[10], 6);
    });
  });
}

ColourPixelsFlipped _event({
  int policyVersion = 1,
  required int revision,
  required int colour,
  required List<int> pixels,
}) {
  return ColourPixelsFlipped(
    player: 'player',
    policyVersion: policyVersion,
    revision: revision,
    gameIndex: 0,
    sectionIndex: 0,
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

int _revision(ColourCanvasBuffer canvas, int pixel) => ByteData.sublistView(
  canvas.pixelRevisions,
).getUint64(pixel * 8, Endian.little);
