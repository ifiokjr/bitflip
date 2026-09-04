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
class UpdateConfigInstructionData {
  const UpdateConfigInstructionData({
    required this.treasury,
    required this.collectionAuthority,
    required this.claimPriceLamports,
    required this.flipFeeLamports,
    required this.minimumFlipFeeLamports,
    required this.maximumFlipFeeLamports,
    required this.unlockIntervalSeconds,
    required this.earlyUnlockFlips,
  }) :
      discriminator = 1;

  final int discriminator;
  final Address treasury;
  final Address collectionAuthority;
  final BigInt claimPriceLamports;
  final BigInt flipFeeLamports;
  final BigInt minimumFlipFeeLamports;
  final BigInt maximumFlipFeeLamports;
  final int unlockIntervalSeconds;
  final int earlyUnlockFlips;
}

Encoder<UpdateConfigInstructionData> getUpdateConfigInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('treasury', getAddressEncoder()),
    ('collectionAuthority', getAddressEncoder()),
    ('claimPriceLamports', getU64Encoder()),
    ('flipFeeLamports', getU64Encoder()),
    ('minimumFlipFeeLamports', getU64Encoder()),
    ('maximumFlipFeeLamports', getU64Encoder()),
    ('unlockIntervalSeconds', getU32Encoder()),
    ('earlyUnlockFlips', getU32Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (UpdateConfigInstructionData value) => <String, Object?>{
      'discriminator': 1,
      'treasury': value.treasury,
      'collectionAuthority': value.collectionAuthority,
      'claimPriceLamports': value.claimPriceLamports,
      'flipFeeLamports': value.flipFeeLamports,
      'minimumFlipFeeLamports': value.minimumFlipFeeLamports,
      'maximumFlipFeeLamports': value.maximumFlipFeeLamports,
      'unlockIntervalSeconds': value.unlockIntervalSeconds,
      'earlyUnlockFlips': value.earlyUnlockFlips,
    },
  );
}

Decoder<UpdateConfigInstructionData> getUpdateConfigInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('treasury', getAddressDecoder()),
    ('collectionAuthority', getAddressDecoder()),
    ('claimPriceLamports', getU64Decoder()),
    ('flipFeeLamports', getU64Decoder()),
    ('minimumFlipFeeLamports', getU64Decoder()),
    ('maximumFlipFeeLamports', getU64Decoder()),
    ('unlockIntervalSeconds', getU32Decoder()),
    ('earlyUnlockFlips', getU32Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'updateConfig instruction decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (UpdateConfigInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(1),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      UpdateConfigInstructionData(
      treasury: map['treasury']! as Address,
      collectionAuthority: map['collectionAuthority']! as Address,
      claimPriceLamports: map['claimPriceLamports']! as BigInt,
      flipFeeLamports: map['flipFeeLamports']! as BigInt,
      minimumFlipFeeLamports: map['minimumFlipFeeLamports']! as BigInt,
      maximumFlipFeeLamports: map['maximumFlipFeeLamports']! as BigInt,
      unlockIntervalSeconds: map['unlockIntervalSeconds']! as int,
      earlyUnlockFlips: map['earlyUnlockFlips']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<UpdateConfigInstructionData>(
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
      VariableSizeDecoder<UpdateConfigInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<UpdateConfigInstructionData, UpdateConfigInstructionData> getUpdateConfigInstructionDataCodec() {
  return combineCodec(getUpdateConfigInstructionDataEncoder(), getUpdateConfigInstructionDataDecoder());
}

/// Creates a [UpdateConfig] instruction.
Instruction getUpdateConfigInstruction({
  required Address programAddress,
  required Address authority,
  required Address config,
  required Address treasury,
  required Address collectionAuthority,
  required BigInt claimPriceLamports,
  required BigInt flipFeeLamports,
  required BigInt minimumFlipFeeLamports,
  required BigInt maximumFlipFeeLamports,
  required int unlockIntervalSeconds,
  required int earlyUnlockFlips,
}) {
  final instructionData = UpdateConfigInstructionData(
      treasury: treasury,
      collectionAuthority: collectionAuthority,
      claimPriceLamports: claimPriceLamports,
      flipFeeLamports: flipFeeLamports,
      minimumFlipFeeLamports: minimumFlipFeeLamports,
      maximumFlipFeeLamports: maximumFlipFeeLamports,
      unlockIntervalSeconds: unlockIntervalSeconds,
      earlyUnlockFlips: earlyUnlockFlips,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
    AccountMeta(address: authority, role: AccountRole.readonlySigner),
    AccountMeta(address: config, role: AccountRole.writable),
    ],
    data: getUpdateConfigInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [UpdateConfig] instruction from raw instruction data.
UpdateConfigInstructionData parseUpdateConfigInstruction(Instruction instruction) {
  return getUpdateConfigInstructionDataDecoder().decode(instruction.data!);
}
