// Auto-generated. Do not edit.
// ignore_for_file: type=lint


import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_accounts/solana_kit_accounts.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';


@immutable
class GameState {
  const GameState({
    required this.gameIndex,
    required this.status,
    required this.bump,
    required this.economyVersion,
    required this.startsAt,
    required this.nextSection,
    required this.mintedSections,
    required this.flipFeeLamports,
    required this.totalFlips,
    required this.sectionAllocationTokens,
    required this.emissionDurationSeconds,
    required this.windowSeconds,
    required this.targetTokensPerWindow,
    required this.startPriceLamports,
    required this.minimumPriceLamports,
    required this.maximumPriceLamports,
    required this.startFloorPriceLamports,
    required this.endFloorPriceLamports,
    required this.changeDenominator,
    required this.burstElasticity,
    required this.ownerShareBasisPoints,
  }) :
      discriminator = 2;

  final int discriminator;
  final int gameIndex;
  final int status;
  final int bump;
  final int economyVersion;
  final BigInt startsAt;
  final int nextSection;
  final int mintedSections;
  final BigInt flipFeeLamports;
  final BigInt totalFlips;
  final BigInt sectionAllocationTokens;
  final BigInt emissionDurationSeconds;
  final BigInt windowSeconds;
  final BigInt targetTokensPerWindow;
  final BigInt startPriceLamports;
  final BigInt minimumPriceLamports;
  final BigInt maximumPriceLamports;
  final BigInt startFloorPriceLamports;
  final BigInt endFloorPriceLamports;
  final BigInt changeDenominator;
  final BigInt burstElasticity;
  final int ownerShareBasisPoints;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is GameState &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          gameIndex == other.gameIndex &&
          status == other.status &&
          bump == other.bump &&
          economyVersion == other.economyVersion &&
          startsAt == other.startsAt &&
          nextSection == other.nextSection &&
          mintedSections == other.mintedSections &&
          flipFeeLamports == other.flipFeeLamports &&
          totalFlips == other.totalFlips &&
          sectionAllocationTokens == other.sectionAllocationTokens &&
          emissionDurationSeconds == other.emissionDurationSeconds &&
          windowSeconds == other.windowSeconds &&
          targetTokensPerWindow == other.targetTokensPerWindow &&
          startPriceLamports == other.startPriceLamports &&
          minimumPriceLamports == other.minimumPriceLamports &&
          maximumPriceLamports == other.maximumPriceLamports &&
          startFloorPriceLamports == other.startFloorPriceLamports &&
          endFloorPriceLamports == other.endFloorPriceLamports &&
          changeDenominator == other.changeDenominator &&
          burstElasticity == other.burstElasticity &&
          ownerShareBasisPoints == other.ownerShareBasisPoints;

  @override
  int get hashCode => Object.hashAll([discriminator, gameIndex, status, bump, economyVersion, startsAt, nextSection, mintedSections, flipFeeLamports, totalFlips, sectionAllocationTokens, emissionDurationSeconds, windowSeconds, targetTokensPerWindow, startPriceLamports, minimumPriceLamports, maximumPriceLamports, startFloorPriceLamports, endFloorPriceLamports, changeDenominator, burstElasticity, ownerShareBasisPoints]);

  @override
  String toString() => 'GameState(discriminator: $discriminator, gameIndex: $gameIndex, status: $status, bump: $bump, economyVersion: $economyVersion, startsAt: $startsAt, nextSection: $nextSection, mintedSections: $mintedSections, flipFeeLamports: $flipFeeLamports, totalFlips: $totalFlips, sectionAllocationTokens: $sectionAllocationTokens, emissionDurationSeconds: $emissionDurationSeconds, windowSeconds: $windowSeconds, targetTokensPerWindow: $targetTokensPerWindow, startPriceLamports: $startPriceLamports, minimumPriceLamports: $minimumPriceLamports, maximumPriceLamports: $maximumPriceLamports, startFloorPriceLamports: $startFloorPriceLamports, endFloorPriceLamports: $endFloorPriceLamports, changeDenominator: $changeDenominator, burstElasticity: $burstElasticity, ownerShareBasisPoints: $ownerShareBasisPoints)';
}


Encoder<GameState> getGameStateEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('status', getU8Encoder()),
    ('bump', getU8Encoder()),
    ('economyVersion', getU8Encoder()),
    ('startsAt', getI64Encoder()),
    ('nextSection', getU16Encoder()),
    ('mintedSections', getU16Encoder()),
    ('flipFeeLamports', getU64Encoder()),
    ('totalFlips', getU64Encoder()),
    ('sectionAllocationTokens', getU64Encoder()),
    ('emissionDurationSeconds', getU64Encoder()),
    ('windowSeconds', getU64Encoder()),
    ('targetTokensPerWindow', getU64Encoder()),
    ('startPriceLamports', getU64Encoder()),
    ('minimumPriceLamports', getU64Encoder()),
    ('maximumPriceLamports', getU64Encoder()),
    ('startFloorPriceLamports', getU64Encoder()),
    ('endFloorPriceLamports', getU64Encoder()),
    ('changeDenominator', getU64Encoder()),
    ('burstElasticity', getU64Encoder()),
    ('ownerShareBasisPoints', getU16Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (GameState value) => <String, Object?>{
      'discriminator': 2,
      'gameIndex': value.gameIndex,
      'status': value.status,
      'bump': value.bump,
      'economyVersion': value.economyVersion,
      'startsAt': value.startsAt,
      'nextSection': value.nextSection,
      'mintedSections': value.mintedSections,
      'flipFeeLamports': value.flipFeeLamports,
      'totalFlips': value.totalFlips,
      'sectionAllocationTokens': value.sectionAllocationTokens,
      'emissionDurationSeconds': value.emissionDurationSeconds,
      'windowSeconds': value.windowSeconds,
      'targetTokensPerWindow': value.targetTokensPerWindow,
      'startPriceLamports': value.startPriceLamports,
      'minimumPriceLamports': value.minimumPriceLamports,
      'maximumPriceLamports': value.maximumPriceLamports,
      'startFloorPriceLamports': value.startFloorPriceLamports,
      'endFloorPriceLamports': value.endFloorPriceLamports,
      'changeDenominator': value.changeDenominator,
      'burstElasticity': value.burstElasticity,
      'ownerShareBasisPoints': value.ownerShareBasisPoints,
    },
  );
}

Decoder<GameState> getGameStateDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('status', getU8Decoder()),
    ('bump', getU8Decoder()),
    ('economyVersion', getU8Decoder()),
    ('startsAt', getI64Decoder()),
    ('nextSection', getU16Decoder()),
    ('mintedSections', getU16Decoder()),
    ('flipFeeLamports', getU64Decoder()),
    ('totalFlips', getU64Decoder()),
    ('sectionAllocationTokens', getU64Decoder()),
    ('emissionDurationSeconds', getU64Decoder()),
    ('windowSeconds', getU64Decoder()),
    ('targetTokensPerWindow', getU64Decoder()),
    ('startPriceLamports', getU64Decoder()),
    ('minimumPriceLamports', getU64Decoder()),
    ('maximumPriceLamports', getU64Decoder()),
    ('startFloorPriceLamports', getU64Decoder()),
    ('endFloorPriceLamports', getU64Decoder()),
    ('changeDenominator', getU64Decoder()),
    ('burstElasticity', getU64Decoder()),
    ('ownerShareBasisPoints', getU16Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(
      SolanaErrorCode.codecsInvalidByteLength,
      {
        'codecDescription': 'gameState account decoder',
        'expected': expected,
        'bytesLength': bytesLength,
      },
    );
  }

  (GameState, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(
      getU8Encoder().encode(2),
    ).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);

    return (
      GameState(
      gameIndex: map['gameIndex']! as int,
      status: map['status']! as int,
      bump: map['bump']! as int,
      economyVersion: map['economyVersion']! as int,
      startsAt: map['startsAt']! as BigInt,
      nextSection: map['nextSection']! as int,
      mintedSections: map['mintedSections']! as int,
      flipFeeLamports: map['flipFeeLamports']! as BigInt,
      totalFlips: map['totalFlips']! as BigInt,
      sectionAllocationTokens: map['sectionAllocationTokens']! as BigInt,
      emissionDurationSeconds: map['emissionDurationSeconds']! as BigInt,
      windowSeconds: map['windowSeconds']! as BigInt,
      targetTokensPerWindow: map['targetTokensPerWindow']! as BigInt,
      startPriceLamports: map['startPriceLamports']! as BigInt,
      minimumPriceLamports: map['minimumPriceLamports']! as BigInt,
      maximumPriceLamports: map['maximumPriceLamports']! as BigInt,
      startFloorPriceLamports: map['startFloorPriceLamports']! as BigInt,
      endFloorPriceLamports: map['endFloorPriceLamports']! as BigInt,
      changeDenominator: map['changeDenominator']! as BigInt,
      burstElasticity: map['burstElasticity']! as BigInt,
      ownerShareBasisPoints: map['ownerShareBasisPoints']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<GameState>(
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
      VariableSizeDecoder<GameState>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<GameState, GameState> getGameStateCodec() {
  return combineCodec(getGameStateEncoder(), getGameStateDecoder());
}

Account<GameState> decodeGameState(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getGameStateDecoder());
}
