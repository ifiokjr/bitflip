import 'dart:async';

import 'package:bitflip_server_server/src/minting/mint_endpoint.dart';
import 'package:test/test.dart';

void main() {
  group('ChallengeRateLimiter', () {
    test('rejects a source before it can trigger expensive RPC work', () {
      var now = DateTime.utc(2026);
      final limiter = ChallengeRateLimiter(
        maximumPerSource: 2,
        maximumGlobal: 10,
        clock: () => now,
      );

      limiter.record('198.51.100.1');
      limiter.record('198.51.100.1');
      expect(
        () => limiter.record('198.51.100.1'),
        throwsA(isA<StateError>()),
      );

      now = now.add(const Duration(minutes: 1, seconds: 1));
      expect(() => limiter.record('198.51.100.1'), returnsNormally);
    });

    test('enforces a global ceiling across sources', () {
      final limiter = ChallengeRateLimiter(
        maximumPerSource: 10,
        maximumGlobal: 2,
      );

      limiter.record('198.51.100.1');
      limiter.record('198.51.100.2');
      expect(
        () => limiter.record('198.51.100.3'),
        throwsA(isA<StateError>()),
      );
    });
  });

  test('MintOperatorGate rejects excess work without queueing', () async {
    final gate = MintOperatorGate(maximumInFlight: 1);
    final release = Completer<void>();
    final first = gate.run(() async {
      await release.future;
      return 1;
    });
    await Future<void>.delayed(Duration.zero);

    await expectLater(
      gate.run(() async => 2),
      throwsA(isA<StateError>()),
    );
    expect(gate.inFlight, 1);

    release.complete();
    expect(await first, 1);
    expect(gate.inFlight, 0);
  });
}
