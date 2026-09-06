import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event_source.dart';
import 'package:bs58/bs58.dart';
import 'package:solana_kit_rpc_spec/solana_kit_rpc_spec.dart';
import 'package:test/test.dart';

void main() {
  group('SolanaColourFlipEventSource signature history', () {
    test(
      'requests confirmed program signatures with durable cursors',
      () async {
        final before = _signature(1);
        final until = _signature(2);
        final newest = _signature(3);
        final failed = _signature(4);
        List<Object?>? capturedParameters;
        final rpc = Rpc(
          api: MapRpcApi({
            'getSignaturesForAddress': (parameters) {
              capturedParameters = parameters;
              return RpcPlan(
                execute: (_) async => <Object?>[
                  {'signature': newest, 'err': null},
                  {
                    'signature': failed,
                    'err': {
                      'InstructionError': [0, 'Custom'],
                    },
                  },
                ],
              );
            },
          }),
          transport: (_) async => null,
        );

        final page = await SolanaColourFlipEventSource(rpc).signatures(
          limit: 2,
          before: before,
          until: until,
        );

        expect(page.newestSignature, newest);
        expect(page.oldestSignature, failed);
        expect(page.entries.first.succeeded, isTrue);
        expect(page.entries.last.succeeded, isFalse);
        expect(capturedParameters, [
          bitflipProgramProgramAddress.value,
          {
            'before': before,
            'commitment': 'confirmed',
            'limit': 2,
            'until': until,
          },
        ]);
      },
    );

    test('rejects malformed provider signature entries', () async {
      final rpc = Rpc(
        api: MapRpcApi({
          'getSignaturesForAddress': (_) => RpcPlan(
            execute: (_) async => <Object?>[
              {'signature': 'not-a-signature', 'err': null},
            ],
          ),
        }),
        transport: (_) async => null,
      );

      await expectLater(
        SolanaColourFlipEventSource(rpc).signatures(limit: 1),
        throwsFormatException,
      );
    });
  });
}

String _signature(int seed) => base58.encoder.convert(
  Uint8List.fromList(List.generate(64, (index) => (index + seed) % 256)),
);
