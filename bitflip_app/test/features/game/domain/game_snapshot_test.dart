import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('trySolToLamports', () {
    test('converts whole and fractional SOL without floating point', () {
      expect(trySolToLamports('1'), BigInt.from(1000000000));
      expect(trySolToLamports('0.25'), BigInt.from(250000000));
      expect(trySolToLamports('0.000000001'), BigInt.one);
    });

    test('rejects zero, negative, and over-precise values', () {
      expect(trySolToLamports('0'), isNull);
      expect(trySolToLamports('-1'), isNull);
      expect(trySolToLamports('0.0000000001'), isNull);
      expect(trySolToLamports('999999999999999999999999'), isNull);
      expect(trySolToLamports('one'), isNull);
    });
  });

  test('claim unlock requires the next sector and either threshold', () {
    final locked = GameSnapshot.empty(gameIndex: 0).copyWith(
      nextSection: 1,
      section: SectionSnapshot(
        index: 1,
        lifecycle: SectionLifecycle.unclaimed,
        bitmap: PixelBitmap.empty(),
        owner: null,
        flipCount: BigInt.zero,
        revision: BigInt.zero,
        salePriceLamports: BigInt.zero,
      ),
    );
    final snapshot = GameSnapshot(
      gameIndex: locked.gameIndex,
      isDemo: false,
      nextSection: locked.nextSection,
      totalFlips: BigInt.zero,
      mintedSections: 0,
      claimPriceLamports: BigInt.zero,
      flipFeeLamports: BigInt.zero,
      startsAtUnixSeconds: BigInt.from(1000),
      unlockIntervalSeconds: 100,
      earlyUnlockFlips: 10,
      previousSectionFlipCount: BigInt.from(9),
      treasury: null,
      section: locked.section,
    );

    expect(snapshot.canClaimSectionAt(BigInt.from(1099)), isFalse);
    expect(snapshot.canClaimSectionAt(BigInt.from(1100)), isTrue);
    expect(
      GameSnapshot(
        gameIndex: snapshot.gameIndex,
        isDemo: snapshot.isDemo,
        nextSection: snapshot.nextSection,
        totalFlips: snapshot.totalFlips,
        mintedSections: snapshot.mintedSections,
        claimPriceLamports: snapshot.claimPriceLamports,
        flipFeeLamports: snapshot.flipFeeLamports,
        startsAtUnixSeconds: snapshot.startsAtUnixSeconds,
        unlockIntervalSeconds: snapshot.unlockIntervalSeconds,
        earlyUnlockFlips: snapshot.earlyUnlockFlips,
        previousSectionFlipCount: BigInt.from(10),
        treasury: snapshot.treasury,
        section: snapshot.section,
      ).canClaimSectionAt(BigInt.from(1001)),
      isTrue,
    );
  });
}
