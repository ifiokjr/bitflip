import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program.dart' as pina_client;
import 'package:bitflip_program/bitflip_program_constraints.dart';

const colourPixelsFlippedEventDiscriminator =
    pina_client.colourPixelsFlippedEventEventDiscriminator;
const colourPixelsFlippedEventMigrationVersion =
    pina_client.colourPixelsFlippedEventEventMigrationVersion;
const colourPixelsFlippedEventSize =
    pina_client.colourPixelsFlippedEventEventSize;
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
  final pina_client.ColourPixelsFlippedEventEvent event;
  try {
    event = pina_client.decodeColourPixelsFlippedEventEvent(bytes);

  } on RangeError {
    throw const FormatException('Invalid Bitflip colour event layout.');
  }
  final maximumSignedInt = BigInt.from(0x7fffffffffffffff);
  if (event.policyVersion == BigInt.zero ||
      event.policyVersion > maximumSignedInt ||
      event.revision == BigInt.zero ||
      event.revision > maximumSignedInt ||
      event.gameIndex >= bitflipGameCount ||
      event.count == 0 ||
      event.count > maximumColourFlipBatch ||
      event.colour >= colourPaletteSize) {
    throw const FormatException('Invalid Bitflip colour event values.');
  }
  final coordinates = <ColourPixelCoordinate>[];
  final seenPixels = <int>{};

  for (var index = 0; index < event.count; index++) {
    final x = event.coordinates[index * 2];
    final y = event.coordinates[index * 2 + 1];

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
    player: event.player.value,
    policyVersion: event.policyVersion.toInt(),
    revision: event.revision.toInt(),
    gameIndex: event.gameIndex,
    sectionIndex: event.sectionIndex,
    colour: event.colour,
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
