// Auto-generated. Do not edit.
// ignore_for_file: type=lint

export 'event_log.dart';
export 'colour_pixels_flipped_event.dart';

import 'event_log.dart';
import 'colour_pixels_flipped_event.dart';

/// Decode every `Program data:` line that names one of this program's events.
///
/// Unrelated lines and programs are skipped. A log that names an event but
/// carries an unknown, future, or non-projectable version throws instead of
/// being silently dropped.
List<BitflipProgramEvent> parseBitflipProgramEventsFromLogs(List<String> logs) {
  final discovered = <BitflipProgramEvent>[];
  for (final log in logs) {
    final colourPixelsFlippedEvent = parseColourPixelsFlippedEventEventFromLog(log);
    if (colourPixelsFlippedEvent != null) {
      discovered.add(colourPixelsFlippedEvent);
      continue;
    }
  }
  return discovered;
}
