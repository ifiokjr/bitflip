import 'dart:math' as math;

import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

class PixelCanvas extends HookWidget {
  const PixelCanvas({
    required this.bitmap,
    required this.queued,
    required this.cursor,
    required this.enabled,
    required this.onPixelPressed,
    super.key,
  });

  final PixelBitmap bitmap;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;
  final bool enabled;
  final ValueChanged<PixelCoordinate> onPixelPressed;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final side = math.min(constraints.maxWidth, 720.0);
        return Center(
          child: Semantics(
            container: true,
            image: true,
            label: context.l10n.canvasLabel,
            child: MouseRegion(
              cursor: enabled
                  ? SystemMouseCursors.precise
                  : SystemMouseCursors.forbidden,
              child: GestureDetector(
                key: BitflipTestKeys.canvas,
                behavior: HitTestBehavior.opaque,
                onTapDown: enabled
                    ? (details) {
                        final cell = side / sectionSide;
                        final x = (details.localPosition.dx / cell)
                            .floor()
                            .clamp(0, sectionSide - 1);
                        final y = (details.localPosition.dy / cell)
                            .floor()
                            .clamp(0, sectionSide - 1);
                        onPixelPressed(PixelCoordinate(x, y));
                      }
                    : null,
                child: RepaintBoundary(
                  child: SizedBox.square(
                    dimension: side,
                    child: CustomPaint(
                      painter: PixelCanvasPainter(
                        bitmap: bitmap,
                        queued: queued,
                        cursor: cursor,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class PixelCanvasPainter extends CustomPainter {
  PixelCanvasPainter({
    required this.bitmap,
    required this.queued,
    required this.cursor,
  });

  final PixelBitmap bitmap;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;

  @override
  void paint(Canvas canvas, Size size) {
    final cell = size.width / sectionSide;
    final background = Paint()..color = BitflipColors.voidColor;
    canvas.drawRect(Offset.zero & size, background);

    final pixelPaint = Paint()..color = BitflipColors.acid;
    final queuedOnPaint = Paint()..color = BitflipColors.coral;
    final queuedOffPaint = Paint()
      ..color = BitflipColors.cyan.withValues(alpha: 0.32);
    for (var y = 0; y < sectionSide; y++) {
      for (var x = 0; x < sectionSide; x++) {
        final coordinate = PixelCoordinate(x, y);
        final queuedHere = queued.contains(coordinate);
        final wasOn = bitmap.isOn(x, y);
        final isOn = queuedHere ? !wasOn : wasOn;
        if (!isOn && !queuedHere) continue;
        final rect = Rect.fromLTWH(x * cell, y * cell, cell, cell);
        canvas.drawRect(
          rect.deflate(cell > 5 ? 0.55 : 0.18),
          queuedHere
              ? isOn
                    ? queuedOnPaint
                    : queuedOffPaint
              : pixelPaint,
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

    final selected = cursor;
    if (selected != null) {
      final cursorPaint = Paint()
        ..color = BitflipColors.paper
        ..style = PaintingStyle.stroke
        ..strokeWidth = math.max(1.0, cell * 0.2);
      canvas.drawRect(
        Rect.fromLTWH(
          selected.x * cell,
          selected.y * cell,
          cell,
          cell,
        ).deflate(0.5),
        cursorPaint,
      );
    }

    final framePaint = Paint()
      ..color = BitflipColors.acid
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;
    canvas.drawRect(Offset.zero & size, framePaint);
  }

  @override
  bool shouldRepaint(PixelCanvasPainter oldDelegate) {
    return oldDelegate.bitmap != bitmap ||
        oldDelegate.queued != queued ||
        oldDelegate.cursor != cursor;
  }
}
