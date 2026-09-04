import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_app/core/bitflip_wallet_option.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_mobile_wallet_adapter/solana_kit_mobile_wallet_adapter.dart';
import 'package:solana_kit_mobile_wallet_adapter_protocol/solana_kit_mobile_wallet_adapter_protocol.dart';

const _walletChain = String.fromEnvironment(
  'SOLANA_WALLET_CHAIN',
  defaultValue: 'solana:devnet',
);

class BitflipWallet {
  BitflipWallet();

  AuthorizationResult? _authorization;

  bool get isSupported => isMwaSupported();

  List<BitflipWalletOption>? get availableWallets => null;

  String? get address {
    final account = _authorization?.accounts.firstOrNull;
    return account == null ? null : _decodeAddress(account.address);
  }

  Future<String> connect([String? walletId]) async {
    if (!isSupported || !await MwaClientHostApi().isWalletEndpointAvailable()) {
      throw StateError('No compatible mobile wallet is available.');
    }
    final authorization = await transact(
      (wallet) => wallet.authorize(
        identity: AppIdentity(
          uri: Uri.parse('https://bitflip.xyz'),
          name: 'Bitflip',
        ),
        chain: _walletChain,
        features: const [
          'solana:signAndSendTransactions',
          'solana:signMessages',
        ],
      ),
    );
    if (authorization.accounts.isEmpty) {
      throw StateError('The wallet did not authorize an account.');
    }
    _authorization = authorization;
    return _decodeAddress(authorization.accounts.first.address);
  }

  Future<String> signAndSend(String wireTransaction) async {
    final authorization = _authorization;
    if (authorization == null) {
      throw StateError('Connect a wallet before signing.');
    }
    final result = await transact((wallet) async {
      final refreshed = await wallet.reauthorize(
        authToken: authorization.authToken,
        identity: AppIdentity(
          uri: Uri.parse('https://bitflip.xyz'),
          name: 'Bitflip',
        ),
      );
      final signatures = await wallet.signAndSendTransactions(
        payloads: [wireTransaction],
      );
      return (refreshed, signatures);
    });
    final (refreshed, signatures) = result;
    if (signatures.length != 1) {
      throw StateError('The wallet returned an invalid transaction response.');
    }
    _authorization = refreshed;
    return signatures.single;
  }

  Future<String> signMessage(String message) async {
    final authorization = _authorization;
    if (authorization == null) {
      throw StateError('Connect a wallet before signing.');
    }
    final result = await transact((wallet) async {
      final refreshed = await wallet.reauthorize(
        authToken: authorization.authToken,
        identity: AppIdentity(
          uri: Uri.parse('https://bitflip.xyz'),
          name: 'Bitflip',
        ),
      );
      if (refreshed.accounts.isEmpty) {
        throw StateError('The wallet returned no authorized account.');
      }
      final signedPayloads = await wallet.signMessages(
        addresses: [refreshed.accounts.first.address],
        payloads: [base64Encode(utf8.encode(message))],
      );
      return (refreshed, signedPayloads);
    });
    final (refreshed, signedPayloads) = result;
    if (signedPayloads.length != 1) {
      throw StateError('The wallet returned an invalid message response.');
    }
    final signedPayload = base64Decode(signedPayloads.single);
    final messageBytes = Uint8List.fromList(utf8.encode(message));
    final signature = switch (signedPayload.length) {
      64 => signedPayload,
      _ when signedPayload.length == messageBytes.length + 64 => () {
        final returnedMessage = signedPayload.sublist(0, messageBytes.length);
        if (!_constantTimeEqual(returnedMessage, messageBytes)) {
          throw StateError('The wallet signed a different message.');
        }
        return signedPayload.sublist(messageBytes.length);
      }(),
      _ => throw StateError(
        'The wallet returned an invalid message signature.',
      ),
    };
    _authorization = refreshed;
    return base64Encode(signature);
  }
}

String _decodeAddress(String encodedAddress) {
  final bytes = base64Decode(encodedAddress);
  if (bytes.length != 32) {
    throw StateError('The wallet returned an invalid Solana address.');
  }
  return getBase58Decoder().decode(Uint8List.fromList(bytes));
}

bool _constantTimeEqual(List<int> left, List<int> right) {
  if (left.length != right.length) return false;
  var difference = 0;
  for (var index = 0; index < left.length; index++) {
    difference |= left[index] ^ right[index];
  }
  return difference == 0;
}
