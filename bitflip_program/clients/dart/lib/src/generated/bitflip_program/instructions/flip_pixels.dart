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
class FlipPixelsInstructionData {
  const FlipPixelsInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
    required this.count,
    required this.coordinates,
    required this.maximumTotalFeeLamports,
  }) :
      discriminator = 6;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
  final int count;
  final Uint8List coordinates;
  final BigInt maximumTotalFeeLamports;
}

Encoder<FlipPixelsInstructionData> getFlipPixelsInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('count', getU8Encoder()),
    ('coordinates', fixEncoderSize(getBytesEncoder(), 32, allowTruncation: false)),
    ('maximumTotalFeeLamports', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (FlipPixelsInstructionData value) => <String, Object?>{
      'discriminator': 6,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'count': value.count,
      'coordinates': value.coordinates,
      'maximumTotalFeeLamports': value.maximumTotalFeeLamports,
    },
  );
}

Decoder<FlipPixelsInstructionData> getFlipPixelsInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('count', getU8Decoder()),
    ('coordinates', fixDecoderSize(getBytesDecoder(), 32)),
    ('maximumTotalFeeLamports', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'flipPixels instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (FlipPixelsInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(6),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      FlipPixelsInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      count: map['count']! as int,
      coordinates: map['coordinates']! as Uint8List,
      maximumTotalFeeLamports: map['maximumTotalFeeLamports']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<FlipPixelsInstructionData>(
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
      VariableSizeDecoder<FlipPixelsInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<FlipPixelsInstructionData, FlipPixelsInstructionData> getFlipPixelsInstructionDataCodec() {
  return combineCodec(getFlipPixelsInstructionDataEncoder(), getFlipPixelsInstructionDataDecoder());
}

/// Creates a [FlipPixels] instruction.
Instruction getFlipPixelsInstruction({
  required Address programAddress,
  required Address player,
  required Address config,
  required Address game,
  required Address section,
  required Address treasury,
  required Address systemProgram,
  required int gameIndex,
  required int sectionIndex,
  required int count,
  required Uint8List coordinates,
  required BigInt maximumTotalFeeLamports,
}) {
  final instructionData = FlipPixelsInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      count: count,
      coordinates: coordinates,
      maximumTotalFeeLamports: maximumTotalFeeLamports,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: player, role: AccountRole.writableSigner),
    AccountMeta(address: config, role: AccountRole.readonly),
    AccountMeta(address: game, role: AccountRole.writable),
    AccountMeta(address: section, role: AccountRole.writable),
    AccountMeta(address: treasury, role: AccountRole.writable),
    AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getFlipPixelsInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [FlipPixels] instruction from raw instruction data.
FlipPixelsInstructionData parseFlipPixelsInstruction(Instruction instruction) {
  return getFlipPixelsInstructionDataDecoder().decode(instruction.data!);
}
