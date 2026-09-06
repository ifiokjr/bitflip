import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_app/core/bitflip_wallet_native.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:solana_kit/solana_kit.dart';

void main() {
  group('embedded mobile wallet', () {
    test('creates one key and restores the same address', () async {
      final storage = _MemoryWalletStorage();
      final first = _wallet(storage: storage);

      await first.initialize();
      final address = first.address;

      expect(address, isNotNull);
      expect(storage.writeCalls, 1);
      expect(storage.value, startsWith('v1:'));

      final restored = _wallet(storage: storage);
      await restored.initialize();

      expect(restored.address, address);
      expect(storage.writeCalls, 1);
    });

    test('fails closed instead of replacing corrupt key material', () async {
      final storage = _MemoryWalletStorage('v1:not-base64');
      final wallet = _wallet(storage: storage);

      await expectLater(wallet.initialize(), throwsA(isA<FormatException>()));

      expect(wallet.address, isNull);
      expect(storage.writeCalls, 0);
      expect(storage.value, 'v1:not-base64');
    });

    test('signs messages with the embedded Ed25519 key', () async {
      final seed = Uint8List.fromList(List.generate(32, (index) => index + 1));
      final storage = _MemoryWalletStorage('v1:${base64Encode(seed)}');
      final wallet = _wallet(storage: storage);

      final encodedSignature = await wallet.signMessage('bitflip challenge');
      final keyPair = createKeyPairFromPrivateKeyBytes(seed);

      expect(
        verifySignature(
          keyPair.publicKey,
          signatureBytes(Uint8List.fromList(base64Decode(encodedSignature))),
          Uint8List.fromList(utf8.encode('bitflip challenge')),
        ),
        isTrue,
      );
      keyPair.dispose();
    });

    test('signs the compiled transaction before submitting it', () async {
      final seed = Uint8List.fromList(List.generate(32, (index) => index + 1));
      final keyPair = createKeyPairFromPrivateKeyBytes(seed);
      Transaction? submitted;
      final wallet = _wallet(
        storage: _MemoryWalletStorage('v1:${base64Encode(seed)}'),
        transactionSender: (wireTransaction) async {
          submitted = getTransactionDecoder().decode(
            Uint8List.fromList(base64Decode(wireTransaction)),
          );
          return 'submitted-signature';
        },
      );
      final transaction = compileTransaction(
        createTransactionMessage(version: TransactionVersion.v0)
            .withFeePayer(getAddressFromPublicKey(keyPair.publicKey))
            .withBlockhashLifetime(
              BlockhashLifetimeConstraint(
                blockhash: '11111111111111111111111111111111',
                lastValidBlockHeight: BigInt.one,
              ),
            ),
      );

      final result = await wallet.signAndSend(
        getBase64EncodedWireTransaction(transaction),
      );

      expect(result, 'submitted-signature');
      expect(submitted, isNotNull);
      expect(isFullySignedTransaction(submitted!), isTrue);
      keyPair.dispose();
    });

    test('funding targets the embedded address without replacing it', () async {
      final storage = _MemoryWalletStorage();
      Address? destination;
      BigInt? amount;
      final wallet = _wallet(
        storage: storage,
        mobileWalletFunder: (target, lamports) async {
          destination = target;
          amount = lamports;
          return 'fund-signature';
        },
      );

      final signature = await wallet.fundWithMobileWallet(
        BigInt.from(50000000),
      );

      expect(signature, 'fund-signature');
      expect(destination?.value, wallet.address);
      expect(amount, BigInt.from(50000000));
      expect(storage.writeCalls, 1);
    });

    test('decodes MWA base64 signatures into Solana base58 signatures', () {
      final signatureBytes = Uint8List.fromList(
        List.generate(64, (index) => index),
      );

      final decoded = decodeMobileWalletSignature(base64Encode(signatureBytes));

      expect(
        getBase58Encoder().encode(decoded.value),
        orderedEquals(signatureBytes),
      );
      expect(
        () => decodeMobileWalletSignature(base64Encode([1, 2, 3])),
        throwsStateError,
      );
    });

    test('desktop native platforms remain view-only', () async {
      final wallet = BitflipWallet(
        walletChain: 'solana:devnet',
        rpcUrl: 'https://api.devnet.solana.com',
        storage: _MemoryWalletStorage(),
        isMobile: false,
        isAndroid: false,
      );

      await wallet.initialize();

      expect(wallet.isSupported, isFalse);
      expect(wallet.address, isNull);
      await expectLater(wallet.connect(), throwsUnsupportedError);
    });
  });
}

BitflipWallet _wallet({
  required EmbeddedWalletStorage storage,
  SignedTransactionSender? transactionSender,
  MobileWalletFunder? mobileWalletFunder,
}) {
  return BitflipWallet(
    walletChain: 'solana:devnet',
    rpcUrl: 'https://api.devnet.solana.com',
    storage: storage,
    isMobile: true,
    isAndroid: true,
    transactionSender: transactionSender,
    mobileWalletFunder: mobileWalletFunder,
  );
}

final class _MemoryWalletStorage implements EmbeddedWalletStorage {
  _MemoryWalletStorage([this.value]);

  String? value;
  int writeCalls = 0;

  @override
  Future<String?> readPrivateKey() async => value;

  @override
  Future<void> writePrivateKey(String value) async {
    writeCalls++;
    this.value = value;
  }
}
