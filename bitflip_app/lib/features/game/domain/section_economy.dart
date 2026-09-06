final class SectionPriceConfig {
  const SectionPriceConfig({
    required this.allocationTokens,
    required this.emissionDurationSeconds,
    required this.windowSeconds,
    required this.targetTokensPerWindow,
    required this.startPriceLamports,
    required this.minimumPriceLamports,
    required this.maximumPriceLamports,
    required this.startFloorPriceLamports,
    required this.endFloorPriceLamports,
    required this.changeDenominator,
    required this.burstElasticity,
  });

  final BigInt allocationTokens;
  final BigInt emissionDurationSeconds;
  final BigInt windowSeconds;
  final BigInt targetTokensPerWindow;
  final BigInt startPriceLamports;
  final BigInt minimumPriceLamports;
  final BigInt maximumPriceLamports;
  final BigInt startFloorPriceLamports;
  final BigInt endFloorPriceLamports;
  final BigInt changeDenominator;
  final BigInt burstElasticity;
}

final class SectionEconomySnapshot {
  const SectionEconomySnapshot({
    required this.launchedAt,
    required this.windowStartedAt,
    required this.lastUpdatedAt,
    required this.windowId,
    required this.windowTargetTokens,
    required this.windowRewardedTokens,
    required this.emittedTokens,
    required this.rewardPoolTokens,
    required this.controllerPriceLamports,
    required this.postedPriceLamports,
    required this.protocolFeeLamports,
  });

  final BigInt launchedAt;
  final BigInt windowStartedAt;
  final BigInt lastUpdatedAt;
  final BigInt windowId;
  final BigInt windowTargetTokens;
  final BigInt windowRewardedTokens;
  final BigInt emittedTokens;
  final BigInt rewardPoolTokens;
  final BigInt controllerPriceLamports;
  final BigInt postedPriceLamports;
  final BigInt protocolFeeLamports;

  SectionFlipQuote quote({
    required SectionPriceConfig config,
    required BigInt now,
    required BigInt requestedRewardTokens,
  }) {
    if (requestedRewardTokens <= BigInt.zero ||
        requestedRewardTokens > BigInt.from(16)) {
      throw ArgumentError.value(
        requestedRewardTokens,
        'requestedRewardTokens',
      );
    }
    if (now < launchedAt || now < lastUpdatedAt) {
      throw StateError('The local clock predates the section economy.');
    }

    var nextWindowStartedAt = windowStartedAt;
    var nextWindowId = windowId;
    var nextWindowTarget = windowTargetTokens;
    var nextWindowRewarded = windowRewardedTokens;
    var nextEmitted = emittedTokens;
    var nextPool = rewardPoolTokens;
    var nextControllerPrice = controllerPriceLamports;
    var nextPostedPrice = postedPriceLamports;
    final emissionEndsAt = launchedAt + config.emissionDurationSeconds;
    final effectiveNow = _min(now, emissionEndsAt);
    final completedWindows =
        (effectiveNow - nextWindowStartedAt) ~/ config.windowSeconds;

    BigInt remainingBase() =>
        config.allocationTokens - nextEmitted - nextPool;

    void accruePool(BigInt requested) {
      nextPool += _min(requested, remainingBase());
    }

    if (completedWindows > BigInt.zero) {
      nextControllerPrice = _adjustedControllerPrice(
        config: config,
        currentPrice: nextControllerPrice,
        rewardedTokens: nextWindowRewarded,
        targetTokens: nextWindowTarget,
      );
      accruePool(_max(BigInt.zero, nextWindowTarget - nextWindowRewarded));

      final missedEmptyWindows = completedWindows - BigInt.one;
      if (missedEmptyWindows > BigInt.zero) {
        final maximumChange =
            config.startPriceLamports ~/ config.changeDenominator;
        nextControllerPrice = _clamp(
          nextControllerPrice - (maximumChange * missedEmptyWindows),
          config.minimumPriceLamports,
          config.maximumPriceLamports,
        );
        accruePool(config.targetTokensPerWindow * missedEmptyWindows);
      }

      nextWindowStartedAt += completedWindows * config.windowSeconds;
      nextWindowId += completedWindows;
      nextWindowRewarded = BigInt.zero;
      if (nextWindowStartedAt >= emissionEndsAt) {
        accruePool(remainingBase());
      }

      final remaining = remainingBase();
      nextWindowTarget = nextWindowStartedAt < emissionEndsAt &&
              remaining > BigInt.zero
          ? _min(config.targetTokensPerWindow, remaining)
          : BigInt.zero;
      nextPostedPrice = _max(
        nextControllerPrice,
        _inventoryFloor(config, nextEmitted),
      );
    }

    final remaining = remainingBase();
    final windowCapacity = _min(
      nextWindowTarget * config.burstElasticity,
      config.allocationTokens,
    );
    final remainingWindowCapacity = _max(
      BigInt.zero,
      windowCapacity - nextWindowRewarded,
    );
    final rewardTokens = _min(
      requestedRewardTokens,
      _min(remaining, remainingWindowCapacity),
    );
    return SectionFlipQuote(
      windowId: nextWindowId,
      rewardTokens: rewardTokens,
      unitPriceLamports: nextPostedPrice,
      totalPriceLamports: nextPostedPrice * rewardTokens,
    );
  }
}

final class SectionFlipQuote {
  const SectionFlipQuote({
    required this.windowId,
    required this.rewardTokens,
    required this.unitPriceLamports,
    required this.totalPriceLamports,
  });

  final BigInt windowId;
  final BigInt rewardTokens;
  final BigInt unitPriceLamports;
  final BigInt totalPriceLamports;
}

BigInt _adjustedControllerPrice({
  required SectionPriceConfig config,
  required BigInt currentPrice,
  required BigInt rewardedTokens,
  required BigInt targetTokens,
}) {
  if (targetTokens == BigInt.zero) return config.minimumPriceLamports;
  final boundedRewarded = _min(rewardedTokens, targetTokens * BigInt.two);
  final distance = (boundedRewarded - targetTokens).abs();
  final maximumChange = config.startPriceLamports ~/ config.changeDenominator;
  final change = maximumChange * distance ~/ targetTokens;
  final adjusted = boundedRewarded >= targetTokens
      ? currentPrice + change
      : currentPrice - change;
  return _clamp(
    adjusted,
    config.minimumPriceLamports,
    config.maximumPriceLamports,
  );
}

BigInt _inventoryFloor(SectionPriceConfig config, BigInt emittedTokens) {
  final range =
      config.endFloorPriceLamports - config.startFloorPriceLamports;
  return config.startFloorPriceLamports +
      (range * emittedTokens ~/ config.allocationTokens);
}

BigInt _min(BigInt left, BigInt right) => left < right ? left : right;
BigInt _max(BigInt left, BigInt right) => left > right ? left : right;
BigInt _clamp(BigInt value, BigInt minimum, BigInt maximum) =>
    _min(_max(value, minimum), maximum);
