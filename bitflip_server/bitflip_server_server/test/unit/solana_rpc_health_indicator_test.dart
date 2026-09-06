import 'dart:async';

import 'package:bitflip_server_server/src/web/solana_rpc_health_indicator.dart';
import 'package:serverpod/serverpod.dart';
import 'package:test/test.dart';

void main() {
  test('passes and reports the observed slot', () async {
    final indicator = SolanaRpcHealthIndicator(
      readSlot: () async => BigInt.from(42),
    );

    final result = await indicator.check();

    expect(result.status, HealthStatus.pass);
    expect(result.observedValue, 42);
    expect(result.observedUnit, 'slot');
  });

  test('fails without exposing dependency details', () async {
    final indicator = SolanaRpcHealthIndicator(
      readSlot: () async => throw StateError('secret upstream detail'),
    );

    final result = await indicator.check();

    expect(result.status, HealthStatus.fail);
    expect(result.output, contains('StateError'));
    expect(result.output, isNot(contains('secret upstream detail')));
  });

  test('fails when the RPC exceeds its deadline', () async {
    final completer = Completer<BigInt>();
    final indicator = SolanaRpcHealthIndicator(
      readSlot: () => completer.future,
      deadline: const Duration(milliseconds: 1),
    );

    final result = await indicator.check();

    expect(result.status, HealthStatus.fail);
    expect(result.output, contains('TimeoutException'));
  });
}
