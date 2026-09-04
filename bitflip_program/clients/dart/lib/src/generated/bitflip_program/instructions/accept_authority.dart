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
class AcceptAuthorityInstructionData {
  const AcceptAuthorityInstructionData() :
      discriminator = 3;

  final int discriminator;
}

Encoder<AcceptAuthorityInstructionData> getAcceptAuthorityInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (AcceptAuthorityInstructionData value) => <String, Object?>{
      'discriminator': 3,
    },
  );
}

Decoder<AcceptAuthorityInstructionData> getAcceptAuthorityInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'acceptAuthority instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (AcceptAuthorityInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(3),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      AcceptAuthorityInstructionData(

      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<AcceptAuthorityInstructionData>(
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
      VariableSizeDecoder<AcceptAuthorityInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<AcceptAuthorityInstructionData, AcceptAuthorityInstructionData> getAcceptAuthorityInstructionDataCodec() {
  return combineCodec(getAcceptAuthorityInstructionDataEncoder(), getAcceptAuthorityInstructionDataDecoder());
}

/// Creates a [AcceptAuthority] instruction.
Instruction getAcceptAuthorityInstruction({
  required Address programAddress,
  required Address pendingAuthority,
  required Address config,

}) {
  final instructionData = AcceptAuthorityInstructionData(

  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: pendingAuthority, role: AccountRole.readonlySigner),
    AccountMeta(address: config, role: AccountRole.writable),
    ],
    data: getAcceptAuthorityInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [AcceptAuthority] instruction from raw instruction data.
AcceptAuthorityInstructionData parseAcceptAuthorityInstruction(Instruction instruction) {
  return getAcceptAuthorityInstructionDataDecoder().decode(instruction.data!);
}
