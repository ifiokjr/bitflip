import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/section_economy.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';
import 'package:bitflip_app/features/game/widgets/game_console.dart';
import 'package:bitflip_app/l10n/generated/app_localizations.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('sealed owner can request a compressed NFT mint', (tester) async {
    var mintCalls = 0;
    await tester.binding.setSurfaceSize(const Size(500, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(SectionLifecycle.sealed),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () => mintCalls++,
          onRefresh: () {},
        ),
      ),
    );

    expect(find.text('MINT COMPRESSED NFT'), findsOneWidget);
    await tester.tap(find.byKey(BitflipTestKeys.mintSection));
    await tester.pump();

    expect(mintCalls, 1);
    expect(tester.takeException(), isNull);
  });

  testWidgets('active section shows batching and permanent seal controls', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(500, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.active,
            queued: {const PixelCoordinate(2, 3)},
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(find.text('COMMIT 1 MOVES'), findsOneWidget);
    expect(find.text('SEAL ARTWORK'), findsOneWidget);
    expect(find.text('1 / 16'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('a depleted reward window disables every custom-client flip', (
    tester,
  ) async {
    final now = BigInt.from(DateTime.now().millisecondsSinceEpoch ~/ 1000);
    final priceConfig = SectionPriceConfig(
      allocationTokens: BigInt.from(26_214_400),
      emissionDurationSeconds: BigInt.from(5_184_000),
      windowSeconds: BigInt.from(300),
      targetTokensPerWindow: BigInt.from(1024),
      startPriceLamports: BigInt.from(10_000),
      minimumPriceLamports: BigInt.from(5000),
      maximumPriceLamports: BigInt.from(1_000_000),
      startFloorPriceLamports: BigInt.from(5000),
      endFloorPriceLamports: BigInt.from(100_000),
      changeDenominator: BigInt.from(8),
      burstElasticity: BigInt.from(2),
    );
    final economy = SectionEconomySnapshot(
      launchedAt: now,
      windowStartedAt: now,
      lastUpdatedAt: now,
      windowId: BigInt.zero,
      windowTargetTokens: BigInt.from(1024),
      windowRewardedTokens: BigInt.from(2048),
      emittedTokens: BigInt.from(2048),
      rewardPoolTokens: BigInt.zero,
      controllerPriceLamports: BigInt.from(10_000),
      postedPriceLamports: BigInt.from(10_000),
      protocolFeeLamports: BigInt.from(20_480_000),
      ownerFeeLamports: BigInt.zero,
    );
    await tester.binding.setSurfaceSize(const Size(500, 1000));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.active,
            queued: {const PixelCoordinate(2, 3)},
            economy: economy,
            priceConfig: priceConfig,
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(find.text('0 BIT'), findsOneWidget);
    expect(find.textContaining('cannot cover the full batch'), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(find.byKey(BitflipTestKeys.commitFlips))
          .onPressed,
      isNull,
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('unsupported native platforms are deliberately view-only', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(500, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.active,
            queued: {const PixelCoordinate(2, 3)},
            isDemo: false,
            isWalletSupported: false,
            walletAddress: null,
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(find.textContaining('View only'), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(find.byKey(BitflipTestKeys.commitFlips))
          .onPressed,
      isNull,
    );
    expect(
      tester
          .widget<OutlinedButton>(find.byKey(BitflipTestKeys.sealSection))
          .onPressed,
      isNull,
    );
  });

  testWidgets('view-only users cannot claim an unclaimed sector', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(500, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.unclaimed,
            isDemo: false,
            isWalletSupported: false,
            walletAddress: null,
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(
      tester
          .widget<FilledButton>(find.byKey(BitflipTestKeys.claimSection))
          .onPressed,
      isNull,
    );
  });

  testWidgets('wallet users cannot transact from unavailable chain state', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(500, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.unclaimed,
            isDemo: false,
            loadStatus: GameLoadStatus.unavailable,
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(
      tester
          .widget<FilledButton>(find.byKey(BitflipTestKeys.claimSection))
          .onPressed,
      isNull,
    );
  });

  testWidgets('owner enters an exact SOL price before listing a sector', (
    tester,
  ) async {
    var listedPrice = BigInt.zero;
    await tester.binding.setSurfaceSize(const Size(500, 1100));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(SectionLifecycle.active),
          onConnect: () {},
          onClaim: () {},
          onList: (price) => listedPrice = price,
          onCancelListing: () {},
          onPurchase: () {},
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    final priceField = find.byKey(BitflipTestKeys.sectionSalePrice);
    await tester.ensureVisible(priceField);
    await tester.enterText(priceField, '0.25');
    await tester.pump();
    final listButton = find.byKey(BitflipTestKeys.listSection);
    await tester.ensureVisible(listButton);
    await tester.tap(listButton);

    expect(listedPrice, BigInt.from(250000000));
    expect(tester.takeException(), isNull);
  });

  testWidgets('listed sector lets another wallet purchase it', (tester) async {
    var purchaseCalls = 0;
    await tester.binding.setSurfaceSize(const Size(500, 1100));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.active,
            owner: 'Seller111111111111111111111111111111111111111',
            walletAddress: 'Buyer1111111111111111111111111111111111111111',
            salePriceLamports: BigInt.from(500000000),
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () => purchaseCalls++,
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(find.text('0.5 SOL'), findsOneWidget);
    final buyButton = find.byKey(BitflipTestKeys.purchaseSection);
    await tester.ensureVisible(buyButton);
    await tester.tap(buyButton);
    expect(purchaseCalls, 1);
    expect(tester.takeException(), isNull);
  });

  testWidgets('owner sees and can withdraw the section-local fee share', (
    tester,
  ) async {
    var withdrawCalls = 0;
    final economy = SectionEconomySnapshot(
      launchedAt: BigInt.zero,
      windowStartedAt: BigInt.zero,
      lastUpdatedAt: BigInt.zero,
      windowId: BigInt.zero,
      windowTargetTokens: BigInt.from(1024),
      windowRewardedTokens: BigInt.zero,
      emittedTokens: BigInt.zero,
      rewardPoolTokens: BigInt.zero,
      controllerPriceLamports: BigInt.from(10_000),
      postedPriceLamports: BigInt.from(10_000),
      protocolFeeLamports: BigInt.from(8000),
      ownerFeeLamports: BigInt.from(2000),
    );
    await tester.binding.setSurfaceSize(const Size(500, 1150));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _TestApp(
        child: GameConsole(
          state: _state(
            SectionLifecycle.active,
            economy: economy,
            ownerShareBasisPoints: 2000,
          ),
          onConnect: () {},
          onClaim: () {},
          onList: (_) {},
          onCancelListing: () {},
          onPurchase: () {},
          onWithdrawOwnerFees: () => withdrawCalls++,
          onCommit: () {},
          onClear: () {},
          onSeal: () {},
          onMint: () {},
          onRefresh: () {},
        ),
      ),
    );

    expect(find.text('EARNS 20% OF FLIP FEES'), findsOneWidget);
    final withdraw = find.byKey(BitflipTestKeys.withdrawOwnerFees);
    expect(find.text('WITHDRAW 0.000002 SOL IN OWNER FEES'), findsOneWidget);
    await tester.ensureVisible(withdraw);
    await tester.tap(withdraw);
    expect(withdrawCalls, 1);
    expect(tester.takeException(), isNull);
  });
}

GameViewState _state(
  SectionLifecycle lifecycle, {
  Set<PixelCoordinate> queued = const {},
  bool isDemo = true,
  bool isWalletSupported = true,
  String? walletAddress = 'DemoWallet111111111111111111111111111111111',
  String owner = 'DemoWallet111111111111111111111111111111111',
  BigInt? salePriceLamports,
  bool isProtocolOwned = false,
  GameLoadStatus? loadStatus,
  SectionEconomySnapshot? economy,
  SectionPriceConfig? priceConfig,
  int ownerShareBasisPoints = 0,
}) {
  return GameViewState(
    snapshot: GameSnapshot(
      gameIndex: 0,
      isDemo: isDemo,
      nextSection: 1,
      totalFlips: BigInt.zero,
      mintedSections: 0,
      claimPriceLamports: BigInt.zero,
      flipFeeLamports: BigInt.from(5000),
      startsAtUnixSeconds: BigInt.zero,
      unlockIntervalSeconds: 3600,
      earlyUnlockFlips: 1024,
      previousSectionFlipCount: BigInt.from(1024),
      treasury: null,
      section: SectionSnapshot(
        index: 0,
        lifecycle: lifecycle,
        bitmap: PixelBitmap.empty(),
        owner: owner,
        flipCount: BigInt.zero,
        revision: BigInt.zero,
        salePriceLamports: salePriceLamports ?? BigInt.zero,
        isProtocolOwned: isProtocolOwned,
        economy: economy,
      ),
      ownerShareBasisPoints: ownerShareBasisPoints,
      priceConfig: priceConfig,
    ),
    queued: queued,
    activity: const GameActivity(GameNotice.ready),
    isBusy: false,
    isWalletSupported: isWalletSupported,
    walletKind: BitflipWalletKind.external,
    canFundWithMobileWallet: false,
    walletAddress: walletAddress,
    walletBalanceLamports: null,
    loadStatus:
        loadStatus ?? (isDemo ? GameLoadStatus.demo : GameLoadStatus.ready),
    walletChain: 'solana:devnet',
    selectedColour: SectionColour.acid,
  );
}

class _TestApp extends StatelessWidget {
  const _TestApp({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      theme: buildBitflipTheme(),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(body: SingleChildScrollView(child: child)),
    );
  }
}
