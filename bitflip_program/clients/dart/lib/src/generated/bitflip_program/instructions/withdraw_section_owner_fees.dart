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
class WithdrawSectionOwnerFeesInstructionData {
  const WithdrawSectionOwnerFeesInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
  }) :
      discriminator = 15;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
}

Encoder<WithdrawSectionOwnerFeesInstructionData> getWithdrawSectionOwnerFeesInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (WithdrawSectionOwnerFeesInstructionData value) => <String, Object?>{
      'discriminator': 15,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
    },
  );
}

Decoder<WithdrawSectionOwnerFeesInstructionData> getWithdrawSectionOwnerFeesInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'withdrawSectionOwnerFees instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (WithdrawSectionOwnerFeesInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(15),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      WithdrawSectionOwnerFeesInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<WithdrawSectionOwnerFeesInstructionData>(
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
      VariableSizeDecoder<WithdrawSectionOwnerFeesInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<WithdrawSectionOwnerFeesInstructionData, WithdrawSectionOwnerFeesInstructionData> getWithdrawSectionOwnerFeesInstructionDataCodec() {
  return combineCodec(getWithdrawSectionOwnerFeesInstructionDataEncoder(), getWithdrawSectionOwnerFeesInstructionDataDecoder());
}

/// Creates a [WithdrawSectionOwnerFees] instruction.
Instruction getWithdrawSectionOwnerFeesInstruction({
  required Address programAddress,
  required Address owner,
  required Address section,
  required int gameIndex,
  required int sectionIndex,
}) {
  final instructionData = WithdrawSectionOwnerFeesInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: owner, role: AccountRole.writableSigner),
    AccountMeta(address: section, role: AccountRole.writable),
    ],
    data: getWithdrawSectionOwnerFeesInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [WithdrawSectionOwnerFees] instruction from raw instruction data.
WithdrawSectionOwnerFeesInstructionData parseWithdrawSectionOwnerFeesInstruction(Instruction instruction) {
  return getWithdrawSectionOwnerFeesInstructionDataDecoder().decode(instruction.data!);
}
