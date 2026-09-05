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
class ClaimSectionInstructionData {
  const ClaimSectionInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
    required this.bump,
    required this.maximumPriceLamports,
  }) :
      discriminator = 5;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
  final int bump;
  final BigInt maximumPriceLamports;
}

Encoder<ClaimSectionInstructionData> getClaimSectionInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('bump', getU8Encoder()),
    ('maximumPriceLamports', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ClaimSectionInstructionData value) => <String, Object?>{
      'discriminator': 5,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'bump': value.bump,
      'maximumPriceLamports': value.maximumPriceLamports,
    },
  );
}

Decoder<ClaimSectionInstructionData> getClaimSectionInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('bump', getU8Decoder()),
    ('maximumPriceLamports', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'claimSection instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (ClaimSectionInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(5),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      ClaimSectionInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      bump: map['bump']! as int,
      maximumPriceLamports: map['maximumPriceLamports']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ClaimSectionInstructionData>(
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
      VariableSizeDecoder<ClaimSectionInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ClaimSectionInstructionData, ClaimSectionInstructionData> getClaimSectionInstructionDataCodec() {
  return combineCodec(getClaimSectionInstructionDataEncoder(), getClaimSectionInstructionDataDecoder());
}

/// Creates a [ClaimSection] instruction.
Instruction getClaimSectionInstruction({
  required Address programAddress,
  required Address owner,
  required Address config,
  required Address game,
  required Address previousSection,
  required Address section,
  required Address treasury,
  required Address systemProgram,
  required int gameIndex,
  required int sectionIndex,
  required int bump,
  required BigInt maximumPriceLamports,
}) {
  final instructionData = ClaimSectionInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      bump: bump,
      maximumPriceLamports: maximumPriceLamports,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: owner, role: AccountRole.writableSigner),
    AccountMeta(address: config, role: AccountRole.readonly),
    AccountMeta(address: game, role: AccountRole.writable),
    AccountMeta(address: previousSection, role: AccountRole.readonly),
    AccountMeta(address: section, role: AccountRole.writable),
    AccountMeta(address: treasury, role: AccountRole.writable),
    AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getClaimSectionInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ClaimSection] instruction from raw instruction data.
ClaimSectionInstructionData parseClaimSectionInstruction(Instruction instruction) {
  return getClaimSectionInstructionDataDecoder().decode(instruction.data!);
}
