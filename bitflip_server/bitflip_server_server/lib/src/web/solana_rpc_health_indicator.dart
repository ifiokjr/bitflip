import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:serverpod/serverpod.dart';
import 'package:solana_kit/solana_kit.dart';

typedef SolanaSlotReader = Future<BigInt> Function();

/// Makes readiness reflect the Solana RPC dependency used by every live action.
final class SolanaRpcHealthIndicator extends HealthIndicator<int> {
  SolanaRpcHealthIndicator({
    required this.readSlot,
    this.deadline = const Duration(seconds: 3),
  });

  factory SolanaRpcHealthIndicator.forService(
    SolanaBitflipMintService service,
  ) {
    return SolanaRpcHealthIndicator(
      readSlot: () => service.rpc.getSlot().send(),
    );
  }

  final SolanaSlotReader readSlot;
  final Duration deadline;

  @override
  String get name => 'solana:rpc';

  @override
  String get componentType => HealthComponentType.component.name;

  @override
  String get observedUnit => 'slot';

  @override
  Duration get timeout => deadline;

  @override
  Future<HealthCheckResult> check() async {
    try {
      final slot = await readSlot().timeout(deadline);
      if (slot.isNegative) {
        return fail(output: 'Solana RPC returned an invalid slot.');
      }
      return pass(observedValue: slot.toInt());
    } on Object catch (error) {
      return fail(output: 'Solana RPC check failed (${error.runtimeType}).');
    }
  }
}
