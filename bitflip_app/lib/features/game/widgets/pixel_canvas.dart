import 'dart:math' as math;

import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

class PixelCanvas extends HookWidget {
  const PixelCanvas({
    required this.bitmap,
    required this.queued,
    required this.cursor,
    required this.enabled,
    required this.onPixelPressed,
    required this.onCursorMoved,
    super.key,
  });

  final PixelBitmap bitmap;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;
  final bool enabled;
  final ValueChanged<PixelCoordinate> onPixelPressed;
  final ValueChanged<PixelCoordinate> onCursorMoved;

  @override
  Widget build(BuildContext context) {
    final focusNode = useFocusNode();
    final transform = useMemoized(TransformationController.new);
    useEffect(() => transform.dispose, [transform]);

    void changeZoom(double delta) {
      final current = transform.value.getMaxScaleOnAxis();
      final next = (current + delta).clamp(1.0, 12.0);
      transform.value = Matrix4.diagonal3Values(next, next, 1);
    }

    KeyEventResult handleKey(FocusNode _, KeyEvent event) {
      if (event is! KeyDownEvent) return KeyEventResult.ignored;
      final selected = cursor ?? const PixelCoordinate(0, 0);
      final next = switch (event.logicalKey) {
        LogicalKeyboardKey.arrowLeft => PixelCoordinate(
          math.max(0, selected.x - 1),
          selected.y,
        ),
        LogicalKeyboardKey.arrowRight => PixelCoordinate(
          math.min(sectionSide - 1, selected.x + 1),
          selected.y,
        ),
        LogicalKeyboardKey.arrowUp => PixelCoordinate(
          selected.x,
          math.max(0, selected.y - 1),
        ),
        LogicalKeyboardKey.arrowDown => PixelCoordinate(
          selected.x,
          math.min(sectionSide - 1, selected.y + 1),
        ),
        _ => null,
      };
      if (next != null) {
        onCursorMoved(next);
        return KeyEventResult.handled;
      }
      if (enabled &&
          (event.logicalKey == LogicalKeyboardKey.space ||
              event.logicalKey == LogicalKeyboardKey.enter)) {
        onPixelPressed(selected);
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        final side = math.min(constraints.maxWidth, 720.0);
        return Center(
          child: SizedBox(
            width: side,
            child: Column(
              children: [
                Align(
                  alignment: Alignment.centerRight,
                  child: SegmentedButton<_ZoomAction>(
                    showSelectedIcon: false,
                    emptySelectionAllowed: true,
                    segments: [
                      ButtonSegment(
                        value: _ZoomAction.out,
                        icon: const Icon(
                          Icons.remove_rounded,
                          key: BitflipTestKeys.canvasZoomOut,
                        ),
                        tooltip: context.l10n.zoomOut,
                      ),
                      ButtonSegment(
                        value: _ZoomAction.reset,
                        icon: const Icon(
                          Icons.center_focus_strong_rounded,
                          key: BitflipTestKeys.canvasZoomReset,
                        ),
                        tooltip: context.l10n.resetZoom,
                      ),
                      ButtonSegment(
                        value: _ZoomAction.increase,
                        icon: const Icon(
                          Icons.add_rounded,
                          key: BitflipTestKeys.canvasZoomIn,
                        ),
                        tooltip: context.l10n.zoomIn,
                      ),
                    ],
                    selected: const {},
                    onSelectionChanged: (actions) {
                      switch (actions.single) {
                        case _ZoomAction.out:
                          changeZoom(-1);
                        case _ZoomAction.reset:
                          transform.value = Matrix4.identity();
                        case _ZoomAction.increase:
                          changeZoom(1);
                      }
                    },
                  ),
                ),
                const SizedBox(height: 8),
                SizedBox.square(
                  dimension: side,
                  child: Semantics(
                    container: true,
                    focusable: true,
                    label: context.l10n.canvasLabel,
                    child: Focus(
                      focusNode: focusNode,
                      onKeyEvent: handleKey,
                      child: InteractiveViewer(
                        transformationController: transform,
                        minScale: 1,
                        maxScale: 12,
                        child: MouseRegion(
                          cursor: enabled
                              ? SystemMouseCursors.precise
                              : SystemMouseCursors.forbidden,
                          child: GestureDetector(
                            key: BitflipTestKeys.canvas,
                            behavior: HitTestBehavior.opaque,
                            onTapDown: enabled
                                ? (details) {
                                    focusNode.requestFocus();
                                    final cell = side / sectionSide;
                                    final x = (details.localPosition.dx / cell)
                                        .floor()
                                        .clamp(0, sectionSide - 1);
                                    final y = (details.localPosition.dy / cell)
                                        .floor()
                                        .clamp(0, sectionSide - 1);
                                    onPixelPressed(PixelCoordinate(x, y));
                                  }
                                : (_) => focusNode.requestFocus(),
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
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

enum _ZoomAction { out, reset, increase }

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
