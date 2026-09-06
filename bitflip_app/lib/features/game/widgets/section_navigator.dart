import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

class SectionNavigator extends HookWidget {
  const SectionNavigator({
    required this.selectedIndex,
    required this.nextSection,
    required this.mintedSections,
    super.key,
  });

  final int selectedIndex;
  final int nextSection;
  final int mintedSections;

  @override
  Widget build(BuildContext context) {
    return AspectRatio(
      aspectRatio: 1,
      child: GridView.builder(
        key: BitflipTestKeys.sectionNavigator,
        physics: const NeverScrollableScrollPhysics(),
        itemCount: sectionCount,
        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
          crossAxisCount: 16,
          mainAxisSpacing: 2,
          crossAxisSpacing: 2,
        ),
        itemBuilder: (context, index) {
          final isSelected = index == selectedIndex;
          final isMinted = index < mintedSections;
          final isClaimed = index < nextSection;
          final background = isMinted
              ? BitflipColors.cyan
              : isClaimed
              ? BitflipColors.acid.withValues(alpha: 0.55)
              : BitflipColors.raised;
          return ExcludeSemantics(
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: background,
                border: Border.all(
                  color: isSelected ? BitflipColors.coral : Colors.transparent,
                  width: isSelected ? 2.5 : 0,
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}
