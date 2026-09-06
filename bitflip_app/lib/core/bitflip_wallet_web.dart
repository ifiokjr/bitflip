import 'dart:convert';
import 'dart:js_interop';

import 'package:bitflip_app/core/bitflip_wallet_option.dart';

@JS('bitflipWallet')
external _WalletBridge? get _bitflipWalletBridge;

extension type _WalletBridge(JSObject _) implements JSObject {
  external JSString? address();

  external JSString listWallets(JSString chain);

  external JSPromise<JSString> connect(JSString walletId, JSString chain);

  external JSPromise<JSString> signAndSend(JSString wireTransaction);

  external JSPromise<JSString> signMessage(JSString message);
}

class BitflipWallet {
  const BitflipWallet({required this.walletChain, required String rpcUrl});

  final String walletChain;

  bool get isSupported => _bitflipWalletBridge != null;

  BitflipWalletKind get kind =>
      isSupported ? BitflipWalletKind.external : BitflipWalletKind.unavailable;

  bool get canFundWithMobileWallet => false;

  List<BitflipWalletOption>? get availableWallets {
    final bridge = _bitflipWalletBridge;
    if (bridge == null) return const [];
    final decoded = jsonDecode(bridge.listWallets(walletChain.toJS).toDart);
    if (decoded is! List<Object?>) {
      throw StateError('The wallet bridge returned invalid wallet options.');
    }
    return List.unmodifiable(decoded.map(_decodeWalletOption));
  }

  String? get address => _bitflipWalletBridge?.address()?.toDart;

  Future<void> initialize() async {}

  Future<String> connect([String? walletId]) async {
    final bridge = _requireBridge();
    final options = availableWallets ?? const [];
    final selectedId =
        walletId ?? (options.length == 1 ? options.single.id : null);
    if (selectedId == null ||
        !options.any((option) => option.id == selectedId)) {
      throw StateError('Choose an available Solana wallet before connecting.');
    }
    return (await bridge.connect(selectedId.toJS, walletChain.toJS).toDart)
        .toDart;
  }

  Future<String> signAndSend(String wireTransaction) async {
    return (await _requireBridge().signAndSend(wireTransaction.toJS).toDart)
        .toDart;
  }

  Future<String> signMessage(String message) async {
    return (await _requireBridge().signMessage(message.toJS).toDart).toDart;
  }

  Future<String> fundWithMobileWallet(BigInt lamports) {
    throw UnsupportedError('Mobile wallet funding is unavailable on the web.');
  }
}

BitflipWalletOption _decodeWalletOption(Object? value) {
  if (value is! Map<String, Object?>) {
    throw StateError('The wallet bridge returned an invalid wallet option.');
  }
  final id = value['id'];
  final name = value['name'];
  if (id is! String || id.isEmpty || name is! String || name.isEmpty) {
    throw StateError('The wallet bridge returned an invalid wallet option.');
  }
  return BitflipWalletOption(id: id, name: name);
}

_WalletBridge _requireBridge() {
  final bridge = _bitflipWalletBridge;
  if (bridge == null) {
    throw UnsupportedError(
      'Solana Wallet Standard is unavailable in this browser.',
    );
  }
  return bridge;
}
