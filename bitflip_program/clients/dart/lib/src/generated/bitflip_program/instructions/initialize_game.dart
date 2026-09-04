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
class InitializeGameInstructionData {
  const InitializeGameInstructionData({
    required this.gameIndex,
    required this.bump,
  }) :
      discriminator = 4;

  final int discriminator;
  final int gameIndex;
  final int bump;
}

Encoder<InitializeGameInstructionData> getInitializeGameInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('bump', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (InitializeGameInstructionData value) => <String, Object?>{
      'discriminator': 4,
      'gameIndex': value.gameIndex,
      'bump': value.bump,
    },
  );
}

Decoder<InitializeGameInstructionData> getInitializeGameInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('bump', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'initializeGame instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (InitializeGameInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(4),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      InitializeGameInstructionData(
      gameIndex: map['gameIndex']! as int,
      bump: map['bump']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<InitializeGameInstructionData>(
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
      VariableSizeDecoder<InitializeGameInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<InitializeGameInstructionData, InitializeGameInstructionData> getInitializeGameInstructionDataCodec() {
  return combineCodec(getInitializeGameInstructionDataEncoder(), getInitializeGameInstructionDataDecoder());
}

/// Creates a [InitializeGame] instruction.
Instruction getInitializeGameInstruction({
  required Address programAddress,
  required Address payer,
  required Address config,
  required Address game,
  required Address systemProgram,
  required int gameIndex,
  required int bump,
}) {
  final instructionData = InitializeGameInstructionData(
      gameIndex: gameIndex,
      bump: bump,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: payer, role: AccountRole.writableSigner),
    AccountMeta(address: config, role: AccountRole.writable),
    AccountMeta(address: game, role: AccountRole.writable),
    AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getInitializeGameInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [InitializeGame] instruction from raw instruction data.
InitializeGameInstructionData parseInitializeGameInstruction(Instruction instruction) {
  return getInitializeGameInstructionDataDecoder().decode(instruction.data!);
}
