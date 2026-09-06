import 'package:bitflip_app/features/game/domain/section_economy.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  final config = SectionPriceConfig(
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

  SectionEconomySnapshot economy({
    BigInt? windowRewardedTokens,
    BigInt? emittedTokens,
  }) => SectionEconomySnapshot(
    launchedAt: BigInt.from(1000),
    windowStartedAt: BigInt.from(1000),
    lastUpdatedAt: BigInt.from(1000),
    windowId: BigInt.zero,
    windowTargetTokens: BigInt.from(1024),
    windowRewardedTokens: windowRewardedTokens ?? BigInt.zero,
    emittedTokens: emittedTokens ?? BigInt.zero,
    rewardPoolTokens: BigInt.zero,
    controllerPriceLamports: BigInt.from(10_000),
    postedPriceLamports: BigInt.from(10_000),
    protocolFeeLamports: BigInt.zero,
  );

  test('quotes whole BIT at the fixed price within a window', () {
    final quote = economy().quote(
      config: config,
      now: BigInt.from(1200),
      requestedRewardTokens: BigInt.from(16),
    );

    expect(quote.windowId, BigInt.zero);
    expect(quote.rewardTokens, BigInt.from(16));
    expect(quote.unitPriceLamports, BigInt.from(10_000));
    expect(quote.totalPriceLamports, BigInt.from(160_000));
  });

  test('matches the under-target on-chain rollover price', () {
    final quote = economy(windowRewardedTokens: BigInt.from(512)).quote(
      config: config,
      now: BigInt.from(1300),
      requestedRewardTokens: BigInt.one,
    );

    expect(quote.windowId, BigInt.one);
    expect(quote.unitPriceLamports, BigInt.from(9375));
    expect(quote.rewardTokens, BigInt.one);
  });

  test('caps a quote at remaining reward-window capacity', () {
    final quote = economy(
      windowRewardedTokens: BigInt.from(2040),
      emittedTokens: BigInt.from(2040),
    ).quote(
      config: config,
      now: BigInt.from(1200),
      requestedRewardTokens: BigInt.from(16),
    );

    expect(quote.rewardTokens, BigInt.from(8));
    expect(quote.totalPriceLamports, BigInt.from(80_000));
  });

  test('quotes zero base reward after the emission period', () {
    final quote = economy().quote(
      config: config,
      now: BigInt.from(1000) + config.emissionDurationSeconds,
      requestedRewardTokens: BigInt.one,
    );

    expect(quote.rewardTokens, BigInt.zero);
    expect(quote.totalPriceLamports, BigInt.zero);
  });
}
