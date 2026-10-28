import 'package:bitflip_program/bitflip_mint_authorization.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/mint_authorization_fixture.dart';

void main() {
  const wallet = '11111111111111111111111111111111';
  final now = DateTime.utc(2026, 10, 28, 12);

  test('expiry bounds are exclusive at now and include clock tolerance', () {
    for (final offset in [
      Duration.zero,
      const Duration(microseconds: -1),
      const Duration(minutes: 5, seconds: 30, microseconds: 1),
    ]) {
      final fixture = mintAuthorizationFixture(
        wallet,
        expiresAt: now.add(offset),
      );
      expect(
        () => fixture.authorization.verifyChallenge(fixture.message, now: now),
        throwsStateError,
      );
    }

    final valid = mintAuthorizationFixture(
      wallet,
      expiresAt: now.add(const Duration(minutes: 5, seconds: 30)),
    );
    expect(
      () => valid.authorization.verifyChallenge(valid.message, now: now),
      returnsNormally,
    );
  });

  test('field validation rejects out-of-range indices and text injection', () {
    for (final (address, game, section) in [
      ('$wallet\n', 0, 0),
      ('invalid-wallet', 0, 0),
      (wallet, -1, 0),
      (wallet, 4, 0),
      (wallet, 0, -1),
      (wallet, 0, 256),
    ]) {
      expect(
        () => MintAuthorization(
          walletAddress: address,
          gameIndex: game,
          sectionIndex: section,
          nonce: 'abcdefghijklmnopqrstuvwxyz012345',
          expiresAt: now,
        ),
        throwsA(anyOf(isA<FormatException>(), isA<RangeError>())),
      );
    }
  });

  test('UTC normalization gives one canonical expiry representation', () {
    final fixture = mintAuthorizationFixture(
      wallet,
      expiresAt: DateTime.parse('2026-10-28T13:04:00+01:00'),
    );

    expect(
      fixture.authorization.message,
      contains('Expires: 2026-10-28T12:04:00.000Z'),
    );
    expect(fixture.authorization.message, fixture.message);
  });
}
