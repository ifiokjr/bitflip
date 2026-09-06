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
class SectionState {
  const SectionState({
    required this.owner,
    required this.assetId,
    required this.merkleTree,
    required this.bitVault,
    required this.gameIndex,
    required this.sectionIndex,
    required this.status,
    required this.bump,
    required this.onPixels,
    required this.leafIndex,
    required this.flipCount,
    required this.revision,
    required this.lastFlipAt,
    required this.salePriceLamports,
    required this.economyLaunchedAt,
    required this.economyWindowStartedAt,
    required this.economyLastUpdatedAt,
    required this.economyWindowId,
    required this.economyWindowTargetTokens,
    required this.economyWindowRewardedTokens,
    required this.emittedTokens,
    required this.rewardPoolTokens,
    required this.controllerPriceLamports,
    required this.postedPriceLamports,
    required this.pixels,
  }) :
      discriminator = 3;

  final int discriminator;
  final Address owner;
  final Address assetId;
  final Address merkleTree;
  final Address bitVault;
  final int gameIndex;
  final int sectionIndex;
  final int status;
  final int bump;
  final int onPixels;
  final int leafIndex;
  final BigInt flipCount;
  final BigInt revision;
  final BigInt lastFlipAt;
  final BigInt salePriceLamports;
  final BigInt economyLaunchedAt;
  final BigInt economyWindowStartedAt;
  final BigInt economyLastUpdatedAt;
  final BigInt economyWindowId;
  final BigInt economyWindowTargetTokens;
  final BigInt economyWindowRewardedTokens;
  final BigInt emittedTokens;
  final BigInt rewardPoolTokens;
  final BigInt controllerPriceLamports;
  final BigInt postedPriceLamports;
  final Uint8List pixels;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SectionState &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          owner == other.owner &&
          assetId == other.assetId &&
          merkleTree == other.merkleTree &&
          bitVault == other.bitVault &&
          gameIndex == other.gameIndex &&
          sectionIndex == other.sectionIndex &&
          status == other.status &&
          bump == other.bump &&
          onPixels == other.onPixels &&
          leafIndex == other.leafIndex &&
          flipCount == other.flipCount &&
          revision == other.revision &&
          lastFlipAt == other.lastFlipAt &&
          salePriceLamports == other.salePriceLamports &&
          economyLaunchedAt == other.economyLaunchedAt &&
          economyWindowStartedAt == other.economyWindowStartedAt &&
          economyLastUpdatedAt == other.economyLastUpdatedAt &&
          economyWindowId == other.economyWindowId &&
          economyWindowTargetTokens == other.economyWindowTargetTokens &&
          economyWindowRewardedTokens == other.economyWindowRewardedTokens &&
          emittedTokens == other.emittedTokens &&
          rewardPoolTokens == other.rewardPoolTokens &&
          controllerPriceLamports == other.controllerPriceLamports &&
          postedPriceLamports == other.postedPriceLamports &&
          pixels == other.pixels;

  @override
  int get hashCode => Object.hashAll([discriminator, owner, assetId, merkleTree, bitVault, gameIndex, sectionIndex, status, bump, onPixels, leafIndex, flipCount, revision, lastFlipAt, salePriceLamports, economyLaunchedAt, economyWindowStartedAt, economyLastUpdatedAt, economyWindowId, economyWindowTargetTokens, economyWindowRewardedTokens, emittedTokens, rewardPoolTokens, controllerPriceLamports, postedPriceLamports, pixels]);

  @override
  String toString() => 'SectionState(discriminator: $discriminator, owner: $owner, assetId: $assetId, merkleTree: $merkleTree, bitVault: $bitVault, gameIndex: $gameIndex, sectionIndex: $sectionIndex, status: $status, bump: $bump, onPixels: $onPixels, leafIndex: $leafIndex, flipCount: $flipCount, revision: $revision, lastFlipAt: $lastFlipAt, salePriceLamports: $salePriceLamports, economyLaunchedAt: $economyLaunchedAt, economyWindowStartedAt: $economyWindowStartedAt, economyLastUpdatedAt: $economyLastUpdatedAt, economyWindowId: $economyWindowId, economyWindowTargetTokens: $economyWindowTargetTokens, economyWindowRewardedTokens: $economyWindowRewardedTokens, emittedTokens: $emittedTokens, rewardPoolTokens: $rewardPoolTokens, controllerPriceLamports: $controllerPriceLamports, postedPriceLamports: $postedPriceLamports, pixels: $pixels)';
}


Encoder<SectionState> getSectionStateEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('owner', getAddressEncoder()),
    ('assetId', getAddressEncoder()),
    ('merkleTree', getAddressEncoder()),
    ('bitVault', getAddressEncoder()),
    ('gameIndex', getU8Encoder()),
    ('sectionIndex', getU8Encoder()),
    ('status', getU8Encoder()),
    ('bump', getU8Encoder()),
    ('onPixels', getU16Encoder()),
    ('leafIndex', getU32Encoder()),
    ('flipCount', getU64Encoder()),
    ('revision', getU64Encoder()),
    ('lastFlipAt', getI64Encoder()),
    ('salePriceLamports', getU64Encoder()),
    ('economyLaunchedAt', getU64Encoder()),
    ('economyWindowStartedAt', getU64Encoder()),
    ('economyLastUpdatedAt', getU64Encoder()),
    ('economyWindowId', getU64Encoder()),
    ('economyWindowTargetTokens', getU64Encoder()),
    ('economyWindowRewardedTokens', getU64Encoder()),
    ('emittedTokens', getU64Encoder()),
    ('rewardPoolTokens', getU64Encoder()),
    ('controllerPriceLamports', getU64Encoder()),
    ('postedPriceLamports', getU64Encoder()),
    ('pixels', fixEncoderSize(getBytesEncoder(), 512, allowTruncation: false)),
  ]);

  return transformEncoder(
    structEncoder,
    (SectionState value) => <String, Object?>{
      'discriminator': 3,
      'owner': value.owner,
      'assetId': value.assetId,
      'merkleTree': value.merkleTree,
      'bitVault': value.bitVault,
      'gameIndex': value.gameIndex,
      'sectionIndex': value.sectionIndex,
      'status': value.status,
      'bump': value.bump,
      'onPixels': value.onPixels,
      'leafIndex': value.leafIndex,
      'flipCount': value.flipCount,
      'revision': value.revision,
      'lastFlipAt': value.lastFlipAt,
      'salePriceLamports': value.salePriceLamports,
      'economyLaunchedAt': value.economyLaunchedAt,
      'economyWindowStartedAt': value.economyWindowStartedAt,
      'economyLastUpdatedAt': value.economyLastUpdatedAt,
      'economyWindowId': value.economyWindowId,
      'economyWindowTargetTokens': value.economyWindowTargetTokens,
      'economyWindowRewardedTokens': value.economyWindowRewardedTokens,
      'emittedTokens': value.emittedTokens,
      'rewardPoolTokens': value.rewardPoolTokens,
      'controllerPriceLamports': value.controllerPriceLamports,
      'postedPriceLamports': value.postedPriceLamports,
      'pixels': value.pixels,
    },
  );
}

Decoder<SectionState> getSectionStateDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('owner', getAddressDecoder()),
    ('assetId', getAddressDecoder()),
    ('merkleTree', getAddressDecoder()),
    ('bitVault', getAddressDecoder()),
    ('gameIndex', getU8Decoder()),
    ('sectionIndex', getU8Decoder()),
    ('status', getU8Decoder()),
    ('bump', getU8Decoder()),
    ('onPixels', getU16Decoder()),
    ('leafIndex', getU32Decoder()),
    ('flipCount', getU64Decoder()),
    ('revision', getU64Decoder()),
    ('lastFlipAt', getI64Decoder()),
    ('salePriceLamports', getU64Decoder()),
    ('economyLaunchedAt', getU64Decoder()),
    ('economyWindowStartedAt', getU64Decoder()),
    ('economyLastUpdatedAt', getU64Decoder()),
    ('economyWindowId', getU64Decoder()),
    ('economyWindowTargetTokens', getU64Decoder()),
    ('economyWindowRewardedTokens', getU64Decoder()),
    ('emittedTokens', getU64Decoder()),
    ('rewardPoolTokens', getU64Decoder()),
    ('controllerPriceLamports', getU64Decoder()),
    ('postedPriceLamports', getU64Decoder()),
    ('pixels', fixDecoderSize(getBytesDecoder(), 512)),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'sectionState account decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (SectionState, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(3),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);

    return (
      SectionState(
      owner: map['owner']! as Address,
      assetId: map['assetId']! as Address,
      merkleTree: map['merkleTree']! as Address,
      bitVault: map['bitVault']! as Address,
      gameIndex: map['gameIndex']! as int,
      sectionIndex: map['sectionIndex']! as int,
      status: map['status']! as int,
      bump: map['bump']! as int,
      onPixels: map['onPixels']! as int,
      leafIndex: map['leafIndex']! as int,
      flipCount: map['flipCount']! as BigInt,
      revision: map['revision']! as BigInt,
      lastFlipAt: map['lastFlipAt']! as BigInt,
      salePriceLamports: map['salePriceLamports']! as BigInt,
      economyLaunchedAt: map['economyLaunchedAt']! as BigInt,
      economyWindowStartedAt: map['economyWindowStartedAt']! as BigInt,
      economyLastUpdatedAt: map['economyLastUpdatedAt']! as BigInt,
      economyWindowId: map['economyWindowId']! as BigInt,
      economyWindowTargetTokens: map['economyWindowTargetTokens']! as BigInt,
      economyWindowRewardedTokens: map['economyWindowRewardedTokens']! as BigInt,
      emittedTokens: map['emittedTokens']! as BigInt,
      rewardPoolTokens: map['rewardPoolTokens']! as BigInt,
      controllerPriceLamports: map['controllerPriceLamports']! as BigInt,
      postedPriceLamports: map['postedPriceLamports']! as BigInt,
      pixels: map['pixels']! as Uint8List,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SectionState>(
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
      VariableSizeDecoder<SectionState>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SectionState, SectionState> getSectionStateCodec() {
  return combineCodec(getSectionStateEncoder(), getSectionStateDecoder());
}

Account<SectionState> decodeSectionState(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getSectionStateDecoder());
}
