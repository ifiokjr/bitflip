import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/widgets/pixel_canvas.dart';
import 'package:bitflip_app/l10n/generated/app_localizations.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('canvas supports keyboard cursor movement and toggling', (
    tester,
  ) async {
    var cursor = const PixelCoordinate(3, 4);
    final toggled = <PixelCoordinate>[];
    await tester.pumpWidget(
      _TestApp(
        child: StatefulBuilder(
          builder: (context, setState) => PixelCanvas(
            bitmap: PixelBitmap.empty(),
            queued: const {},
            cursor: cursor,
            enabled: true,
            onPixelPressed: toggled.add,
            onCursorMoved: (coordinate) {
              setState(() => cursor = coordinate);
            },
          ),
        ),
      ),
    );

    await tester.tap(find.byKey(BitflipTestKeys.canvas));
    toggled.clear();
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.space);

    expect(cursor, const PixelCoordinate(4, 4));
    expect(toggled, [const PixelCoordinate(4, 4)]);
    expect(tester.takeException(), isNull);
  });

  testWidgets('canvas exposes zoom controls and screen-reader guidance', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await tester.pumpWidget(
      _TestApp(
        child: PixelCanvas(
          bitmap: PixelBitmap.empty(),
          queued: const {},
          cursor: null,
          enabled: true,
          onPixelPressed: (_) {},
          onCursorMoved: (_) {},
        ),
      ),
    );

    expect(find.byKey(BitflipTestKeys.canvasZoomIn), findsOneWidget);
    expect(find.byKey(BitflipTestKeys.canvasZoomOut), findsOneWidget);
    expect(find.byKey(BitflipTestKeys.canvasZoomReset), findsOneWidget);
    expect(find.bySemanticsLabel(RegExp('Use arrow keys')), findsOneWidget);

    await tester.tap(find.byKey(BitflipTestKeys.canvasZoomIn));
    await tester.tap(find.byKey(BitflipTestKeys.canvasZoomReset));
    expect(tester.takeException(), isNull);
    semantics.dispose();
  });
}

class _TestApp extends StatelessWidget {
  const _TestApp({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      theme: buildBitflipTheme(),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(body: SingleChildScrollView(child: child)),
    );
  }
}
