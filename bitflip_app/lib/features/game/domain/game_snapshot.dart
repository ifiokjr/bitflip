import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/section_economy.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';

enum SectionLifecycle { unclaimed, active, sealed, minted }

final class SectionSnapshot {
  const SectionSnapshot({
    required this.index,
    required this.lifecycle,
    required this.bitmap,
    required this.owner,
    required this.flipCount,
    required this.revision,
    required this.salePriceLamports,
    this.isProtocolOwned = false,
    this.assetId,
    this.bitVault,
    this.economy,
    this.policy,
  });

  final int index;
  final SectionLifecycle lifecycle;
  final PixelBitmap bitmap;
  final String? owner;
  final BigInt flipCount;
  final BigInt revision;
  final BigInt salePriceLamports;
  final bool isProtocolOwned;
  final String? assetId;
  final String? bitVault;
  final SectionEconomySnapshot? economy;
  final SectionPolicySnapshot? policy;

  bool get isEditable => lifecycle == SectionLifecycle.active;
  bool get isClaimed => lifecycle != SectionLifecycle.unclaimed;
  bool get isListed => salePriceLamports > BigInt.zero;

  SectionSnapshot copyWith({
    SectionLifecycle? lifecycle,
    PixelBitmap? bitmap,
    String? owner,
    BigInt? flipCount,
    BigInt? revision,
    BigInt? salePriceLamports,
    bool? isProtocolOwned,
    String? assetId,
    String? bitVault,
    SectionEconomySnapshot? economy,
    SectionPolicySnapshot? policy,
  }) {
    return SectionSnapshot(
      index: index,
      lifecycle: lifecycle ?? this.lifecycle,
      bitmap: bitmap ?? this.bitmap,
      owner: owner ?? this.owner,
      flipCount: flipCount ?? this.flipCount,
      revision: revision ?? this.revision,
      salePriceLamports: salePriceLamports ?? this.salePriceLamports,
      isProtocolOwned: isProtocolOwned ?? this.isProtocolOwned,
      assetId: assetId ?? this.assetId,
      bitVault: bitVault ?? this.bitVault,
      economy: economy ?? this.economy,
      policy: policy ?? this.policy,
    );
  }
}

final class GameSnapshot {
  const GameSnapshot({
    required this.gameIndex,
    required this.isDemo,
    required this.nextSection,
    required this.totalFlips,
    required this.mintedSections,
    required this.claimPriceLamports,
    required this.flipFeeLamports,
    required this.startsAtUnixSeconds,
    required this.unlockIntervalSeconds,
    required this.earlyUnlockFlips,
    required this.previousSectionFlipCount,
    required this.treasury,
    required this.section,
    this.ownerShareBasisPoints = 0,
    this.bitMint,
    this.bitReserve,
    this.priceConfig,
  });

  factory GameSnapshot.demo({int sectionIndex = 0}) {
    return GameSnapshot(
      gameIndex: 0,
      isDemo: true,
      nextSection: 18,
      totalFlips: BigInt.from(48192 + sectionIndex * 37),
      mintedSections: 5,
      claimPriceLamports: BigInt.from(100000000),
      flipFeeLamports: BigInt.from(5000),
      startsAtUnixSeconds: BigInt.zero,
      unlockIntervalSeconds: 3600,
      earlyUnlockFlips: 1024,
      previousSectionFlipCount: BigInt.from(2048),
      treasury: null,
      section: SectionSnapshot(
        index: sectionIndex,
        lifecycle: sectionIndex < 5
            ? SectionLifecycle.minted
            : sectionIndex < 18
            ? SectionLifecycle.active
            : SectionLifecycle.unclaimed,
        bitmap: PixelBitmap.demo(sectionIndex),
        owner: sectionIndex < 18 ? '8iT…fL1P' : null,
        flipCount: BigInt.from(2048 + sectionIndex * 13),
        revision: BigInt.from(120 + sectionIndex),
        salePriceLamports: BigInt.zero,
        assetId: sectionIndex < 5 ? 'cnft:$sectionIndex' : null,
      ),
    );
  }

  factory GameSnapshot.empty({required int gameIndex, int sectionIndex = 0}) {
    return GameSnapshot(
      gameIndex: gameIndex,
      isDemo: false,
      nextSection: 0,
      totalFlips: BigInt.zero,
      mintedSections: 0,
      claimPriceLamports: BigInt.zero,
      flipFeeLamports: BigInt.zero,
      startsAtUnixSeconds: BigInt.zero,
      unlockIntervalSeconds: 0,
      earlyUnlockFlips: 0,
      previousSectionFlipCount: null,
      treasury: null,
      section: SectionSnapshot(
        index: sectionIndex,
        lifecycle: SectionLifecycle.unclaimed,
        bitmap: PixelBitmap.empty(),
        owner: null,
        flipCount: BigInt.zero,
        revision: BigInt.zero,
        salePriceLamports: BigInt.zero,
      ),
    );
  }

  final int gameIndex;
  final bool isDemo;
  final int nextSection;
  final BigInt totalFlips;
  final int mintedSections;
  final BigInt claimPriceLamports;
  final BigInt flipFeeLamports;
  final BigInt startsAtUnixSeconds;
  final int unlockIntervalSeconds;
  final int earlyUnlockFlips;
  final BigInt? previousSectionFlipCount;
  final String? treasury;
  final SectionSnapshot section;
  final int ownerShareBasisPoints;
  final String? bitMint;
  final String? bitReserve;
  final SectionPriceConfig? priceConfig;

  int get claimedSections => nextSection.clamp(0, sectionCount);

  BigInt get selectedSectionUnlockAt =>
      startsAtUnixSeconds + BigInt.from(section.index * unlockIntervalSeconds);

  bool canClaimSectionAt(BigInt unixSeconds) {
    if (section.isClaimed || section.index != nextSection) return false;
    final unlockedByTime = unixSeconds >= selectedSectionUnlockAt;
    final unlockedByActivity =
        section.index > 0 &&
        (previousSectionFlipCount ?? BigInt.zero) >=
            BigInt.from(earlyUnlockFlips);
    return unlockedByTime || unlockedByActivity;
  }

  GameSnapshot copyWith({
    bool? isDemo,
    int? nextSection,
    BigInt? totalFlips,
    int? mintedSections,
    SectionSnapshot? section,
  }) {
    return GameSnapshot(
      gameIndex: gameIndex,
      isDemo: isDemo ?? this.isDemo,
      nextSection: nextSection ?? this.nextSection,
      totalFlips: totalFlips ?? this.totalFlips,
      mintedSections: mintedSections ?? this.mintedSections,
      claimPriceLamports: claimPriceLamports,
      flipFeeLamports: flipFeeLamports,
      startsAtUnixSeconds: startsAtUnixSeconds,
      unlockIntervalSeconds: unlockIntervalSeconds,
      earlyUnlockFlips: earlyUnlockFlips,
      previousSectionFlipCount: previousSectionFlipCount,
      treasury: treasury,
      section: section ?? this.section,
      ownerShareBasisPoints: ownerShareBasisPoints,
      bitMint: bitMint,
      bitReserve: bitReserve,
      priceConfig: priceConfig,
    );
  }
}

String formatSectionIndex(int index) => index.toString().padLeft(3, '0');

String lamportsToSol(BigInt lamports) {
  final whole = lamports ~/ BigInt.from(1000000000);
  final fraction = (lamports % BigInt.from(1000000000))
      .toString()
      .padLeft(9, '0')
      .replaceFirst(RegExp(r'0+$'), '');
  return fraction.isEmpty ? '$whole' : '$whole.$fraction';
}

BigInt? trySolToLamports(String value) {
  final match = RegExp(r'^(\d+)(?:\.(\d{0,9}))?$').firstMatch(value.trim());
  if (match == null) return null;
  final whole = BigInt.parse(match.group(1)!);
  final fraction = (match.group(2) ?? '').padRight(9, '0');
  final lamports =
      whole * BigInt.from(1000000000) +
      (fraction.isEmpty ? BigInt.zero : BigInt.parse(fraction));
  final maximumU64 = (BigInt.one << 64) - BigInt.one;
  return lamports > BigInt.zero && lamports <= maximumU64 ? lamports : null;
}
