import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' show PointMode;

import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/pixel_colour_map.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/foundation.dart' show setEquals;
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
    this.activeColour,
    this.colourMap,
    super.key,
  });

  final PixelBitmap bitmap;
  final PixelColourMap? colourMap;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;
  final bool enabled;
  final ValueChanged<PixelCoordinate> onPixelPressed;
  final ValueChanged<PixelCoordinate> onCursorMoved;
  final SectionColour? activeColour;

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
                                    colourMap: colourMap,
                                    queued: queued,
                                    cursor: cursor,
                                    activeColour: activeColour,
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
    required this.colourMap,
    required this.queued,
    required this.cursor,
    required this.activeColour,
  });

  final PixelBitmap bitmap;
  final PixelColourMap? colourMap;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;
  final SectionColour? activeColour;

  @override
  void paint(Canvas canvas, Size size) {
    final cell = size.width / sectionSide;
    final background = Paint()..color = BitflipColors.voidColor;
    canvas.drawRect(Offset.zero & size, background);

    final queuedMask = Uint8List(sectionPixelCount);

    for (final coordinate in queued) {
      queuedMask[coordinate.linearIndex] = 1;
    }

    final committed = List.generate(
      SectionColour.values.length,
      (_) => _CellBatch(),
      growable: false,
    );
    final on = _CellBatch();
    final queuedOn = _CellBatch();
    final queuedOff = _CellBatch();
    final preview = _CellBatch();
    final bytes = bitmap.bytes;

    for (var index = 0; index < sectionPixelCount; index++) {
      final queuedHere = queuedMask[index] != 0;

      if (queuedHere && activeColour != null) {
        preview.add(index);
        continue;
      }

      final committedCode = colourMap?.codeAt(index) ?? noSectionColour;

      if (committedCode != noSectionColour) {
        committed[committedCode].add(index);
        continue;
      }

      final wasOn = (bytes[index >> 3] & (1 << (index & 7))) != 0;
      final isOn = queuedHere ? !wasOn : wasOn;

      if (!isOn && !queuedHere) continue;
      (queuedHere ? (isOn ? queuedOn : queuedOff) : on).add(index);
    }

    final inset = cell > 5 ? 0.55 : 0.18;
    final cellSide = math.max(0.0, cell - inset * 2);

    void drawBatch(_CellBatch batch, Color colour) {
      if (batch.isEmpty) return;
      canvas.drawRawPoints(
        PointMode.points,
        batch.centres(cell),
        Paint()
          ..color = colour
          ..strokeCap = StrokeCap.square
          ..strokeWidth = cellSide,
      );
    }

    for (var code = 0; code < committed.length; code++) {
      drawBatch(committed[code], BitflipColors.sectionPalette[code]);
    }

    drawBatch(on, BitflipColors.acid);
    drawBatch(queuedOn, BitflipColors.coral);
    drawBatch(queuedOff, BitflipColors.cyan.withValues(alpha: 0.32));

    if (activeColour case final colour?) {
      drawBatch(preview, BitflipColors.sectionPalette[colour.code]);
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
        oldDelegate.colourMap != colourMap ||
        !setEquals(oldDelegate.queued, queued) ||
        oldDelegate.cursor != cursor ||
        oldDelegate.activeColour != activeColour;
  }
}

/// Cells that share one paint. Square stroke caps turn each centre point into
/// the same inset square `drawRect` produced, so a whole colour is one call.
final class _CellBatch {
  final List<int> _linearIndices = [];

  bool get isEmpty => _linearIndices.isEmpty;

  void add(int linearIndex) => _linearIndices.add(linearIndex);

  Float32List centres(double cell) {
    final points = Float32List(_linearIndices.length * 2);
    final half = cell / 2;

    for (var i = 0; i < _linearIndices.length; i++) {
      final index = _linearIndices[i];
      points[i * 2] = (index % sectionSide) * cell + half;
      points[i * 2 + 1] = (index ~/ sectionSide) * cell + half;
    }

    return points;
  }
}
