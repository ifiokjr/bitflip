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
      },
    );

    test('does not mint an active section', () async {
      repository.snapshot = _liveSnapshot(SectionLifecycle.active);
      final controller = container.read(gameControllerProvider.notifier);
      await controller.refresh();

      await controller.mintSection();

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
