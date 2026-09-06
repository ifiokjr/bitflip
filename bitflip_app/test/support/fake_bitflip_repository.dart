import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/data/bitflip_repository.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';

final class FakeBitflipRepository implements BitflipRepository {
  FakeBitflipRepository({
    GameSnapshot? snapshot,
    this.isWalletSupported = true,
    this.walletKind = BitflipWalletKind.external,
    this.canFundWithMobileWallet = false,
    this.availableWallets,
    this.walletAddress = 'DemoWallet111111111111111111111111111111111',
    BigInt? walletBalanceLamports,
    this.returnNullOnLoad = false,
    this.initializeError,
    this.balanceError,
    this.loadError,
    this.connectError,
    this.fundError,
    this.claimError,
    this.listError,
    this.cancelListingError,
    this.purchaseError,
    this.withdrawOwnerFeesError,
    this.flipError,
    this.sealError,
    this.mintError,
  }) : walletBalanceLamports = walletBalanceLamports ?? BigInt.zero,
       snapshot = snapshot ?? GameSnapshot.demo(sectionIndex: 12);

  GameSnapshot snapshot;
  @override
  bool get isDemoMode => snapshot.isDemo;
  @override
  final bool isWalletSupported;
  @override
  final BitflipWalletKind walletKind;
  @override
  final bool canFundWithMobileWallet;
  @override
  String get walletChain => 'solana:devnet';
  @override
  final List<BitflipWalletOption>? availableWallets;
  @override
  String? walletAddress;
  BigInt? walletBalanceLamports;
  Object? balanceError;
  final bool returnNullOnLoad;
  final Object? initializeError;
  final Object? loadError;
  final Object? connectError;
  final Object? fundError;
  final Object? claimError;
  final Object? listError;
  final Object? cancelListingError;
  final Object? purchaseError;
  final Object? withdrawOwnerFeesError;
  final Object? flipError;
  final Object? sealError;
  final Object? mintError;

  int claimCalls = 0;
  int listCalls = 0;
  int cancelListingCalls = 0;
  int purchaseCalls = 0;
  int withdrawOwnerFeesCalls = 0;
  int connectCalls = 0;
  int fundCalls = 0;
  int initializeCalls = 0;
  int flipCalls = 0;
  int mintCalls = 0;
  int sealCalls = 0;
  int loadCalls = 0;
  List<PixelCoordinate> lastFlips = const [];
  String? lastWalletId;
  BigInt? lastFundingLamports;
  BigInt? lastListingPriceLamports;

  @override
  Future<void> initializeWallet() async {
    initializeCalls++;
    final error = initializeError;
    if (error != null) throw error;
    if (walletKind == BitflipWalletKind.embedded) {
      walletAddress ??= 'Embedded111111111111111111111111111111111111';
    }
  }

  @override
  Future<BigInt?> loadWalletBalance() async {
    final error = balanceError;
    if (error != null) throw error;
    return walletBalanceLamports;
  }

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
  Future<String> listSection(
    GameSnapshot snapshot,
    BigInt priceLamports,
  ) async {
    listCalls++;
    lastListingPriceLamports = priceLamports;
    final error = listError;
    if (error != null) throw error;
    this.snapshot = this.snapshot.copyWith(
      section: this.snapshot.section.copyWith(salePriceLamports: priceLamports),
    );
    return 'list-signature';
  }

  @override
  Future<String> cancelSectionListing(GameSnapshot snapshot) async {
    cancelListingCalls++;
    final error = cancelListingError;
    if (error != null) throw error;
    this.snapshot = this.snapshot.copyWith(
      section: this.snapshot.section.copyWith(salePriceLamports: BigInt.zero),
    );
    return 'cancel-listing-signature';
  }

  @override
  Future<String> purchaseSection(GameSnapshot snapshot) async {
    purchaseCalls++;
    final error = purchaseError;
    if (error != null) throw error;
    this.snapshot = this.snapshot.copyWith(
      section: this.snapshot.section.copyWith(
        owner: walletAddress,
        salePriceLamports: BigInt.zero,
      ),
    );
    return 'purchase-signature';
  }

  @override
  Future<String> withdrawSectionOwnerFees(GameSnapshot snapshot) async {
    withdrawOwnerFeesCalls++;
    final error = withdrawOwnerFeesError;
    if (error != null) throw error;
    return 'withdraw-owner-fees-signature';
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
  Future<String> fundWithMobileWallet(BigInt lamports) async {
    fundCalls++;
    lastFundingLamports = lamports;
    final error = fundError;
    if (error != null) throw error;
    walletBalanceLamports = (walletBalanceLamports ?? BigInt.zero) + lamports;
    return 'fund-signature';
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
