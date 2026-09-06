import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:bitflip_app/core/bitflip_wallet_option.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_mobile_wallet_adapter/solana_kit_mobile_wallet_adapter.dart';
import 'package:solana_kit_mobile_wallet_adapter_protocol/solana_kit_mobile_wallet_adapter_protocol.dart';
import 'package:solana_kit_rpc_spec/solana_kit_rpc_spec.dart';
import 'package:solana_kit_system/solana_kit_system.dart';

abstract interface class EmbeddedWalletStorage {
  Future<String?> readPrivateKey();

  Future<void> writePrivateKey(String value);
}

final class SecureEmbeddedWalletStorage implements EmbeddedWalletStorage {
  const SecureEmbeddedWalletStorage({FlutterSecureStorage? storage})
    : _storage =
          storage ??
          const FlutterSecureStorage(
            aOptions: AndroidOptions(
              resetOnError: false,
              migrateOnAlgorithmChange: true,
              sharedPreferencesName: 'bitflip_embedded_wallet',
              preferencesKeyPrefix: 'bitflip_',
            ),
            iOptions: IOSOptions(
              accountName: 'com.ifiokjr.bitflip.embedded-wallet',
              accessibility: KeychainAccessibility.unlocked_this_device,
              synchronizable: false,
            ),
          );

  static const _privateKeyName = 'ed25519-private-key-v1';
  final FlutterSecureStorage _storage;

  @override
  Future<String?> readPrivateKey() => _storage.read(key: _privateKeyName);

  @override
  Future<void> writePrivateKey(String value) =>
      _storage.write(key: _privateKeyName, value: value);
}

typedef SignedTransactionSender = Future<String> Function(String transaction);
typedef MobileWalletFunder = Future<String> Function(
  Address destination,
  BigInt lamports,
);

class BitflipWallet {
  BitflipWallet({
    required this.walletChain,
    required String rpcUrl,
    EmbeddedWalletStorage? storage,
    bool? isMobile,
    bool? isAndroid,
    Rpc? rpc,
    SignedTransactionSender? transactionSender,
    MobileWalletFunder? mobileWalletFunder,
  }) : _storage = storage ?? const SecureEmbeddedWalletStorage(),
       _isMobile = isMobile ?? (Platform.isAndroid || Platform.isIOS),
       _isAndroid = isAndroid ?? Platform.isAndroid,
       _rpc =
           rpc ??
           createSolanaRpc(
             url: rpcUrl,
             allowInsecureHttp: _isLoopback(rpcUrl),
           ) {
    _transactionSender = transactionSender ?? _sendSignedTransaction;
    _mobileWalletFunder = mobileWalletFunder ?? _fundFromExternalWallet;
  }

  final String walletChain;
  final EmbeddedWalletStorage _storage;
  final bool _isMobile;
  final bool _isAndroid;
  final Rpc _rpc;
  late final SignedTransactionSender _transactionSender;
  late final MobileWalletFunder _mobileWalletFunder;

  KeyPair? _keyPair;
  Future<void>? _initialization;

  bool get isSupported => _isMobile;

  BitflipWalletKind get kind =>
      _isMobile ? BitflipWalletKind.embedded : BitflipWalletKind.unavailable;

  bool get canFundWithMobileWallet => _isAndroid;

  List<BitflipWalletOption>? get availableWallets => null;

  String? get address {
    final keyPair = _keyPair;
    return keyPair == null
        ? null
        : getAddressFromPublicKey(keyPair.publicKey).value;
  }

  Future<void> initialize() {
    if (!_isMobile || _keyPair != null) return Future.value();
    return _initialization ??= _loadOrCreateKeyPair();
  }

  Future<String> connect([String? walletId]) async {
    if (!_isMobile) {
      throw UnsupportedError('Embedded signing is only available on mobile.');
    }
    await initialize();
    return address!;
  }

  Future<String> signAndSend(String wireTransaction) async {
    final keyPair = await _requireKeyPair();
    final transaction = getTransactionDecoder().decode(
      Uint8List.fromList(base64Decode(wireTransaction)),
    );
    final signed = await signTransaction([keyPair], transaction);
    return _transactionSender(getBase64EncodedWireTransaction(signed));
  }

  Future<String> signMessage(String message) async {
    final keyPair = await _requireKeyPair();
    final signature = signBytes(
      keyPair.privateKey,
      Uint8List.fromList(utf8.encode(message)),
    );
    return base64Encode(signature.value);
  }

  Future<String> fundWithMobileWallet(BigInt lamports) async {
    if (!_isAndroid) {
      throw UnsupportedError(
        'Mobile Wallet Adapter funding is only available on Android.',
      );
    }
    if (lamports <= BigInt.zero) {
      throw ArgumentError.value(lamports, 'lamports', 'Must be positive.');
    }
    await initialize();
    return _mobileWalletFunder(Address(address!), lamports);
  }

  Future<void> _loadOrCreateKeyPair() async {
    try {
      final stored = await _storage.readPrivateKey();
      if (stored != null) {
        _keyPair = createKeyPairFromPrivateKeyBytes(_decodePrivateKey(stored));
        return;
      }

      final generated = generateKeyPair();
      final encoded = 'v1:${base64Encode(generated.privateKey)}';
      try {
        await _storage.writePrivateKey(encoded);
      } on Object {
        generated.dispose();
        rethrow;
      }
      _keyPair = generated;
    } on Object {
      _initialization = null;
      rethrow;
    }
  }

  Future<KeyPair> _requireKeyPair() async {
    if (!_isMobile) {
      throw UnsupportedError('Embedded signing is only available on mobile.');
    }
    await initialize();
    return _keyPair!;
  }

  Future<String> _sendSignedTransaction(String transaction) =>
      _rpc.sendTransaction(transaction).send();

  Future<String> _fundFromExternalWallet(
    Address destination,
    BigInt lamports,
  ) async {
    if (!isMwaSupported() ||
        !await MwaClientHostApi().isWalletEndpointAvailable()) {
      throw StateError('No compatible mobile wallet is available.');
    }

    return transact((wallet) async {
      final authorization = await wallet.authorize(
        identity: AppIdentity(
          uri: Uri.parse('https://bitflip.xyz'),
          name: 'Bitflip',
        ),
        chain: walletChain,
        features: const ['solana:signAndSendTransactions'],
      );
      if (authorization.accounts.isEmpty) {
        throw StateError('The funding wallet did not authorize an account.');
      }
      final source = Address(
        _decodeMobileWalletAddress(authorization.accounts.first.address),
      );
      final latest = await _rpc.getLatestBlockhashValue().send();
      final transaction = compileTransaction(
        createTransactionMessage(version: TransactionVersion.v0)
            .withFeePayer(source)
            .withBlockhashLifetime(
              BlockhashLifetimeConstraint(
                blockhash: latest.value.blockhash.value,
                lastValidBlockHeight: latest.value.lastValidBlockHeight,
              ),
            )
            .appendInstruction(
              getTransferSolInstruction(
                programAddress: systemProgramAddress,
                source: source,
                destination: destination,
                amount: lamports,
              ),
            ),
      );
      final signatures = await wallet.signAndSendTransactions(
        payloads: [getBase64EncodedWireTransaction(transaction)],
      );
      if (signatures.length != 1) {
        throw StateError('The funding wallet returned an invalid response.');
      }
      final transactionSignature = decodeMobileWalletSignature(
        signatures.single,
      );
      await waitForTransactionConfirmation(
        rpc: _rpc,
        signature: transactionSignature,
        transaction: transaction,
        config: const RpcTransactionConfirmationConfig(
          commitment: Commitment.confirmed,
          pollInterval: Duration(milliseconds: 400),
        ),
      );
      return transactionSignature.value;
    });
  }
}

Uint8List _decodePrivateKey(String value) {
  if (!value.startsWith('v1:')) {
    throw const FormatException('Unsupported embedded wallet key version.');
  }
  final bytes = base64Decode(value.substring(3));
  if (bytes.length != 32) {
    throw const FormatException('Invalid embedded wallet private key.');
  }
  return Uint8List.fromList(bytes);
}

String _decodeMobileWalletAddress(String encodedAddress) {
  final bytes = base64Decode(encodedAddress);
  if (bytes.length != 32) {
    throw StateError('The mobile wallet returned an invalid Solana address.');
  }
  return getBase58Decoder().decode(Uint8List.fromList(bytes));
}

Signature decodeMobileWalletSignature(String encodedSignature) {
  final bytes = base64Decode(encodedSignature);
  if (bytes.length != 64) {
    throw StateError(
      'The mobile wallet returned an invalid transaction signature.',
    );
  }
  return signature(getBase58Decoder().decode(Uint8List.fromList(bytes)));
}

bool _isLoopback(String value) {
  final host = Uri.tryParse(value)?.host.toLowerCase();
  return host == '127.0.0.1' || host == 'localhost' || host == '::1';
}
