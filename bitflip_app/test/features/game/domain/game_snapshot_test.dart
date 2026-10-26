import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/pixel_colour_map.dart';
import 'package:bitflip_app/features/game/domain/section_economy.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';
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

  test('snapshots compare by value so unchanged polls are detectable', () {
    GameSnapshot build({BigInt? totalFlips, PixelCoordinate? extraPixel}) {
      return GameSnapshot(
        gameIndex: 0,
        isDemo: false,
        nextSection: 3,
        totalFlips: totalFlips ?? BigInt.from(77),
        mintedSections: 1,
        claimPriceLamports: BigInt.from(1_000_000),
        flipFeeLamports: BigInt.from(5000),
        startsAtUnixSeconds: BigInt.from(1000),
        unlockIntervalSeconds: 3600,
        earlyUnlockFlips: 1024,
        previousSectionFlipCount: BigInt.from(2048),
        treasury: '11111111111111111111111111111111',
        ownerShareBasisPoints: 2500,
        bitMint: 'Mint111111111111111111111111111111111111111',
        bitReserve: 'Reserve11111111111111111111111111111111111',
        priceConfig: SectionPriceConfig(
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
        ),
        section: SectionSnapshot(
          index: 2,
          lifecycle: SectionLifecycle.active,
          bitmap: PixelBitmap.demo(2).toggled([?extraPixel]),
          owner: 'Owner11111111111111111111111111111111111111',
          flipCount: BigInt.from(40),
          revision: BigInt.from(9),
          salePriceLamports: BigInt.zero,
          bitVault: 'Vault111111111111111111111111111111111111111',
          colourMap: PixelColourMap.empty().painted(const [
            PixelCoordinate(1, 1),
          ], SectionColour.pink),
          economy: SectionEconomySnapshot(
            launchedAt: BigInt.from(1000),
            windowStartedAt: BigInt.from(1000),
            lastUpdatedAt: BigInt.from(1000),
            windowId: BigInt.zero,
            windowTargetTokens: BigInt.from(1024),
            windowRewardedTokens: BigInt.zero,
            emittedTokens: BigInt.zero,
            rewardPoolTokens: BigInt.zero,
            controllerPriceLamports: BigInt.from(10_000),
            postedPriceLamports: BigInt.from(10_000),
            protocolFeeLamports: BigInt.zero,
            ownerFeeLamports: BigInt.zero,
          ),
          policy: SectionPolicySnapshot(
            version: BigInt.one,
            startsAtUnixSeconds: BigInt.from(100),
            endsAtUnixSeconds: BigInt.from(200),
            entryPriceTokens: BigInt.zero,
            rewardPerActionTokens: BigInt.zero,
            rulesDigest: List.filled(32, 1),
            mode: SectionPolicyMode.colourCanvas,
            paletteId: 0,
            rewardPolicy: SectionRewardPolicy.none,
          ),
        ),
      );
    }

    expect(build(), equals(build()));
    expect(build().hashCode, build().hashCode);
    expect(build().section, equals(build().section));
    expect(build().section.hashCode, build().section.hashCode);
    expect(build(), isNot(equals(build(totalFlips: BigInt.from(78)))));
    expect(
      build().section,
      isNot(equals(build(extraPixel: const PixelCoordinate(7, 7)).section)),
    );
  });
}
