// Auto-generated. Do not edit.
// ignore_for_file: type=lint



import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';


@immutable
class GameSeeds {
  const GameSeeds({
    required this.gameIndex,
  });

  final int gameIndex;
}

/// Finds the program derived address for [Game].
Future<(Address, int)> findGamePda({
  required GameSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'game',
    getU8Encoder().encode(seeds.gameIndex),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
