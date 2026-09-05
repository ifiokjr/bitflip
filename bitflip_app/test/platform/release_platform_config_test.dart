import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android release can use the network and discover MWA wallets', () {
    final manifest = File('android/app/src/main/AndroidManifest.xml')
        .readAsStringSync();

    expect(manifest, contains('android.permission.INTERNET'));
    expect(manifest, contains('android.intent.action.VIEW'));
    expect(manifest, contains('android.intent.category.BROWSABLE'));
    expect(manifest, contains('android:scheme="solana-wallet"'));
  });

  test('macOS release permits outbound network connections', () {
    final entitlements = File('macos/Runner/Release.entitlements')
        .readAsStringSync();

    expect(entitlements, contains('com.apple.security.network.client'));
  });
}
