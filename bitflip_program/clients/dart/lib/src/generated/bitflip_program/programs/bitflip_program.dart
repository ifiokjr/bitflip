// Auto-generated. Do not edit.
// ignore_for_file: type=lint


import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';

import '../instructions/instructions.dart';


/// The address of the BitflipProgram program.
const bitflipProgramProgramAddress = Address('5AuNvfV9Xi9gskJpW2qQJndQkFcwbWNV6fjaf2VvuEcM');

/// Known accounts for the BitflipProgram program.
enum BitflipProgramAccount {
  configState,
  gameState,
  sectionState,
}

/// Known instructions for the BitflipProgram program.
enum BitflipProgramInstruction {
  initializeConfig,
  updateConfig,
  proposeAuthority,
  acceptAuthority,
  initializeGame,
  claimSection,
  flipPixels,
  sealSection,
  recordSectionMint,
  listSection,
  cancelSectionListing,
  purchaseSection,
  settleSectionEconomy,
}

/// Identifies the type of a BitflipProgram instruction.
BitflipProgramInstruction identifyBitflipProgramInstruction(
  Uint8List data,
) {
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return BitflipProgramInstruction.initializeConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return BitflipProgramInstruction.updateConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return BitflipProgramInstruction.proposeAuthority;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return BitflipProgramInstruction.acceptAuthority;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return BitflipProgramInstruction.initializeGame;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return BitflipProgramInstruction.claimSection;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return BitflipProgramInstruction.flipPixels;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return BitflipProgramInstruction.sealSection;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return BitflipProgramInstruction.recordSectionMint;
  }
  if (containsBytes(data, getU8Encoder().encode(9), 0)) {
    return BitflipProgramInstruction.listSection;
  }
  if (containsBytes(data, getU8Encoder().encode(10), 0)) {
    return BitflipProgramInstruction.cancelSectionListing;
  }
  if (containsBytes(data, getU8Encoder().encode(11), 0)) {
    return BitflipProgramInstruction.purchaseSection;
  }
  if (containsBytes(data, getU8Encoder().encode(12), 0)) {
    return BitflipProgramInstruction.settleSectionEconomy;
  }

  throw SolanaError(
    SolanaErrorCode.programClientsFailedToIdentifyInstruction,
    {
      'instructionData': data,
      'programName': 'bitflipProgram',
    },
  );
}

/// A parsed instruction from the BitflipProgram program.
sealed class ParsedBitflipProgramInstruction {
  const ParsedBitflipProgramInstruction(this.instructionType);

  final BitflipProgramInstruction instructionType;
}

/// A parsed InitializeConfig instruction.
final class ParsedInitializeConfig extends ParsedBitflipProgramInstruction {
  const ParsedInitializeConfig({required this.data})
      : super(BitflipProgramInstruction.initializeConfig);

  final InitializeConfigInstructionData data;
}

/// A parsed UpdateConfig instruction.
final class ParsedUpdateConfig extends ParsedBitflipProgramInstruction {
  const ParsedUpdateConfig({required this.data})
      : super(BitflipProgramInstruction.updateConfig);

  final UpdateConfigInstructionData data;
}

/// A parsed ProposeAuthority instruction.
final class ParsedProposeAuthority extends ParsedBitflipProgramInstruction {
  const ParsedProposeAuthority({required this.data})
      : super(BitflipProgramInstruction.proposeAuthority);

  final ProposeAuthorityInstructionData data;
}

/// A parsed AcceptAuthority instruction.
final class ParsedAcceptAuthority extends ParsedBitflipProgramInstruction {
  const ParsedAcceptAuthority({required this.data})
      : super(BitflipProgramInstruction.acceptAuthority);

  final AcceptAuthorityInstructionData data;
}

/// A parsed InitializeGame instruction.
final class ParsedInitializeGame extends ParsedBitflipProgramInstruction {
  const ParsedInitializeGame({required this.data})
      : super(BitflipProgramInstruction.initializeGame);

  final InitializeGameInstructionData data;
}

/// A parsed ClaimSection instruction.
final class ParsedClaimSection extends ParsedBitflipProgramInstruction {
  const ParsedClaimSection({required this.data})
      : super(BitflipProgramInstruction.claimSection);

  final ClaimSectionInstructionData data;
}

/// A parsed FlipPixels instruction.
final class ParsedFlipPixels extends ParsedBitflipProgramInstruction {
  const ParsedFlipPixels({required this.data})
      : super(BitflipProgramInstruction.flipPixels);

  final FlipPixelsInstructionData data;
}

/// A parsed SealSection instruction.
final class ParsedSealSection extends ParsedBitflipProgramInstruction {
  const ParsedSealSection({required this.data})
      : super(BitflipProgramInstruction.sealSection);

  final SealSectionInstructionData data;
}

/// A parsed RecordSectionMint instruction.
final class ParsedRecordSectionMint extends ParsedBitflipProgramInstruction {
  const ParsedRecordSectionMint({required this.data})
      : super(BitflipProgramInstruction.recordSectionMint);

  final RecordSectionMintInstructionData data;
}

/// A parsed ListSection instruction.
final class ParsedListSection extends ParsedBitflipProgramInstruction {
  const ParsedListSection({required this.data})
      : super(BitflipProgramInstruction.listSection);

  final ListSectionInstructionData data;
}

/// A parsed CancelSectionListing instruction.
final class ParsedCancelSectionListing extends ParsedBitflipProgramInstruction {
  const ParsedCancelSectionListing({required this.data})
      : super(BitflipProgramInstruction.cancelSectionListing);

  final CancelSectionListingInstructionData data;
}

/// A parsed PurchaseSection instruction.
final class ParsedPurchaseSection extends ParsedBitflipProgramInstruction {
  const ParsedPurchaseSection({required this.data})
      : super(BitflipProgramInstruction.purchaseSection);

  final PurchaseSectionInstructionData data;
}

/// A parsed SettleSectionEconomy instruction.
final class ParsedSettleSectionEconomy extends ParsedBitflipProgramInstruction {
  const ParsedSettleSectionEconomy({required this.data})
      : super(BitflipProgramInstruction.settleSectionEconomy);

  final SettleSectionEconomyInstructionData data;
}

/// Parses a BitflipProgram instruction.
ParsedBitflipProgramInstruction parseBitflipProgramInstruction(
  Instruction instruction,
) {
  return switch (identifyBitflipProgramInstruction(
    instruction.data ?? Uint8List(0),
  )) {
    BitflipProgramInstruction.initializeConfig => ParsedInitializeConfig(
      data: parseInitializeConfigInstruction(instruction),
    ),
    BitflipProgramInstruction.updateConfig => ParsedUpdateConfig(
      data: parseUpdateConfigInstruction(instruction),
    ),
    BitflipProgramInstruction.proposeAuthority => ParsedProposeAuthority(
      data: parseProposeAuthorityInstruction(instruction),
    ),
    BitflipProgramInstruction.acceptAuthority => ParsedAcceptAuthority(
      data: parseAcceptAuthorityInstruction(instruction),
    ),
    BitflipProgramInstruction.initializeGame => ParsedInitializeGame(
      data: parseInitializeGameInstruction(instruction),
    ),
    BitflipProgramInstruction.claimSection => ParsedClaimSection(
      data: parseClaimSectionInstruction(instruction),
    ),
    BitflipProgramInstruction.flipPixels => ParsedFlipPixels(
      data: parseFlipPixelsInstruction(instruction),
    ),
    BitflipProgramInstruction.sealSection => ParsedSealSection(
      data: parseSealSectionInstruction(instruction),
    ),
    BitflipProgramInstruction.recordSectionMint => ParsedRecordSectionMint(
      data: parseRecordSectionMintInstruction(instruction),
    ),
    BitflipProgramInstruction.listSection => ParsedListSection(
      data: parseListSectionInstruction(instruction),
    ),
    BitflipProgramInstruction.cancelSectionListing => ParsedCancelSectionListing(
      data: parseCancelSectionListingInstruction(instruction),
    ),
    BitflipProgramInstruction.purchaseSection => ParsedPurchaseSection(
      data: parsePurchaseSectionInstruction(instruction),
    ),
    BitflipProgramInstruction.settleSectionEconomy => ParsedSettleSectionEconomy(
      data: parseSettleSectionEconomyInstruction(instruction),
    ),
  };
}
