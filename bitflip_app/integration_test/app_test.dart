import 'package:bitflip_app/app/app.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:integration_test/integration_test.dart';

import '../test/support/fake_bitflip_repository.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('visitor queues and commits a pixel batch', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          bitflipRepositoryProvider.overrideWithValue(
            FakeBitflipRepository(isWalletSupported: false),
          ),
        ],
        child: const BitflipApp(),
      ),
    );
    await tester.pump(const Duration(milliseconds: 500));

    final openCanvasButton = find.text('OPEN CANVAS');
    await tester.ensureVisible(openCanvasButton);
    await tester.pump(const Duration(milliseconds: 600));
    await tester.tap(openCanvasButton);
    await tester.pump(const Duration(milliseconds: 600));
    final canvas = find.byKey(BitflipTestKeys.canvas);
    expect(canvas, findsOneWidget);
    await tester.ensureVisible(canvas);
    await tester.pump(const Duration(milliseconds: 600));

    await tester.tapAt(tester.getCenter(canvas) + const Offset(2, 2));
    await tester.pump();
    expect(find.byKey(BitflipTestKeys.commitFlips), findsOneWidget);

    final commitButton = find.byKey(BitflipTestKeys.commitFlips);
    await tester.ensureVisible(commitButton);
    await tester.pump(const Duration(milliseconds: 600));
    await tester.tap(commitButton);
    await tester.pump(const Duration(milliseconds: 600));
    expect(
      find.text('Moves committed as one atomic transaction.'),
      findsOneWidget,
    );
  });
}
