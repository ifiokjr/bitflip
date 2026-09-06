import 'dart:convert';
import 'dart:io';

import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:test/test.dart';

void main() {
  group('production mint configuration', () {
    test('rejects every missing required value', () {
      const requiredNames = [
        'SOLANA_RPC_URL',
        'BITFLIP_METADATA_BASE_URL',
        'BITFLIP_GAME_INDEX',
        'BITFLIP_CLUSTER',
        'BITFLIP_MERKLE_TREE',
        'BITFLIP_OPERATOR_PRIVATE_KEY',
        'BITFLIP_PRIORITY_FEE_MICROLAMPORTS',
      ];
      for (final name in requiredNames) {
        final environment = _validEnvironment()..remove(name);
        expect(
          () => SolanaBitflipMintService.fromEnvironment(
            environment,
            production: true,
          ),
          throwsA(isA<StateError>()),
          reason: name,
        );
      }
    });

    test('rejects non-mainnet and non-HTTPS production values', () {
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment()..['BITFLIP_CLUSTER'] = 'devnet',
          production: true,
        ),
        throwsA(isA<StateError>()),
      );
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment()
            ..['SOLANA_RPC_URL'] = 'https://api.devnet.solana.com',
          production: true,
        ),
        throwsA(isA<StateError>()),
      );
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment()
            ..['BITFLIP_METADATA_BASE_URL'] = 'http://localhost:8082',
          production: true,
        ),
        throwsA(isA<StateError>()),
      );
    });

    test('validates signer and tree syntax before startup', () {
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment()..['BITFLIP_OPERATOR_PRIVATE_KEY'] = 'secret',
          production: true,
        ),
        throwsA(anything),
      );
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment()..['BITFLIP_MERKLE_TREE'] = 'not-an-address',
          production: true,
        ),
        throwsA(anything),
      );
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment()..['BITFLIP_PRIORITY_FEE_MICROLAMPORTS'] = '-1',
          production: true,
        ),
        throwsA(isA<StateError>()),
      );
    });

    test('accepts a complete production environment', () {
      final service = SolanaBitflipMintService.fromEnvironment(
        _validEnvironment(),
        production: true,
      );

      expect(service.gameIndex, 7);
      expect(service.metadataBaseUrl, 'https://metadata.bitflip.xyz');
      expect(service.priorityFeeMicroLamports, 1000);
    });

    test('staging requires explicit devnet signing configuration', () {
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          const {},
          production: false,
          requireExplicitConfiguration: true,
        ),
        throwsA(isA<StateError>()),
      );

      final service = SolanaBitflipMintService.fromEnvironment(
        _validEnvironment()
          ..['BITFLIP_CLUSTER'] = 'devnet'
          ..['SOLANA_RPC_URL'] = 'https://api.devnet.solana.com',
        production: false,
        requireExplicitConfiguration: true,
      );

      expect(service.gameIndex, 7);
      expect(service.priorityFeeMicroLamports, 1000);
    });

    test('staging rejects an explicit mainnet cluster', () {
      expect(
        () => SolanaBitflipMintService.fromEnvironment(
          _validEnvironment(),
          production: false,
          requireExplicitConfiguration: true,
        ),
        throwsA(isA<StateError>()),
      );
    });

    test('development retains explicit local defaults', () {
      final service = SolanaBitflipMintService.fromEnvironment(
        const {},
        production: false,
      );

      expect(service.gameIndex, 0);
      expect(service.metadataBaseUrl, 'http://localhost:8082');
    });

    test('does not expose Serverpod Insights in production', () {
      final configuration = File('config/production.yaml').readAsStringSync();

      expect(configuration, isNot(contains('\ninsightsServer:')));
    });
  });
}

Map<String, String> _validEnvironment() => {
  'SOLANA_RPC_URL': 'https://rpc.bitflip.xyz',
  'BITFLIP_METADATA_BASE_URL': 'https://metadata.bitflip.xyz',
  'BITFLIP_GAME_INDEX': '7',
  'BITFLIP_CLUSTER': 'mainnet',
  'BITFLIP_MERKLE_TREE': '11111111111111111111111111111111',
  'BITFLIP_OPERATOR_PRIVATE_KEY': jsonEncode(List<int>.filled(32, 1)),
  'BITFLIP_PRIORITY_FEE_MICROLAMPORTS': '1000',
};
