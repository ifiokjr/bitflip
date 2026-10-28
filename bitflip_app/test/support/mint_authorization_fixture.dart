import 'package:bitflip_program/bitflip_mint_authorization.dart';

/// A common text vector for the real native signer and browser adapter tests.
({MintAuthorization authorization, String message}) mintAuthorizationFixture(
  String walletAddress, {
  DateTime? expiresAt,
}) {
  final expiry = (expiresAt ?? DateTime.now().add(const Duration(minutes: 4)))
      .toUtc();

  return (
    authorization: MintAuthorization(
      walletAddress: walletAddress,
      gameIndex: 0,
      sectionIndex: 12,
      nonce: 'abcdefghijklmnopqrstuvwxyz012345',
      expiresAt: expiry,
    ),
    message:
        'Bitflip compressed NFT authorization v1\n'
        'Domain: bitflip/mint-section\n'
        'Wallet: $walletAddress\n'
        'Game: 0\n'
        'Section: 12\n'
        'Nonce: abcdefghijklmnopqrstuvwxyz012345\n'
        'Expires: ${expiry.toIso8601String()}\n'
        'Action: Mint the sealed section to this wallet.',
  );
}
