// Auto-generated. Do not edit.
// ignore_for_file: type=lint


import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';


@immutable
class FundSectionVaultInstructionData {
  const FundSectionVaultInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
  }) :
      discriminator = 14;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
}

Encoder<FundSectionVaultInstructionData> getFundSectionVaultInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (FundSectionVaultInstructionData value) => <String, Object?>{
      'discriminator': 14,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
    },
  );
}

Decoder<FundSectionVaultInstructionData> getFundSectionVaultInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'fundSectionVault instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (FundSectionVaultInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(14),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      FundSectionVaultInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<FundSectionVaultInstructionData>(
        fixedSize: structDecoder.fixedSize,
        read: (bytes, offset) {
          final bytesLength = bytes.length - offset;
          if (bytesLength != structDecoder.fixedSize) {
            throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
          }
          return readTopLevel(bytes, offset);
        },
      ),
    VariableSizeDecoder<Map<String, Object?>>() =>
      VariableSizeDecoder<FundSectionVaultInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<FundSectionVaultInstructionData, FundSectionVaultInstructionData> getFundSectionVaultInstructionDataCodec() {
  return combineCodec(getFundSectionVaultInstructionDataEncoder(), getFundSectionVaultInstructionDataDecoder());
}

/// Creates a [FundSectionVault] instruction.
Instruction getFundSectionVaultInstruction({
  required Address programAddress,
  required Address funder,
  required Address config,
  required Address section,
  required Address bitMint,
  required Address bitReserve,
  required Address sectionVault,
  required Address associatedTokenProgram,
  required Address tokenProgram,
  required Address systemProgram,
  required int gameIndex,
  required int sectionIndex,
}) {
  final instructionData = FundSectionVaultInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: funder, role: AccountRole.writableSigner),
    AccountMeta(address: config, role: AccountRole.readonly),
    AccountMeta(address: section, role: AccountRole.writable),
    AccountMeta(address: bitMint, role: AccountRole.readonly),
    AccountMeta(address: bitReserve, role: AccountRole.writable),
    AccountMeta(address: sectionVault, role: AccountRole.writable),
    AccountMeta(address: associatedTokenProgram, role: AccountRole.readonly),
    AccountMeta(address: tokenProgram, role: AccountRole.readonly),
    AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getFundSectionVaultInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [FundSectionVault] instruction from raw instruction data.
FundSectionVaultInstructionData parseFundSectionVaultInstruction(Instruction instruction) {
  return getFundSectionVaultInstructionDataDecoder().decode(instruction.data!);
}
