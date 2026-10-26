import 'package:bitflip_program/bitflip_program.dart';
import 'package:solana_kit/solana_kit.dart';

/// Derives every Bitflip PDA once. The config, game, and section addresses
/// are constant for a program and session, so the polling refresh loop reuses
/// memoized addresses instead of re-running the bump search on each cycle.
final class BitflipAddressBook {
  BitflipAddressBook({Address? programAddress})
    : _programAddress = programAddress ?? bitflipProgramProgramAddress;

  final Address _programAddress;
  final Map<int, Address> _games = {};
  final Map<(int, int), (Address, int)> _sections = {};
  Address? _config;

  Future<Address> config() async {
    return _config ??= (await findConfigPda(programAddress: _programAddress))
        .$1;
  }

  Future<Address> game(int gameIndex) async {
    return _games[gameIndex] ??= (await findGamePda(
      programAddress: _programAddress,
      seeds: GameSeeds(gameIndex: gameIndex),
    )).$1;
  }

  /// The memoized section address and bump, as instruction builders need both.
  Future<(Address, int)> sectionAccount(int gameIndex, int sectionIndex) async {
    return _sections[(gameIndex, sectionIndex)] ??= await findSectionPda(
      programAddress: _programAddress,
      seeds: SectionSeeds(gameIndex: gameIndex, sectionIndex: sectionIndex),
    );
  }

  Future<Address> section(int gameIndex, int sectionIndex) async {
    return (await sectionAccount(gameIndex, sectionIndex)).$1;
  }
}
