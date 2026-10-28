@TestOn('browser')
library;

import 'dart:convert';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';

import 'package:bitflip_app/core/bitflip_wallet_web.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/mint_authorization_fixture.dart';

@JS('bitflipWallet')
external set _bridge(JSObject? value);

void main() {
  const address = '11111111111111111111111111111111';
  const wallet = BitflipWallet(walletChain: 'solana:devnet', rpcUrl: 'unused');
  late List<String> signedMessages;

  setUp(() {
    signedMessages = [];
    final bridge = JSObject();
    bridge.setProperty('address'.toJS, (() => address.toJS).toJS);
    bridge.setProperty(
      'signMessage'.toJS,
      ((JSString message) {
        signedMessages.add(message.toDart);

        return Future.value('signature'.toJS).toJS;
      }).toJS,
    );
    _bridge = bridge;
  });

  tearDown(() => _bridge = null);

  test(
    'web forwards the same canonical UTF-8 vector as the native signer',
    () async {
      final fixture = mintAuthorizationFixture(address);

      expect(
        await wallet.signMintAuthorization(fixture.authorization),
        'signature',
      );
      expect(signedMessages, hasLength(1));
      expect(utf8.encode(signedMessages.single), utf8.encode(fixture.message));
    },
  );

  test(
    'web refuses wrong-wallet, expired, and distant-expiry statements',
    () async {
      for (final authorization in [
        mintAuthorizationFixture('11111111111111111111111111111112')
            .authorization,
        mintAuthorizationFixture(
          address,
          expiresAt: DateTime.now().subtract(const Duration(seconds: 1)),
        ).authorization,
        mintAuthorizationFixture(
          address,
          expiresAt: DateTime.now().add(const Duration(days: 1)),
        ).authorization,
      ]) {
        await expectLater(
          wallet.signMintAuthorization(authorization),
          throwsStateError,
        );
      }

      expect(signedMessages, isEmpty);
    },
  );
}
