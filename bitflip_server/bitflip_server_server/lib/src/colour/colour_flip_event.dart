import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program_constraints.dart';
import 'package:bs58/bs58.dart';

const colourPixelsFlippedEventDiscriminator = 1;
const colourPixelsFlippedEventSize = 85;
const colourPaletteSize = 8;
const colourCanvasSide = 64;
const colourCanvasPixelCount = colourCanvasSide * colourCanvasSide;
const maximumColourFlipBatch = 16;

final class ColourPixelsFlipped {
  const ColourPixelsFlipped({
    required this.player,
    required this.policyVersion,
    required this.revision,
    required this.gameIndex,
    required this.sectionIndex,
    required this.colour,
    required this.coordinates,
  });

  final String player;
  final int policyVersion;
  final int revision;
  final int gameIndex;
  final int sectionIndex;
  final int colour;
  final List<ColourPixelCoordinate> coordinates;
}

final class ColourPixelCoordinate {
  const ColourPixelCoordinate(this.x, this.y);

  final int x;
  final int y;

  int get linearIndex => y * colourCanvasSide + x;
}

ColourPixelsFlipped decodeColourPixelsFlippedEvent(String encoded) {
  final Uint8List bytes;
  try {
    bytes = base64Decode(encoded);
  } on FormatException {
    throw const FormatException('Invalid Bitflip colour event encoding.');
  }
  if (bytes.length != colourPixelsFlippedEventSize ||
      bytes[0] != colourPixelsFlippedEventDiscriminator) {
    throw const FormatException('Invalid Bitflip colour event layout.');
  }
  final view = ByteData.sublistView(bytes);
  final policyVersion = view.getUint64(33, Endian.little);
  final revision = view.getUint64(41, Endian.little);
  final gameIndex = bytes[81];
  final sectionIndex = bytes[82];
  final count = bytes[83];
  final colour = bytes[84];
  if (policyVersion == 0 ||
      policyVersion > 0x7fffffffffffffff ||
      revision == 0 ||
      revision > 0x7fffffffffffffff ||
      gameIndex >= bitflipGameCount ||
      count == 0 ||
      count > maximumColourFlipBatch ||
      colour >= colourPaletteSize) {
    throw const FormatException('Invalid Bitflip colour event values.');
  }
  final coordinates = <ColourPixelCoordinate>[];
  final seenPixels = <int>{};
  for (var index = 0; index < count; index++) {
    final x = bytes[49 + index * 2];
    final y = bytes[50 + index * 2];
    if (x >= colourCanvasSide || y >= colourCanvasSide) {
      throw const FormatException('Invalid Bitflip colour coordinates.');
    }
    final coordinate = ColourPixelCoordinate(x, y);
    if (!seenPixels.add(coordinate.linearIndex)) {
      throw const FormatException('Duplicate Bitflip colour coordinates.');
    }
    coordinates.add(coordinate);
  }
  return ColourPixelsFlipped(
    player: base58.encoder.convert(Uint8List.sublistView(bytes, 1, 33)),
    policyVersion: policyVersion,
    revision: revision,
    gameIndex: gameIndex,
    sectionIndex: sectionIndex,
    colour: colour,
    coordinates: List.unmodifiable(coordinates),
  );
}

List<ColourPixelsFlipped> colourEventsFromProgramLogs(
  Iterable<String> logs, {
  required String programAddress,
}) {
  final invocationStack = <String>[];
  final events = <ColourPixelsFlipped>[];
  for (final log in logs) {
    final invoked = _programFromSuffix(log, ' invoke [');
    if (invoked != null) {
      invocationStack.add(invoked);
      continue;
    }
    if (log.startsWith('Program data: ') &&
        invocationStack.lastOrNull == programAddress) {
      final fields = log.substring('Program data: '.length).trim().split(' ');
      for (final field in fields) {
        if (field.isEmpty) continue;
        try {
          events.add(decodeColourPixelsFlippedEvent(field));
        } on FormatException {
          // A Bitflip instruction may log unrelated binary data. Only the
          // exact, versioned colour event layout is accepted.
        }
      }
      continue;
    }
    final completed =
        _programFromSuffix(log, ' success') ??
        _programFromSuffix(log, ' failed:');
    if (completed == null || invocationStack.isEmpty) continue;
    if (invocationStack.last == completed) {
      invocationStack.removeLast();
    } else {
      invocationStack.clear();
    }
  }
  return List.unmodifiable(events);
}

String? _programFromSuffix(String log, String suffix) {
  const prefix = 'Program ';
  if (!log.startsWith(prefix)) return null;
  final suffixIndex = log.indexOf(suffix, prefix.length);
  if (suffixIndex < 0) return null;
  return log.substring(prefix.length, suffixIndex);
}

extension<T> on List<T> {
  T? get lastOrNull => isEmpty ? null : last;
}
