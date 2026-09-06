import 'package:bitflip_app/features/wallet/widgets/wallet_sheet.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('parseSolToLamports', () {
    test('parses whole and fractional SOL without floating-point rounding', () {
      expect(parseSolToLamports('1'), BigInt.from(1000000000));
      expect(parseSolToLamports('0.125'), BigInt.from(125000000));
      expect(parseSolToLamports('0.000000001'), BigInt.one);
    });

    test('rejects zero, negative, and over-precise amounts', () {
      expect(parseSolToLamports('0'), isNull);
      expect(parseSolToLamports('-1'), isNull);
      expect(parseSolToLamports('0.0000000001'), isNull);
      expect(parseSolToLamports('not SOL'), isNull);
    });
  });
}
