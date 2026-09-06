import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../../../support/fake_bitflip_repository.dart';

void main() {
  group('GameController', () {
    late FakeBitflipRepository repository;
    late ProviderContainer container;

    setUp(() {
      repository = FakeBitflipRepository();
      container = ProviderContainer(
        overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
      );
    });

    tearDown(() => container.dispose());

    test('queues unique pixels and toggles a repeated selection out', () {
      final controller = container.read(gameControllerProvider.notifier);
      const coordinate = PixelCoordinate(4, 9);

      controller.togglePixel(coordinate);
      expect(container.read(gameControllerProvider).queued, {coordinate});

      controller.togglePixel(coordinate);
      expect(container.read(gameControllerProvider).queued, isEmpty);
    });

    test('caps each atomic batch at 16 unique coordinates', () {
      final controller = container.read(gameControllerProvider.notifier);

      for (var x = 0; x < maxFlipBatch + 1; x++) {
        controller.togglePixel(PixelCoordinate(x, 0));
      }
      final state = container.read(gameControllerProvider);

      expect(state.queued, hasLength(maxFlipBatch));
      expect(state.queued, isNot(contains(const PixelCoordinate(16, 0))));
      expect(state.activity.notice, GameNotice.batchFull);
    });

    test('moves the accessible cursor without changing queued pixels', () {
      final controller = container.read(gameControllerProvider.notifier);

      controller.selectPixel(const PixelCoordinate(18, 27));

      final state = container.read(gameControllerProvider);
      expect(state.cursor, const PixelCoordinate(18, 27));
      expect(state.queued, isEmpty);
    });

    test('commits a demo batch locally and clears the preview queue', () async {
      final controller = container.read(gameControllerProvider.notifier);
      final before = container.read(gameControllerProvider).snapshot;
      const coordinates = [PixelCoordinate(3, 2), PixelCoordinate(1, 1)];
      for (final coordinate in coordinates) {
        controller.togglePixel(coordinate);
      }

      await controller.commitMoves();
      final state = container.read(gameControllerProvider);

      expect(state.queued, isEmpty);
      expect(state.snapshot.totalFlips, before.totalFlips + BigInt.from(2));
      expect(
        state.snapshot.section.revision,
        before.section.revision + BigInt.one,
      );
      expect(state.activity.notice, GameNotice.committed);
    });

    test('sends sorted live coordinates through the repository', () async {
      repository.snapshot = _liveSnapshot(SectionLifecycle.active);
      final controller = container.read(gameControllerProvider.notifier);
      await controller.refresh();
      controller.togglePixel(const PixelCoordinate(9, 8));
      controller.togglePixel(const PixelCoordinate(1, 0));

      await controller.commitMoves();

      expect(repository.flipCalls, 1);
      expect(repository.lastFlips, const [
        PixelCoordinate(1, 0),
        PixelCoordinate(9, 8),
      ]);
      expect(
        container.read(gameControllerProvider).activity.transactionSignature,
        'flip-signature',
      );
      await Future<void>.delayed(Duration.zero);
      expect(
        container.read(gameControllerProvider).snapshot.section.revision,
        BigInt.one,
        reason:
            'an older RPC response must not overwrite confirmed local state',
      );
    });

    test(
      'mints only a sealed section and records the returned asset',
      () async {
        repository.snapshot = _liveSnapshot(SectionLifecycle.sealed);
        final controller = container.read(gameControllerProvider.notifier);
        await controller.refresh();

        await controller.mintSection();
        final state = container.read(gameControllerProvider);

        expect(repository.mintCalls, 1);
        expect(state.snapshot.section.lifecycle, SectionLifecycle.minted);
        expect(state.snapshot.section.assetId, startsWith('Asset'));
        expect(state.snapshot.mintedSections, 1);
        expect(state.activity.notice, GameNotice.minted);
        expect(state.activity.transactionSignature, 'mint-signature');
        expect(state.activity.assetId, startsWith('Asset'));
      },
    );

    test('does not mint an active section', () async {
      repository.snapshot = _liveSnapshot(SectionLifecycle.active);
      final controller = container.read(gameControllerProvider.notifier);
      await controller.refresh();

      await controller.mintSection();

      expect(repository.mintCalls, 0);
    });

    test('owner can list and cancel a section at an exact SOL price', () async {
      repository.snapshot = _liveSnapshot(SectionLifecycle.active);
      final controller = container.read(gameControllerProvider.notifier);
      await controller.refresh();

      await controller.listSection(BigInt.from(250000000));
      expect(repository.listCalls, 1);
      expect(repository.lastListingPriceLamports, BigInt.from(250000000));
      expect(
        container
            .read(gameControllerProvider)
            .snapshot
            .section
            .salePriceLamports,
        BigInt.from(250000000),
      );

      await controller.cancelSectionListing();
      expect(repository.cancelListingCalls, 1);
      expect(
        container
            .read(gameControllerProvider)
            .snapshot
            .section
            .salePriceLamports,
        BigInt.zero,
      );
      expect(
        container.read(gameControllerProvider).activity.notice,
        GameNotice.listingCancelled,
      );
    });

    test(
      'buyer can purchase a listed section but cannot list section zero',
      () async {
        repository = FakeBitflipRepository(
          snapshot: _liveSnapshot(
            SectionLifecycle.active,
            owner: 'Seller111111111111111111111111111111111111111',
            salePriceLamports: BigInt.from(500000000),
          ),
          walletAddress: 'Buyer1111111111111111111111111111111111111111',
        );
        final buyerContainer = ProviderContainer(
          overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
        );
        addTearDown(buyerContainer.dispose);
        final controller = buyerContainer.read(gameControllerProvider.notifier);
        await controller.refresh();

        await controller.purchaseSection();

        expect(repository.purchaseCalls, 1);
        expect(
          buyerContainer.read(gameControllerProvider).snapshot.section.owner,
          repository.walletAddress,
        );
        expect(
          buyerContainer.read(gameControllerProvider).activity.notice,
          GameNotice.purchased,
        );

        repository.snapshot = _liveSnapshot(
          SectionLifecycle.active,
          isProtocolOwned: true,
        );
        await controller.refresh();
        await controller.listSection(BigInt.one);
        expect(repository.listCalls, 0);
      },
    );

    test('live mode starts blank instead of displaying demo art', () {
      repository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
      );
      final liveContainer = ProviderContainer(
        overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(liveContainer.dispose);

      final state = liveContainer.read(gameControllerProvider);

      expect(state.loadStatus, GameLoadStatus.loading);
      expect(state.snapshot.isDemo, isFalse);
      expect(state.snapshot.section.bitmap.onCount, 0);
    });

    test(
      'initializes and restores the embedded wallet during refresh',
      () async {
        repository = FakeBitflipRepository(
          snapshot: _liveSnapshot(SectionLifecycle.active),
          walletKind: BitflipWalletKind.embedded,
          canFundWithMobileWallet: true,
          walletAddress: null,
          walletBalanceLamports: BigInt.from(25000000),
        );
        final walletContainer = ProviderContainer(
          overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
        );
        addTearDown(walletContainer.dispose);

        await walletContainer.read(gameControllerProvider.notifier).refresh();
        final state = walletContainer.read(gameControllerProvider);

        expect(repository.initializeCalls, 1);
        expect(state.walletAddress, startsWith('Embedded'));
        expect(state.walletBalanceLamports, BigInt.from(25000000));
        expect(state.walletKind, BitflipWalletKind.embedded);
        expect(state.canFundWithMobileWallet, isTrue);
      },
    );

    test('funds the embedded wallet and refreshes its balance', () async {
      repository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
        walletKind: BitflipWalletKind.embedded,
        canFundWithMobileWallet: true,
        walletAddress: null,
      );
      final walletContainer = ProviderContainer(
        overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(walletContainer.dispose);
      final controller = walletContainer.read(gameControllerProvider.notifier);
      await controller.refresh();

      await controller.fundWithMobileWallet(BigInt.from(50000000));
      final state = walletContainer.read(gameControllerProvider);

      expect(repository.fundCalls, 1);
      expect(repository.lastFundingLamports, BigInt.from(50000000));
      expect(state.walletBalanceLamports, BigInt.from(50000000));
      expect(state.activity.notice, GameNotice.funded);
      expect(state.activity.transactionSignature, 'fund-signature');
    });

    test('reports confirmed funding when the balance refresh fails', () async {
      repository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
        walletKind: BitflipWalletKind.embedded,
        canFundWithMobileWallet: true,
        walletAddress: null,
      );
      final walletContainer = ProviderContainer(
        overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(walletContainer.dispose);
      final controller = walletContainer.read(gameControllerProvider.notifier);
      await controller.refresh();
      repository.balanceError = StateError('balance RPC unavailable');

      await controller.fundWithMobileWallet(BigInt.from(50000000));
      final state = walletContainer.read(gameControllerProvider);

      expect(repository.fundCalls, 1);
      expect(state.activity.notice, GameNotice.funded);
      expect(state.activity.transactionSignature, 'fund-signature');
      expect(state.walletBalanceLamports, BigInt.zero);
    });

    test('missing and failed RPC loads have explicit states', () async {
      final unavailableRepository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
        returnNullOnLoad: true,
      );
      final unavailableContainer = ProviderContainer(
        overrides: [
          bitflipRepositoryProvider.overrideWithValue(unavailableRepository),
        ],
      );
      addTearDown(unavailableContainer.dispose);
      await unavailableContainer
          .read(gameControllerProvider.notifier)
          .refresh();
      expect(
        unavailableContainer.read(gameControllerProvider).loadStatus,
        GameLoadStatus.unavailable,
      );

      final errorRepository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
        loadError: StateError('RPC unavailable'),
      );
      final errorContainer = ProviderContainer(
        overrides: [
          bitflipRepositoryProvider.overrideWithValue(errorRepository),
        ],
      );
      addTearDown(errorContainer.dispose);
      await errorContainer.read(gameControllerProvider.notifier).refresh();
      expect(
        errorContainer.read(gameControllerProvider).loadStatus,
        GameLoadStatus.error,
      );
      expect(errorContainer.read(gameControllerProvider).canTransact, isFalse);
    });

    test('unsupported live platforms cannot queue or transact', () async {
      repository = FakeBitflipRepository(
        snapshot: _liveSnapshot(SectionLifecycle.active),
        isWalletSupported: false,
        walletAddress: null,
      );
      final viewOnlyContainer = ProviderContainer(
        overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(viewOnlyContainer.dispose);
      final controller = viewOnlyContainer.read(
        gameControllerProvider.notifier,
      );
      await controller.refresh();

      controller.togglePixel(const PixelCoordinate(3, 2));
      await controller.claimSection();
      await controller.commitMoves();
      await controller.sealSection();
      await controller.mintSection();

      expect(viewOnlyContainer.read(gameControllerProvider).queued, isEmpty);
      expect(repository.claimCalls, 0);
      expect(repository.flipCalls, 0);
      expect(repository.sealCalls, 0);
      expect(repository.mintCalls, 0);
    });
  });
}

GameSnapshot _liveSnapshot(
  SectionLifecycle lifecycle, {
  String owner = 'DemoWallet111111111111111111111111111111111',
  BigInt? salePriceLamports,
  bool isProtocolOwned = false,
}) {
  return GameSnapshot(
    gameIndex: 0,
    isDemo: false,
    nextSection: 1,
    totalFlips: BigInt.zero,
    mintedSections: 0,
    claimPriceLamports: BigInt.from(1000000),
    flipFeeLamports: BigInt.from(5000),
    startsAtUnixSeconds: BigInt.zero,
    unlockIntervalSeconds: 3600,
    earlyUnlockFlips: 1024,
    previousSectionFlipCount: BigInt.from(1024),
    treasury: '11111111111111111111111111111111',
    section: SectionSnapshot(
      index: 0,
      lifecycle: lifecycle,
      bitmap: PixelBitmap.empty(),
      owner: owner,
      flipCount: BigInt.zero,
      revision: BigInt.zero,
      salePriceLamports: salePriceLamports ?? BigInt.zero,
      isProtocolOwned: isProtocolOwned,
    ),
  );
}
