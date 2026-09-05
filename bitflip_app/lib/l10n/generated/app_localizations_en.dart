// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'BITFLIP';

  @override
  String get liveNetwork => 'LIVE // SOLANA';

  @override
  String get heroTitle => 'A million pixels. One irreversible canvas.';

  @override
  String get heroBody =>
      'Claim a 64 × 64 sector, draw in public, then seal the final state into a compressed NFT. Every mark is a tiny on-chain decision.';

  @override
  String get openCanvas => 'OPEN CANVAS';

  @override
  String get connectWallet => 'CONNECT WALLET';

  @override
  String get walletUnavailable =>
      'View only on this device. Signing is currently supported on Android and the web.';

  @override
  String get chooseWallet => 'Choose a wallet';

  @override
  String get chooseWalletBody =>
      'Bitflip only shows wallets that support this Solana network, versioned transactions, and message signing.';

  @override
  String get noWalletTitle => 'No compatible wallet found';

  @override
  String get noWalletBody =>
      'Install a Solana Wallet Standard browser wallet, then reload Bitflip to claim, flip, seal, and mint.';

  @override
  String get close => 'CLOSE';

  @override
  String get demoMode => 'SIGNAL MIRROR';

  @override
  String get chainMode => 'ON-CHAIN';

  @override
  String get demoNotice =>
      'Showing a local signal while this deployment has no live game data.';

  @override
  String get gameLoading => 'Loading live on-chain canvas state…';

  @override
  String get gameUnavailable =>
      'This game has not been initialized on the configured network.';

  @override
  String get gameOffline =>
      'Live chain state is unavailable. No demo data has been substituted; retry when the connection returns.';

  @override
  String get canvasLabel => 'Interactive 64 by 64 pixel sector';

  @override
  String sectionLabel(String section) {
    return 'SECTOR $section';
  }

  @override
  String sectionPosition(int current, int total) {
    return '$current / $total';
  }

  @override
  String get previousSection => 'PREVIOUS SECTOR';

  @override
  String get nextSection => 'NEXT SECTOR';

  @override
  String get overview => 'THE WHOLE SIGNAL';

  @override
  String get overviewBody =>
      '256 sectors unlock in sequence. Pick one to inspect its pulse.';

  @override
  String get console => 'MOVE CONSOLE';

  @override
  String get active => 'ACTIVE';

  @override
  String get sealed => 'SEALED';

  @override
  String get minted => 'MINTED';

  @override
  String get unclaimed => 'UNCLAIMED';

  @override
  String get selectedPixel => 'CURSOR';

  @override
  String get noPixelSelected => '— / —';

  @override
  String get queuedMoves => 'QUEUED MOVES';

  @override
  String moveCount(int count) {
    return '$count / 16';
  }

  @override
  String get onPixels => 'LIT PIXELS';

  @override
  String get network => 'NETWORK';

  @override
  String get revision => 'REVISION';

  @override
  String pixelCount(int count) {
    return '$count / 4096';
  }

  @override
  String get moveFee => 'MOVE FEE';

  @override
  String get claimPrice => 'CLAIM PRICE';

  @override
  String feeValue(String sol) {
    return '$sol SOL';
  }

  @override
  String commitMoves(int count) {
    return 'COMMIT $count MOVES';
  }

  @override
  String get clearQueue => 'CLEAR QUEUE';

  @override
  String get sealArtwork => 'SEAL ARTWORK';

  @override
  String get confirmSealTitle => 'Seal this artwork permanently?';

  @override
  String get confirmSealBody =>
      'Sealing cannot be undone. Confirm the sector, network, wallet, and final pixels before continuing.';

  @override
  String get confirmSealAction => 'SEAL PERMANENTLY';

  @override
  String get cancel => 'CANCEL';

  @override
  String get claimSector => 'CLAIM THIS SECTOR';

  @override
  String get mintCompressedNft => 'MINT COMPRESSED NFT';

  @override
  String get sectionOwner => 'CREATOR';

  @override
  String get anonymousOwner => 'NOT CLAIMED';

  @override
  String get activity => 'LIVE PULSE';

  @override
  String get activityReady =>
      'Canvas synchronized. Tap cells to compose your next transaction.';

  @override
  String activityQueued(int x, int y) {
    return 'Move queued at $x:$y.';
  }

  @override
  String get activityCommitted => 'Moves committed as one atomic transaction.';

  @override
  String activitySectionChanged(String section) {
    return 'Receiver tuned to sector $section.';
  }

  @override
  String get activityConnected =>
      'Wallet connected. You control the signing boundary.';

  @override
  String get activityClaimed => 'Sector claim confirmed on-chain.';

  @override
  String get activitySealed =>
      'Artwork sealed. Its pixels can never change again.';

  @override
  String get activityMinted =>
      'Compressed NFT minted. The asset and Bitflip receipt landed atomically.';

  @override
  String get connectionIssue =>
      'The chain signal dropped. Your queued moves are still safe locally.';

  @override
  String get viewTransaction => 'VIEW CONFIRMED TRANSACTION';

  @override
  String get viewAsset => 'VIEW MINTED ASSET';

  @override
  String get batchFull => 'A transaction can contain at most 16 unique moves.';

  @override
  String get howItWorks => 'THREE MOVES. THAT’S THE GAME.';

  @override
  String get claimStep => '01 / CLAIM';

  @override
  String get claimStepBody =>
      'Sectors open from top-left to bottom-right—by time or by collective activity.';

  @override
  String get flipStep => '02 / FLIP';

  @override
  String get flipStepBody =>
      'Tap up to 16 cells. They toggle atomically, with an exact fee you approve before signing.';

  @override
  String get sealStep => '03 / SEAL';

  @override
  String get sealStepBody =>
      'Only the sector creator can freeze the artwork. Sealing is permanent and unlocks minting.';

  @override
  String get builtWith => 'BUILT WITH PINA · FLUTTER · BUBBLEGUM V1';

  @override
  String get securityNote =>
      'No reward faucet. No no-op payout. Fixed fees are checked on-chain.';

  @override
  String get status => 'STATUS';

  @override
  String get gameStats => 'GAME TELEMETRY';

  @override
  String claimedValue(int count) {
    return '$count claimed';
  }

  @override
  String flipsValue(String count) {
    return '$count flips';
  }

  @override
  String mintedValue(int count) {
    return '$count minted';
  }

  @override
  String get refresh => 'REFRESH';

  @override
  String get refreshing => 'SYNCING…';

  @override
  String get learnMore => 'READ THE PROTOCOL';

  @override
  String get dismiss => 'DISMISS';
}
