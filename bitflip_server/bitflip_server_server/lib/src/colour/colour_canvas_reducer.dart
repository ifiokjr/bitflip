import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';

const noPixelColour = 255;
const colourRevisionByteCount = colourCanvasPixelCount * 8;

final class ColourCanvasBuffer {
  ColourCanvasBuffer._({
    required this.policyVersion,
    required this.highestRevision,
    required this.colours,
    required this.pixelRevisions,
  });

  factory ColourCanvasBuffer.empty({int policyVersion = 0}) {
    return ColourCanvasBuffer._(
      policyVersion: policyVersion,
      highestRevision: 0,
      colours: Uint8List(colourCanvasPixelCount)
        ..fillRange(
          0,
          colourCanvasPixelCount,
          noPixelColour,
        ),
      pixelRevisions: Uint8List(colourRevisionByteCount),
    );
  }

  factory ColourCanvasBuffer.fromBytes({
    required int policyVersion,
    required int highestRevision,
    required Uint8List colours,
    required Uint8List pixelRevisions,
  }) {
    if (colours.length != colourCanvasPixelCount ||
        pixelRevisions.length != colourRevisionByteCount) {
      throw const FormatException('Stored colour canvas has an invalid size.');
    }
    return ColourCanvasBuffer._(
      policyVersion: policyVersion,
      highestRevision: highestRevision,
      colours: Uint8List.fromList(colours),
      pixelRevisions: Uint8List.fromList(pixelRevisions),
    );
  }

  int policyVersion;
  int highestRevision;
  final Uint8List colours;
  final Uint8List pixelRevisions;

  bool apply(ColourPixelsFlipped event) {
    if (event.policyVersion < policyVersion) return false;
    if (event.policyVersion > policyVersion) {
      policyVersion = event.policyVersion;
      highestRevision = 0;
      colours.fillRange(0, colours.length, noPixelColour);
      pixelRevisions.fillRange(0, pixelRevisions.length, 0);
    }
    final revisions = ByteData.sublistView(pixelRevisions);
    var changed = false;
    for (final coordinate in event.coordinates) {
      final pixel = coordinate.linearIndex;
      final revisionOffset = pixel * 8;
      if (event.revision <=
          revisions.getUint64(revisionOffset, Endian.little)) {
        continue;
      }
      revisions.setUint64(revisionOffset, event.revision, Endian.little);
      colours[pixel] = event.colour;
      changed = true;
    }
    if (event.revision > highestRevision) highestRevision = event.revision;
    return changed;
  }
}
