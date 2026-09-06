import 'dart:typed_data';

import 'package:bitflip_server_server/src/colour/colour_event_indexer.dart';
import 'package:bs58/bs58.dart';
import 'package:test/test.dart';

void main() {
  group('ColourIndexerConfiguration', () {
    test('is disabled by default for local development', () {
      final configuration = ColourIndexerConfiguration.fromEnvironment(
        const {},
        requireEnabled: false,
      );
      expect(configuration.enabled, isFalse);
      expect(configuration.cluster, 'development');
    });

    test('requires durable indexing for a release deployment', () {
      expect(
        () => ColourIndexerConfiguration.fromEnvironment(
          const {},
          requireEnabled: true,
        ),
        throwsA(isA<StateError>()),
      );
    });

    test('requires an anchor and validates operational bounds', () {
      expect(
        () => ColourIndexerConfiguration.fromEnvironment(
          const {
            'BITFLIP_COLOUR_INDEXER_ENABLED': 'true',
            'BITFLIP_CLUSTER': 'devnet',
          },
          requireEnabled: false,
        ),
        throwsA(isA<StateError>()),
      );

      final signature = base58.encoder.convert(
        Uint8List.fromList(List.generate(64, (index) => index + 1)),
      );
      final configuration = ColourIndexerConfiguration.fromEnvironment(
        {
          'BITFLIP_COLOUR_INDEXER_ENABLED': 'true',
          'BITFLIP_CLUSTER': 'devnet',
          'BITFLIP_COLOUR_INDEXER_START_SIGNATURE': signature,
          'BITFLIP_COLOUR_INDEXER_INTERVAL_SECONDS': '3',
          'BITFLIP_COLOUR_INDEXER_PAGE_SIZE': '128',
          'BITFLIP_COLOUR_INDEXER_CONCURRENCY': '8',
        },
        requireEnabled: true,
      );
      expect(configuration.enabled, isTrue);
      expect(configuration.startSignature, signature);
      expect(configuration.interval, const Duration(seconds: 3));
      expect(configuration.pageSize, 128);
      expect(configuration.maximumConcurrentTransactions, 8);
    });
  });
}
