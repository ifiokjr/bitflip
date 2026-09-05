import 'package:flutter/widgets.dart';

abstract final class BitflipTestKeys {
  static const connectWallet = Key('connect-wallet');
  static const canvas = Key('pixel-canvas');
  static const canvasZoomIn = Key('canvas-zoom-in');
  static const canvasZoomOut = Key('canvas-zoom-out');
  static const canvasZoomReset = Key('canvas-zoom-reset');
  static const pixelX = Key('pixel-x');
  static const pixelY = Key('pixel-y');
  static const toggleCoordinate = Key('toggle-coordinate');
  static const claimSection = Key('claim-section');
  static const commitFlips = Key('commit-flips');
  static const clearFlips = Key('clear-flips');
  static const nextSection = Key('next-section');
  static const previousSection = Key('previous-section');
  static const sealSection = Key('seal-section');
  static const confirmSeal = Key('confirm-seal');
  static const mintSection = Key('mint-section');
  static const sectionNavigator = Key('section-navigator');
  static const sectionPicker = Key('section-picker');
  static const viewResult = Key('view-result');
}
