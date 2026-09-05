import 'dart:typed_data';

import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('PixelBitmap', () {
    test('stores exactly one bit per pixel', () {
      expect(sectionByteCount, 512);
      expect(sectionPixelCount, 4096);
      expect(PixelBitmap.empty().bytes, hasLength(sectionByteCount));
    });

    test('rejects a bitmap with the wrong byte length', () {
      expect(() => PixelBitmap.fromBytes(Uint8List(511)), throwsArgumentError);
    });

    test('defensively copies incoming and outgoing bytes', () {
      final source = Uint8List(sectionByteCount);
      final bitmap = PixelBitmap.fromBytes(source);
      source[0] = 255;
      final exported = bitmap.bytes..[0] = 255;

      expect(bitmap.isOn(0, 0), isFalse);
      expect(exported[0], 255);
      expect(bitmap.bytes[0], 0);
    });

    test('maps boundary coordinates to the correct bits', () {
      final bitmap = PixelBitmap.empty().toggled(const [
        PixelCoordinate(0, 0),
        PixelCoordinate(63, 0),
        PixelCoordinate(0, 63),
        PixelCoordinate(63, 63),
      ]);

      expect(bitmap.onCount, 4);
      expect(bitmap.isOn(0, 0), isTrue);
      expect(bitmap.isOn(63, 0), isTrue);
      expect(bitmap.isOn(0, 63), isTrue);
      expect(bitmap.isOn(63, 63), isTrue);
      expect(bitmap.bytes.first, 1);
      expect(bitmap.bytes.last, 128);
    });

    test('toggle is immutable and reversible', () {
      final empty = PixelBitmap.empty();
      final enabled = empty.toggled(const [PixelCoordinate(9, 7)]);
      final disabled = enabled.toggled(const [PixelCoordinate(9, 7)]);

      expect(empty.isOn(9, 7), isFalse);
      expect(enabled.isOn(9, 7), isTrue);
      expect(disabled.isOn(9, 7), isFalse);
    });

    test('duplicate coordinates cancel in the local preview', () {
      final bitmap = PixelBitmap.empty().toggled(const [
        PixelCoordinate(3, 4),
        PixelCoordinate(3, 4),
      ]);

      expect(bitmap.onCount, 0);
    });

    test('rejects reads outside the 64 by 64 section', () {
      final bitmap = PixelBitmap.empty();

      expect(() => bitmap.isOn(-1, 0), throwsRangeError);
      expect(() => bitmap.isOn(64, 0), throwsRangeError);
      expect(() => bitmap.isOn(0, -1), throwsRangeError);
      expect(() => bitmap.isOn(0, 64), throwsRangeError);
    });

    test('demo art is deterministic and non-empty', () {
      final first = PixelBitmap.demo(8);
      final second = PixelBitmap.demo(8);
      final different = PixelBitmap.demo(9);

      expect(first.bytes, orderedEquals(second.bytes));
      expect(first.bytes, isNot(orderedEquals(different.bytes)));
      expect(first.onCount, greaterThan(0));
    });
  });

  group('PixelCoordinate', () {
    test('sorts in row-major order and deduplicates by value', () {
      final uniqueCoordinates = <PixelCoordinate>{};
      uniqueCoordinates.addAll(const [
        PixelCoordinate(1, 1),
        PixelCoordinate(0, 1),
        PixelCoordinate(1, 1),
        PixelCoordinate(63, 0),
      ]);
      final coordinates = uniqueCoordinates.toList()..sort();

      expect(coordinates, const [
        PixelCoordinate(63, 0),
        PixelCoordinate(0, 1),
        PixelCoordinate(1, 1),
      ]);
    });
  });
}
