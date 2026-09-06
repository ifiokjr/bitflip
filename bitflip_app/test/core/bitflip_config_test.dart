import 'package:bitflip_app/core/bitflip_config.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('BitflipConfig', () {
    test('keeps safe local defaults outside release mode', () {
      final config = BitflipConfig.parse(
        environment: '',
        walletChain: '',
        rpcUrl: '',
        serverpodUrl: '',
        gameIndex: '',
        releaseMode: false,
      );

      expect(config.environment, BitflipEnvironment.development);
      expect(config.walletChain, 'solana:devnet');
      expect(config.rpcUrl, 'https://api.devnet.solana.com');
      expect(config.serverpodUrl, 'http://localhost:8080/');
      expect(config.gameIndex, 0);
    });

    test('release mode rejects missing configuration', () {
      expect(
        () => BitflipConfig.parse(
          environment: '',
          walletChain: '',
          rpcUrl: '',
          serverpodUrl: '',
          gameIndex: '',
          releaseMode: true,
        ),
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            contains('BITFLIP_ENVIRONMENT'),
          ),
        ),
      );
    });

    test('production requires mainnet and public HTTPS endpoints', () {
      expect(
        () => _productionConfig(walletChain: 'solana:devnet'),
        throwsA(isA<StateError>()),
      );
      expect(
        () => _productionConfig(rpcUrl: 'http://127.0.0.1:8899'),
        throwsA(isA<StateError>()),
      );
      expect(
        () => _productionConfig(serverpodUrl: 'http://localhost:8080/'),
        throwsA(isA<StateError>()),
      );
    });

    test('accepts complete mainnet production configuration', () {
      final config = _productionConfig();

      expect(config.environment, BitflipEnvironment.production);
      expect(config.walletChain, 'solana:mainnet');
      expect(config.gameIndex, 3);
    });

    test('rejects a game outside the four configured games', () {
      expect(
        () => _productionConfig(gameIndex: '4'),
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            contains('between 0 and 3'),
          ),
        ),
      );
    });
  });
}

BitflipConfig _productionConfig({
  String walletChain = 'solana:mainnet',
  String rpcUrl = 'https://rpc.bitflip.xyz',
  String serverpodUrl = 'https://api.bitflip.xyz/',
  String gameIndex = '3',
}) {
  return BitflipConfig.parse(
    environment: 'production',
    walletChain: walletChain,
    rpcUrl: rpcUrl,
    serverpodUrl: serverpodUrl,
    gameIndex: gameIndex,
    releaseMode: true,
  );
}
