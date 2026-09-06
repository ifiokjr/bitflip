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
class ConfigureSectionPolicyInstructionData {
  const ConfigureSectionPolicyInstructionData({
    required this.gameIndex,
    required this.sectionIndex,
    required this.expectedPolicyVersion,
    required this.mode,
    required this.paletteId,
    required this.rewardPolicy,
    required this.startsAt,
    required this.endsAt,
    required this.entryPriceTokens,
    required this.rewardPerActionTokens,
    required this.rulesDigest,
  }) :
      discriminator = 16;

  final int discriminator;
  final int gameIndex;
  final int sectionIndex;
  final BigInt expectedPolicyVersion;
  final int mode;
  final int paletteId;
  final int rewardPolicy;
  final BigInt startsAt;
  final BigInt endsAt;
  final BigInt entryPriceTokens;
  final BigInt rewardPerActionTokens;
  final Uint8List rulesDigest;
}

Encoder<ConfigureSectionPolicyInstructionData> getConfigureSectionPolicyInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('expectedPolicyVersion', getU64Encoder()),
    ('mode', getU8Encoder()),
    ('paletteId', getU8Encoder()),
    ('rewardPolicy', getU8Encoder()),
    ('startsAt', getI64Encoder()),
    ('endsAt', getI64Encoder()),
    ('entryPriceTokens', getU64Encoder()),
    ('rewardPerActionTokens', getU64Encoder()),
    ('rulesDigest', fixEncoderSize(getBytesEncoder(), 32, allowTruncation: false)),
  ]);

  return transformEncoder(
    structEncoder,
    (ConfigureSectionPolicyInstructionData value) => <String, Object?>{
      'discriminator': 16,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'expectedPolicyVersion': value.expectedPolicyVersion,
      'mode': value.mode,
      'paletteId': value.paletteId,
      'rewardPolicy': value.rewardPolicy,
      'startsAt': value.startsAt,
      'endsAt': value.endsAt,
      'entryPriceTokens': value.entryPriceTokens,
      'rewardPerActionTokens': value.rewardPerActionTokens,
      'rulesDigest': value.rulesDigest,
    },
  );
}

Decoder<ConfigureSectionPolicyInstructionData> getConfigureSectionPolicyInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('expectedPolicyVersion', getU64Decoder()),
    ('mode', getU8Decoder()),
    ('paletteId', getU8Decoder()),
    ('rewardPolicy', getU8Decoder()),
    ('startsAt', getI64Decoder()),
    ('endsAt', getI64Decoder()),
    ('entryPriceTokens', getU64Decoder()),
    ('rewardPerActionTokens', getU64Decoder()),
    ('rulesDigest', fixDecoderSize(getBytesDecoder(), 32)),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'configureSectionPolicy instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (ConfigureSectionPolicyInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(16),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      ConfigureSectionPolicyInstructionData(
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      expectedPolicyVersion: map['expectedPolicyVersion']! as BigInt,
      mode: map['mode']! as int,
      paletteId: map['paletteId']! as int,
      rewardPolicy: map['rewardPolicy']! as int,
      startsAt: map['startsAt']! as BigInt,
      endsAt: map['endsAt']! as BigInt,
      entryPriceTokens: map['entryPriceTokens']! as BigInt,
      rewardPerActionTokens: map['rewardPerActionTokens']! as BigInt,
      rulesDigest: map['rulesDigest']! as Uint8List,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ConfigureSectionPolicyInstructionData>(
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
      VariableSizeDecoder<ConfigureSectionPolicyInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ConfigureSectionPolicyInstructionData, ConfigureSectionPolicyInstructionData> getConfigureSectionPolicyInstructionDataCodec() {
  return combineCodec(getConfigureSectionPolicyInstructionDataEncoder(), getConfigureSectionPolicyInstructionDataDecoder());
}

/// Creates a [ConfigureSectionPolicy] instruction.
Instruction getConfigureSectionPolicyInstruction({
  required Address programAddress,
  required Address owner,
  required Address section,
  required int gameIndex,
  required int sectionIndex,
  required BigInt expectedPolicyVersion,
  required int mode,
  required int paletteId,
  required int rewardPolicy,
  required BigInt startsAt,
  required BigInt endsAt,
  required BigInt entryPriceTokens,
  required BigInt rewardPerActionTokens,
  required Uint8List rulesDigest,
}) {
  final instructionData = ConfigureSectionPolicyInstructionData(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      expectedPolicyVersion: expectedPolicyVersion,
      mode: mode,
      paletteId: paletteId,
      rewardPolicy: rewardPolicy,
      startsAt: startsAt,
      endsAt: endsAt,
      entryPriceTokens: entryPriceTokens,
      rewardPerActionTokens: rewardPerActionTokens,
      rulesDigest: rulesDigest,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: owner, role: AccountRole.readonlySigner),
    AccountMeta(address: section, role: AccountRole.writable),
    ],
    data: getConfigureSectionPolicyInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ConfigureSectionPolicy] instruction from raw instruction data.
ConfigureSectionPolicyInstructionData parseConfigureSectionPolicyInstruction(Instruction instruction) {
  return getConfigureSectionPolicyInstructionDataDecoder().decode(instruction.data!);
}
