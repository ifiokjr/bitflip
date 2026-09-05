import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_server_server/src/minting/solana_signature.dart';
import 'package:bs58/bs58.dart';
import 'package:ed25519_edwards/ed25519_edwards.dart' as ed25519;
import 'package:test/test.dart';

void main() {
  group('Solana signature verification', () {
    test('accepts a valid Ed25519 signature', () {
      final keyPair = ed25519.generateKey();
      const message = 'Bitflip authorization';
      final signature = ed25519.sign(
        keyPair.privateKey,
        Uint8List.fromList(utf8.encode(message)),
      );

      expect(
        verifySolanaSignature(
          publicKeyBytes: Uint8List.fromList(keyPair.publicKey.bytes),
          message: message,
          signatureBytes: signature,
        ),
        isTrue,
      );
    });

    test('rejects a changed message or signer', () {
      final keyPair = ed25519.generateKey();
      final otherKeyPair = ed25519.generateKey();
      const message = 'Bitflip authorization';
      final signature = ed25519.sign(
        keyPair.privateKey,
        Uint8List.fromList(utf8.encode(message)),
      );

      expect(
        verifySolanaSignature(
          publicKeyBytes: Uint8List.fromList(keyPair.publicKey.bytes),
          message: '$message changed',
          signatureBytes: signature,
        ),
        isFalse,
      );
      expect(
        verifySolanaSignature(
          publicKeyBytes: Uint8List.fromList(otherKeyPair.publicKey.bytes),
          message: message,
          signatureBytes: signature,
        ),
        isFalse,
      );
    });

    test('enforces canonical public-key and signature lengths', () {
      expect(
        () => decodeBase58PublicKey('not-a-wallet'),
        throwsFormatException,
      );
      expect(
        () => decodeBase64Signature(base64Encode([1, 2])),
        throwsFormatException,
      );
      expect(() => decodeBase64Signature('a' * 129), throwsFormatException);
    });

    test('decodes a 32-byte base58 public key', () {
      final publicKey = Uint8List.fromList(List<int>.generate(32, (i) => i));
      final encoded = base58.encoder.convert(publicKey);

      expect(decodeBase58PublicKey(encoded), orderedEquals(publicKey));
    });
  });
}
