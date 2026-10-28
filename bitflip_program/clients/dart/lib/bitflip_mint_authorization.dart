import 'package:bitflip_program/bitflip_program_constraints.dart';

/// The only off-chain statement a Bitflip wallet may sign.
///
/// Shared by the server and both wallet adapters. The fixed domain, version,
/// and action prevent a server response from choosing arbitrary signing bytes.
/// Changing this format requires a protocol version.
final class MintAuthorization {
  factory MintAuthorization({
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
    required String nonce,
    required DateTime expiresAt,
  }) {
    if (walletAddress.length < 32 ||
        walletAddress.length > 44 ||
        _walletPattern.stringMatch(walletAddress) != walletAddress) {
      throw const FormatException('Invalid mint wallet address.');
    }

    RangeError.checkValueInInterval(
      gameIndex,
      0,
      bitflipMaximumGameIndex,
      'gameIndex',
    );
    RangeError.checkValueInInterval(
      sectionIndex,
      0,
      bitflipMaximumSectionIndex,
      'sectionIndex',
    );

    if (nonce.length != 32 || _noncePattern.stringMatch(nonce) != nonce) {
      throw const FormatException('Invalid mint authorization nonce.');
    }

    return MintAuthorization._(
      walletAddress,
      gameIndex,
      sectionIndex,
      nonce,
      expiresAt.toUtc(),
    );
  }

  const MintAuthorization._(
    this.walletAddress,
    this.gameIndex,
    this.sectionIndex,
    this.nonce,
    this.expiresAt,
  );

  /// The extra thirty seconds accepted by clients allow a small clock difference.
  static const lifetime = Duration(minutes: 5);
  static const _clockTolerance = Duration(seconds: 30);
  static final _noncePattern = RegExp(r'^[A-Za-z0-9_-]+$');
  static final _walletPattern = RegExp(r'^[1-9A-HJ-NP-Za-km-z]+$');

  final String walletAddress;
  final int gameIndex;
  final int sectionIndex;
  final String nonce;
  final DateTime expiresAt;

  /// Canonical UTF-8 text, with LF separators and no final newline.
  String get message => [
    'Bitflip compressed NFT authorization v1',
    'Domain: bitflip/mint-section',
    'Wallet: $walletAddress',
    'Game: $gameIndex',
    'Section: $sectionIndex',
    'Nonce: $nonce',
    'Expires: ${expiresAt.toIso8601String()}',
    'Action: Mint the sealed section to this wallet.',
  ].join('\n');

  /// Check the connected wallet and freshness immediately before signing.
  String messageForSigning({required String? connectedWallet, DateTime? now}) {
    final remaining = expiresAt.difference((now ?? DateTime.now()).toUtc());

    if (connectedWallet != walletAddress ||
        remaining <= Duration.zero ||
        remaining > lifetime + _clockTolerance) {
      throw StateError('The mint authorization is no longer valid.');
    }

    return message;
  }

  /// Compare server text with independently constructed intent before signing.
  void verifyChallenge(String serverMessage, {DateTime? now}) {
    final expected = messageForSigning(
      connectedWallet: walletAddress,
      now: now,
    );

    if (serverMessage != expected) {
      throw StateError('The server returned an unexpected mint authorization.');
    }
  }
}
