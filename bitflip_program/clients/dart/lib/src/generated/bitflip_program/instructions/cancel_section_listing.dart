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
class CancelSectionListingInstructionData {
  const CancelSectionListingInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
  }) :
      discriminator = 10;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
}

Encoder<CancelSectionListingInstructionData> getCancelSectionListingInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (CancelSectionListingInstructionData value) => <String, Object?>{
      'discriminator': 10,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
    },
  );
}

Decoder<CancelSectionListingInstructionData> getCancelSectionListingInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'cancelSectionListing instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (CancelSectionListingInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(10),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      CancelSectionListingInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<CancelSectionListingInstructionData>(
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
      VariableSizeDecoder<CancelSectionListingInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<CancelSectionListingInstructionData, CancelSectionListingInstructionData> getCancelSectionListingInstructionDataCodec() {
  return combineCodec(getCancelSectionListingInstructionDataEncoder(), getCancelSectionListingInstructionDataDecoder());
}

/// Creates a [CancelSectionListing] instruction.
Instruction getCancelSectionListingInstruction({
  required Address programAddress,
  required Address owner,
  required Address game,
  required Address section,
  required int gameIndex,
  required int sectionIndex,
}) {
  final instructionData = CancelSectionListingInstructionData(
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
    data: getCancelSectionListingInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [CancelSectionListing] instruction from raw instruction data.
CancelSectionListingInstructionData parseCancelSectionListingInstruction(Instruction instruction) {
  return getCancelSectionListingInstructionDataDecoder().decode(instruction.data!);
}
