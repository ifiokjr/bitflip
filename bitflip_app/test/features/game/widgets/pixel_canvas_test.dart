import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/pixel_colour_map.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';
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

  group('PixelCanvasPainter', () {
    for (final scene in [_Scene.openCanvas(), _Scene.colourRound()]) {
      test('${scene.name} renders identically to per-cell drawRect', () async {
        final batched = await _rasterise(scene.painter);
        final reference = await _rasterise(_PerCellReferencePainter(scene));

        expect(batched, orderedEquals(reference));
      });
    }

    test('issues one draw call per colour instead of one per pixel', () {
      final canvas = _RecordingCanvas();

      _Scene.colourRound().painter.paint(canvas, const Size.square(640));

      // Background, cursor, and frame are the only rectangles; every pixel
      // colour (eight palette entries plus acid, coral, cyan, and the active
      // preview) collapses into a single point batch.
      expect(canvas.calls[#drawRect], 3);
      expect(canvas.calls[#drawRawPoints], lessThanOrEqualTo(12));
      expect(canvas.calls[#drawLine], 18);
    });

    test('skips repainting when fresh instances carry the same content', () {
      final bytes = PixelBitmap.demo(4).bytes;
      final colours = PixelColourMap.empty().painted(const [
        PixelCoordinate(2, 2),
      ], SectionColour.cyan).codes;
      PixelCanvasPainter painter({int toggleIndex = -1}) {
        final mutable = Uint8List.fromList(bytes);

        if (toggleIndex >= 0) mutable[toggleIndex] ^= 1;

        return PixelCanvasPainter(
          bitmap: PixelBitmap.fromBytes(mutable),
          colourMap: PixelColourMap.fromBytes(Uint8List.fromList(colours)),
          queued: {const PixelCoordinate(1, 1)},
          cursor: const PixelCoordinate(3, 3),
          activeColour: null,
        );
      }

      expect(painter().shouldRepaint(painter()), isFalse);
      expect(painter(toggleIndex: 7).shouldRepaint(painter()), isTrue);
      expect(
        PixelCanvasPainter(
          bitmap: PixelBitmap.fromBytes(bytes),
          colourMap: null,
          queued: const {},
          cursor: null,
          activeColour: null,
        ).shouldRepaint(painter()),
        isTrue,
      );
    });
  });
}

Future<List<int>> _rasterise(CustomPainter painter) async {
  const side = 640;
  final recorder = ui.PictureRecorder();
  painter.paint(Canvas(recorder), const Size.square(640));
  final picture = recorder.endRecording();
  final image = await picture.toImage(side, side);
  final pixels = (await image.toByteData())!.buffer.asUint8List();
  image.dispose();
  picture.dispose();

  return pixels;
}

final class _Scene {
  _Scene._(this.name, this.painter);

  factory _Scene.openCanvas() {
    final bitmap = PixelBitmap.demo(3);

    return _Scene._(
      'open canvas',
      PixelCanvasPainter(
        bitmap: bitmap,
        colourMap: null,
        queued: _queuedCells(bitmap),
        cursor: const PixelCoordinate(5, 5),
        activeColour: null,
      ),
    );
  }

  factory _Scene.colourRound() {
    final bitmap = PixelBitmap.demo(3);
    var colourMap = PixelColourMap.empty();

    for (final colour in SectionColour.values) {
      colourMap = colourMap.painted([
        for (var x = 0; x < 8; x++) PixelCoordinate(x, colour.code * 7),
      ], colour);
    }

    return _Scene._(
      'colour round',
      PixelCanvasPainter(
        bitmap: bitmap,
        colourMap: colourMap,
        queued: _queuedCells(bitmap),
        cursor: const PixelCoordinate(60, 1),
        activeColour: SectionColour.violet,
      ),
    );
  }

  final String name;
  final PixelCanvasPainter painter;

  /// Eight queued cells that are currently on and eight that are off, so the
  /// coral, translucent cyan, and active-colour previews all take part.
  static Set<PixelCoordinate> _queuedCells(PixelBitmap bitmap) {
    final on = <PixelCoordinate>[];
    final off = <PixelCoordinate>[];

    for (var y = 0; y < sectionSide && (on.length < 8 || off.length < 8); y++) {
      for (var x = 0; x < sectionSide; x++) {
        final target = bitmap.isOn(x, y) ? on : off;

        if (target.length < 8) target.add(PixelCoordinate(x, y));
      }
    }

    return {...on, ...off};
  }
}

/// The original one-`drawRect`-per-cell algorithm, kept as the rendering
/// oracle for the batched painter.
final class _PerCellReferencePainter extends CustomPainter {
  _PerCellReferencePainter(this.scene);

  final _Scene scene;

  @override
  void paint(Canvas canvas, Size size) {
    final painter = scene.painter;
    final cell = size.width / sectionSide;
    final inset = cell > 5 ? 0.55 : 0.18;
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = BitflipColors.voidColor,
    );
    final activeColour = painter.activeColour;
    final colourPaint = Paint()
      ..color = activeColour == null
          ? BitflipColors.coral
          : BitflipColors.sectionPalette[activeColour.code];

    for (var y = 0; y < sectionSide; y++) {
      for (var x = 0; x < sectionSide; x++) {
        final rect = Rect.fromLTWH(
          x * cell,
          y * cell,
          cell,
          cell,
        ).deflate(inset);
        final queuedHere = painter.queued.contains(PixelCoordinate(x, y));

        if (queuedHere && activeColour != null) {
          canvas.drawRect(rect, colourPaint);
          continue;
        }

        final committed = painter.colourMap?.colourAt(x, y);

        if (committed != null) {
          canvas.drawRect(
            rect,
            Paint()..color = BitflipColors.sectionPalette[committed.code],
          );
          continue;
        }

        final wasOn = painter.bitmap.isOn(x, y);
        final isOn = queuedHere ? !wasOn : wasOn;

        if (!isOn && !queuedHere) continue;
        canvas.drawRect(
          rect,
          Paint()
            ..color = queuedHere
                ? isOn
                      ? BitflipColors.coral
                      : BitflipColors.cyan.withValues(alpha: 0.32)
                : BitflipColors.acid,
        );
      }
    }

    final gridPaint = Paint()
      ..color = BitflipColors.line.withValues(alpha: 0.52)
      ..strokeWidth = 1;

    for (var index = 0; index <= sectionSide; index += 8) {
      final position = index * cell;
      canvas
        ..drawLine(
          Offset(position, 0),
          Offset(position, size.height),
          gridPaint,
        )
        ..drawLine(
          Offset(0, position),
          Offset(size.width, position),
          gridPaint,
        );
    }

    if (painter.cursor case final selected?) {
      canvas.drawRect(
        Rect.fromLTWH(
          selected.x * cell,
          selected.y * cell,
          cell,
          cell,
        ).deflate(0.5),
        Paint()
          ..color = BitflipColors.paper
          ..style = PaintingStyle.stroke
          ..strokeWidth = math.max(1.0, cell * 0.2),
      );
    }

    canvas.drawRect(
      Offset.zero & size,
      Paint()
        ..color = BitflipColors.acid
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(_PerCellReferencePainter oldDelegate) => true;
}

/// Counts every drawing command instead of rasterising it.
final class _RecordingCanvas implements Canvas {
  final Map<Symbol, int> calls = {};

  @override
  dynamic noSuchMethod(Invocation invocation) {
    calls.update(
      invocation.memberName,
      (count) => count + 1,
      ifAbsent: () => 1,
    );

    return null;
  }
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
