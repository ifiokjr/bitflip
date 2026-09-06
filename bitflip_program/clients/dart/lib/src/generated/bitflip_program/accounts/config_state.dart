// Auto-generated. Do not edit.
// ignore_for_file: type=lint


import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_accounts/solana_kit_accounts.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';


@immutable
class ConfigState {
  const ConfigState({
    required this.version,
    required this.authority,
    required this.pendingAuthority,
    required this.treasury,
    required this.collectionAuthority,
    required this.claimPriceLamports,
    required this.flipFeeLamports,
    required this.minimumFlipFeeLamports,
    required this.maximumFlipFeeLamports,
    required this.unlockIntervalSeconds,
    required this.earlyUnlockFlips,
    required this.gameCount,
    required this.bump,
  }) :
      discriminator = 1;

  final int discriminator;
  final int version;
  final Address authority;
  final Address pendingAuthority;
  final Address treasury;
  final Address collectionAuthority;
  final BigInt claimPriceLamports;
  final BigInt flipFeeLamports;
  final BigInt minimumFlipFeeLamports;
  final BigInt maximumFlipFeeLamports;
  final int unlockIntervalSeconds;
  final int earlyUnlockFlips;
  final int gameCount;
  final int bump;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ConfigState &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          version == other.version &&
          authority == other.authority &&
          pendingAuthority == other.pendingAuthority &&
          treasury == other.treasury &&
          collectionAuthority == other.collectionAuthority &&
          claimPriceLamports == other.claimPriceLamports &&
          flipFeeLamports == other.flipFeeLamports &&
          minimumFlipFeeLamports == other.minimumFlipFeeLamports &&
          maximumFlipFeeLamports == other.maximumFlipFeeLamports &&
          unlockIntervalSeconds == other.unlockIntervalSeconds &&
          earlyUnlockFlips == other.earlyUnlockFlips &&
          gameCount == other.gameCount &&
          bump == other.bump;

  @override
  int get hashCode => Object.hashAll([discriminator, version, authority, pendingAuthority, treasury, collectionAuthority, claimPriceLamports, flipFeeLamports, minimumFlipFeeLamports, maximumFlipFeeLamports, unlockIntervalSeconds, earlyUnlockFlips, gameCount, bump]);

  @override
  String toString() => 'ConfigState(discriminator: $discriminator, version: $version, authority: $authority, pendingAuthority: $pendingAuthority, treasury: $treasury, collectionAuthority: $collectionAuthority, claimPriceLamports: $claimPriceLamports, flipFeeLamports: $flipFeeLamports, minimumFlipFeeLamports: $minimumFlipFeeLamports, maximumFlipFeeLamports: $maximumFlipFeeLamports, unlockIntervalSeconds: $unlockIntervalSeconds, earlyUnlockFlips: $earlyUnlockFlips, gameCount: $gameCount, bump: $bump)';
}


Encoder<ConfigState> getConfigStateEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('version', getU8Encoder()),
    ('authority', getAddressEncoder()),
    ('pendingAuthority', getAddressEncoder()),
    ('treasury', getAddressEncoder()),
    ('collectionAuthority', getAddressEncoder()),
    ('claimPriceLamports', getU64Encoder()),
    ('flipFeeLamports', getU64Encoder()),
    ('minimumFlipFeeLamports', getU64Encoder()),
    ('maximumFlipFeeLamports', getU64Encoder()),
    ('unlockIntervalSeconds', getU32Encoder()),
    ('earlyUnlockFlips', getU32Encoder()),
    ('gameCount', getU16Encoder()),
    ('bump', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ConfigState value) => <String, Object?>{
      'discriminator': 1,
      'version': value.version,
      'authority': value.authority,
      'pendingAuthority': value.pendingAuthority,
      'treasury': value.treasury,
      'collectionAuthority': value.collectionAuthority,
      'claimPriceLamports': value.claimPriceLamports,
      'flipFeeLamports': value.flipFeeLamports,
      'minimumFlipFeeLamports': value.minimumFlipFeeLamports,
      'maximumFlipFeeLamports': value.maximumFlipFeeLamports,
      'unlockIntervalSeconds': value.unlockIntervalSeconds,
      'earlyUnlockFlips': value.earlyUnlockFlips,
      'gameCount': value.gameCount,
      'bump': value.bump,
    },
  );
}

Decoder<ConfigState> getConfigStateDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('version', getU8Decoder()),
    ('authority', getAddressDecoder()),
    ('pendingAuthority', getAddressDecoder()),
    ('treasury', getAddressDecoder()),
    ('collectionAuthority', getAddressDecoder()),
    ('claimPriceLamports', getU64Decoder()),
    ('flipFeeLamports', getU64Decoder()),
    ('minimumFlipFeeLamports', getU64Decoder()),
    ('maximumFlipFeeLamports', getU64Decoder()),
    ('unlockIntervalSeconds', getU32Decoder()),
    ('earlyUnlockFlips', getU32Decoder()),
    ('gameCount', getU16Decoder()),
    ('bump', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'configState account decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (ConfigState, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(1),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);

    return (
      ConfigState(
      version: map['version']! as int,
      authority: map['authority']! as Address,
      pendingAuthority: map['pendingAuthority']! as Address,
      treasury: map['treasury']! as Address,
      collectionAuthority: map['collectionAuthority']! as Address,
      claimPriceLamports: map['claimPriceLamports']! as BigInt,
      flipFeeLamports: map['flipFeeLamports']! as BigInt,
      minimumFlipFeeLamports: map['minimumFlipFeeLamports']! as BigInt,
      maximumFlipFeeLamports: map['maximumFlipFeeLamports']! as BigInt,
      unlockIntervalSeconds: map['unlockIntervalSeconds']! as int,
      earlyUnlockFlips: map['earlyUnlockFlips']! as int,
      gameCount: map['gameCount']! as int,
      bump: map['bump']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ConfigState>(
        fixedSize: structDecoder.fixedSize,
        read: (bytes, offset) {
          final bytesLength = bytes.length - offset;
          if (bytesLength < structDecoder.fixedSize) {
            throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
          }
          return readTopLevel(bytes, offset);
        },
      ),
    VariableSizeDecoder<Map<String, Object?>>() =>
      VariableSizeDecoder<ConfigState>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ConfigState, ConfigState> getConfigStateCodec() {
  return combineCodec(getConfigStateEncoder(), getConfigStateDecoder());
}

Account<ConfigState> decodeConfigState(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getConfigStateDecoder());
}
