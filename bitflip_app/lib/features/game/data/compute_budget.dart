import 'dart:typed_data';

import 'package:solana_kit/solana_kit.dart';

/// Compute-unit ceilings for each player transaction.
///
/// The program's Surfpool suite measures every instruction on the real SBF
/// artifact and fails when one exceeds its budget. These limits sit above
/// those budgets plus any token-account creation a transaction bundles, so a
/// wallet prices what the transaction can actually consume instead of the
/// 200k-per-instruction default.
abstract final class ComputeUnitLimit {
  /// Claim plus vault funding: the claim creates the section account and the
  /// funding creates and fills its BIT vault.
  static const claimAndFundSection = 80000;

  /// A flip batch preceded by the idempotent creation of the player's BIT
  /// account, which costs the most the first time it runs.
  static const flipPixels = 65000;

  /// Listing, cancelling, purchasing, sealing, withdrawing fees, and
  /// configuring a policy each touch one or two accounts without CPIs.
  static const sectionAdministration = 10000;
}

Instruction setComputeUnitLimitInstruction(int units) {
  final data = Uint8List(5)..[0] = 2;
  ByteData.sublistView(data).setUint32(1, units, Endian.little);

  return Instruction(
    programAddress: computeBudgetProgramAddress,
    accounts: const [],
    data: data,
  );
}

Instruction setComputeUnitPriceInstruction(int microLamports) {
  final data = Uint8List(9)..[0] = 3;
  ByteData.sublistView(data).setUint64(1, microLamports, Endian.little);

  return Instruction(
    programAddress: computeBudgetProgramAddress,
    accounts: const [],
    data: data,
  );
}
