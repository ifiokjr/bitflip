// Auto-generated. Do not edit.
// ignore_for_file: type=lint



import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';


@immutable
class SectionSeeds {
  const SectionSeeds({
    required this.gameIndex,
    required this.sectionIndex,
  });

  final int gameIndex;
  final int sectionIndex;
}

/// Finds the program derived address for [Section].
Future<(Address, int)> findSectionPda({
  required SectionSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'section',
    getU8Encoder().encode(seeds.gameIndex),
    getU8Encoder().encode(seeds.sectionIndex),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
