import 'dart:convert';

import 'package:crypto/crypto.dart';

enum SectionPolicyMode {
  openCanvas(0),
  colourCanvas(1);

  const SectionPolicyMode(this.code);

  factory SectionPolicyMode.fromCode(int code) => switch (code) {
    0 => SectionPolicyMode.openCanvas,
    1 => SectionPolicyMode.colourCanvas,
    _ => throw StateError('The section has an unknown game mode.'),
  };

  final int code;
}

enum SectionRewardPolicy {
  none(0);

  const SectionRewardPolicy(this.code);

  factory SectionRewardPolicy.fromCode(int code) => switch (code) {
    0 => SectionRewardPolicy.none,
    _ => throw StateError('The section has an unknown reward policy.'),
  };

  final int code;
}

enum SectionColour {
  acid(0),
  coral(1),
  cyan(2),
  paper(3),
  violet(4),
  amber(5),
  blue(6),
  pink(7);

  const SectionColour(this.code);

  factory SectionColour.fromCode(int code) => switch (code) {
    0 => SectionColour.acid,
    1 => SectionColour.coral,
    2 => SectionColour.cyan,
    3 => SectionColour.paper,
    4 => SectionColour.violet,
    5 => SectionColour.amber,
    6 => SectionColour.blue,
    7 => SectionColour.pink,
    _ => throw StateError('The pixel has an unknown palette colour.'),
  };

  final int code;
}

final class SectionPolicySnapshot {
  SectionPolicySnapshot({
    required this.version,
    required this.startsAtUnixSeconds,
    required this.endsAtUnixSeconds,
    required this.entryPriceTokens,
    required this.rewardPerActionTokens,
    required List<int> rulesDigest,
    required this.mode,
    required this.paletteId,
    required this.rewardPolicy,
  }) : rulesDigest = List.unmodifiable(rulesDigest);

  final BigInt version;
  final BigInt startsAtUnixSeconds;
  final BigInt endsAtUnixSeconds;
  final BigInt entryPriceTokens;
  final BigInt rewardPerActionTokens;
  final List<int> rulesDigest;
  final SectionPolicyMode mode;
  final int paletteId;
  final SectionRewardPolicy rewardPolicy;

  bool get isConfigured => version > BigInt.zero;

  bool isLiveAt(BigInt unixSeconds) =>
      isConfigured &&
      unixSeconds >= startsAtUnixSeconds &&
      unixSeconds < endsAtUnixSeconds;

  String get rulesDigestHex =>
      rulesDigest.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

final class SectionPolicyDraft {
  SectionPolicyDraft({
    required this.mode,
    required this.startsAtUnixSeconds,
    required this.endsAtUnixSeconds,
    required List<int> rulesDigest,
  }) : rulesDigest = List.unmodifiable(rulesDigest) {
    final duration = endsAtUnixSeconds - startsAtUnixSeconds;
    if (duration <= BigInt.zero || duration > BigInt.from(30 * 24 * 60 * 60)) {
      throw ArgumentError.value(duration, 'duration');
    }
    if (this.rulesDigest.length != 32) {
      throw ArgumentError.value(this.rulesDigest.length, 'rulesDigest.length');
    }
  }

  factory SectionPolicyDraft.startingNow({
    required SectionPolicyMode mode,
    required Duration duration,
    DateTime? now,
  }) {
    final clock = now ?? DateTime.now();
    final startsAt = BigInt.from(clock.millisecondsSinceEpoch ~/ 1000);
    final rulesId = switch (mode) {
      SectionPolicyMode.openCanvas => 'bitflip:section-rules:v1:open-canvas',
      SectionPolicyMode.colourCanvas =>
        'bitflip:section-rules:v1:eight-colour-canvas',
    };
    return SectionPolicyDraft(
      mode: mode,
      startsAtUnixSeconds: startsAt,
      endsAtUnixSeconds: startsAt + BigInt.from(duration.inSeconds),
      rulesDigest: sha256.convert(utf8.encode(rulesId)).bytes,
    );
  }

  final SectionPolicyMode mode;
  final BigInt startsAtUnixSeconds;
  final BigInt endsAtUnixSeconds;
  final List<int> rulesDigest;
}
