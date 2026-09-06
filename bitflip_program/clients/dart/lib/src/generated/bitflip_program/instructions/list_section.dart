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
class ListSectionInstructionData {
  const ListSectionInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
    required this.priceLamports,
  }) :
      discriminator = 9;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
  final BigInt priceLamports;
}

Encoder<ListSectionInstructionData> getListSectionInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('priceLamports', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ListSectionInstructionData value) => <String, Object?>{
      'discriminator': 9,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'priceLamports': value.priceLamports,
    },
  );
}

Decoder<ListSectionInstructionData> getListSectionInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('priceLamports', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'listSection instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (ListSectionInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(9),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      ListSectionInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      priceLamports: map['priceLamports']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ListSectionInstructionData>(
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
      VariableSizeDecoder<ListSectionInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ListSectionInstructionData, ListSectionInstructionData> getListSectionInstructionDataCodec() {
  return combineCodec(getListSectionInstructionDataEncoder(), getListSectionInstructionDataDecoder());
}

/// Creates a [ListSection] instruction.
Instruction getListSectionInstruction({
  required Address programAddress,
  required Address owner,
  required Address game,
  required Address section,
  required int gameIndex,
  required int sectionIndex,
  required BigInt priceLamports,
}) {
  final instructionData = ListSectionInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      priceLamports: priceLamports,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: owner, role: AccountRole.readonlySigner),
    AccountMeta(address: game, role: AccountRole.readonly),
    AccountMeta(address: section, role: AccountRole.writable),
    ],
    data: getListSectionInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ListSection] instruction from raw instruction data.
ListSectionInstructionData parseListSectionInstruction(Instruction instruction) {
  return getListSectionInstructionDataDecoder().decode(instruction.data!);
}
