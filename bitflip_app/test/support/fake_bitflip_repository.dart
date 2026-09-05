import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/data/bitflip_repository.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';

final class FakeBitflipRepository implements BitflipRepository {
  FakeBitflipRepository({
    GameSnapshot? snapshot,
    this.isWalletSupported = true,
    this.availableWallets,
    this.walletAddress = 'DemoWallet111111111111111111111111111111111',
  }) : snapshot = snapshot ?? GameSnapshot.demo(sectionIndex: 12);

  GameSnapshot snapshot;
  @override
  final bool isWalletSupported;
  @override
  final List<BitflipWalletOption>? availableWallets;
  @override
  String? walletAddress;

  int claimCalls = 0;
  int flipCalls = 0;
  int mintCalls = 0;
  int sealCalls = 0;
  List<PixelCoordinate> lastFlips = const [];
  String? lastWalletId;

  @override
  Future<String> claimSection(GameSnapshot snapshot) async {
    claimCalls++;
    return 'claim-signature';
  }

  @override
  Future<String> connectWallet([String? walletId]) async {
    lastWalletId = walletId;
    walletAddress ??= 'DemoWallet111111111111111111111111111111111';
    return walletAddress!;
  }

  @override
  Future<String> flipPixels(
    GameSnapshot snapshot,
    List<PixelCoordinate> coordinates,
  ) async {
    flipCalls++;
    lastFlips = List.unmodifiable(coordinates);
    return 'flip-signature';
  }

  @override
  Future<GameSnapshot?> loadSection(int sectionIndex) async => snapshot;

  @override
  Future<String> mintSection(GameSnapshot snapshot) async {
    mintCalls++;
    return 'Asset11111111111111111111111111111111111111';
  }

  @override
  Future<String> sealSection(GameSnapshot snapshot) async {
    sealCalls++;
    return 'seal-signature';
  }
}
