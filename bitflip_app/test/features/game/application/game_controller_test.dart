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

GameSnapshot _liveSnapshot(SectionLifecycle lifecycle) {
  const owner = 'DemoWallet111111111111111111111111111111111';
  return GameSnapshot(
    gameIndex: 0,
    isDemo: false,
    nextSection: 1,
    totalFlips: BigInt.zero,
    mintedSections: 0,
    claimPriceLamports: BigInt.from(1000000),
    flipFeeLamports: BigInt.from(5000),
    treasury: '11111111111111111111111111111111',
    section: SectionSnapshot(
      index: 0,
      lifecycle: lifecycle,
      bitmap: PixelBitmap.empty(),
      owner: owner,
      flipCount: BigInt.zero,
      revision: BigInt.zero,
    ),
  );
}
