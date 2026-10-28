import 'package:bitflip_program/bitflip_mint_authorization.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:test/test.dart';

void main() {
  test(
    'mint authorization binds wallet, section, nonce, expiry, and action',
    () {
      final expiry = DateTime.utc(2026, 9, 4, 12, 30);
      final message = MintAuthorization(
        walletAddress: '11111111111111111111111111111111',
        gameIndex: 3,
        sectionIndex: 219,
        nonce: 'abcdefghijklmnopqrstuvwxyz012345',
        expiresAt: expiry,
      ).message;

      expect(
        message,
        'Bitflip compressed NFT authorization v1\n'
        'Domain: bitflip/mint-section\n'
        'Wallet: 11111111111111111111111111111111\n'
        'Game: 3\n'
        'Section: 219\n'
        'Nonce: abcdefghijklmnopqrstuvwxyz012345\n'
        'Expires: 2026-09-04T12:30:00.000Z\n'
        'Action: Mint the sealed section to this wallet.',
      );
    },
  );

  group('metadata origin validation', () {
    test('accepts HTTPS and normalizes trailing slashes', () {
      final service = SolanaBitflipMintService(
        rpc: createSolanaRpc(
          url: 'http://127.0.0.1:8899',
          allowInsecureHttp: true,
        ),
        merkleTreeAddress: null,
        operatorPrivateKey: null,
        metadataBaseUrl: '  https://bitflip.xyz/content///  ',
      );

      expect(service.metadataBaseUrl, 'https://bitflip.xyz/content');
    });

    test('allows insecure HTTP only for local development', () {
      final service = SolanaBitflipMintService(
        rpc: createSolanaRpc(
          url: 'http://127.0.0.1:8899',
          allowInsecureHttp: true,
        ),
        merkleTreeAddress: null,
        operatorPrivateKey: null,
        metadataBaseUrl: 'http://localhost:8082/',
      );

      expect(service.metadataBaseUrl, 'http://localhost:8082');
    });

    test('rejects insecure or ambiguous permanent metadata URLs', () {
      for (final value in [
        'http://bitflip.xyz',
        'https://user:secret@bitflip.xyz',
        'https://bitflip.xyz?network=mainnet',
        'https://bitflip.xyz#metadata',
        'bitflip.xyz',
      ]) {
        expect(
          () => SolanaBitflipMintService(
            rpc: createSolanaRpc(
              url: 'http://127.0.0.1:8899',
              allowInsecureHttp: true,
            ),
            merkleTreeAddress: null,
            operatorPrivateKey: null,
            metadataBaseUrl: value,
          ),
          throwsA(isA<StateError>()),
          reason: value,
        );
      }
    });
  });
}
