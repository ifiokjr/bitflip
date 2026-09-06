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
    this.connectError,
    this.claimError,
    this.flipError,
    this.sealError,
    this.mintError,
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
  final Object? connectError;
  final Object? claimError;
  final Object? flipError;
  final Object? sealError;
  final Object? mintError;

  int claimCalls = 0;
  int connectCalls = 0;
  int flipCalls = 0;
  int mintCalls = 0;
  int sealCalls = 0;
  int loadCalls = 0;
  List<PixelCoordinate> lastFlips = const [];
  String? lastWalletId;

  @override
  Future<String> claimSection(GameSnapshot snapshot) async {
    claimCalls++;
    final error = claimError;
    if (error != null) throw error;
    final owner = walletAddress;
    this.snapshot = this.snapshot.copyWith(
      nextSection: this.snapshot.nextSection + 1,
      section: this.snapshot.section.copyWith(
        lifecycle: SectionLifecycle.active,
        owner: owner,
      ),
    );
    return 'claim-signature';
  }

  @override
  Future<String> connectWallet([String? walletId]) async {
    connectCalls++;
    lastWalletId = walletId;
    final error = connectError;
    if (error != null) throw error;
    walletAddress ??= 'DemoWallet111111111111111111111111111111111';
    return walletAddress!;
  }

  @override
  Future<String> flipPixels(
    GameSnapshot snapshot,
    List<PixelCoordinate> coordinates,
  ) async {
    flipCalls++;
    final error = flipError;
    if (error != null) throw error;
    lastFlips = List.unmodifiable(coordinates);
    this.snapshot = this.snapshot.copyWith(
      totalFlips: this.snapshot.totalFlips + BigInt.from(coordinates.length),
      section: this.snapshot.section.copyWith(
        bitmap: this.snapshot.section.bitmap.toggled(coordinates),
        flipCount:
            this.snapshot.section.flipCount + BigInt.from(coordinates.length),
        revision: this.snapshot.section.revision + BigInt.one,
      ),
    );
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
    final error = mintError;
    if (error != null) throw error;
    this.snapshot = this.snapshot.copyWith(
      mintedSections: this.snapshot.mintedSections + 1,
      section: this.snapshot.section.copyWith(
        lifecycle: SectionLifecycle.minted,
        assetId: 'Asset11111111111111111111111111111111111111',
      ),
    );
    return const BitflipMintResult(
      assetId: 'Asset11111111111111111111111111111111111111',
      transactionSignature: 'mint-signature',
      alreadyMinted: false,
    );
  }

  @override
  Future<String> sealSection(GameSnapshot snapshot) async {
    sealCalls++;
    final error = sealError;
    if (error != null) throw error;
    this.snapshot = this.snapshot.copyWith(
      section: this.snapshot.section.copyWith(
        lifecycle: SectionLifecycle.sealed,
      ),
    );
    return 'seal-signature';
  }
}
