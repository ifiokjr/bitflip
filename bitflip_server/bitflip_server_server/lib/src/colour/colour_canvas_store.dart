import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_canvas_reducer.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:serverpod/serverpod.dart';

abstract final class ColourCanvasStore {
  static Future<ColourCanvasState> applySectionEvents(
    Session session, {
    required int gameIndex,
    required int sectionIndex,
    required Iterable<ColourPixelsFlipped> events,
  }) async {
    final orderedEvents = events.toList()
      ..sort((left, right) => left.revision.compareTo(right.revision));
    if (orderedEvents.isEmpty) {
      throw ArgumentError.value(events, 'events', 'Events cannot be empty.');
    }
    late ColourCanvasState result;
    await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
      transaction,
    ) async {
      await session.db.unsafeQuery(
        'SELECT pg_advisory_xact_lock(@game, @section);',
        transaction: transaction,
        parameters: QueryParameters.named({
          'game': gameIndex,
          'section': sectionIndex,
        }),
      );
      final stored = await ColourCanvasState.db.findFirstRow(
        session,
        where: (table) =>
            table.gameIndex.equals(gameIndex) &
            table.sectionIndex.equals(sectionIndex),
        transaction: transaction,
        lockMode: LockMode.forUpdate,
      );
      final canvas = stored == null
          ? ColourCanvasBuffer.empty()
          : ColourCanvasBuffer.fromBytes(
              policyVersion: stored.policyVersion,
              highestRevision: stored.highestRevision,
              colours: bytesFromByteData(stored.colours),
              pixelRevisions: bytesFromByteData(stored.pixelRevisions),
            );
      for (final event in orderedEvents) {
        if (event.gameIndex != gameIndex ||
            event.sectionIndex != sectionIndex) {
          throw ArgumentError('A colour event belongs to another section.');
        }
        canvas.apply(event);
      }
      final next = ColourCanvasState(
        id: stored?.id,
        gameIndex: gameIndex,
        sectionIndex: sectionIndex,
        policyVersion: canvas.policyVersion,
        highestRevision: canvas.highestRevision,
        colours: ByteData.sublistView(canvas.colours),
        pixelRevisions: ByteData.sublistView(canvas.pixelRevisions),
        updatedAt: DateTime.now().toUtc(),
      );
      result = stored == null
          ? await ColourCanvasState.db.insertRow(
              session,
              next,
              transaction: transaction,
            )
          : await ColourCanvasState.db.updateRow(
              session,
              next,
              transaction: transaction,
            );
    });
    return result;
  }

  static Future<void> applyEvents(
    Session session,
    Iterable<ColourPixelsFlipped> events,
  ) async {
    final bySection = <(int, int), List<ColourPixelsFlipped>>{};
    for (final event in events) {
      bySection
          .putIfAbsent((event.gameIndex, event.sectionIndex), () => [])
          .add(event);
    }
    for (final entry in bySection.entries) {
      await applySectionEvents(
        session,
        gameIndex: entry.key.$1,
        sectionIndex: entry.key.$2,
        events: entry.value,
      );
    }
  }
}

Uint8List bytesFromByteData(ByteData data) => data.buffer.asUint8List(
  data.offsetInBytes,
  data.lengthInBytes,
);
