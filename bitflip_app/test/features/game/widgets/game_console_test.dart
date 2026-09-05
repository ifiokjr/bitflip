import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
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
}

GameViewState _state(
  SectionLifecycle lifecycle, {
  Set<PixelCoordinate> queued = const {},
  bool isDemo = true,
  bool isWalletSupported = true,
  String? walletAddress = 'DemoWallet111111111111111111111111111111111',
}) {
  const owner = 'DemoWallet111111111111111111111111111111111';
  return GameViewState(
    snapshot: GameSnapshot(
      gameIndex: 0,
      isDemo: isDemo,
      nextSection: 1,
      totalFlips: BigInt.zero,
      mintedSections: 0,
      claimPriceLamports: BigInt.zero,
      flipFeeLamports: BigInt.from(5000),
      treasury: null,
      section: SectionSnapshot(
        index: 0,
        lifecycle: lifecycle,
        bitmap: PixelBitmap.empty(),
        owner: owner,
        flipCount: BigInt.zero,
        revision: BigInt.zero,
      ),
    ),
    queued: queued,
    activity: const GameActivity(GameNotice.ready),
    isBusy: false,
    isWalletSupported: isWalletSupported,
    walletAddress: walletAddress,
    loadStatus: isDemo ? GameLoadStatus.demo : GameLoadStatus.ready,
    walletChain: 'solana:devnet',
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
