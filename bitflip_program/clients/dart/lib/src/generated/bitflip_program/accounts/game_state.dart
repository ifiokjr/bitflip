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
    required this.startsAt,
    required this.nextSection,
    required this.mintedSections,
    required this.flipFeeLamports,
    required this.totalFlips,
  }) :
      discriminator = 2;

  final int discriminator;
  final int gameIndex;
  final int status;
  final int bump;
  final BigInt startsAt;
  final int nextSection;
  final int mintedSections;
  final BigInt flipFeeLamports;
  final BigInt totalFlips;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is GameState &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          gameIndex == other.gameIndex &&
          status == other.status &&
          bump == other.bump &&
          startsAt == other.startsAt &&
          nextSection == other.nextSection &&
          mintedSections == other.mintedSections &&
          flipFeeLamports == other.flipFeeLamports &&
          totalFlips == other.totalFlips;

  @override
  int get hashCode => Object.hash(discriminator, gameIndex, status, bump, startsAt, nextSection, mintedSections, flipFeeLamports, totalFlips);

  @override
  String toString() => 'GameState(discriminator: $discriminator, gameIndex: $gameIndex, status: $status, bump: $bump, startsAt: $startsAt, nextSection: $nextSection, mintedSections: $mintedSections, flipFeeLamports: $flipFeeLamports, totalFlips: $totalFlips)';
}


Encoder<GameState> getGameStateEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('gameIndex', getU8Encoder()),
    ('status', getU8Encoder()),
    ('bump', getU8Encoder()),
    ('startsAt', getI64Encoder()),
    ('nextSection', getU16Encoder()),
    ('mintedSections', getU16Encoder()),
    ('flipFeeLamports', getU64Encoder()),
    ('totalFlips', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (GameState value) => <String, Object?>{
      'discriminator': 2,
      'gameIndex': value.gameIndex,
      'status': value.status,
      'bump': value.bump,
      'startsAt': value.startsAt,
      'nextSection': value.nextSection,
      'mintedSections': value.mintedSections,
      'flipFeeLamports': value.flipFeeLamports,
      'totalFlips': value.totalFlips,
    },
  );
}

Decoder<GameState> getGameStateDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('gameIndex', getU8Decoder()),
    ('status', getU8Decoder()),
    ('bump', getU8Decoder()),
    ('startsAt', getI64Decoder()),
    ('nextSection', getU16Decoder()),
    ('mintedSections', getU16Decoder()),
    ('flipFeeLamports', getU64Decoder()),
    ('totalFlips', getU64Decoder()),
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
      startsAt: map['startsAt']! as BigInt,
      nextSection: map['nextSection']! as int,
      mintedSections: map['mintedSections']! as int,
      flipFeeLamports: map['flipFeeLamports']! as BigInt,
      totalFlips: map['totalFlips']! as BigInt,
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
