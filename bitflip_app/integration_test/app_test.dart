import 'package:bitflip_app/app/app.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:integration_test/integration_test.dart';

import '../test/support/fake_bitflip_repository.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('visitor queues and commits a demo pixel batch', (tester) async {
    await _pumpApp(tester, FakeBitflipRepository(isWalletSupported: false));
    await _openCanvas(tester);

    await _queueOnePixel(tester);
    await _tapKey(tester, BitflipTestKeys.commitFlips);

    expect(
      find.text('Moves committed as one atomic transaction.'),
      findsOneWidget,
    );
  });

  testWidgets('live journey connects, claims, flips, seals, and mints', (
    tester,
  ) async {
    final repository = FakeBitflipRepository(
      snapshot: _liveSnapshot(SectionLifecycle.unclaimed, owner: null),
      walletAddress: null,
    );
    await _pumpApp(tester, repository);
    await _openCanvas(tester);

    await _tapKey(tester, BitflipTestKeys.connectWallet);
    expect(
      find.text('Wallet connected. You control the signing boundary.'),
      findsOneWidget,
    );

    await _tapKey(tester, BitflipTestKeys.claimSection);
    expect(repository.claimCalls, 1);
    expect(find.text('Sector claim confirmed on-chain.'), findsOneWidget);
    expect(find.byKey(BitflipTestKeys.viewResult), findsOneWidget);

    await _queueOnePixel(tester);
    await _tapKey(tester, BitflipTestKeys.commitFlips);
    expect(repository.flipCalls, 1);
    expect(
      find.text('Moves committed as one atomic transaction.'),
      findsOneWidget,
    );

    await _tapKey(tester, BitflipTestKeys.sealSection);
    expect(find.text('Seal this artwork permanently?'), findsOneWidget);
    await _tapKey(tester, BitflipTestKeys.confirmSeal);
    expect(repository.sealCalls, 1);
    expect(
      find.text('Artwork sealed. Its pixels can never change again.'),
      findsOneWidget,
    );

    await _tapKey(tester, BitflipTestKeys.mintSection);
    expect(repository.mintCalls, 1);
    expect(
      find.text(
        'Compressed NFT minted. The asset and Bitflip receipt landed atomically.',
      ),
      findsOneWidget,
    );
    expect(find.text('VIEW MINTED ASSET'), findsOneWidget);
  });

  testWidgets('wallet cancellation keeps the app usable and disconnected', (
    tester,
  ) async {
    final repository = FakeBitflipRepository(
      snapshot: _liveSnapshot(SectionLifecycle.unclaimed, owner: null),
      walletAddress: null,
      connectError: StateError('The wallet request was cancelled.'),
    );
    await _pumpApp(tester, repository);

    await _tapKey(tester, BitflipTestKeys.connectWallet);

    expect(repository.connectCalls, 1);
    expect(repository.walletAddress, isNull);
    expect(find.byKey(BitflipTestKeys.connectWallet), findsWidgets);
    expect(
      find.text(
        'The chain signal dropped. Your queued moves are still safe locally.',
      ),
      findsOneWidget,
    );
  });

  for (final failure in <(String, Object)>[
    ('insufficient funds', StateError('insufficient funds for transaction')),
    ('stale revision', StateError('the section revision changed')),
    ('RPC outage', StateError('RPC unavailable')),
  ]) {
    testWidgets('${failure.$1} preserves a queued live move', (tester) async {
      final repository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
        flipError: failure.$2,
      );
      await _pumpApp(tester, repository);
      await _openCanvas(tester);
      await _queueOnePixel(tester);

      await _tapKey(tester, BitflipTestKeys.commitFlips);

      expect(repository.flipCalls, 1);
      expect(find.byKey(BitflipTestKeys.commitFlips), findsOneWidget);
      expect(
        find.text(
          'The chain signal dropped. Your queued moves are still safe locally.',
        ),
        findsOneWidget,
      );
    });
  }

  for (final failure in <String>['expired', 'replayed']) {
    testWidgets('$failure mint authorization leaves the section retryable', (
      tester,
    ) async {
      final repository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.sealed),
        mintError: StateError('The mint authorization is $failure.'),
      );
      await _pumpApp(tester, repository);
      await _openCanvas(tester);

      await _tapKey(tester, BitflipTestKeys.mintSection);

      expect(repository.mintCalls, 1);
      expect(find.byKey(BitflipTestKeys.mintSection), findsOneWidget);
      expect(
        find.text(
          'The chain signal dropped. Your queued moves are still safe locally.',
        ),
        findsOneWidget,
      );
    });
  }
}

Future<void> _pumpApp(
  WidgetTester tester,
  FakeBitflipRepository repository,
) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
      child: const BitflipApp(),
    ),
  );
  await tester.pump(const Duration(milliseconds: 500));
}

Future<void> _openCanvas(WidgetTester tester) async {
  final openCanvasButton = find.text('OPEN CANVAS');
  await tester.ensureVisible(openCanvasButton);
  await tester.pump(const Duration(milliseconds: 600));
  await tester.tap(openCanvasButton);
  await tester.pump(const Duration(milliseconds: 600));
  final canvas = find.byKey(BitflipTestKeys.canvas);
  expect(canvas, findsOneWidget);
  await tester.ensureVisible(canvas);
  await tester.pump(const Duration(milliseconds: 600));
}

Future<void> _queueOnePixel(WidgetTester tester) async {
  final canvas = find.byKey(BitflipTestKeys.canvas);
  await tester.ensureVisible(canvas);
  await tester.pump(const Duration(milliseconds: 300));
  await tester.tapAt(tester.getCenter(canvas) + const Offset(2, 2));
  await tester.pump();
  expect(find.byKey(BitflipTestKeys.commitFlips), findsOneWidget);
}

Future<void> _tapKey(WidgetTester tester, Key key) async {
  final finder = find.byKey(key).first;
  expect(finder, findsOneWidget);
  await tester.ensureVisible(finder);
  await tester.pump(const Duration(milliseconds: 300));
  await tester.tap(finder);
  await tester.pump(const Duration(milliseconds: 500));
}

GameSnapshot _liveSnapshot(
  SectionLifecycle lifecycle, {
  String? owner = 'DemoWallet111111111111111111111111111111111',
}) {
  return GameSnapshot(
    gameIndex: 0,
    isDemo: false,
    nextSection: lifecycle == SectionLifecycle.unclaimed ? 0 : 1,
    totalFlips: BigInt.zero,
    mintedSections: lifecycle == SectionLifecycle.minted ? 1 : 0,
    claimPriceLamports: BigInt.from(100000000),
    flipFeeLamports: BigInt.from(5000),
    treasury: '11111111111111111111111111111111',
    section: SectionSnapshot(
      index: 0,
      lifecycle: lifecycle,
      bitmap: PixelBitmap.empty(),
      owner: owner,
      flipCount: BigInt.zero,
      revision: BigInt.zero,
      assetId: lifecycle == SectionLifecycle.minted
          ? 'Asset11111111111111111111111111111111111111'
          : null,
    ),
  );
}
