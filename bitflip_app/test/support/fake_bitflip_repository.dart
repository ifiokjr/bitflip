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
    this.returnNullOnLoad = false,
    this.loadError,
  }) : snapshot = snapshot ?? GameSnapshot.demo(sectionIndex: 12);

  GameSnapshot snapshot;
  @override
  bool get isDemoMode => snapshot.isDemo;
  @override
  final bool isWalletSupported;
  @override
  String get walletChain => 'solana:devnet';
  @override
  final List<BitflipWalletOption>? availableWallets;
  @override
  String? walletAddress;
  final bool returnNullOnLoad;
  final Object? loadError;

  int claimCalls = 0;
  int flipCalls = 0;
  int mintCalls = 0;
  int sealCalls = 0;
  int loadCalls = 0;
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
  Future<GameSnapshot?> loadSection(int sectionIndex) async {
    loadCalls++;
    final error = loadError;
    if (error != null) throw error;
    return returnNullOnLoad ? null : snapshot;
  }

  @override
  Future<BitflipMintResult> mintSection(GameSnapshot snapshot) async {
    mintCalls++;
    return const BitflipMintResult(
      assetId: 'Asset11111111111111111111111111111111111111',
      transactionSignature: 'mint-signature',
      alreadyMinted: false,
    );
  }

  @override
  Future<String> sealSection(GameSnapshot snapshot) async {
    sealCalls++;
    return 'seal-signature';
  }
}
