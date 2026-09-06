import 'package:bitflip_app/app/router/app_router.dart';
import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class BitflipApp extends HookConsumerWidget {
  const BitflipApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(appRouterProvider);
    final theme = buildBitflipTheme();

    return MaterialApp.router(
      title: 'Bitflip',
      debugShowCheckedModeBanner: false,
      theme: theme,
      darkTheme: theme,
      themeMode: ThemeMode.dark,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      routerConfig: router,
    );
  }
}
