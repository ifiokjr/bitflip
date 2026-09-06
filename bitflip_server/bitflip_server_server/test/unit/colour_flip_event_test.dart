import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:test/test.dart';

void main() {
  group('Bitflip colour event parsing', () {
    test('decodes the exact versioned event layout', () {
      final event = decodeColourPixelsFlippedEvent(
        _encodedEvent(
          policyVersion: 7,
          revision: 42,
          gameIndex: 3,
          sectionIndex: 255,
          colour: 6,
          coordinates: const [
            ColourPixelCoordinate(1, 2),
            ColourPixelCoordinate(63, 0),
          ],
        ),
      );

      expect(event.policyVersion, 7);
      expect(event.revision, 42);
      expect(event.gameIndex, 3);
      expect(event.sectionIndex, 255);
      expect(event.colour, 6);
      expect(event.coordinates.map((coordinate) => coordinate.linearIndex), [
        129,
        63,
      ]);
    });

    test('accepts data only while the Bitflip program is executing', () {
      final encoded = _encodedEvent(
        revision: 1,
        coordinates: const [ColourPixelCoordinate(4, 5)],
      );
      final events = colourEventsFromProgramLogs([
        'Program attacker invoke [1]',
        'Program data: $encoded',
        'Program attacker success',
        'Program bitflip invoke [1]',
        'Program cpi invoke [2]',
        'Program data: $encoded',
        'Program cpi success',
        'Program data: $encoded',
        'Program bitflip success',
      ], programAddress: 'bitflip');

      expect(events, hasLength(1));
      expect(events.single.coordinates.single.linearIndex, 324);
    });

    test('rejects malformed or out-of-range events', () {
      expect(
        () => decodeColourPixelsFlippedEvent(base64Encode(Uint8List(84))),
        throwsFormatException,
      );
      expect(
        () => decodeColourPixelsFlippedEvent(
          _encodedEvent(
            colour: 8,
            coordinates: const [ColourPixelCoordinate(0, 0)],
          ),
        ),
        throwsFormatException,
      );
      expect(
        () => decodeColourPixelsFlippedEvent(
          _encodedEvent(
            policyVersion: 0,
            coordinates: const [ColourPixelCoordinate(0, 0)],
          ),
        ),
        throwsFormatException,
      );
      expect(
        () => decodeColourPixelsFlippedEvent(
          _encodedEvent(
            coordinates: const [
              ColourPixelCoordinate(3, 2),
              ColourPixelCoordinate(3, 2),
            ],
          ),
        ),
        throwsFormatException,
      );
      expect(
        () => decodeColourPixelsFlippedEvent(
          _encodedEvent(
            revision: 0,
            coordinates: const [ColourPixelCoordinate(0, 0)],
          ),
        ),
        throwsFormatException,
      );
    });
  });
}

String _encodedEvent({
  int policyVersion = 1,
  int revision = 1,
  int gameIndex = 0,
  int sectionIndex = 0,
  int colour = 0,
  required List<ColourPixelCoordinate> coordinates,
}) {
  final bytes = Uint8List(colourPixelsFlippedEventSize);
  bytes[0] = colourPixelsFlippedEventDiscriminator;
  for (var index = 0; index < 32; index++) {
    bytes[1 + index] = index;
  }
  ByteData.sublistView(bytes)
    ..setUint64(33, policyVersion, Endian.little)
    ..setUint64(41, revision, Endian.little);
  for (var index = 0; index < coordinates.length; index++) {
    bytes[49 + index * 2] = coordinates[index].x;
    bytes[50 + index * 2] = coordinates[index].y;
  }
  bytes[81] = gameIndex;
  bytes[82] = sectionIndex;
  bytes[83] = coordinates.length;
  bytes[84] = colour;
  return base64Encode(bytes);
}
