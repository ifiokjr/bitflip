import 'package:flutter/material.dart';

abstract final class BitflipColors {
  static const voidColor = Color(0xFF050B0A);
  static const panel = Color(0xFF0C1714);
  static const raised = Color(0xFF12241F);
  static const acid = Color(0xFFC9FF3D);
  static const coral = Color(0xFFFF654F);
  static const cyan = Color(0xFF5AE8FF);
  static const paper = Color(0xFFF4F0DF);
  static const muted = Color(0xFF8EA39D);
  static const line = Color(0xFF24433A);
  static const sectionPalette = [
    acid,
    coral,
    cyan,
    paper,
    Color(0xFFAD7CFF),
    Color(0xFFFFC857),
    Color(0xFF5386FF),
    Color(0xFFFF70C6),
  ];
}

ThemeData buildBitflipTheme() {
  final colorScheme = ColorScheme.fromSeed(
    seedColor: BitflipColors.acid,
    brightness: Brightness.dark,
    surface: BitflipColors.panel,
    primary: BitflipColors.acid,
    secondary: BitflipColors.coral,
    tertiary: BitflipColors.cyan,
  );
  final base = ThemeData(
    brightness: Brightness.dark,
    colorScheme: colorScheme,
    scaffoldBackgroundColor: BitflipColors.voidColor,
    fontFamily: 'monospace',
    useMaterial3: true,
  );
  final textTheme = base.textTheme.apply(
    bodyColor: BitflipColors.paper,
    displayColor: BitflipColors.paper,
  );

  return base.copyWith(
    textTheme: textTheme.copyWith(
      displayLarge: textTheme.displayLarge?.copyWith(
        fontSize: 68,
        height: 0.94,
        fontWeight: FontWeight.w900,
        letterSpacing: -4,
      ),
      displaySmall: textTheme.displaySmall?.copyWith(
        fontSize: 42,
        height: 0.98,
        fontWeight: FontWeight.w900,
        letterSpacing: -2.3,
      ),
      headlineSmall: textTheme.headlineSmall?.copyWith(
        fontWeight: FontWeight.w800,
        letterSpacing: -1,
      ),
      titleLarge: textTheme.titleLarge?.copyWith(
        fontWeight: FontWeight.w800,
        letterSpacing: -0.7,
      ),
      labelLarge: textTheme.labelLarge?.copyWith(
        fontWeight: FontWeight.w800,
        letterSpacing: 0.5,
      ),
      bodyMedium: textTheme.bodyMedium?.copyWith(
        color: BitflipColors.muted,
        height: 1.5,
      ),
    ),
    dividerColor: BitflipColors.line,
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        foregroundColor: BitflipColors.voidColor,
        backgroundColor: BitflipColors.acid,
        minimumSize: const Size(48, 52),
        padding: const EdgeInsets.symmetric(horizontal: 22, vertical: 15),
        shape: const RoundedRectangleBorder(),
        textStyle: const TextStyle(fontWeight: FontWeight.w900),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: BitflipColors.paper,
        side: const BorderSide(color: BitflipColors.line),
        minimumSize: const Size(48, 52),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
        shape: const RoundedRectangleBorder(),
        textStyle: const TextStyle(fontWeight: FontWeight.w800),
      ),
    ),
    iconTheme: const IconThemeData(color: BitflipColors.paper),
    tooltipTheme: const TooltipThemeData(
      decoration: BoxDecoration(color: BitflipColors.paper),
      textStyle: TextStyle(color: BitflipColors.voidColor),
    ),
  );
}
