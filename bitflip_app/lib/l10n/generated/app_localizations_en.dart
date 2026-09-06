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
      'One public sector starts the signal. As play grows, new 64 × 64 sectors unlock for players to own, trade, draw on, and eventually seal.';

  @override
  String get openCanvas => 'OPEN CANVAS';

  @override
  String get connectWallet => 'CONNECT WALLET';

  @override
  String get walletUnavailable =>
      'View only on this device. Mobile signing is available on Android and iOS; the web uses compatible browser wallets.';

  @override
  String get openWalletDetails => 'Open wallet details';

  @override
  String get embeddedWalletTitle => 'YOUR BITFLIP WALLET';

  @override
  String get externalWalletTitle => 'CONNECTED WALLET';

  @override
  String get walletBalance => 'AVAILABLE TO PLAY';

  @override
  String get walletAddress => 'WALLET ADDRESS';

  @override
  String get walletAddressCopied => 'Wallet address copied.';

  @override
  String get copyWalletAddress => 'COPY ADDRESS';

  @override
  String embeddedWalletBody(String network) {
    return 'This device-bound wallet signs Bitflip actions without opening another app. To add funds, send SOL on $network from any compatible wallet to the address above.';
  }

  @override
  String get externalWalletBody =>
      'Bitflip signs through this third-party Wallet Standard wallet. Bitflip never receives its private key.';

  @override
  String get embeddedWalletWarning =>
      'This is a small spending wallet, not a vault. Its key is encrypted by this device, but backup and export are not available yet. Reinstalling or losing the device can permanently lose access, so keep only enough SOL here to play.';

  @override
  String get fundingAmount => 'AMOUNT TO ADD';

  @override
  String get fundingAmountHelp =>
      'Your external wallet will show the transfer before approval.';

  @override
  String get fundWithMobileWallet => 'FUND WITH MOBILE WALLET';

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
  String get canvasLabel =>
      'Interactive 64 by 64 pixel sector. Use arrow keys to move the cursor and Space or Enter to toggle a pixel.';

  @override
  String get zoomOut => 'Zoom out';

  @override
  String get resetZoom => 'Reset zoom';

  @override
  String get zoomIn => 'Zoom in';

  @override
  String get pixelX => 'Pixel X';

  @override
  String get pixelY => 'Pixel Y';

  @override
  String togglePixel(int x, int y) {
    return 'TOGGLE PIXEL $x:$y';
  }

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
  String get selectSection => 'Select sector';

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
  String get moveReward => 'BIT REWARD';

  @override
  String get claimPrice => 'CLAIM PRICE';

  @override
  String feeValue(String sol) {
    return '$sol SOL';
  }

  @override
  String rewardValue(String bits) {
    return '$bits BIT';
  }

  @override
  String get rewardWindowUnavailable =>
      'This five-minute reward window cannot cover the full batch. Refresh after the next window.';

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
  String get sectorLocked => 'SECTOR NOT YET UNLOCKED';

  @override
  String get sectorReady => 'Unlocked and available to claim.';

  @override
  String sectorUnlockProgress(String current, int required, String time) {
    return 'Unlocks at $current / $required previous-sector flips, or after $time.';
  }

  @override
  String waitingForSector(String section) {
    return 'Sector $section must be claimed first.';
  }

  @override
  String get bitflipProgram => 'BITFLIP PROGRAM';

  @override
  String get forSale => 'LISTED FOR SALE';

  @override
  String get salePrice => 'SALE PRICE';

  @override
  String get salePriceHelp =>
      'The buyer pays you directly. Up to 9 decimal places.';

  @override
  String get invalidSalePrice =>
      'Enter a positive SOL amount with no more than 9 decimal places.';

  @override
  String get solUnit => 'SOL';

  @override
  String get listSection => 'LIST SECTOR FOR SALE';

  @override
  String get cancelListing => 'CANCEL LISTING';

  @override
  String get buySection => 'BUY THIS SECTOR';

  @override
  String get mintCompressedNft => 'MINT COMPRESSED NFT';

  @override
  String get sectionOwner => 'OWNER';

  @override
  String ownerFeeShare(String percent) {
    return 'EARNS $percent% OF FLIP FEES';
  }

  @override
  String withdrawOwnerFees(String amount) {
    return 'WITHDRAW $amount SOL IN OWNER FEES';
  }

  @override
  String get sectionMode => 'SECTION MODE';

  @override
  String get baseCanvasMode => 'BASE CANVAS';

  @override
  String get openCanvasMode => 'OPEN CANVAS';

  @override
  String get colourCanvasMode => '8-COLOUR CANVAS';

  @override
  String get noCampaign => 'No owner campaign is configured.';

  @override
  String campaignScheduledFor(String time) {
    return 'Scheduled for $time.';
  }

  @override
  String campaignLiveUntil(String time) {
    return 'Terms locked until $time.';
  }

  @override
  String get campaignEnded => 'This campaign has ended.';

  @override
  String get policyRewardsDisabled =>
      'Protocol-pool rewards remain disabled; this round changes mode only.';

  @override
  String get startOpenRound => 'START 24H OPEN';

  @override
  String get startColourRound => 'START 24H COLOUR';

  @override
  String confirmPolicyTitle(String mode) {
    return 'Start a 24-hour $mode round?';
  }

  @override
  String get confirmPolicyBody =>
      'The mode and rules digest are written on-chain. They cannot be changed while the round is live and remain attached if the section is sold.';

  @override
  String get confirmPolicyAction => 'START ROUND';

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
  String get activityFunded => 'Funds transferred to your Bitflip wallet.';

  @override
  String get activityClaimed => 'Sector claim confirmed on-chain.';

  @override
  String get activityListed => 'Sector listed for a fixed on-chain price.';

  @override
  String get activityListingCancelled => 'Sector listing cancelled.';

  @override
  String get activityPurchased =>
      'Sector purchased. Ownership transferred atomically.';

  @override
  String get activityOwnerFeesWithdrawn =>
      'Accrued owner fees withdrawn from this sector.';

  @override
  String get activityPolicyConfigured =>
      'Section policy published. Live terms are now locked on-chain.';

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
  String get walletIssue =>
      'The wallet could not complete that request. No transaction was reported as confirmed.';

  @override
  String get howItWorks => 'THREE MOVES. THAT’S THE GAME.';

  @override
  String get claimStep => '01 / CLAIM';

  @override
  String get claimStepBody =>
      'Bitflip funds one public starting sector. Each later buyer creates and fully funds the BIT vault for the next sector only after time or collective activity unlocks it.';

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
      'BIT has zero decimals. Dynamic fees and exact rewards are checked on-chain before any pixel changes.';

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

  @override
  String get privacyLink => 'PRIVACY';

  @override
  String get termsLink => 'TERMS';

  @override
  String get supportLink => 'SUPPORT';

  @override
  String get backToBitflip => 'BACK TO BITFLIP';

  @override
  String get legalEffectiveDate => 'EFFECTIVE 6 SEPTEMBER 2026';

  @override
  String get privacyTitle => 'Privacy';

  @override
  String get privacyIntroduction =>
      'Bitflip is built around a public blockchain. This notice explains what becomes public, what the service processes, and what never belongs in a support request.';

  @override
  String get privacyPublicTitle => 'Public blockchain data';

  @override
  String get privacyPublicBody =>
      'Your wallet address, claims, flips, fees, sealed artwork, mint records, and transaction signatures can be permanently visible on Solana. Bitflip cannot erase or make public chain history private.';

  @override
  String get privacyServiceTitle => 'Service data';

  @override
  String get privacyServiceBody =>
      'The service processes wallet-signed mint challenges and limited technical data such as request timing, error class, and network address for security, reliability, and abuse prevention. Operational data is retained only as needed for those purposes and protected with access controls.';

  @override
  String get privacyControlTitle => 'Your keys and choices';

  @override
  String get privacyControlBody =>
      'On mobile, Bitflip creates a device-bound private key and stores it using Android Keystore-backed encryption or iOS Keychain; it is not sent to the Bitflip service. On the web, signing stays in your selected third-party wallet. macOS remains view-only in this release.';

  @override
  String get privacyContactTitle => 'Providers and questions';

  @override
  String get privacyContactBody =>
      'Wallets, Solana RPC providers, app stores, and hosting providers process data under their own notices. Use the Support page to ask a privacy question without posting seed phrases, private keys, or other secrets.';

  @override
  String get termsTitle => 'Terms';

  @override
  String get termsIntroduction =>
      'By using Bitflip, you choose to interact with experimental software and a public blockchain. Verify the network, wallet, sector, pixels, and fee before approving an in-app action or external-wallet funding prompt.';

  @override
  String get termsActionsTitle => 'Fees and irreversible actions';

  @override
  String get termsActionsBody =>
      'Claims and flips can require program and network fees shown before signing. Confirmed blockchain transactions are not refundable or reversible. Sealing permanently freezes a sector and enables compressed-NFT minting.';

  @override
  String get termsWalletTitle => 'Wallet responsibility';

  @override
  String get termsWalletBody =>
      'The mobile Bitflip wallet is device-bound and has no backup or export yet, so keep only a small playing balance. Web wallet keys remain with the selected provider. You are responsible for device security, available funds, and every approval; Bitflip support will never ask for private key material.';

  @override
  String get termsArtworkTitle => 'Public artwork and metadata';

  @override
  String get termsArtworkBody =>
      'Only submit artwork you are entitled to publish. Pixels and mint records are public. The pixels are stored on chain, while the metadata and rendered image use an operated HTTPS service and should not be described as fully immutable.';

  @override
  String get termsRiskTitle => 'Experimental service';

  @override
  String get termsRiskBody =>
      'Bitflip is provided without a promise of uninterrupted availability, future value, or financial return. Wallets, RPC services, Solana, and marketplaces can fail or change independently. Use only funds you can afford to spend on the game.';

  @override
  String get supportTitle => 'Support';

  @override
  String get supportIntroduction =>
      'Start with the transaction signature and the status shown in Bitflip. Public chain evidence usually tells us whether a request was cancelled, failed, expired, or confirmed.';

  @override
  String get supportSafetyTitle => 'Keep secrets out of support';

  @override
  String get supportSafetyBody =>
      'Never send a seed phrase, private key, keystore, password, or wallet backup. Bitflip support does not need them. Anyone asking for one is not helping you safely.';

  @override
  String get supportTransactionTitle => 'Before retrying';

  @override
  String get supportTransactionBody =>
      'Check the signature in a Solana explorer on the same network. If its status is unknown, wait and refresh before signing again. Minting is safe to re-enter once chain state is known; do not submit duplicates while a transaction is indeterminate.';

  @override
  String get supportIncludeTitle => 'What to include';

  @override
  String get supportIncludeBody =>
      'Include the platform and version, wallet name, selected network, game and sector numbers, approximate UTC time, public transaction signature if one exists, and the exact error message. Remove personal information and all secrets.';

  @override
  String get openSupportRequest => 'OPEN SUPPORT REQUEST';
}
