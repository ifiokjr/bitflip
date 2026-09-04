import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';

enum SectionLifecycle { unclaimed, active, sealed, minted }

final class SectionSnapshot {
  const SectionSnapshot({
    required this.index,
    required this.lifecycle,
    required this.bitmap,
    required this.owner,
    required this.flipCount,
    required this.revision,
    this.assetId,
  });

  final int index;
  final SectionLifecycle lifecycle;
  final PixelBitmap bitmap;
  final String? owner;
  final BigInt flipCount;
  final BigInt revision;
  final String? assetId;

  bool get isEditable => lifecycle == SectionLifecycle.active;
  bool get isClaimed => lifecycle != SectionLifecycle.unclaimed;

  SectionSnapshot copyWith({
    SectionLifecycle? lifecycle,
    PixelBitmap? bitmap,
    String? owner,
    BigInt? flipCount,
    BigInt? revision,
    String? assetId,
  }) {
    return SectionSnapshot(
      index: index,
      lifecycle: lifecycle ?? this.lifecycle,
      bitmap: bitmap ?? this.bitmap,
      owner: owner ?? this.owner,
      flipCount: flipCount ?? this.flipCount,
      revision: revision ?? this.revision,
      assetId: assetId ?? this.assetId,
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
    required this.treasury,
    required this.section,
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
        assetId: sectionIndex < 5 ? 'cnft:$sectionIndex' : null,
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
  final String? treasury;
  final SectionSnapshot section;

  int get claimedSections => nextSection.clamp(0, sectionCount);

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
      treasury: treasury,
      section: section ?? this.section,
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
