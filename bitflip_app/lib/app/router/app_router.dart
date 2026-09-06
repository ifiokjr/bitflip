import 'package:bitflip_app/features/game/widgets/game_screen.dart';
import 'package:bitflip_app/features/legal/widgets/legal_screen.dart';
import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'app_router.g.dart';

final GlobalKey<NavigatorState> _rootNavigatorKey = GlobalKey<NavigatorState>();

@Riverpod(keepAlive: true)
GoRouter appRouter(Ref ref) {
  return GoRouter(
    navigatorKey: _rootNavigatorKey,
    initialLocation: '/',
    routes: [
      GoRoute(path: '/', builder: (context, state) => const GameScreen()),
      GoRoute(
        path: '/privacy',
        builder: (context, state) =>
            const LegalScreen(document: LegalDocument.privacy),
      ),
      GoRoute(
        path: '/terms',
        builder: (context, state) =>
            const LegalScreen(document: LegalDocument.terms),
      ),
      GoRoute(
        path: '/support',
        builder: (context, state) =>
            const LegalScreen(document: LegalDocument.support),
      ),
    ],
  );
}
