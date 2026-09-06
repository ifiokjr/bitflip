import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'generated/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('en')];

  /// No description provided for @appName.
  ///
  /// In en, this message translates to:
  /// **'BITFLIP'**
  String get appName;

  /// No description provided for @liveNetwork.
  ///
  /// In en, this message translates to:
  /// **'LIVE // SOLANA'**
  String get liveNetwork;

  /// No description provided for @heroTitle.
  ///
  /// In en, this message translates to:
  /// **'A million pixels. One irreversible canvas.'**
  String get heroTitle;

  /// No description provided for @heroBody.
  ///
  /// In en, this message translates to:
  /// **'One public sector starts the signal. As play grows, new 64 × 64 sectors unlock for players to own, trade, draw on, and eventually seal.'**
  String get heroBody;

  /// No description provided for @openCanvas.
  ///
  /// In en, this message translates to:
  /// **'OPEN CANVAS'**
  String get openCanvas;

  /// No description provided for @connectWallet.
  ///
  /// In en, this message translates to:
  /// **'CONNECT WALLET'**
  String get connectWallet;

  /// No description provided for @walletUnavailable.
  ///
  /// In en, this message translates to:
  /// **'View only on this device. Mobile signing is available on Android and iOS; the web uses compatible browser wallets.'**
  String get walletUnavailable;

  /// No description provided for @openWalletDetails.
  ///
  /// In en, this message translates to:
  /// **'Open wallet details'**
  String get openWalletDetails;

  /// No description provided for @embeddedWalletTitle.
  ///
  /// In en, this message translates to:
  /// **'YOUR BITFLIP WALLET'**
  String get embeddedWalletTitle;

  /// No description provided for @externalWalletTitle.
  ///
  /// In en, this message translates to:
  /// **'CONNECTED WALLET'**
  String get externalWalletTitle;

  /// No description provided for @walletBalance.
  ///
  /// In en, this message translates to:
  /// **'AVAILABLE TO PLAY'**
  String get walletBalance;

  /// No description provided for @walletAddress.
  ///
  /// In en, this message translates to:
  /// **'WALLET ADDRESS'**
  String get walletAddress;

  /// No description provided for @walletAddressCopied.
  ///
  /// In en, this message translates to:
  /// **'Wallet address copied.'**
  String get walletAddressCopied;

  /// No description provided for @copyWalletAddress.
  ///
  /// In en, this message translates to:
  /// **'COPY ADDRESS'**
  String get copyWalletAddress;

  /// No description provided for @embeddedWalletBody.
  ///
  /// In en, this message translates to:
  /// **'This device-bound wallet signs Bitflip actions without opening another app. To add funds, send SOL on {network} from any compatible wallet to the address above.'**
  String embeddedWalletBody(String network);

  /// No description provided for @externalWalletBody.
  ///
  /// In en, this message translates to:
  /// **'Bitflip signs through this third-party Wallet Standard wallet. Bitflip never receives its private key.'**
  String get externalWalletBody;

  /// No description provided for @embeddedWalletWarning.
  ///
  /// In en, this message translates to:
  /// **'This is a small spending wallet, not a vault. Its key is encrypted by this device, but backup and export are not available yet. Reinstalling or losing the device can permanently lose access, so keep only enough SOL here to play.'**
  String get embeddedWalletWarning;

  /// No description provided for @fundingAmount.
  ///
  /// In en, this message translates to:
  /// **'AMOUNT TO ADD'**
  String get fundingAmount;

  /// No description provided for @fundingAmountHelp.
  ///
  /// In en, this message translates to:
  /// **'Your external wallet will show the transfer before approval.'**
  String get fundingAmountHelp;

  /// No description provided for @fundWithMobileWallet.
  ///
  /// In en, this message translates to:
  /// **'FUND WITH MOBILE WALLET'**
  String get fundWithMobileWallet;

  /// No description provided for @chooseWallet.
  ///
  /// In en, this message translates to:
  /// **'Choose a wallet'**
  String get chooseWallet;

  /// No description provided for @chooseWalletBody.
  ///
  /// In en, this message translates to:
  /// **'Bitflip only shows wallets that support this Solana network, versioned transactions, and message signing.'**
  String get chooseWalletBody;

  /// No description provided for @noWalletTitle.
  ///
  /// In en, this message translates to:
  /// **'No compatible wallet found'**
  String get noWalletTitle;

  /// No description provided for @noWalletBody.
  ///
  /// In en, this message translates to:
  /// **'Install a Solana Wallet Standard browser wallet, then reload Bitflip to claim, flip, seal, and mint.'**
  String get noWalletBody;

  /// No description provided for @close.
  ///
  /// In en, this message translates to:
  /// **'CLOSE'**
  String get close;

  /// No description provided for @demoMode.
  ///
  /// In en, this message translates to:
  /// **'SIGNAL MIRROR'**
  String get demoMode;

  /// No description provided for @chainMode.
  ///
  /// In en, this message translates to:
  /// **'ON-CHAIN'**
  String get chainMode;

  /// No description provided for @demoNotice.
  ///
  /// In en, this message translates to:
  /// **'Showing a local signal while this deployment has no live game data.'**
  String get demoNotice;

  /// No description provided for @gameLoading.
  ///
  /// In en, this message translates to:
  /// **'Loading live on-chain canvas state…'**
  String get gameLoading;

  /// No description provided for @gameUnavailable.
  ///
  /// In en, this message translates to:
  /// **'This game has not been initialized on the configured network.'**
  String get gameUnavailable;

  /// No description provided for @gameOffline.
  ///
  /// In en, this message translates to:
  /// **'Live chain state is unavailable. No demo data has been substituted; retry when the connection returns.'**
  String get gameOffline;

  /// No description provided for @canvasLabel.
  ///
  /// In en, this message translates to:
  /// **'Interactive 64 by 64 pixel sector. Use arrow keys to move the cursor and Space or Enter to toggle a pixel.'**
  String get canvasLabel;

  /// No description provided for @zoomOut.
  ///
  /// In en, this message translates to:
  /// **'Zoom out'**
  String get zoomOut;

  /// No description provided for @resetZoom.
  ///
  /// In en, this message translates to:
  /// **'Reset zoom'**
  String get resetZoom;

  /// No description provided for @zoomIn.
  ///
  /// In en, this message translates to:
  /// **'Zoom in'**
  String get zoomIn;

  /// No description provided for @pixelX.
  ///
  /// In en, this message translates to:
  /// **'Pixel X'**
  String get pixelX;

  /// No description provided for @pixelY.
  ///
  /// In en, this message translates to:
  /// **'Pixel Y'**
  String get pixelY;

  /// No description provided for @togglePixel.
  ///
  /// In en, this message translates to:
  /// **'TOGGLE PIXEL {x}:{y}'**
  String togglePixel(int x, int y);

  /// No description provided for @sectionLabel.
  ///
  /// In en, this message translates to:
  /// **'SECTOR {section}'**
  String sectionLabel(String section);

  /// No description provided for @sectionPosition.
  ///
  /// In en, this message translates to:
  /// **'{current} / {total}'**
  String sectionPosition(int current, int total);

  /// No description provided for @previousSection.
  ///
  /// In en, this message translates to:
  /// **'PREVIOUS SECTOR'**
  String get previousSection;

  /// No description provided for @nextSection.
  ///
  /// In en, this message translates to:
  /// **'NEXT SECTOR'**
  String get nextSection;

  /// No description provided for @overview.
  ///
  /// In en, this message translates to:
  /// **'THE WHOLE SIGNAL'**
  String get overview;

  /// No description provided for @overviewBody.
  ///
  /// In en, this message translates to:
  /// **'256 sectors unlock in sequence. Pick one to inspect its pulse.'**
  String get overviewBody;

  /// No description provided for @selectSection.
  ///
  /// In en, this message translates to:
  /// **'Select sector'**
  String get selectSection;

  /// No description provided for @console.
  ///
  /// In en, this message translates to:
  /// **'MOVE CONSOLE'**
  String get console;

  /// No description provided for @active.
  ///
  /// In en, this message translates to:
  /// **'ACTIVE'**
  String get active;

  /// No description provided for @sealed.
  ///
  /// In en, this message translates to:
  /// **'SEALED'**
  String get sealed;

  /// No description provided for @minted.
  ///
  /// In en, this message translates to:
  /// **'MINTED'**
  String get minted;

  /// No description provided for @unclaimed.
  ///
  /// In en, this message translates to:
  /// **'UNCLAIMED'**
  String get unclaimed;

  /// No description provided for @selectedPixel.
  ///
  /// In en, this message translates to:
  /// **'CURSOR'**
  String get selectedPixel;

  /// No description provided for @noPixelSelected.
  ///
  /// In en, this message translates to:
  /// **'— / —'**
  String get noPixelSelected;

  /// No description provided for @queuedMoves.
  ///
  /// In en, this message translates to:
  /// **'QUEUED MOVES'**
  String get queuedMoves;

  /// No description provided for @moveCount.
  ///
  /// In en, this message translates to:
  /// **'{count} / 16'**
  String moveCount(int count);

  /// No description provided for @onPixels.
  ///
  /// In en, this message translates to:
  /// **'LIT PIXELS'**
  String get onPixels;

  /// No description provided for @network.
  ///
  /// In en, this message translates to:
  /// **'NETWORK'**
  String get network;

  /// No description provided for @revision.
  ///
  /// In en, this message translates to:
  /// **'REVISION'**
  String get revision;

  /// No description provided for @pixelCount.
  ///
  /// In en, this message translates to:
  /// **'{count} / 4096'**
  String pixelCount(int count);

  /// No description provided for @moveFee.
  ///
  /// In en, this message translates to:
  /// **'MOVE FEE'**
  String get moveFee;

  /// No description provided for @moveReward.
  ///
  /// In en, this message translates to:
  /// **'BIT REWARD'**
  String get moveReward;

  /// No description provided for @claimPrice.
  ///
  /// In en, this message translates to:
  /// **'CLAIM PRICE'**
  String get claimPrice;

  /// No description provided for @feeValue.
  ///
  /// In en, this message translates to:
  /// **'{sol} SOL'**
  String feeValue(String sol);

  /// No description provided for @rewardValue.
  ///
  /// In en, this message translates to:
  /// **'{bits} BIT'**
  String rewardValue(String bits);

  /// No description provided for @rewardWindowUnavailable.
  ///
  /// In en, this message translates to:
  /// **'This five-minute reward window cannot cover the full batch. Refresh after the next window.'**
  String get rewardWindowUnavailable;

  /// No description provided for @commitMoves.
  ///
  /// In en, this message translates to:
  /// **'COMMIT {count} MOVES'**
  String commitMoves(int count);

  /// No description provided for @clearQueue.
  ///
  /// In en, this message translates to:
  /// **'CLEAR QUEUE'**
  String get clearQueue;

  /// No description provided for @sealArtwork.
  ///
  /// In en, this message translates to:
  /// **'SEAL ARTWORK'**
  String get sealArtwork;

  /// No description provided for @confirmSealTitle.
  ///
  /// In en, this message translates to:
  /// **'Seal this artwork permanently?'**
  String get confirmSealTitle;

  /// No description provided for @confirmSealBody.
  ///
  /// In en, this message translates to:
  /// **'Sealing cannot be undone. Confirm the sector, network, wallet, and final pixels before continuing.'**
  String get confirmSealBody;

  /// No description provided for @confirmSealAction.
  ///
  /// In en, this message translates to:
  /// **'SEAL PERMANENTLY'**
  String get confirmSealAction;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'CANCEL'**
  String get cancel;

  /// No description provided for @claimSector.
  ///
  /// In en, this message translates to:
  /// **'CLAIM THIS SECTOR'**
  String get claimSector;

  /// No description provided for @sectorLocked.
  ///
  /// In en, this message translates to:
  /// **'SECTOR NOT YET UNLOCKED'**
  String get sectorLocked;

  /// No description provided for @sectorReady.
  ///
  /// In en, this message translates to:
  /// **'Unlocked and available to claim.'**
  String get sectorReady;

  /// No description provided for @sectorUnlockProgress.
  ///
  /// In en, this message translates to:
  /// **'Unlocks at {current} / {required} previous-sector flips, or after {time}.'**
  String sectorUnlockProgress(String current, int required, String time);

  /// No description provided for @waitingForSector.
  ///
  /// In en, this message translates to:
  /// **'Sector {section} must be claimed first.'**
  String waitingForSector(String section);

  /// No description provided for @bitflipProgram.
  ///
  /// In en, this message translates to:
  /// **'BITFLIP PROGRAM'**
  String get bitflipProgram;

  /// No description provided for @forSale.
  ///
  /// In en, this message translates to:
  /// **'LISTED FOR SALE'**
  String get forSale;

  /// No description provided for @salePrice.
  ///
  /// In en, this message translates to:
  /// **'SALE PRICE'**
  String get salePrice;

  /// No description provided for @salePriceHelp.
  ///
  /// In en, this message translates to:
  /// **'The buyer pays you directly. Up to 9 decimal places.'**
  String get salePriceHelp;

  /// No description provided for @invalidSalePrice.
  ///
  /// In en, this message translates to:
  /// **'Enter a positive SOL amount with no more than 9 decimal places.'**
  String get invalidSalePrice;

  /// No description provided for @solUnit.
  ///
  /// In en, this message translates to:
  /// **'SOL'**
  String get solUnit;

  /// No description provided for @listSection.
  ///
  /// In en, this message translates to:
  /// **'LIST SECTOR FOR SALE'**
  String get listSection;

  /// No description provided for @cancelListing.
  ///
  /// In en, this message translates to:
  /// **'CANCEL LISTING'**
  String get cancelListing;

  /// No description provided for @buySection.
  ///
  /// In en, this message translates to:
  /// **'BUY THIS SECTOR'**
  String get buySection;

  /// No description provided for @mintCompressedNft.
  ///
  /// In en, this message translates to:
  /// **'MINT COMPRESSED NFT'**
  String get mintCompressedNft;

  /// No description provided for @sectionOwner.
  ///
  /// In en, this message translates to:
  /// **'OWNER'**
  String get sectionOwner;

  /// No description provided for @ownerFeeShare.
  ///
  /// In en, this message translates to:
  /// **'EARNS {percent}% OF FLIP FEES'**
  String ownerFeeShare(String percent);

  /// No description provided for @withdrawOwnerFees.
  ///
  /// In en, this message translates to:
  /// **'WITHDRAW {amount} SOL IN OWNER FEES'**
  String withdrawOwnerFees(String amount);

  /// No description provided for @sectionMode.
  ///
  /// In en, this message translates to:
  /// **'SECTION MODE'**
  String get sectionMode;

  /// No description provided for @baseCanvasMode.
  ///
  /// In en, this message translates to:
  /// **'BASE CANVAS'**
  String get baseCanvasMode;

  /// No description provided for @openCanvasMode.
  ///
  /// In en, this message translates to:
  /// **'OPEN CANVAS'**
  String get openCanvasMode;

  /// No description provided for @colourCanvasMode.
  ///
  /// In en, this message translates to:
  /// **'8-COLOUR CANVAS'**
  String get colourCanvasMode;

  /// No description provided for @chooseColour.
  ///
  /// In en, this message translates to:
  /// **'PAINT COLOUR'**
  String get chooseColour;

  /// No description provided for @noCampaign.
  ///
  /// In en, this message translates to:
  /// **'No owner campaign is configured.'**
  String get noCampaign;

  /// No description provided for @campaignScheduledFor.
  ///
  /// In en, this message translates to:
  /// **'Scheduled for {time}.'**
  String campaignScheduledFor(String time);

  /// No description provided for @campaignLiveUntil.
  ///
  /// In en, this message translates to:
  /// **'Terms locked until {time}.'**
  String campaignLiveUntil(String time);

  /// No description provided for @campaignEnded.
  ///
  /// In en, this message translates to:
  /// **'This campaign has ended.'**
  String get campaignEnded;

  /// No description provided for @policyRewardsDisabled.
  ///
  /// In en, this message translates to:
  /// **'Protocol-pool rewards remain disabled; this round changes mode only.'**
  String get policyRewardsDisabled;

  /// No description provided for @startOpenRound.
  ///
  /// In en, this message translates to:
  /// **'START 24H OPEN'**
  String get startOpenRound;

  /// No description provided for @startColourRound.
  ///
  /// In en, this message translates to:
  /// **'START 24H COLOUR'**
  String get startColourRound;

  /// No description provided for @confirmPolicyTitle.
  ///
  /// In en, this message translates to:
  /// **'Start a 24-hour {mode} round?'**
  String confirmPolicyTitle(String mode);

  /// No description provided for @confirmPolicyBody.
  ///
  /// In en, this message translates to:
  /// **'The mode and rules digest are written on-chain. They cannot be changed while the round is live and remain attached if the section is sold.'**
  String get confirmPolicyBody;

  /// No description provided for @confirmPolicyAction.
  ///
  /// In en, this message translates to:
  /// **'START ROUND'**
  String get confirmPolicyAction;

  /// No description provided for @anonymousOwner.
  ///
  /// In en, this message translates to:
  /// **'NOT CLAIMED'**
  String get anonymousOwner;

  /// No description provided for @activity.
  ///
  /// In en, this message translates to:
  /// **'LIVE PULSE'**
  String get activity;

  /// No description provided for @activityReady.
  ///
  /// In en, this message translates to:
  /// **'Canvas synchronized. Tap cells to compose your next transaction.'**
  String get activityReady;

  /// No description provided for @activityQueued.
  ///
  /// In en, this message translates to:
  /// **'Move queued at {x}:{y}.'**
  String activityQueued(int x, int y);

  /// No description provided for @activityCommitted.
  ///
  /// In en, this message translates to:
  /// **'Moves committed as one atomic transaction.'**
  String get activityCommitted;

  /// No description provided for @activitySectionChanged.
  ///
  /// In en, this message translates to:
  /// **'Receiver tuned to sector {section}.'**
  String activitySectionChanged(String section);

  /// No description provided for @activityConnected.
  ///
  /// In en, this message translates to:
  /// **'Wallet connected. You control the signing boundary.'**
  String get activityConnected;

  /// No description provided for @activityFunded.
  ///
  /// In en, this message translates to:
  /// **'Funds transferred to your Bitflip wallet.'**
  String get activityFunded;

  /// No description provided for @activityClaimed.
  ///
  /// In en, this message translates to:
  /// **'Sector claim confirmed on-chain.'**
  String get activityClaimed;

  /// No description provided for @activityListed.
  ///
  /// In en, this message translates to:
  /// **'Sector listed for a fixed on-chain price.'**
  String get activityListed;

  /// No description provided for @activityListingCancelled.
  ///
  /// In en, this message translates to:
  /// **'Sector listing cancelled.'**
  String get activityListingCancelled;

  /// No description provided for @activityPurchased.
  ///
  /// In en, this message translates to:
  /// **'Sector purchased. Ownership transferred atomically.'**
  String get activityPurchased;

  /// No description provided for @activityOwnerFeesWithdrawn.
  ///
  /// In en, this message translates to:
  /// **'Accrued owner fees withdrawn from this sector.'**
  String get activityOwnerFeesWithdrawn;

  /// No description provided for @activityPolicyConfigured.
  ///
  /// In en, this message translates to:
  /// **'Section policy published. Live terms are now locked on-chain.'**
  String get activityPolicyConfigured;

  /// No description provided for @activitySealed.
  ///
  /// In en, this message translates to:
  /// **'Artwork sealed. Its pixels can never change again.'**
  String get activitySealed;

  /// No description provided for @activityMinted.
  ///
  /// In en, this message translates to:
  /// **'Compressed NFT minted. The asset and Bitflip receipt landed atomically.'**
  String get activityMinted;

  /// No description provided for @connectionIssue.
  ///
  /// In en, this message translates to:
  /// **'The chain signal dropped. Your queued moves are still safe locally.'**
  String get connectionIssue;

  /// No description provided for @viewTransaction.
  ///
  /// In en, this message translates to:
  /// **'VIEW CONFIRMED TRANSACTION'**
  String get viewTransaction;

  /// No description provided for @viewAsset.
  ///
  /// In en, this message translates to:
  /// **'VIEW MINTED ASSET'**
  String get viewAsset;

  /// No description provided for @batchFull.
  ///
  /// In en, this message translates to:
  /// **'A transaction can contain at most 16 unique moves.'**
  String get batchFull;

  /// No description provided for @walletIssue.
  ///
  /// In en, this message translates to:
  /// **'The wallet could not complete that request. No transaction was reported as confirmed.'**
  String get walletIssue;

  /// No description provided for @howItWorks.
  ///
  /// In en, this message translates to:
  /// **'THREE MOVES. THAT’S THE GAME.'**
  String get howItWorks;

  /// No description provided for @claimStep.
  ///
  /// In en, this message translates to:
  /// **'01 / CLAIM'**
  String get claimStep;

  /// No description provided for @claimStepBody.
  ///
  /// In en, this message translates to:
  /// **'Bitflip funds one public starting sector. Each later buyer creates and fully funds the BIT vault for the next sector only after time or collective activity unlocks it.'**
  String get claimStepBody;

  /// No description provided for @flipStep.
  ///
  /// In en, this message translates to:
  /// **'02 / FLIP'**
  String get flipStep;

  /// No description provided for @flipStepBody.
  ///
  /// In en, this message translates to:
  /// **'Tap up to 16 cells. They toggle atomically, with an exact fee you approve before signing.'**
  String get flipStepBody;

  /// No description provided for @sealStep.
  ///
  /// In en, this message translates to:
  /// **'03 / SEAL'**
  String get sealStep;

  /// No description provided for @sealStepBody.
  ///
  /// In en, this message translates to:
  /// **'Only the sector creator can freeze the artwork. Sealing is permanent and unlocks minting.'**
  String get sealStepBody;

  /// No description provided for @builtWith.
  ///
  /// In en, this message translates to:
  /// **'BUILT WITH PINA · FLUTTER · BUBBLEGUM V1'**
  String get builtWith;

  /// No description provided for @securityNote.
  ///
  /// In en, this message translates to:
  /// **'BIT has zero decimals. Dynamic fees and exact rewards are checked on-chain before any pixel changes.'**
  String get securityNote;

  /// No description provided for @status.
  ///
  /// In en, this message translates to:
  /// **'STATUS'**
  String get status;

  /// No description provided for @gameStats.
  ///
  /// In en, this message translates to:
  /// **'GAME TELEMETRY'**
  String get gameStats;

  /// No description provided for @claimedValue.
  ///
  /// In en, this message translates to:
  /// **'{count} claimed'**
  String claimedValue(int count);

  /// No description provided for @flipsValue.
  ///
  /// In en, this message translates to:
  /// **'{count} flips'**
  String flipsValue(String count);

  /// No description provided for @mintedValue.
  ///
  /// In en, this message translates to:
  /// **'{count} minted'**
  String mintedValue(int count);

  /// No description provided for @refresh.
  ///
  /// In en, this message translates to:
  /// **'REFRESH'**
  String get refresh;

  /// No description provided for @refreshing.
  ///
  /// In en, this message translates to:
  /// **'SYNCING…'**
  String get refreshing;

  /// No description provided for @learnMore.
  ///
  /// In en, this message translates to:
  /// **'READ THE PROTOCOL'**
  String get learnMore;

  /// No description provided for @dismiss.
  ///
  /// In en, this message translates to:
  /// **'DISMISS'**
  String get dismiss;

  /// No description provided for @privacyLink.
  ///
  /// In en, this message translates to:
  /// **'PRIVACY'**
  String get privacyLink;

  /// No description provided for @termsLink.
  ///
  /// In en, this message translates to:
  /// **'TERMS'**
  String get termsLink;

  /// No description provided for @supportLink.
  ///
  /// In en, this message translates to:
  /// **'SUPPORT'**
  String get supportLink;

  /// No description provided for @backToBitflip.
  ///
  /// In en, this message translates to:
  /// **'BACK TO BITFLIP'**
  String get backToBitflip;

  /// No description provided for @legalEffectiveDate.
  ///
  /// In en, this message translates to:
  /// **'EFFECTIVE 6 SEPTEMBER 2026'**
  String get legalEffectiveDate;

  /// No description provided for @privacyTitle.
  ///
  /// In en, this message translates to:
  /// **'Privacy'**
  String get privacyTitle;

  /// No description provided for @privacyIntroduction.
  ///
  /// In en, this message translates to:
  /// **'Bitflip is built around a public blockchain. This notice explains what becomes public, what the service processes, and what never belongs in a support request.'**
  String get privacyIntroduction;

  /// No description provided for @privacyPublicTitle.
  ///
  /// In en, this message translates to:
  /// **'Public blockchain data'**
  String get privacyPublicTitle;

  /// No description provided for @privacyPublicBody.
  ///
  /// In en, this message translates to:
  /// **'Your wallet address, claims, flips, fees, sealed artwork, mint records, and transaction signatures can be permanently visible on Solana. Bitflip cannot erase or make public chain history private.'**
  String get privacyPublicBody;

  /// No description provided for @privacyServiceTitle.
  ///
  /// In en, this message translates to:
  /// **'Service data'**
  String get privacyServiceTitle;

  /// No description provided for @privacyServiceBody.
  ///
  /// In en, this message translates to:
  /// **'The service processes wallet-signed mint challenges and limited technical data such as request timing, error class, and network address for security, reliability, and abuse prevention. Operational data is retained only as needed for those purposes and protected with access controls.'**
  String get privacyServiceBody;

  /// No description provided for @privacyControlTitle.
  ///
  /// In en, this message translates to:
  /// **'Your keys and choices'**
  String get privacyControlTitle;

  /// No description provided for @privacyControlBody.
  ///
  /// In en, this message translates to:
  /// **'On mobile, Bitflip creates a device-bound private key and stores it using Android Keystore-backed encryption or iOS Keychain; it is not sent to the Bitflip service. On the web, signing stays in your selected third-party wallet. macOS remains view-only in this release.'**
  String get privacyControlBody;

  /// No description provided for @privacyContactTitle.
  ///
  /// In en, this message translates to:
  /// **'Providers and questions'**
  String get privacyContactTitle;

  /// No description provided for @privacyContactBody.
  ///
  /// In en, this message translates to:
  /// **'Wallets, Solana RPC providers, app stores, and hosting providers process data under their own notices. Use the Support page to ask a privacy question without posting seed phrases, private keys, or other secrets.'**
  String get privacyContactBody;

  /// No description provided for @termsTitle.
  ///
  /// In en, this message translates to:
  /// **'Terms'**
  String get termsTitle;

  /// No description provided for @termsIntroduction.
  ///
  /// In en, this message translates to:
  /// **'By using Bitflip, you choose to interact with experimental software and a public blockchain. Verify the network, wallet, sector, pixels, and fee before approving an in-app action or external-wallet funding prompt.'**
  String get termsIntroduction;

  /// No description provided for @termsActionsTitle.
  ///
  /// In en, this message translates to:
  /// **'Fees and irreversible actions'**
  String get termsActionsTitle;

  /// No description provided for @termsActionsBody.
  ///
  /// In en, this message translates to:
  /// **'Claims and flips can require program and network fees shown before signing. Confirmed blockchain transactions are not refundable or reversible. Sealing permanently freezes a sector and enables compressed-NFT minting.'**
  String get termsActionsBody;

  /// No description provided for @termsWalletTitle.
  ///
  /// In en, this message translates to:
  /// **'Wallet responsibility'**
  String get termsWalletTitle;

  /// No description provided for @termsWalletBody.
  ///
  /// In en, this message translates to:
  /// **'The mobile Bitflip wallet is device-bound and has no backup or export yet, so keep only a small playing balance. Web wallet keys remain with the selected provider. You are responsible for device security, available funds, and every approval; Bitflip support will never ask for private key material.'**
  String get termsWalletBody;

  /// No description provided for @termsArtworkTitle.
  ///
  /// In en, this message translates to:
  /// **'Public artwork and metadata'**
  String get termsArtworkTitle;

  /// No description provided for @termsArtworkBody.
  ///
  /// In en, this message translates to:
  /// **'Only submit artwork you are entitled to publish. Pixels and mint records are public. The pixels are stored on chain, while the metadata and rendered image use an operated HTTPS service and should not be described as fully immutable.'**
  String get termsArtworkBody;

  /// No description provided for @termsRiskTitle.
  ///
  /// In en, this message translates to:
  /// **'Experimental service'**
  String get termsRiskTitle;

  /// No description provided for @termsRiskBody.
  ///
  /// In en, this message translates to:
  /// **'Bitflip is provided without a promise of uninterrupted availability, future value, or financial return. Wallets, RPC services, Solana, and marketplaces can fail or change independently. Use only funds you can afford to spend on the game.'**
  String get termsRiskBody;

  /// No description provided for @supportTitle.
  ///
  /// In en, this message translates to:
  /// **'Support'**
  String get supportTitle;

  /// No description provided for @supportIntroduction.
  ///
  /// In en, this message translates to:
  /// **'Start with the transaction signature and the status shown in Bitflip. Public chain evidence usually tells us whether a request was cancelled, failed, expired, or confirmed.'**
  String get supportIntroduction;

  /// No description provided for @supportSafetyTitle.
  ///
  /// In en, this message translates to:
  /// **'Keep secrets out of support'**
  String get supportSafetyTitle;

  /// No description provided for @supportSafetyBody.
  ///
  /// In en, this message translates to:
  /// **'Never send a seed phrase, private key, keystore, password, or wallet backup. Bitflip support does not need them. Anyone asking for one is not helping you safely.'**
  String get supportSafetyBody;

  /// No description provided for @supportTransactionTitle.
  ///
  /// In en, this message translates to:
  /// **'Before retrying'**
  String get supportTransactionTitle;

  /// No description provided for @supportTransactionBody.
  ///
  /// In en, this message translates to:
  /// **'Check the signature in a Solana explorer on the same network. If its status is unknown, wait and refresh before signing again. Minting is safe to re-enter once chain state is known; do not submit duplicates while a transaction is indeterminate.'**
  String get supportTransactionBody;

  /// No description provided for @supportIncludeTitle.
  ///
  /// In en, this message translates to:
  /// **'What to include'**
  String get supportIncludeTitle;

  /// No description provided for @supportIncludeBody.
  ///
  /// In en, this message translates to:
  /// **'Include the platform and version, wallet name, selected network, game and sector numbers, approximate UTC time, public transaction signature if one exists, and the exact error message. Remove personal information and all secrets.'**
  String get supportIncludeBody;

  /// No description provided for @openSupportRequest.
  ///
  /// In en, this message translates to:
  /// **'OPEN SUPPORT REQUEST'**
  String get openSupportRequest;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
