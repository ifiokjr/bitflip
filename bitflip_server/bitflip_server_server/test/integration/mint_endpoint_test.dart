import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bs58/bs58.dart';
import 'package:ed25519_edwards/ed25519_edwards.dart' as ed25519;
import 'package:solana_kit/solana_kit.dart';
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

void main() {
  withServerpod('Bitflip mint authorization', (sessionBuilder, endpoints) {
    late ed25519.KeyPair owner;
    late _FakeMintService mintService;

    setUp(() {
      owner = ed25519.generateKey();
      mintService = _FakeMintService(owner: _walletAddress(owner));
      MintServiceRegistry.configure(mintService);
    });

    test('mints a sealed section after a valid wallet signature', () async {
      final walletAddress = _walletAddress(owner);
      final challenge = await endpoints.mint.createChallenge(
        sessionBuilder,
        walletAddress: walletAddress,
        gameIndex: 0,
        sectionIndex: 12,
      );

      final result = await endpoints.mint.mintSection(
        sessionBuilder,
        walletAddress: walletAddress,
        gameIndex: 0,
        sectionIndex: 12,
        nonce: challenge.nonce,
        signatureBase64: _signature(owner, challenge.message),
      );

      expect(result.assetId, _address.value);
      expect(result.leafIndex, 42);
      expect(result.transactionSignature, 'surfpool-signature');
      expect(result.alreadyMinted, isFalse);
      expect(mintService.mintCalls, 1);
    });

    test('rejects replaying a consumed authorization', () async {
      final walletAddress = _walletAddress(owner);
      final challenge = await endpoints.mint.createChallenge(
        sessionBuilder,
        walletAddress: walletAddress,
        gameIndex: 0,
        sectionIndex: 12,
      );
      final signature = _signature(owner, challenge.message);

      await endpoints.mint.mintSection(
        sessionBuilder,
        walletAddress: walletAddress,
        gameIndex: 0,
        sectionIndex: 12,
        nonce: challenge.nonce,
        signatureBase64: signature,
      );
      await expectLater(
        endpoints.mint.mintSection(
          sessionBuilder,
          walletAddress: walletAddress,
          gameIndex: 0,
          sectionIndex: 12,
          nonce: challenge.nonce,
          signatureBase64: signature,
        ),
        throwsA(isA<StateError>()),
      );

      expect(mintService.mintCalls, 1);
    });

    test('rejects a valid signature from a different key', () async {
      final walletAddress = _walletAddress(owner);
      final challenge = await endpoints.mint.createChallenge(
        sessionBuilder,
        walletAddress: walletAddress,
        gameIndex: 0,
        sectionIndex: 12,
      );
      final attacker = ed25519.generateKey();

      await expectLater(
        endpoints.mint.mintSection(
          sessionBuilder,
          walletAddress: walletAddress,
          gameIndex: 0,
          sectionIndex: 12,
          nonce: challenge.nonce,
          signatureBase64: _signature(attacker, challenge.message),
        ),
        throwsA(isA<StateError>()),
      );

      expect(mintService.mintCalls, 0);
    });

    test('rejects malformed authorization inputs before minting', () async {
      await expectLater(
        endpoints.mint.mintSection(
          sessionBuilder,
          walletAddress: _walletAddress(owner),
          gameIndex: 0,
          sectionIndex: 12,
          nonce: 'not-a-canonical-nonce',
          signatureBase64: base64Encode(Uint8List(64)),
        ),
        throwsA(isA<FormatException>()),
      );
      await expectLater(
        endpoints.mint.mintSection(
          sessionBuilder,
          walletAddress: _walletAddress(owner),
          gameIndex: 256,
          sectionIndex: 12,
          nonce: 'a' * 32,
          signatureBase64: base64Encode(Uint8List(64)),
        ),
        throwsA(isA<RangeError>()),
      );

      expect(mintService.mintCalls, 0);
    });

    test('rejects challenge requests from a non-owner', () async {
      final attacker = ed25519.generateKey();

      await expectLater(
        endpoints.mint.createChallenge(
          sessionBuilder,
          walletAddress: _walletAddress(attacker),
          gameIndex: 0,
          sectionIndex: 12,
        ),
        throwsA(isA<StateError>()),
      );
    });

    test('rejects minting before the section is sealed', () async {
      mintService.status = 1;

      await expectLater(
        endpoints.mint.createChallenge(
          sessionBuilder,
          walletAddress: _walletAddress(owner),
          gameIndex: 0,
          sectionIndex: 12,
        ),
        throwsA(isA<StateError>()),
      );
    });

    test('rate limits repeated challenges for one wallet', () async {
      final walletAddress = _walletAddress(owner);
      for (var request = 0; request < 6; request++) {
        await endpoints.mint.createChallenge(
          sessionBuilder,
          walletAddress: walletAddress,
          gameIndex: 0,
          sectionIndex: 12,
        );
      }

      await expectLater(
        endpoints.mint.createChallenge(
          sessionBuilder,
          walletAddress: walletAddress,
          gameIndex: 0,
          sectionIndex: 12,
        ),
        throwsA(isA<StateError>()),
      );
    });
  });
}

const _address = Address('11111111111111111111111111111111');

final class _FakeMintService implements BitflipMintService {
  _FakeMintService({required this.owner});

  final String owner;
  int status = 2;
  int mintCalls = 0;

  @override
  Future<MintableSection> loadSection(int gameIndex, int sectionIndex) async {
    return MintableSection(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      owner: Address(owner),
      status: status,
      collectionAuthority: _address,
      configAddress: _address,
      gameAddress: _address,
      sectionAddress: _address,
      bitmap: Uint8List(512),
    );
  }

  @override
  Future<MintSubmission> mint(MintableSection section) async {
    mintCalls++;
    return const MintSubmission(
      assetId: _address,
      merkleTree: _address,
      leafIndex: 42,
      transactionSignature: 'surfpool-signature',
      alreadyMinted: false,
    );
  }
}

String _walletAddress(ed25519.KeyPair keyPair) {
  return base58.encoder.convert(Uint8List.fromList(keyPair.publicKey.bytes));
}

String _signature(ed25519.KeyPair keyPair, String message) {
  final signature = ed25519.sign(
    keyPair.privateKey,
    Uint8List.fromList(utf8.encode(message)),
  );
  return base64Encode(signature);
}
