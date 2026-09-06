import 'package:bitflip_app/app/app.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../../../support/fake_bitflip_repository.dart';

void main() {
  for (final size in [const Size(390, 844), const Size(1440, 1100)]) {
    testWidgets('game screen renders without overflow at ${size.width}px', (
      tester,
    ) async {
      await tester.binding.setSurfaceSize(size);
      addTearDown(() => tester.binding.setSurfaceSize(null));
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
      await tester.pumpAndSettle();

      expect(
        find.text('A million pixels. One irreversible canvas.'),
        findsOneWidget,
      );
      expect(find.text('OPEN CANVAS'), findsOneWidget);
      await tester.scrollUntilVisible(
        find.byKey(BitflipTestKeys.sectionPicker),
        500,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.byKey(BitflipTestKeys.sectionPicker), findsOneWidget);
      expect(
        find.descendant(
          of: find.byKey(BitflipTestKeys.sectionNavigator),
          matching: find.byType(InkWell),
        ),
        findsNothing,
        reason:
            'the compact overview is visual; the labelled picker is the '
            'accessible section control',
      );
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets(
    'web wallet picker connects the selected Wallet Standard wallet',
    (tester) async {
      final repository = FakeBitflipRepository(
        walletAddress: null,
        availableWallets: const [
          BitflipWalletOption(id: 'phantom', name: 'Phantom'),
          BitflipWalletOption(id: 'backpack', name: 'Backpack'),
        ],
      );
      await tester.binding.setSurfaceSize(const Size(900, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        ProviderScope(
          overrides: [bitflipRepositoryProvider.overrideWithValue(repository)],
          child: const BitflipApp(),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(BitflipTestKeys.connectWallet).first);
      await tester.pumpAndSettle();

      expect(find.text('Choose a wallet'), findsOneWidget);
      expect(find.text('Phantom'), findsOneWidget);
      expect(find.text('Backpack'), findsOneWidget);

      await tester.tap(find.text('Backpack'));
      await tester.pumpAndSettle();

      expect(repository.lastWalletId, 'backpack');
      expect(find.text('Choose a wallet'), findsNothing);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('web wallet picker explains when no compatible wallet exists', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(900, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          bitflipRepositoryProvider.overrideWithValue(
            FakeBitflipRepository(
              walletAddress: null,
              availableWallets: const [],
            ),
          ),
        ],
        child: const BitflipApp(),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(BitflipTestKeys.connectWallet).first);
    await tester.pumpAndSettle();

    expect(find.text('No compatible wallet found'), findsOneWidget);
    expect(
      find.textContaining('Wallet Standard browser wallet'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('failed live loads never substitute demo artwork', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(900, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          bitflipRepositoryProvider.overrideWithValue(
            FakeBitflipRepository(
              snapshot: _liveSnapshot(),
              loadError: StateError('offline'),
            ),
          ),
        ],
        child: const BitflipApp(),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(
      find.textContaining('No demo data has been substituted'),
      findsOneWidget,
    );
    expect(find.text('SIGNAL MIRROR'), findsNothing);
    expect(tester.takeException(), isNull);
  });
}

GameSnapshot _liveSnapshot() => GameSnapshot.empty(gameIndex: 0).copyWith(
  section: GameSnapshot.empty(gameIndex: 0).section.copyWith(
    lifecycle: SectionLifecycle.active,
    owner: 'DemoWallet111111111111111111111111111111111',
  ),
);
