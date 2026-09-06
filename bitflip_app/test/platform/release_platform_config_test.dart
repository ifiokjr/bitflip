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
    expect(manifest, contains('android:allowBackup="false"'));
  });

  test('iOS release grants the app access to its Keychain group', () {
    final entitlements = File('ios/Runner/Runner.entitlements')
        .readAsStringSync();
    final project = File('ios/Runner.xcodeproj/project.pbxproj')
        .readAsStringSync();

    expect(entitlements, contains('keychain-access-groups'));
    expect(
      project,
      contains('CODE_SIGN_ENTITLEMENTS = Runner/Runner.entitlements'),
    );
  });

  test('macOS release permits outbound network connections', () {
    final entitlements = File('macos/Runner/Release.entitlements')
        .readAsStringSync();

    expect(entitlements, contains('com.apple.security.network.client'));
  });
}
