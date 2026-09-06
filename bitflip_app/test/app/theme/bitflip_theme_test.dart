import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('release theme is deliberately dark with legible body contrast', () {
    final theme = buildBitflipTheme();

    expect(theme.brightness, Brightness.dark);
    expect(theme.scaffoldBackgroundColor, BitflipColors.voidColor);
    expect(
      ThemeData.estimateBrightnessForColor(theme.textTheme.bodyLarge!.color!),
      Brightness.light,
    );
  });
}
