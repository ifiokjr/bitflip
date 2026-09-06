import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/legal/widgets/legal_screen.dart';
import 'package:bitflip_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  for (final (document, title) in [
    (LegalDocument.privacy, 'Privacy'),
    (LegalDocument.terms, 'Terms'),
    (LegalDocument.support, 'Support'),
  ]) {
    testWidgets('$title is readable at a compact width and large text scale', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(390, 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        MaterialApp(
          theme: buildBitflipTheme(),
          darkTheme: buildBitflipTheme(),
          themeMode: ThemeMode.dark,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(context)
                .copyWith(textScaler: const TextScaler.linear(1.6)),
            child: child!,
          ),
          home: LegalScreen(document: document),
        ),
      );

      expect(find.text(title), findsOneWidget);
      expect(find.text('BACK TO BITFLIP'), findsOneWidget);
      expect(tester.takeException(), isNull);
    });
  }
}
