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
  /// **'Claim a 64 × 64 sector, draw in public, then seal the final state into a compressed NFT. Every mark is a tiny on-chain decision.'**
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
  /// **'Wallet signing is unavailable on this device. The canvas stays fully explorable.'**
  String get walletUnavailable;

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

  /// No description provided for @canvasLabel.
  ///
  /// In en, this message translates to:
  /// **'Interactive 64 by 64 pixel sector'**
  String get canvasLabel;

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

  /// No description provided for @feeValue.
  ///
  /// In en, this message translates to:
  /// **'{sol} SOL'**
  String feeValue(String sol);

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

  /// No description provided for @claimSector.
  ///
  /// In en, this message translates to:
  /// **'CLAIM THIS SECTOR'**
  String get claimSector;

  /// No description provided for @mintCompressedNft.
  ///
  /// In en, this message translates to:
  /// **'MINT COMPRESSED NFT'**
  String get mintCompressedNft;

  /// No description provided for @sectionOwner.
  ///
  /// In en, this message translates to:
  /// **'CREATOR'**
  String get sectionOwner;

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

  /// No description provided for @batchFull.
  ///
  /// In en, this message translates to:
  /// **'A transaction can contain at most 16 unique moves.'**
  String get batchFull;

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
  /// **'Sectors open from top-left to bottom-right—by time or by collective activity.'**
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
  /// **'No reward faucet. No no-op payout. Fixed fees are checked on-chain.'**
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
