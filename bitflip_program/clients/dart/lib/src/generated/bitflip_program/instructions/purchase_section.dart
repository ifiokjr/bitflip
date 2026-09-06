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
class PurchaseSectionInstructionData {
  const PurchaseSectionInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
    required this.maximumPriceLamports,
  }) :
      discriminator = 11;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
  final BigInt maximumPriceLamports;
}

Encoder<PurchaseSectionInstructionData> getPurchaseSectionInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('maximumPriceLamports', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (PurchaseSectionInstructionData value) => <String, Object?>{
      'discriminator': 11,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'maximumPriceLamports': value.maximumPriceLamports,
    },
  );
}

Decoder<PurchaseSectionInstructionData> getPurchaseSectionInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('maximumPriceLamports', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'purchaseSection instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (PurchaseSectionInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(11),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      PurchaseSectionInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      maximumPriceLamports: map['maximumPriceLamports']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<PurchaseSectionInstructionData>(
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
      VariableSizeDecoder<PurchaseSectionInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<PurchaseSectionInstructionData, PurchaseSectionInstructionData> getPurchaseSectionInstructionDataCodec() {
  return combineCodec(getPurchaseSectionInstructionDataEncoder(), getPurchaseSectionInstructionDataDecoder());
}

/// Creates a [PurchaseSection] instruction.
Instruction getPurchaseSectionInstruction({
  required Address programAddress,
  required Address buyer,
  required Address seller,
  required Address game,
  required Address section,
  required Address systemProgram,
  required int gameIndex,
  required int sectionIndex,
  required BigInt maximumPriceLamports,
}) {
  final instructionData = PurchaseSectionInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      maximumPriceLamports: maximumPriceLamports,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: buyer, role: AccountRole.writableSigner),
    AccountMeta(address: seller, role: AccountRole.writable),
    AccountMeta(address: game, role: AccountRole.readonly),
    AccountMeta(address: section, role: AccountRole.writable),
    AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getPurchaseSectionInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [PurchaseSection] instruction from raw instruction data.
PurchaseSectionInstructionData parsePurchaseSectionInstruction(Instruction instruction) {
  return getPurchaseSectionInstructionDataDecoder().decode(instruction.data!);
}
