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
class ConfigureBitCustodyInstructionData {
  const ConfigureBitCustodyInstructionData() :
      discriminator = 13;

  final int discriminator;
}

Encoder<ConfigureBitCustodyInstructionData> getConfigureBitCustodyInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ConfigureBitCustodyInstructionData value) => <String, Object?>{
      'discriminator': 13,
    },
  );
}

Decoder<ConfigureBitCustodyInstructionData> getConfigureBitCustodyInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'configureBitCustody instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (ConfigureBitCustodyInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(13),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      ConfigureBitCustodyInstructionData(

      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ConfigureBitCustodyInstructionData>(
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
      VariableSizeDecoder<ConfigureBitCustodyInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ConfigureBitCustodyInstructionData, ConfigureBitCustodyInstructionData> getConfigureBitCustodyInstructionDataCodec() {
  return combineCodec(getConfigureBitCustodyInstructionDataEncoder(), getConfigureBitCustodyInstructionDataDecoder());
}

/// Creates a [ConfigureBitCustody] instruction.
Instruction getConfigureBitCustodyInstruction({
  required Address programAddress,
  required Address authority,
  required Address config,
  required Address bitMint,
  required Address bitReserve,
  required Address tokenProgram,

}) {
  final instructionData = ConfigureBitCustodyInstructionData(

  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: authority, role: AccountRole.readonlySigner),
    AccountMeta(address: config, role: AccountRole.writable),
    AccountMeta(address: bitMint, role: AccountRole.readonly),
    AccountMeta(address: bitReserve, role: AccountRole.readonly),
    AccountMeta(address: tokenProgram, role: AccountRole.readonly),
    ],
    data: getConfigureBitCustodyInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ConfigureBitCustody] instruction from raw instruction data.
ConfigureBitCustodyInstructionData parseConfigureBitCustodyInstruction(Instruction instruction) {
  return getConfigureBitCustodyInstructionDataDecoder().decode(instruction.data!);
}
