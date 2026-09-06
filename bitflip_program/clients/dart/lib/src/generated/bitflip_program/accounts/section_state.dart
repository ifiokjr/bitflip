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
    required this.pixels,
  }) :
      discriminator = 3;

  final int discriminator;
  final Address owner;
  final Address assetId;
  final Address merkleTree;
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
          pixels == other.pixels;

  @override
  int get hashCode => Object.hash(discriminator, owner, assetId, merkleTree, gameIndex, sectionIndex, status, bump, onPixels, leafIndex, flipCount, revision, lastFlipAt, salePriceLamports, pixels);

  @override
  String toString() => 'SectionState(discriminator: $discriminator, owner: $owner, assetId: $assetId, merkleTree: $merkleTree, gameIndex: $gameIndex, sectionIndex: $sectionIndex, status: $status, bump: $bump, onPixels: $onPixels, leafIndex: $leafIndex, flipCount: $flipCount, revision: $revision, lastFlipAt: $lastFlipAt, salePriceLamports: $salePriceLamports, pixels: $pixels)';
}


Encoder<SectionState> getSectionStateEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('owner', getAddressEncoder()),
    ('assetId', getAddressEncoder()),
    ('merkleTree', getAddressEncoder()),
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
    ('pixels', fixEncoderSize(getBytesEncoder(), 512, allowTruncation: false)),
  ]);

  return transformEncoder(
    structEncoder,
    (SectionState value) => <String, Object?>{
      'discriminator': 3,
      'owner': value.owner,
      'assetId': value.assetId,
      'merkleTree': value.merkleTree,
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
