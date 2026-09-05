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
class ProposeAuthorityInstructionData {
  const ProposeAuthorityInstructionData({
    required this.pendingAuthority,
  }) :
      discriminator = 2;

  final int discriminator;
  final Address pendingAuthority;
}

Encoder<ProposeAuthorityInstructionData> getProposeAuthorityInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('pendingAuthority', getAddressEncoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ProposeAuthorityInstructionData value) => <String, Object?>{
      'discriminator': 2,
      'pendingAuthority': value.pendingAuthority,
    },
  );
}

Decoder<ProposeAuthorityInstructionData> getProposeAuthorityInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('pendingAuthority', getAddressDecoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'proposeAuthority instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (ProposeAuthorityInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(2),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      ProposeAuthorityInstructionData(
      pendingAuthority: map['pendingAuthority']! as Address,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ProposeAuthorityInstructionData>(
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
      VariableSizeDecoder<ProposeAuthorityInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ProposeAuthorityInstructionData, ProposeAuthorityInstructionData> getProposeAuthorityInstructionDataCodec() {
  return combineCodec(getProposeAuthorityInstructionDataEncoder(), getProposeAuthorityInstructionDataDecoder());
}

/// Creates a [ProposeAuthority] instruction.
Instruction getProposeAuthorityInstruction({
  required Address programAddress,
  required Address authority,
  required Address config,
  required Address pendingAuthority,
}) {
  final instructionData = ProposeAuthorityInstructionData(
      pendingAuthority: pendingAuthority,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: authority, role: AccountRole.readonlySigner),
    AccountMeta(address: config, role: AccountRole.writable),
    ],
    data: getProposeAuthorityInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ProposeAuthority] instruction from raw instruction data.
ProposeAuthorityInstructionData parseProposeAuthorityInstruction(Instruction instruction) {
  return getProposeAuthorityInstructionDataDecoder().decode(instruction.data!);
}
