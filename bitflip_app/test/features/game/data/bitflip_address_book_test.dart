import 'package:bitflip_app/features/game/data/bitflip_address_book.dart';
import 'package:bitflip_program/bitflip_program.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('BitflipAddressBook', () {
    test('memoizes every derived address', () async {
      final book = BitflipAddressBook();
      final directSection = await findSectionPda(
        programAddress: bitflipProgramProgramAddress,
        seeds: const SectionSeeds(gameIndex: 0, sectionIndex: 5),
      );
      final directGame = await findGamePda(
        programAddress: bitflipProgramProgramAddress,
        seeds: const GameSeeds(gameIndex: 0),
      );

      final firstSection = await book.sectionAccount(0, 5);
      final secondSection = await book.sectionAccount(0, 5);
      final otherSection = await book.sectionAccount(0, 6);

      expect(firstSection, directSection);
      expect(identical(firstSection, secondSection), isTrue);
      expect(await book.section(0, 5), firstSection.$1);
      expect(otherSection, isNot(equals(firstSection)));
      expect(identical(await book.game(0), await book.game(0)), isTrue);
      expect(await book.game(0), directGame.$1);
      expect(identical(await book.config(), await book.config()), isTrue);
      expect(
        await book.config(),
        (await findConfigPda(programAddress: bitflipProgramProgramAddress)).$1,
      );
    });
  });
}
