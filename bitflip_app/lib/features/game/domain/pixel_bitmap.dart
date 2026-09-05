import 'dart:math' as math;
import 'dart:typed_data';

const sectionSide = 64;
const sectionPixelCount = sectionSide * sectionSide;
const sectionByteCount = sectionPixelCount ~/ 8;
const sectionCount = 256;
const maxFlipBatch = 16;

final class PixelCoordinate implements Comparable<PixelCoordinate> {
  const PixelCoordinate(this.x, this.y)
    : assert(x >= 0 && x < sectionSide),
      assert(y >= 0 && y < sectionSide);

  final int x;
  final int y;

  int get linearIndex => (y * sectionSide) + x;

  @override
  int compareTo(PixelCoordinate other) =>
      linearIndex.compareTo(other.linearIndex);

  @override
  bool operator ==(Object other) {
    return other is PixelCoordinate && other.x == x && other.y == y;
  }

  @override
  int get hashCode => Object.hash(x, y);
}

final class PixelBitmap {
  PixelBitmap._(this._bytes);

  factory PixelBitmap.empty() => PixelBitmap._(Uint8List(sectionByteCount));

  factory PixelBitmap.fromBytes(Uint8List bytes) {
    if (bytes.length != sectionByteCount) {
      throw ArgumentError.value(bytes.length, 'bytes.length', 'must be 512');
    }
    return PixelBitmap._(Uint8List.fromList(bytes));
  }

  factory PixelBitmap.demo(int seed) {
    final bitmap = PixelBitmap.empty();
    for (var y = 0; y < sectionSide; y++) {
      for (var x = 0; x < sectionSide; x++) {
        final wave = 31 + math.sin((x + seed * 5) * 0.19) * 9;
        final echo = 31 + math.cos((x - seed * 3) * 0.11) * 16;
        final ring = math.sqrt(math.pow(x - 31.5, 2) + math.pow(y - 31.5, 2));
        final isSignal = (y - wave).abs() < 1.2;
        final isEcho = (y - echo).abs() < 0.7 && (x + seed).isEven;
        final isRing = (ring - (11 + seed % 7)).abs() < 0.65;
        final isMarker = (x + (y * 3) + (seed * 7)) % 97 == 0;
        if (isSignal || isEcho || isRing || isMarker) {
          bitmap._set(x, y, true);
        }
      }
    }
    return bitmap;
  }

  final Uint8List _bytes;

  Uint8List get bytes => Uint8List.fromList(_bytes);

  int get onCount {
    var count = 0;
    for (final byte in _bytes) {
      count += _bitCount[byte];
    }
    return count;
  }

  bool isOn(int x, int y) {
    _validateCoordinate(x, y);
    final index = (y * sectionSide) + x;
    return (_bytes[index >> 3] & (1 << (index & 7))) != 0;
  }

  PixelBitmap toggled(Iterable<PixelCoordinate> coordinates) {
    final next = PixelBitmap.fromBytes(_bytes);
    for (final coordinate in coordinates) {
      next._set(
        coordinate.x,
        coordinate.y,
        !next.isOn(coordinate.x, coordinate.y),
      );
    }
    return next;
  }

  void _set(int x, int y, bool value) {
    final index = (y * sectionSide) + x;
    final byteIndex = index >> 3;
    final mask = 1 << (index & 7);
    _bytes[byteIndex] = value
        ? _bytes[byteIndex] | mask
        : _bytes[byteIndex] & ~mask;
  }

  static void _validateCoordinate(int x, int y) {
    if (x < 0 || x >= sectionSide || y < 0 || y >= sectionSide) {
      throw RangeError('Pixel coordinates must be within 0–63.');
    }
  }
}

const _bitCount = <int>[
  0,
  1,
  1,
  2,
  1,
  2,
  2,
  3,
  1,
  2,
  2,
  3,
  2,
  3,
  3,
  4,
  1,
  2,
  2,
  3,
  2,
  3,
  3,
  4,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  1,
  2,
  2,
  3,
  2,
  3,
  3,
  4,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  1,
  2,
  2,
  3,
  2,
  3,
  3,
  4,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  4,
  5,
  5,
  6,
  5,
  6,
  6,
  7,
  1,
  2,
  2,
  3,
  2,
  3,
  3,
  4,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  4,
  5,
  5,
  6,
  5,
  6,
  6,
  7,
  2,
  3,
  3,
  4,
  3,
  4,
  4,
  5,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  4,
  5,
  5,
  6,
  5,
  6,
  6,
  7,
  3,
  4,
  4,
  5,
  4,
  5,
  5,
  6,
  4,
  5,
  5,
  6,
  5,
  6,
  6,
  7,
  4,
  5,
  5,
  6,
  5,
  6,
  6,
  7,
  5,
  6,
  6,
  7,
  6,
  7,
  7,
  8,
];
