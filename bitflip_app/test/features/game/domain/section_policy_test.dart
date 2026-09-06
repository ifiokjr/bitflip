import 'package:bitflip_app/features/game/domain/section_policy.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('a configured policy is live only inside its committed interval', () {
    final policy = SectionPolicySnapshot(
      version: BigInt.one,
      startsAtUnixSeconds: BigInt.from(100),
      endsAtUnixSeconds: BigInt.from(200),
      entryPriceTokens: BigInt.zero,
      rewardPerActionTokens: BigInt.zero,
      rulesDigest: List.filled(32, 0xab),
      mode: SectionPolicyMode.colourCanvas,
      paletteId: 0,
      rewardPolicy: SectionRewardPolicy.none,
    );

    expect(policy.isConfigured, isTrue);
    expect(policy.isLiveAt(BigInt.from(99)), isFalse);
    expect(policy.isLiveAt(BigInt.from(100)), isTrue);
    expect(policy.isLiveAt(BigInt.from(199)), isTrue);
    expect(policy.isLiveAt(BigInt.from(200)), isFalse);
    expect(policy.rulesDigestHex, List.filled(32, 'ab').join());
  });

  test('unknown policy identifiers fail closed', () {
    expect(() => SectionPolicyMode.fromCode(2), throwsStateError);
    expect(() => SectionRewardPolicy.fromCode(1), throwsStateError);
  });
}
