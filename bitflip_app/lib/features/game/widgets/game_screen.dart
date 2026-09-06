import 'dart:async';
import 'dart:math' as math;

import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/widgets/game_console.dart';
import 'package:bitflip_app/features/game/widgets/pixel_canvas.dart';
import 'package:bitflip_app/features/game/widgets/section_navigator.dart';
import 'package:bitflip_app/features/wallet/widgets/wallet_sheet.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';

class GameScreen extends HookConsumerWidget {
  const GameScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(gameControllerProvider);
    final controller = ref.read(gameControllerProvider.notifier);
    final workspaceKey = useMemoized(() => GlobalKey());
    final requestedSection = int.tryParse(
      GoRouterState.of(context).uri.queryParameters['section'] ?? '',
    );
    useEffect(() {
      final initialRefresh = Timer(Duration.zero, () {
        if (requestedSection != null &&
            requestedSection >= 0 &&
            requestedSection < sectionCount) {
          unawaited(controller.selectSection(requestedSection));
        } else {
          unawaited(controller.refresh());
        }
      });
      final liveRefresh = Timer.periodic(
        const Duration(seconds: 12),
        (_) => unawaited(controller.refresh()),
      );
      return () {
        initialRefresh.cancel();
        liveRefresh.cancel();
      };
    }, [controller, requestedSection]);

    return Scaffold(
      body: Stack(
        children: [
          const Positioned.fill(child: _Atmosphere()),
          Positioned.fill(
            child: SafeArea(
              child: SelectionArea(
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: _PageWidth(
                        child: _Header(
                          state: state,
                          onConnect: () =>
                              unawaited(_connectWallet(context, controller)),
                          onWallet: () => unawaited(
                            showBitflipWalletSheet(
                              context,
                              state: state,
                              controller: controller,
                            ),
                          ),
                        ),
                      ),
                    ),
                    SliverToBoxAdapter(
                      child: _PageWidth(
                        child: _Hero(
                          onOpenCanvas: () {
                            final target = workspaceKey.currentContext;
                            if (target != null) {
                              unawaited(
                                Scrollable.ensureVisible(
                                  target,
                                  duration: const Duration(milliseconds: 520),
                                  curve: Curves.easeOutCubic,
                                ),
                              );
                            }
                          },
                        ),
                      ),
                    ),
                    SliverToBoxAdapter(
                      child: _SignalTicker(snapshot: state.snapshot),
                    ),
                    SliverToBoxAdapter(
                      key: workspaceKey,
                      child: _PageWidth(
                        padding: const EdgeInsets.only(top: 72),
                        child: _GameWorkspace(
                          state: state,
                          controller: controller,
                        ),
                      ),
                    ),
                    SliverToBoxAdapter(
                      child: _PageWidth(
                        padding: const EdgeInsets.only(top: 96),
                        child: _HowItWorks(),
                      ),
                    ),
                    SliverToBoxAdapter(
                      child: _PageWidth(
                        padding: const EdgeInsets.only(top: 84, bottom: 44),
                        child: _Footer(),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _PageWidth extends HookWidget {
  const _PageWidth({required this.child, this.padding = EdgeInsets.zero});

  final Widget child;
  final EdgeInsets padding;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 1440),
        child: Padding(
          padding: EdgeInsets.fromLTRB(
            MediaQuery.sizeOf(context).width < 700 ? 18 : 36,
            padding.top,
            MediaQuery.sizeOf(context).width < 700 ? 18 : 36,
            padding.bottom,
          ),
          child: child,
        ),
      ),
    );
  }
}

class _Header extends HookWidget {
  const _Header({
    required this.state,
    required this.onConnect,
    required this.onWallet,
  });

  final GameViewState state;
  final VoidCallback onConnect;
  final VoidCallback onWallet;

  @override
  Widget build(BuildContext context) {
    final wallet = state.walletAddress;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 22),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final compact = constraints.maxWidth < 460;
          return Row(
            children: [
              const _BitflipMark(size: 34),
              if (!compact) ...[
                const SizedBox(width: 13),
                Text(
                  context.l10n.appName,
                  style: Theme.of(context).textTheme.titleLarge
                      ?.copyWith(letterSpacing: -1.4),
                ),
              ],
              const Spacer(),
              if (wallet != null)
                _StatusPill(
                  key: BitflipTestKeys.walletDetails,
                  label: _shortAddress(wallet),
                  color: BitflipColors.cyan,
                  onTap: onWallet,
                )
              else if (state.isWalletSupported)
                compact
                    ? IconButton.outlined(
                        key: BitflipTestKeys.connectWallet,
                        onPressed: state.isBusy ? null : onConnect,
                        tooltip: context.l10n.connectWallet,
                        icon: const Icon(
                          Icons.account_balance_wallet_outlined,
                          size: 18,
                        ),
                      )
                    : OutlinedButton.icon(
                        key: BitflipTestKeys.connectWallet,
                        onPressed: state.isBusy ? null : onConnect,
                        icon: const Icon(
                          Icons.account_balance_wallet_outlined,
                          size: 18,
                        ),
                        label: Text(context.l10n.connectWallet),
                      )
              else
                _StatusPill(
                  label: state.snapshot.isDemo
                      ? context.l10n.demoMode
                      : context.l10n.chainMode,
                  color: state.snapshot.isDemo
                      ? BitflipColors.coral
                      : BitflipColors.acid,
                ),
            ],
          );
        },
      ),
    );
  }
}

class _Hero extends HookWidget {
  const _Hero({required this.onOpenCanvas});

  final VoidCallback onOpenCanvas;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(0, 64, 0, 82),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final wide = constraints.maxWidth >= 900;
          final copy = Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _Eyebrow(label: context.l10n.liveNetwork),
              const SizedBox(height: 26),
              Text(
                context.l10n.heroTitle,
                style: wide
                    ? Theme.of(context).textTheme.displayLarge
                    : Theme.of(context).textTheme.displaySmall,
              ),
              const SizedBox(height: 26),
              ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 680),
                child: Text(
                  context.l10n.heroBody,
                  style: Theme.of(context).textTheme.bodyLarge
                      ?.copyWith(color: BitflipColors.muted, height: 1.55),
                ),
              ),
              const SizedBox(height: 34),
              FilledButton.icon(
                onPressed: onOpenCanvas,
                icon: const Icon(Icons.south_east_rounded),
                label: Text(context.l10n.openCanvas),
              ),
            ],
          );
          if (!wide) return copy;
          return Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Expanded(flex: 7, child: copy),
              const SizedBox(width: 52),
              const Expanded(flex: 3, child: _HeroSignal()),
            ],
          );
        },
      ),
    );
  }
}

class _HeroSignal extends HookWidget {
  const _HeroSignal();

  @override
  Widget build(BuildContext context) {
    return AspectRatio(
      aspectRatio: 1,
      child: Transform.rotate(
        angle: math.pi / 18,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: BitflipColors.panel,
            border: Border.all(color: BitflipColors.line),
            boxShadow: [
              BoxShadow(
                color: BitflipColors.acid.withValues(alpha: 0.12),
                blurRadius: 60,
                spreadRadius: 8,
              ),
            ],
          ),
          child: const Padding(
            padding: EdgeInsets.all(32),
            child: _BitflipMark(size: 240),
          ),
        ),
      ),
    );
  }
}

class _SignalTicker extends HookWidget {
  const _SignalTicker({required this.snapshot});

  final GameSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    final (message, color) = switch (snapshot.isDemo) {
      true => (context.l10n.demoNotice, BitflipColors.coral),
      false => (context.l10n.securityNote, BitflipColors.acid),
    };
    return Container(
      color: color,
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 13),
      child: Center(
        child: Text(
          message,
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.labelLarge
              ?.copyWith(color: BitflipColors.voidColor, letterSpacing: 0.4),
        ),
      ),
    );
  }
}

class _GameWorkspace extends HookWidget {
  const _GameWorkspace({required this.state, required this.controller});

  final GameViewState state;
  final GameController controller;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final wide = constraints.maxWidth >= 1080;
        final canvas = _CanvasPanel(state: state, controller: controller);
        final console = GameConsole(
          state: state,
          onConnect: () => unawaited(_connectWallet(context, controller)),
          onClaim: () => unawaited(controller.claimSection()),
          onList: (price) => unawaited(controller.listSection(price)),
          onCancelListing: () => unawaited(controller.cancelSectionListing()),
          onPurchase: () => unawaited(controller.purchaseSection()),
          onWithdrawOwnerFees: () =>
              unawaited(controller.withdrawSectionOwnerFees()),
          onCommit: () => unawaited(controller.commitMoves()),
          onClear: controller.clearQueue,
          onSeal: () => unawaited(_confirmSeal(context, controller)),
          onMint: () => unawaited(controller.mintSection()),
          onRefresh: () => unawaited(controller.refresh()),
          onViewResult: () => unawaited(_openResult(state)),
        );
        if (!wide) {
          return Column(
            children: [canvas, const SizedBox(height: 22), console],
          );
        }
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(child: canvas),
            const SizedBox(width: 28),
            SizedBox(width: 356, child: console),
          ],
        );
      },
    );
  }
}

Future<void> _confirmSeal(
  BuildContext context,
  GameController controller,
) async {
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      icon: const Icon(Icons.lock_outline_rounded),
      title: Text(context.l10n.confirmSealTitle),
      content: Text(context.l10n.confirmSealBody),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: Text(context.l10n.cancel),
        ),
        FilledButton(
          key: BitflipTestKeys.confirmSeal,
          onPressed: () => Navigator.of(context).pop(true),
          child: Text(context.l10n.confirmSealAction),
        ),
      ],
    ),
  );
  if (confirmed ?? false) await controller.sealSection();
}

Future<void> _openResult(GameViewState state) async {
  final activity = state.activity;
  final assetId = activity.assetId;
  final path = assetId == null
      ? '/tx/${activity.transactionSignature}'
      : '/account/$assetId';
  final cluster = state.walletChain.split(':').last;
  final uri = Uri.https(
    'solscan.io',
    path,
    cluster == 'mainnet' ? null : {'cluster': cluster},
  );
  if (!await launchUrl(uri, mode: LaunchMode.externalApplication)) {
    throw StateError('Could not open the Solana explorer.');
  }
}

Future<void> _connectWallet(
  BuildContext context,
  GameController controller,
) async {
  final wallets = controller.availableWallets;
  if (wallets == null) {
    await controller.connectWallet();
    return;
  }
  if (!context.mounted) return;
  if (wallets.isEmpty) {
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.account_balance_wallet_outlined),
        title: Text(context.l10n.noWalletTitle),
        content: Text(context.l10n.noWalletBody),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text(context.l10n.close),
          ),
        ],
      ),
    );
    return;
  }
  final selected = wallets.length == 1
      ? wallets.single
      : await showModalBottomSheet<BitflipWalletOption>(
          context: context,
          backgroundColor: BitflipColors.panel,
          showDragHandle: true,
          builder: (context) => _WalletPicker(wallets: wallets),
        );
  if (selected == null) return;
  await controller.connectWallet(selected.id);
}

class _WalletPicker extends HookWidget {
  const _WalletPicker({required this.wallets});

  final List<BitflipWalletOption> wallets;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 520),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(24, 8, 24, 24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                context.l10n.chooseWallet,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: 8),
              Text(
                context.l10n.chooseWalletBody,
                style: Theme.of(context).textTheme.bodyMedium
                    ?.copyWith(color: BitflipColors.muted),
              ),
              const SizedBox(height: 18),
              Flexible(
                child: ListView.separated(
                  shrinkWrap: true,
                  itemCount: wallets.length,
                  separatorBuilder: (_, _) => const SizedBox(height: 8),
                  itemBuilder: (context, index) {
                    final wallet = wallets[index];
                    return ListTile(
                      shape: const Border.fromBorderSide(
                        BorderSide(color: BitflipColors.line),
                      ),
                      leading: const Icon(
                        Icons.account_balance_wallet_outlined,
                        color: BitflipColors.cyan,
                      ),
                      title: Text(
                        wallet.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      trailing: const Icon(Icons.arrow_forward_rounded),
                      onTap: () => Navigator.of(context).pop(wallet),
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _CanvasPanel extends HookWidget {
  const _CanvasPanel({required this.state, required this.controller});

  final GameViewState state;
  final GameController controller;

  @override
  Widget build(BuildContext context) {
    final section = state.snapshot.section;
    final canEdit =
        (state.loadStatus == GameLoadStatus.ready ||
            state.loadStatus == GameLoadStatus.demo) &&
        section.isEditable &&
        !state.isBusy;
    final selectedPixel = state.cursor ?? const PixelCoordinate(0, 0);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _Eyebrow(
                    label: state.snapshot.isDemo
                        ? context.l10n.demoMode
                        : context.l10n.chainMode,
                  ),
                  const SizedBox(height: 10),
                  Text(
                    context.l10n.sectionLabel(
                      formatSectionIndex(section.index),
                    ),
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                ],
              ),
            ),
            Text(
              context.l10n.sectionPosition(section.index + 1, sectionCount),
              style: Theme.of(context).textTheme.labelLarge
                  ?.copyWith(color: BitflipColors.muted),
            ),
          ],
        ),
        const SizedBox(height: 18),
        if (state.loadStatus != GameLoadStatus.ready &&
            state.loadStatus != GameLoadStatus.demo) ...[
          _GameLoadBanner(status: state.loadStatus),
          const SizedBox(height: 14),
        ],
        PixelCanvas(
          bitmap: section.bitmap,
          queued: state.queued,
          cursor: state.cursor,
          enabled: canEdit,
          onPixelPressed: controller.togglePixel,
          onCursorMoved: controller.selectPixel,
        ),
        const SizedBox(height: 15),
        _CoordinatePicker(
          coordinate: selectedPixel,
          enabled: canEdit,
          onChanged: controller.selectPixel,
          onToggle: () => controller.togglePixel(selectedPixel),
        ),
        const SizedBox(height: 15),
        Row(
          children: [
            Expanded(
              child: OutlinedButton.icon(
                key: BitflipTestKeys.previousSection,
                onPressed: section.index == 0 || state.isBusy
                    ? null
                    : () => unawaited(
                        controller.selectSection(section.index - 1),
                      ),
                icon: const Icon(Icons.west_rounded),
                label: Text(context.l10n.previousSection),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: OutlinedButton.icon(
                key: BitflipTestKeys.nextSection,
                onPressed: section.index == sectionCount - 1 || state.isBusy
                    ? null
                    : () => unawaited(
                        controller.selectSection(section.index + 1),
                      ),
                iconAlignment: IconAlignment.end,
                icon: const Icon(Icons.east_rounded),
                label: Text(context.l10n.nextSection),
              ),
            ),
          ],
        ),
        const SizedBox(height: 52),
        LayoutBuilder(
          builder: (context, constraints) {
            final navigator = SectionNavigator(
              selectedIndex: section.index,
              nextSection: state.snapshot.nextSection,
              mintedSections: state.snapshot.mintedSections,
            );
            final copy = Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  context.l10n.overview,
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const SizedBox(height: 10),
                Text(context.l10n.overviewBody),
                const SizedBox(height: 16),
                DropdownButtonFormField<int>(
                  key: BitflipTestKeys.sectionPicker,
                  initialValue: section.index,
                  decoration: InputDecoration(
                    labelText: context.l10n.selectSection,
                  ),
                  items: [
                    for (var index = 0; index < sectionCount; index++)
                      DropdownMenuItem(
                        value: index,
                        child: Text(formatSectionIndex(index)),
                      ),
                  ],
                  onChanged: state.isBusy
                      ? null
                      : (index) {
                          if (index != null) {
                            unawaited(controller.selectSection(index));
                          }
                        },
                ),
              ],
            );
            if (constraints.maxWidth < 700) {
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [copy, const SizedBox(height: 22), navigator],
              );
            }
            return Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(child: copy),
                const SizedBox(width: 30),
                SizedBox(width: 310, child: navigator),
              ],
            );
          },
        ),
      ],
    );
  }
}

class _CoordinatePicker extends HookWidget {
  const _CoordinatePicker({
    required this.coordinate,
    required this.enabled,
    required this.onChanged,
    required this.onToggle,
  });

  final PixelCoordinate coordinate;
  final bool enabled;
  final ValueChanged<PixelCoordinate> onChanged;
  final VoidCallback onToggle;

  @override
  Widget build(BuildContext context) {
    Widget axisPicker({
      required Key key,
      required String label,
      required int value,
      required ValueChanged<int> onChanged,
    }) {
      return InputDecorator(
        decoration: InputDecoration(labelText: label),
        child: DropdownButtonHideUnderline(
          child: DropdownButton<int>(
            key: key,
            value: value,
            isExpanded: true,
            items: [
              for (var index = 0; index < sectionSide; index++)
                DropdownMenuItem(value: index, child: Text('$index')),
            ],
            onChanged: (next) {
              if (next != null) onChanged(next);
            },
          ),
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        LayoutBuilder(
          builder: (context, constraints) {
            final xPicker = axisPicker(
              key: BitflipTestKeys.pixelX,
              label: context.l10n.pixelX,
              value: coordinate.x,
              onChanged: (x) => onChanged(PixelCoordinate(x, coordinate.y)),
            );
            final yPicker = axisPicker(
              key: BitflipTestKeys.pixelY,
              label: context.l10n.pixelY,
              value: coordinate.y,
              onChanged: (y) => onChanged(PixelCoordinate(coordinate.x, y)),
            );
            if (constraints.maxWidth < 420) {
              return Column(
                children: [xPicker, const SizedBox(height: 10), yPicker],
              );
            }
            return Row(
              children: [
                Expanded(child: xPicker),
                const SizedBox(width: 12),
                Expanded(child: yPicker),
              ],
            );
          },
        ),
        const SizedBox(height: 10),
        OutlinedButton.icon(
          key: BitflipTestKeys.toggleCoordinate,
          onPressed: enabled ? onToggle : null,
          icon: const Icon(Icons.flip_rounded),
          label: Text(context.l10n.togglePixel(coordinate.x, coordinate.y)),
        ),
      ],
    );
  }
}

class _GameLoadBanner extends HookWidget {
  const _GameLoadBanner({required this.status});

  final GameLoadStatus status;

  @override
  Widget build(BuildContext context) {
    final (icon, message) = switch (status) {
      GameLoadStatus.loading => (Icons.sync_rounded, context.l10n.gameLoading),
      GameLoadStatus.unavailable => (
        Icons.hourglass_empty_rounded,
        context.l10n.gameUnavailable,
      ),
      GameLoadStatus.error => (
        Icons.cloud_off_outlined,
        context.l10n.gameOffline,
      ),
      GameLoadStatus.ready || GameLoadStatus.demo => (
        Icons.check_circle_outline,
        context.l10n.activityReady,
      ),
    };
    return Semantics(
      liveRegion: true,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: BitflipColors.raised,
          border: Border.all(color: BitflipColors.line),
        ),
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Row(
            children: [
              Icon(icon, size: 20, color: BitflipColors.coral),
              const SizedBox(width: 10),
              Expanded(child: Text(message)),
            ],
          ),
        ),
      ),
    );
  }
}

class _HowItWorks extends HookWidget {
  @override
  Widget build(BuildContext context) {
    final steps = [
      (context.l10n.claimStep, context.l10n.claimStepBody, BitflipColors.acid),
      (context.l10n.flipStep, context.l10n.flipStepBody, BitflipColors.coral),
      (context.l10n.sealStep, context.l10n.sealStepBody, BitflipColors.cyan),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          context.l10n.howItWorks,
          style: Theme.of(context).textTheme.displaySmall,
        ),
        const SizedBox(height: 30),
        LayoutBuilder(
          builder: (context, constraints) {
            final width = constraints.maxWidth >= 900
                ? (constraints.maxWidth - 32) / 3
                : constraints.maxWidth;
            return Wrap(
              spacing: 16,
              runSpacing: 16,
              children: [
                for (final step in steps)
                  SizedBox(
                    width: width,
                    child: _StepCard(
                      title: step.$1,
                      body: step.$2,
                      color: step.$3,
                    ),
                  ),
              ],
            );
          },
        ),
      ],
    );
  }
}

class _StepCard extends HookWidget {
  const _StepCard({
    required this.title,
    required this.body,
    required this.color,
  });

  final String title;
  final String body;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(minHeight: 210),
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: BitflipColors.panel,
        border: Border(top: BorderSide(color: color, width: 4)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.titleLarge
                ?.copyWith(color: color),
          ),
          const SizedBox(height: 22),
          Text(body, style: Theme.of(context).textTheme.bodyLarge),
        ],
      ),
    );
  }
}

class _Footer extends HookWidget {
  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: const BoxDecoration(
        border: Border(top: BorderSide(color: BitflipColors.line)),
      ),
      child: Padding(
        padding: const EdgeInsets.only(top: 28),
        child: Wrap(
          alignment: WrapAlignment.spaceBetween,
          runAlignment: WrapAlignment.center,
          spacing: 24,
          runSpacing: 14,
          children: [
            Text(
              context.l10n.builtWith,
              style: Theme.of(context).textTheme.labelLarge
                  ?.copyWith(color: BitflipColors.muted),
            ),
            Text(
              context.l10n.securityNote,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            Wrap(
              spacing: 4,
              children: [
                TextButton(
                  onPressed: () => context.go('/privacy'),
                  child: Text(context.l10n.privacyLink),
                ),
                TextButton(
                  onPressed: () => context.go('/terms'),
                  child: Text(context.l10n.termsLink),
                ),
                TextButton(
                  onPressed: () => context.go('/support'),
                  child: Text(context.l10n.supportLink),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _StatusPill extends HookWidget {
  const _StatusPill({
    required this.label,
    required this.color,
    this.onTap,
    super.key,
  });

  final String label;
  final Color color;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: onTap != null,
      label: onTap == null ? null : context.l10n.openWalletDetails,
      child: InkWell(
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            border: Border.all(color: color.withValues(alpha: 0.5)),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 7,
                height: 7,
                decoration: BoxDecoration(color: color, shape: BoxShape.circle),
              ),
              const SizedBox(width: 8),
              Text(
                label,
                style: Theme.of(context).textTheme.labelSmall
                    ?.copyWith(color: color, fontWeight: FontWeight.w800),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Eyebrow extends HookWidget {
  const _Eyebrow({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        const SizedBox(
          width: 28,
          child: Divider(color: BitflipColors.coral, thickness: 2),
        ),
        const SizedBox(width: 10),
        Flexible(
          child: Text(
            label,
            style: Theme.of(context).textTheme.labelLarge
                ?.copyWith(color: BitflipColors.coral, letterSpacing: 1.4),
          ),
        ),
      ],
    );
  }
}

class _BitflipMark extends HookWidget {
  const _BitflipMark({required this.size});

  final double size;

  @override
  Widget build(BuildContext context) {
    return SizedBox.square(
      dimension: size,
      child: CustomPaint(painter: _BitflipMarkPainter()),
    );
  }
}

class _BitflipMarkPainter extends CustomPainter {
  static const pattern = <int>[0x1E, 0x11, 0x1E, 0x11, 0x1E];

  @override
  void paint(Canvas canvas, Size size) {
    final cell = size.width / 5;
    final paint = Paint()..color = BitflipColors.acid;
    final alt = Paint()..color = BitflipColors.coral;
    for (var y = 0; y < pattern.length; y++) {
      for (var x = 0; x < 5; x++) {
        if ((pattern[y] & (1 << (4 - x))) == 0) continue;
        canvas.drawRect(
          Rect.fromLTWH(x * cell, y * cell, cell - 1, cell - 1),
          x == 4 && y.isOdd ? alt : paint,
        );
      }
    }
  }

  @override
  bool shouldRepaint(_BitflipMarkPainter oldDelegate) => false;
}

class _Atmosphere extends HookWidget {
  const _Atmosphere();

  @override
  Widget build(BuildContext context) {
    return IgnorePointer(child: CustomPaint(painter: _AtmospherePainter()));
  }
}

class _AtmospherePainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final glow = Paint()
      ..shader =
          const RadialGradient(colors: [Color(0x262A7763), Colors.transparent])
              .createShader(
                Rect.fromCircle(
                  center: Offset(size.width * 0.72, size.height * 0.11),
                  radius: size.width * 0.52,
                ),
              );
    canvas.drawRect(Offset.zero & size, glow);
    final line = Paint()
      ..color = BitflipColors.line.withValues(alpha: 0.16)
      ..strokeWidth = 1;
    for (var y = 0.0; y < size.height; y += 6) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), line);
    }
  }

  @override
  bool shouldRepaint(_AtmospherePainter oldDelegate) => false;
}

String _shortAddress(String value) {
  if (value.length <= 12) return value;
  return '${value.substring(0, 5)}…${value.substring(value.length - 4)}';
}
