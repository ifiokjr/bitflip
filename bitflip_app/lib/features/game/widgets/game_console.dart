import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

class GameConsole extends HookWidget {
  const GameConsole({
    required this.state,
    required this.onConnect,
    required this.onClaim,
    required this.onList,
    required this.onCancelListing,
    required this.onPurchase,
    required this.onCommit,
    required this.onClear,
    required this.onSeal,
    required this.onMint,
    required this.onRefresh,
    this.onViewResult,
    super.key,
  });

  final GameViewState state;
  final VoidCallback onConnect;
  final VoidCallback onClaim;
  final ValueChanged<BigInt> onList;
  final VoidCallback onCancelListing;
  final VoidCallback onPurchase;
  final VoidCallback onCommit;
  final VoidCallback onClear;
  final VoidCallback onSeal;
  final VoidCallback onMint;
  final VoidCallback onRefresh;
  final VoidCallback? onViewResult;

  @override
  Widget build(BuildContext context) {
    final section = state.snapshot.section;
    final canSign = state.canTransact;
    final now = BigInt.from(DateTime.now().millisecondsSinceEpoch ~/ 1000);
    final canClaimSection = state.snapshot.canClaimSectionAt(now);
    final canSeal =
        section.isEditable &&
        (state.snapshot.isDemo || state.walletAddress == section.owner);
    final canMint =
        section.lifecycle == SectionLifecycle.sealed &&
        (state.snapshot.isDemo || state.walletAddress == section.owner);
    final hasFullQueuedReward =
        state.queuedReward == BigInt.from(state.queued.length);

    return DecoratedBox(
      decoration: BoxDecoration(
        color: BitflipColors.panel.withValues(alpha: 0.96),
        border: Border.all(color: BitflipColors.line),
      ),
      child: Padding(
        padding: const EdgeInsets.all(22),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    context.l10n.console,
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                ),
                IconButton(
                  onPressed: state.isBusy ? null : onRefresh,
                  tooltip: state.isBusy
                      ? context.l10n.refreshing
                      : context.l10n.refresh,
                  icon: state.isBusy
                      ? const SizedBox.square(
                          dimension: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.sync_rounded),
                ),
              ],
            ),
            if (section.lifecycle == SectionLifecycle.unclaimed) ...[
              const SizedBox(height: 10),
              Row(
                children: [
                  Expanded(
                    child: Text(
                      context.l10n.claimPrice,
                      style: Theme.of(context).textTheme.labelLarge
                          ?.copyWith(color: BitflipColors.muted),
                    ),
                  ),
                  Text(
                    context.l10n.feeValue(
                      lamportsToSol(state.snapshot.claimPriceLamports),
                    ),
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      color: BitflipColors.cyan,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                section.index == state.snapshot.nextSection
                    ? canClaimSection
                          ? context.l10n.sectorReady
                          : context.l10n.sectorUnlockProgress(
                              (state.snapshot.previousSectionFlipCount ??
                                      BigInt.zero)
                                  .toString(),
                              state.snapshot.earlyUnlockFlips,
                              _formatUnlockTime(
                                context,
                                state.snapshot.selectedSectionUnlockAt,
                              ),
                            )
                    : context.l10n.waitingForSector(
                        formatSectionIndex(state.snapshot.nextSection),
                      ),
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: canClaimSection
                      ? BitflipColors.acid
                      : BitflipColors.muted,
                ),
              ),
            ],
            const SizedBox(height: 18),
            Wrap(
              runSpacing: 12,
              spacing: 12,
              children: [
                ConsoleDatum(
                  label: context.l10n.status,
                  value: _lifecycleLabel(context, section.lifecycle),
                  accent: _lifecycleColor(section.lifecycle),
                ),
                ConsoleDatum(
                  label: context.l10n.selectedPixel,
                  value: state.cursor == null
                      ? context.l10n.noPixelSelected
                      : '${state.cursor!.x.toString().padLeft(2, '0')} : '
                            '${state.cursor!.y.toString().padLeft(2, '0')}',
                ),
                ConsoleDatum(
                  label: context.l10n.queuedMoves,
                  value: context.l10n.moveCount(state.queued.length),
                  accent: state.queued.isEmpty
                      ? BitflipColors.muted
                      : BitflipColors.coral,
                ),
                ConsoleDatum(
                  label: context.l10n.onPixels,
                  value: context.l10n.pixelCount(state.previewBitmap.onCount),
                ),
                ConsoleDatum(
                  label: context.l10n.network,
                  value: state.walletChain.split(':').last.toUpperCase(),
                  accent: BitflipColors.cyan,
                ),
                ConsoleDatum(
                  label: context.l10n.revision,
                  value: section.revision.toString(),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _SignalMeter(progress: state.queued.length / 16),
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(
                  child: Text(
                    context.l10n.moveFee,
                    style: Theme.of(context).textTheme.labelLarge
                        ?.copyWith(color: BitflipColors.muted),
                  ),
                ),
                Text(
                  context.l10n.feeValue(lamportsToSol(state.queuedFee)),
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    color: BitflipColors.cyan,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: Text(
                    context.l10n.moveReward,
                    style: Theme.of(context).textTheme.labelLarge
                        ?.copyWith(color: BitflipColors.muted),
                  ),
                ),
                Text(
                  context.l10n.rewardValue(state.queuedReward.toString()),
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    color: BitflipColors.coral,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ],
            ),
            if (state.queued.isNotEmpty && !hasFullQueuedReward) ...[
              const SizedBox(height: 8),
              Text(
                context.l10n.rewardWindowUnavailable,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: BitflipColors.coral,
                ),
              ),
            ],
            const SizedBox(height: 18),
            if (state.walletAddress == null && state.isWalletSupported)
              FilledButton.icon(
                key: BitflipTestKeys.connectWallet,
                onPressed: state.isBusy ? null : onConnect,
                icon: const Icon(Icons.account_balance_wallet_outlined),
                label: Text(context.l10n.connectWallet),
              )
            else if (section.lifecycle == SectionLifecycle.unclaimed)
              FilledButton.icon(
                key: BitflipTestKeys.claimSection,
                onPressed: canSign && !state.isBusy && canClaimSection
                    ? onClaim
                    : null,
                icon: const Icon(Icons.flag_outlined),
                label: Text(
                  canClaimSection
                      ? context.l10n.claimSector
                      : context.l10n.sectorLocked,
                ),
              )
            else if (section.lifecycle == SectionLifecycle.sealed)
              FilledButton.icon(
                key: BitflipTestKeys.mintSection,
                onPressed: canMint && !state.isBusy ? onMint : null,
                icon: const Icon(Icons.auto_awesome_outlined),
                label: Text(context.l10n.mintCompressedNft),
              )
            else if (section.lifecycle == SectionLifecycle.minted)
              FilledButton.icon(
                onPressed: null,
                icon: const Icon(Icons.verified_outlined),
                label: Text(context.l10n.minted),
              )
            else
              FilledButton.icon(
                key: BitflipTestKeys.commitFlips,
                onPressed:
                    state.queued.isEmpty ||
                        state.isBusy ||
                        !canSign ||
                        !hasFullQueuedReward
                    ? null
                    : onCommit,
                icon: const Icon(Icons.bolt_rounded),
                label: Text(context.l10n.commitMoves(state.queued.length)),
              ),
            const SizedBox(height: 10),
            OutlinedButton(
              key: BitflipTestKeys.clearFlips,
              onPressed: state.queued.isEmpty || state.isBusy ? null : onClear,
              child: Text(context.l10n.clearQueue),
            ),
            if (section.lifecycle == SectionLifecycle.active) ...[
              const SizedBox(height: 10),
              OutlinedButton.icon(
                key: BitflipTestKeys.sealSection,
                onPressed: canSeal && !state.isBusy ? onSeal : null,
                icon: const Icon(Icons.lock_outline_rounded),
                label: Text(context.l10n.sealArtwork),
              ),
            ],
            if (!state.isWalletSupported && !state.snapshot.isDemo) ...[
              const SizedBox(height: 14),
              Text(
                context.l10n.walletUnavailable,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
            const SizedBox(height: 22),
            const Divider(height: 1),
            const SizedBox(height: 20),
            _OwnerRow(section: section),
            if (section.isClaimed) ...[
              const SizedBox(height: 16),
              _MarketplacePanel(
                state: state,
                onList: onList,
                onCancelListing: onCancelListing,
                onPurchase: onPurchase,
              ),
            ],
            const SizedBox(height: 20),
            _ActivityPulse(activity: state.activity),
            if (onViewResult != null &&
                (state.activity.transactionSignature != null ||
                    state.activity.assetId != null)) ...[
              const SizedBox(height: 10),
              TextButton.icon(
                key: BitflipTestKeys.viewResult,
                onPressed: onViewResult,
                icon: const Icon(Icons.open_in_new_rounded, size: 18),
                label: Text(
                  state.activity.assetId == null
                      ? context.l10n.viewTransaction
                      : context.l10n.viewAsset,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class ConsoleDatum extends HookWidget {
  const ConsoleDatum({
    required this.label,
    required this.value,
    this.accent = BitflipColors.paper,
    super.key,
  });

  final String label;
  final String value;
  final Color accent;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 132,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: Theme.of(context).textTheme.labelSmall
                ?.copyWith(color: BitflipColors.muted, letterSpacing: 1),
          ),
          const SizedBox(height: 4),
          Text(
            value,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.titleSmall
                ?.copyWith(color: accent, fontWeight: FontWeight.w800),
          ),
        ],
      ),
    );
  }
}

class _SignalMeter extends HookWidget {
  const _SignalMeter({required this.progress});

  final double progress;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 5,
      child: LayoutBuilder(
        builder: (context, constraints) {
          return Stack(
            children: [
              Positioned.fill(child: ColoredBox(color: BitflipColors.raised)),
              AnimatedContainer(
                duration: const Duration(milliseconds: 180),
                curve: Curves.easeOutCubic,
                width: constraints.maxWidth * progress.clamp(0, 1),
                color: progress >= 1 ? BitflipColors.coral : BitflipColors.acid,
              ),
            ],
          );
        },
      ),
    );
  }
}

class _OwnerRow extends HookWidget {
  const _OwnerRow({required this.section});

  final SectionSnapshot section;

  @override
  Widget build(BuildContext context) {
    late final String ownerLabel;
    if (section.isProtocolOwned) {
      ownerLabel = context.l10n.bitflipProgram;
    } else if (section.owner == null) {
      ownerLabel = context.l10n.anonymousOwner;
    } else {
      ownerLabel = _shortAddress(section.owner!);
    }
    return Row(
      children: [
        const Icon(Icons.person_outline_rounded, size: 19),
        const SizedBox(width: 10),
        Expanded(
          child: Text(
            context.l10n.sectionOwner,
            style: Theme.of(context).textTheme.labelLarge,
          ),
        ),
        Text(
          ownerLabel,
          style: Theme.of(context).textTheme.labelLarge
              ?.copyWith(color: BitflipColors.cyan),
        ),
      ],
    );
  }
}

class _MarketplacePanel extends HookWidget {
  const _MarketplacePanel({
    required this.state,
    required this.onList,
    required this.onCancelListing,
    required this.onPurchase,
  });

  final GameViewState state;
  final ValueChanged<BigInt> onList;
  final VoidCallback onCancelListing;
  final VoidCallback onPurchase;

  @override
  Widget build(BuildContext context) {
    final section = state.snapshot.section;
    final isOwner = state.walletAddress == section.owner;
    final canTransfer =
        !section.isProtocolOwned &&
        section.lifecycle != SectionLifecycle.minted;
    final priceController = useTextEditingController();
    final enteredPrice = useState<BigInt?>(null);

    if (section.isListed) {
      return DecoratedBox(
        decoration: BoxDecoration(
          color: BitflipColors.raised,
          border: Border.all(color: BitflipColors.cyan.withValues(alpha: 0.5)),
        ),
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                context.l10n.forSale,
                style: Theme.of(context).textTheme.labelLarge
                    ?.copyWith(color: BitflipColors.cyan),
              ),
              const SizedBox(height: 5),
              Text(
                context.l10n.feeValue(lamportsToSol(section.salePriceLamports)),
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 12),
              if (isOwner)
                OutlinedButton(
                  key: BitflipTestKeys.cancelSectionListing,
                  onPressed: state.isBusy ? null : onCancelListing,
                  child: Text(context.l10n.cancelListing),
                )
              else
                FilledButton.icon(
                  key: BitflipTestKeys.purchaseSection,
                  onPressed: state.canTransact && !state.isBusy
                      ? onPurchase
                      : null,
                  icon: const Icon(Icons.shopping_bag_outlined),
                  label: Text(context.l10n.buySection),
                ),
            ],
          ),
        ),
      );
    }

    if (!isOwner || !canTransfer) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          key: BitflipTestKeys.sectionSalePrice,
          controller: priceController,
          enabled: !state.isBusy,
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: InputDecoration(
            labelText: context.l10n.salePrice,
            suffixText: context.l10n.solUnit,
            helperText: context.l10n.salePriceHelp,
            errorText:
                priceController.text.isNotEmpty && enteredPrice.value == null
                ? context.l10n.invalidSalePrice
                : null,
          ),
          onChanged: (value) => enteredPrice.value = trySolToLamports(value),
        ),
        const SizedBox(height: 10),
        OutlinedButton.icon(
          key: BitflipTestKeys.listSection,
          onPressed: state.isBusy || enteredPrice.value == null
              ? null
              : () => onList(enteredPrice.value!),
          icon: const Icon(Icons.sell_outlined),
          label: Text(context.l10n.listSection),
        ),
      ],
    );
  }
}

class _ActivityPulse extends HookWidget {
  const _ActivityPulse({required this.activity});

  final GameActivity activity;

  @override
  Widget build(BuildContext context) {
    final text = switch (activity.notice) {
      GameNotice.ready => context.l10n.activityReady,
      GameNotice.queued => context.l10n.activityQueued(
        activity.coordinate?.x ?? 0,
        activity.coordinate?.y ?? 0,
      ),
      GameNotice.committed => context.l10n.activityCommitted,
      GameNotice.sectionChanged => context.l10n.activitySectionChanged(
        formatSectionIndex(activity.sectionIndex ?? 0),
      ),
      GameNotice.connected => context.l10n.activityConnected,
      GameNotice.funded => context.l10n.activityFunded,
      GameNotice.claimed => context.l10n.activityClaimed,
      GameNotice.listed => context.l10n.activityListed,
      GameNotice.listingCancelled => context.l10n.activityListingCancelled,
      GameNotice.purchased => context.l10n.activityPurchased,
      GameNotice.sealed => context.l10n.activitySealed,
      GameNotice.minted => context.l10n.activityMinted,
      GameNotice.batchFull => context.l10n.batchFull,
      GameNotice.walletIssue => context.l10n.walletIssue,
      GameNotice.connectionIssue => context.l10n.connectionIssue,
    };
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 8,
          height: 8,
          margin: const EdgeInsets.only(top: 5),
          decoration: const BoxDecoration(
            color: BitflipColors.coral,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                context.l10n.activity,
                style: Theme.of(context).textTheme.labelSmall
                    ?.copyWith(color: BitflipColors.coral, letterSpacing: 1.2),
              ),
              const SizedBox(height: 5),
              Text(text, style: Theme.of(context).textTheme.bodySmall),
            ],
          ),
        ),
      ],
    );
  }
}

String _lifecycleLabel(BuildContext context, SectionLifecycle lifecycle) {
  return switch (lifecycle) {
    SectionLifecycle.unclaimed => context.l10n.unclaimed,
    SectionLifecycle.active => context.l10n.active,
    SectionLifecycle.sealed => context.l10n.sealed,
    SectionLifecycle.minted => context.l10n.minted,
  };
}

Color _lifecycleColor(SectionLifecycle lifecycle) {
  return switch (lifecycle) {
    SectionLifecycle.unclaimed => BitflipColors.muted,
    SectionLifecycle.active => BitflipColors.acid,
    SectionLifecycle.sealed => BitflipColors.coral,
    SectionLifecycle.minted => BitflipColors.cyan,
  };
}

String _shortAddress(String value) {
  if (value.length <= 12) return value;
  return '${value.substring(0, 5)}…${value.substring(value.length - 4)}';
}

String _formatUnlockTime(BuildContext context, BigInt unixSeconds) {
  final date = DateTime.fromMillisecondsSinceEpoch(
    unixSeconds.toInt() * 1000,
    isUtc: true,
  ).toLocal();
  final localizations = MaterialLocalizations.of(context);
  return '${localizations.formatMediumDate(date)} '
      '${localizations.formatTimeOfDay(TimeOfDay.fromDateTime(date))}';
}
