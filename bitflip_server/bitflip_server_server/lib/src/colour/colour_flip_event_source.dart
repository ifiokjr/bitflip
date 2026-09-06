import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event.dart';
import 'package:bs58/bs58.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_rpc_api/solana_kit_rpc_api.dart' as rpc_api;
import 'package:solana_kit_rpc_spec/solana_kit_rpc_spec.dart';
import 'package:solana_kit_rpc_types/solana_kit_rpc_types.dart' as rpc_types;

abstract interface class ColourFlipEventSource {
  Future<List<ColourPixelsFlipped>> eventsForSignature(String signature);
}

final class SolanaColourFlipEventSource implements ColourFlipEventSource {
  const SolanaColourFlipEventSource(this.rpc);

  final Rpc rpc;

  @override
  Future<List<ColourPixelsFlipped>> eventsForSignature(String signature) async {
    final normalized = _validatedTransactionSignature(signature);
    final transaction = await rpc
        .getTransaction(
          Signature(normalized),
          const rpc_api.GetTransactionConfig(
            commitment: rpc_types.Commitment.confirmed,
            maxSupportedTransactionVersion: 0,
          ),
        )
        .send();
    if (transaction == null) {
      throw StateError('The confirmed colour transaction is not available.');
    }
    final meta = transaction['meta'];
    if (meta is! Map || meta['err'] != null) {
      throw StateError('The colour transaction did not succeed.');
    }
    final rawLogs = meta['logMessages'];
    if (rawLogs is! List) {
      throw StateError('The colour transaction has no program logs.');
    }
    return colourEventsFromProgramLogs(
      rawLogs.whereType<String>(),
      programAddress: bitflipProgramProgramAddress.value,
    );
  }
}

String _validatedTransactionSignature(String value) {
  final normalized = value.trim();
  if (normalized.length < 80 || normalized.length > 90) {
    throw const FormatException('Invalid Solana transaction signature.');
  }
  try {
    if (base58.decoder.convert(normalized).length != 64) {
      throw const FormatException('Invalid Solana transaction signature.');
    }
  } on FormatException {
    rethrow;
  } on Object {
    throw const FormatException('Invalid Solana transaction signature.');
  }
  return normalized;
}

abstract final class ColourFlipEventSourceRegistry {
  static ColourFlipEventSource? _source;

  static ColourFlipEventSource get source =>
      _source ??
      (throw StateError('The Bitflip colour event source is unavailable.'));

  static void configure(ColourFlipEventSource source) {
    _source = source;
  }
}
