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
class RecordSectionMintInstructionData {
  const RecordSectionMintInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
    required this.expectedOwner,
    required this.assetId,
    required this.merkleTree,
    required this.leafIndex,
  }) :
      discriminator = 8;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
  final Address expectedOwner;
  final Address assetId;
  final Address merkleTree;
  final int leafIndex;
}

Encoder<RecordSectionMintInstructionData> getRecordSectionMintInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('expectedOwner', getAddressEncoder()),
    ('assetId', getAddressEncoder()),
    ('merkleTree', getAddressEncoder()),
    ('leafIndex', getU32Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (RecordSectionMintInstructionData value) => <String, Object?>{
      'discriminator': 8,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'expectedOwner': value.expectedOwner,
      'assetId': value.assetId,
      'merkleTree': value.merkleTree,
      'leafIndex': value.leafIndex,
    },
  );
}

Decoder<RecordSectionMintInstructionData> getRecordSectionMintInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('expectedOwner', getAddressDecoder()),
    ('assetId', getAddressDecoder()),
    ('merkleTree', getAddressDecoder()),
    ('leafIndex', getU32Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'recordSectionMint instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (RecordSectionMintInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(8),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      RecordSectionMintInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      expectedOwner: map['expectedOwner']! as Address,
      assetId: map['assetId']! as Address,
      merkleTree: map['merkleTree']! as Address,
      leafIndex: map['leafIndex']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<RecordSectionMintInstructionData>(
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
      VariableSizeDecoder<RecordSectionMintInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<RecordSectionMintInstructionData, RecordSectionMintInstructionData> getRecordSectionMintInstructionDataCodec() {
  return combineCodec(getRecordSectionMintInstructionDataEncoder(), getRecordSectionMintInstructionDataDecoder());
}

/// Creates a [RecordSectionMint] instruction.
Instruction getRecordSectionMintInstruction({
  required Address programAddress,
  required Address collectionAuthority,
  required Address config,
  required Address game,
  required Address section,
  required int gameIndex,
  required int sectionIndex,
  required Address expectedOwner,
  required Address assetId,
  required Address merkleTree,
  required int leafIndex,
}) {
  final instructionData = RecordSectionMintInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      expectedOwner: expectedOwner,
      assetId: assetId,
      merkleTree: merkleTree,
      leafIndex: leafIndex,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: collectionAuthority, role: AccountRole.readonlySigner),
    AccountMeta(address: config, role: AccountRole.readonly),
    AccountMeta(address: game, role: AccountRole.writable),
    AccountMeta(address: section, role: AccountRole.writable),
    ],
    data: getRecordSectionMintInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [RecordSectionMint] instruction from raw instruction data.
RecordSectionMintInstructionData parseRecordSectionMintInstruction(Instruction instruction) {
  return getRecordSectionMintInstructionDataDecoder().decode(instruction.data!);
}
