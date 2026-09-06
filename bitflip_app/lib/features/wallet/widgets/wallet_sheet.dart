import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/application/game_controller.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:bitflip_app/testing/bitflip_test_keys.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

Future<void> showBitflipWalletSheet(
  BuildContext context, {
  required GameViewState state,
  required GameController controller,
}) async {
  final lamports = await showModalBottomSheet<BigInt>(
    context: context,
    isScrollControlled: true,
    backgroundColor: BitflipColors.panel,
    showDragHandle: true,
    builder: (context) => _WalletSheet(state: state),
  );
  if (lamports != null) {
    await controller.fundWithMobileWallet(lamports);
  }
}

BigInt? parseSolToLamports(String input) {
  final value = input.trim();
  if (!RegExp(r'^\d+(\.\d{1,9})?$').hasMatch(value)) return null;
  final parts = value.split('.');
  final whole = BigInt.parse(parts.first);
  final fraction = parts.length == 1
      ? BigInt.zero
      : BigInt.parse(parts.last.padRight(9, '0'));
  final lamports = whole * BigInt.from(1000000000) + fraction;
  return lamports > BigInt.zero ? lamports : null;
}

class _WalletSheet extends HookWidget {
  const _WalletSheet({required this.state});

  final GameViewState state;

  @override
  Widget build(BuildContext context) {
    final embedded = state.walletKind == BitflipWalletKind.embedded;
    final address = state.walletAddress!;
    final amountController = useTextEditingController(text: '0.05');
    final amount = useState(parseSolToLamports(amountController.text));
    final bottomInset = MediaQuery.viewInsetsOf(context).bottom;

    return SafeArea(
      child: SingleChildScrollView(
        padding: EdgeInsets.fromLTRB(24, 8, 24, 24 + bottomInset),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Container(
                  width: 44,
                  height: 44,
                  decoration: BoxDecoration(
                    color: BitflipColors.cyan.withValues(alpha: 0.1),
                    border: Border.all(
                      color: BitflipColors.cyan.withValues(alpha: 0.5),
                    ),
                  ),
                  child: Icon(
                    embedded
                        ? Icons.shield_outlined
                        : Icons.account_balance_wallet_outlined,
                    color: BitflipColors.cyan,
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        embedded
                            ? context.l10n.embeddedWalletTitle
                            : context.l10n.externalWalletTitle,
                        style: Theme.of(context).textTheme.headlineSmall,
                      ),
                      Text(
                        state.walletChain.split(':').last.toUpperCase(),
                        style: Theme.of(context).textTheme.labelMedium
                            ?.copyWith(
                              color: BitflipColors.cyan,
                              letterSpacing: 1.1,
                            ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 22),
            if (embedded) ...[
              Text(
                context.l10n.walletBalance,
                style: Theme.of(context).textTheme.labelMedium
                    ?.copyWith(color: BitflipColors.muted),
              ),
              const SizedBox(height: 4),
              Text(
                state.walletBalanceLamports == null
                    ? '— SOL'
                    : '${lamportsToSol(state.walletBalanceLamports!)} SOL',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                  color: BitflipColors.acid,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const SizedBox(height: 18),
            ],
            Text(
              context.l10n.walletAddress,
              style: Theme.of(context).textTheme.labelMedium
                  ?.copyWith(color: BitflipColors.muted),
            ),
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: BitflipColors.voidColor,
                border: Border.all(color: BitflipColors.line),
              ),
              child: SelectableText(
                address,
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  fontFamily: 'monospace',
                  color: BitflipColors.paper,
                ),
              ),
            ),
            const SizedBox(height: 10),
            OutlinedButton.icon(
              key: BitflipTestKeys.copyWalletAddress,
              onPressed: () async {
                await Clipboard.setData(ClipboardData(text: address));
                if (!context.mounted) return;
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text(context.l10n.walletAddressCopied)),
                );
              },
              icon: const Icon(Icons.copy_rounded),
              label: Text(context.l10n.copyWalletAddress),
            ),
            const SizedBox(height: 18),
            Text(
              embedded
                  ? context.l10n.embeddedWalletBody(
                      state.walletChain.split(':').last,
                    )
                  : context.l10n.externalWalletBody,
              style: Theme.of(context).textTheme.bodyMedium
                  ?.copyWith(color: BitflipColors.muted, height: 1.5),
            ),
            if (embedded) ...[
              const SizedBox(height: 14),
              Container(
                padding: const EdgeInsets.all(14),
                decoration: BoxDecoration(
                  color: BitflipColors.coral.withValues(alpha: 0.08),
                  border: Border.all(
                    color: BitflipColors.coral.withValues(alpha: 0.45),
                  ),
                ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Icon(
                      Icons.warning_amber_rounded,
                      color: BitflipColors.coral,
                      size: 20,
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: Text(
                        context.l10n.embeddedWalletWarning,
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: BitflipColors.paper,
                          height: 1.45,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ],
            if (embedded && state.canFundWithMobileWallet) ...[
              const SizedBox(height: 22),
              TextField(
                key: BitflipTestKeys.walletFundingAmount,
                controller: amountController,
                keyboardType: const TextInputType.numberWithOptions(
                  decimal: true,
                ),
                inputFormatters: [
                  FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
                ],
                decoration: InputDecoration(
                  labelText: context.l10n.fundingAmount,
                  suffixText: 'SOL',
                  helperText: context.l10n.fundingAmountHelp,
                ),
                onChanged: (value) => amount.value = parseSolToLamports(value),
              ),
              const SizedBox(height: 14),
              FilledButton.icon(
                key: BitflipTestKeys.fundWithMobileWallet,
                onPressed: amount.value == null || state.isBusy
                    ? null
                    : () => Navigator.of(context).pop(amount.value),
                icon: const Icon(Icons.add_card_rounded),
                label: Text(context.l10n.fundWithMobileWallet),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
