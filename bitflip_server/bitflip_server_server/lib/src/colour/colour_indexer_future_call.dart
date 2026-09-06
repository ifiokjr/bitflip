import 'package:bitflip_server_server/src/colour/colour_event_indexer.dart';
import 'package:serverpod/serverpod.dart';

final class ColourIndexerFutureCall extends FutureCall {
  Future<void> scan(Session session) async {
    final startedAt = DateTime.now().toUtc();
    try {
      final result = await ColourEventIndexerRegistry.indexer.runOnce(session);
      session.log(
        'Bitflip colour indexer batch completed.',
        level: LogLevel.info,
        metadata: {
          'operation': 'colour_indexer',
          'outcome': result.outcome,
          'signaturesScanned': result.signaturesScanned,
          'successfulTransactions': result.successfulTransactions,
          'eventsApplied': result.eventsApplied,
          'sweepCompleted': result.sweepCompleted,
          'durationMs': DateTime.now()
              .toUtc()
              .difference(startedAt)
              .inMilliseconds,
        },
      );
    } on Object catch (error, stackTrace) {
      session.log(
        'Bitflip colour indexer batch failed.',
        level: LogLevel.error,
        exception: error,
        stackTrace: stackTrace,
        metadata: {
          'operation': 'colour_indexer',
          'outcome': 'failed',
          'durationMs': DateTime.now()
              .toUtc()
              .difference(startedAt)
              .inMilliseconds,
        },
      );
      Error.throwWithStackTrace(error, stackTrace);
    }
  }
}
