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
class WithdrawProtocolFeesInstructionData {
  const WithdrawProtocolFeesInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
  }) :
      discriminator = 17,
      migrationVersion = 0;

  final int discriminator;
  final int migrationVersion;
  final int gameIndex;
  final int sectionIndex;
}

Encoder<WithdrawProtocolFeesInstructionData> getWithdrawProtocolFeesInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('migrationVersion', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (WithdrawProtocolFeesInstructionData value) => <String, Object?>{
      'discriminator': 17,
      'migrationVersion': 0,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
    },
  );
}

Decoder<WithdrawProtocolFeesInstructionData> getWithdrawProtocolFeesInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('migrationVersion', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'withdrawProtocolFees instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (WithdrawProtocolFeesInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(17),
    ).read(bytes, offset + 0);
    getConstantDecoder(
      getU8Encoder().encode(0),
    ).read(bytes, offset + 1);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      WithdrawProtocolFeesInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<WithdrawProtocolFeesInstructionData>(
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
      VariableSizeDecoder<WithdrawProtocolFeesInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<WithdrawProtocolFeesInstructionData, WithdrawProtocolFeesInstructionData> getWithdrawProtocolFeesInstructionDataCodec() {
  return combineCodec(getWithdrawProtocolFeesInstructionDataEncoder(), getWithdrawProtocolFeesInstructionDataDecoder());
}

/// Creates a [WithdrawProtocolFees] instruction.
Instruction getWithdrawProtocolFeesInstruction({
  required Address programAddress,
  required Address authority,
  required Address config,
  required Address section,
  required Address treasury,
  required int gameIndex,
  required int sectionIndex,
}) {
  final instructionData = WithdrawProtocolFeesInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: authority, role: AccountRole.readonlySigner),
    AccountMeta(address: config, role: AccountRole.readonly),
    AccountMeta(address: section, role: AccountRole.writable),
    AccountMeta(address: treasury, role: AccountRole.writable),
    ],
    data: getWithdrawProtocolFeesInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [WithdrawProtocolFees] instruction from raw instruction data.
WithdrawProtocolFeesInstructionData parseWithdrawProtocolFeesInstruction(Instruction instruction) {
  return getWithdrawProtocolFeesInstructionDataDecoder().decode(instruction.data!);
}
