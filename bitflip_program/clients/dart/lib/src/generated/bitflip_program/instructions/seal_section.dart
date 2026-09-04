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
class SealSectionInstructionData {
  const SealSectionInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
  }) :
      discriminator = 7;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
}

Encoder<SealSectionInstructionData> getSealSectionInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SealSectionInstructionData value) => <String, Object?>{
      'discriminator': 7,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
    },
  );
}

Decoder<SealSectionInstructionData> getSealSectionInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'sealSection instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (SealSectionInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(7),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      SealSectionInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SealSectionInstructionData>(
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
      VariableSizeDecoder<SealSectionInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SealSectionInstructionData, SealSectionInstructionData> getSealSectionInstructionDataCodec() {
  return combineCodec(getSealSectionInstructionDataEncoder(), getSealSectionInstructionDataDecoder());
}

/// Creates a [SealSection] instruction.
Instruction getSealSectionInstruction({
  required Address programAddress,
  required Address owner,
  required Address game,
  required Address section,
  required int gameIndex,
  required int sectionIndex,
}) {
  final instructionData = SealSectionInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: owner, role: AccountRole.readonlySigner),
    AccountMeta(address: game, role: AccountRole.readonly),
    AccountMeta(address: section, role: AccountRole.writable),
    ],
    data: getSealSectionInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SealSection] instruction from raw instruction data.
SealSectionInstructionData parseSealSectionInstruction(Instruction instruction) {
  return getSealSectionInstructionDataDecoder().decode(instruction.data!);
}
