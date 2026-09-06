import 'dart:typed_data';

import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';

const noSectionColour = 255;

final class PixelColourMap {
  PixelColourMap._(this._colours);

  factory PixelColourMap.empty() => PixelColourMap._(
    Uint8List(sectionPixelCount)
      ..fillRange(0, sectionPixelCount, noSectionColour),
  );

  factory PixelColourMap.fromBytes(Uint8List colours) {
    if (colours.length != sectionPixelCount) {
      throw ArgumentError.value(
        colours.length,
        'colours.length',
        'must be $sectionPixelCount',
      );
    }
    if (colours.any(
      (colour) =>
          colour != noSectionColour && colour >= SectionColour.values.length,
    )) {
      throw const FormatException('The canvas contains an unknown colour.');
    }
    return PixelColourMap._(Uint8List.fromList(colours));
  }

  final Uint8List _colours;

  SectionColour? colourAt(int x, int y) {
    if (x < 0 || x >= sectionSide || y < 0 || y >= sectionSide) {
      throw RangeError('Pixel coordinates must be within 0–63.');
    }
    final colour = _colours[y * sectionSide + x];
    return colour == noSectionColour ? null : SectionColour.fromCode(colour);
  }

  PixelColourMap painted(
    Iterable<PixelCoordinate> coordinates,
    SectionColour colour,
  ) {
    final next = PixelColourMap._(Uint8List.fromList(_colours));
    for (final coordinate in coordinates) {
      next._colours[coordinate.linearIndex] = colour.code;
    }
    return next;
  }
}
