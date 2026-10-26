import 'dart:typed_data';

import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/pixel_colour_map.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('paints an immutable eight-colour off-chain layer', () {
    final empty = PixelColourMap.empty();
    final painted = empty.painted(const [
      PixelCoordinate(2, 3),
      PixelCoordinate(63, 63),
    ], SectionColour.violet);

    expect(empty.colourAt(2, 3), isNull);
    expect(painted.colourAt(2, 3), SectionColour.violet);
    expect(painted.colourAt(63, 63), SectionColour.violet);
  });

  test('rejects invalid canvas sizes and palette entries', () {
    expect(
      () => PixelColourMap.fromBytes(Uint8List(sectionPixelCount - 1)),
      throwsArgumentError,
    );
    final colours = Uint8List(sectionPixelCount)
      ..fillRange(0, sectionPixelCount, noSectionColour)
      ..[42] = 8;
    expect(() => PixelColourMap.fromBytes(colours), throwsFormatException);
  });

  test('compares by palette content and exposes raw codes', () {
    final painted = PixelColourMap.empty().painted(const [
      PixelCoordinate(5, 6),
    ], SectionColour.amber);
    final copy = PixelColourMap.fromBytes(painted.codes);

    expect(painted, equals(copy));
    expect(painted.hashCode, copy.hashCode);
    expect(painted, isNot(equals(PixelColourMap.empty())));
    expect(painted.codeAt(const PixelCoordinate(5, 6).linearIndex), 5);
    expect(painted.codeAt(0), noSectionColour);
    expect(painted.codes, hasLength(sectionPixelCount));
  });
}
