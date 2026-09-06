import 'dart:async';

import 'package:bitflip_app/core/bitflip_config.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/data/bitflip_repository.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'game_controller.g.dart';

enum GameNotice {
  ready,
  queued,
  committed,
  sectionChanged,
  connected,
  funded,
  claimed,
  listed,
  listingCancelled,
  purchased,
  sealed,
  minted,
  batchFull,
  walletIssue,
  connectionIssue,
}

enum GameLoadStatus { loading, ready, demo, unavailable, error }

final class GameActivity {
  const GameActivity(
    this.notice, {
    this.coordinate,
    this.sectionIndex,
    this.transactionSignature,
    this.assetId,
  });

  final GameNotice notice;
  final PixelCoordinate? coordinate;
  final int? sectionIndex;
  final String? transactionSignature;
  final String? assetId;
}

final class GameViewState {
  const GameViewState({
    required this.snapshot,
    required this.queued,
    required this.activity,
    required this.isBusy,
    required this.isWalletSupported,
    required this.walletKind,
    required this.canFundWithMobileWallet,
    required this.walletAddress,
    required this.walletBalanceLamports,
    required this.loadStatus,
    required this.walletChain,
    this.cursor,
  });

  factory GameViewState.initial({
    required bool isWalletSupported,
    required BitflipWalletKind walletKind,
    required bool canFundWithMobileWallet,
    required bool isDemoMode,
    required String walletChain,
  }) {
    return GameViewState(
      snapshot: isDemoMode
          ? GameSnapshot.demo(sectionIndex: 12)
          : GameSnapshot.empty(gameIndex: 0),
      queued: const {},
      cursor: null,
      activity: const GameActivity(GameNotice.ready),
      isBusy: false,
      isWalletSupported: isWalletSupported,
      walletKind: walletKind,
      canFundWithMobileWallet: canFundWithMobileWallet,
      walletAddress: null,
      walletBalanceLamports: null,
      loadStatus: isDemoMode ? GameLoadStatus.demo : GameLoadStatus.loading,
      walletChain: walletChain,
    );
  }

  final GameSnapshot snapshot;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;
  final GameActivity activity;
  final bool isBusy;
  final bool isWalletSupported;
  final BitflipWalletKind walletKind;
  final bool canFundWithMobileWallet;
  final String? walletAddress;
  final BigInt? walletBalanceLamports;
  final GameLoadStatus loadStatus;
  final String walletChain;

  PixelBitmap get previewBitmap => snapshot.section.bitmap.toggled(queued);

  BigInt get queuedFee => snapshot.flipFeeLamports * BigInt.from(queued.length);

  bool get canTransact =>
      (loadStatus == GameLoadStatus.ready ||
          loadStatus == GameLoadStatus.demo) &&
      (snapshot.isDemo || (isWalletSupported && walletAddress != null));

  GameViewState copyWith({
    GameSnapshot? snapshot,
    Set<PixelCoordinate>? queued,
    PixelCoordinate? cursor,
    bool clearCursor = false,
    GameActivity? activity,
    bool? isBusy,
    String? walletAddress,
    BigInt? walletBalanceLamports,
    bool clearWalletBalance = false,
    GameLoadStatus? loadStatus,
    String? walletChain,
  }) {
    return GameViewState(
      snapshot: snapshot ?? this.snapshot,
      queued: Set.unmodifiable(queued ?? this.queued),
      cursor: clearCursor ? null : cursor ?? this.cursor,
      activity: activity ?? this.activity,
      isBusy: isBusy ?? this.isBusy,
      isWalletSupported: isWalletSupported,
      walletKind: walletKind,
      canFundWithMobileWallet: canFundWithMobileWallet,
      walletAddress: walletAddress ?? this.walletAddress,
      walletBalanceLamports: clearWalletBalance
          ? null
          : walletBalanceLamports ?? this.walletBalanceLamports,
      loadStatus: loadStatus ?? this.loadStatus,
      walletChain: walletChain ?? this.walletChain,
    );
  }
}

@Riverpod(keepAlive: true)
BitflipRepository bitflipRepository(Ref ref) =>
    SolanaBitflipRepository(config: ref.watch(bitflipConfigProvider));

@Riverpod(keepAlive: true)
class GameController extends _$GameController {
  BitflipRepository get _repository => ref.read(bitflipRepositoryProvider);

  List<BitflipWalletOption>? get availableWallets =>
      _repository.availableWallets;

  @override
  GameViewState build() {
    return GameViewState.initial(
      isWalletSupported: _repository.isWalletSupported,
      walletKind: _repository.walletKind,
      canFundWithMobileWallet: _repository.canFundWithMobileWallet,
      isDemoMode: _repository.isDemoMode,
      walletChain: _repository.walletChain,
    );
  }

  Future<void> refresh() async {
    if (state.isBusy) return;
    if (_repository.isDemoMode) return;
    state = state.copyWith(isBusy: true);
    final sectionIndex = state.snapshot.section.index;
    var walletInitializationFailed = false;
    try {
      try {
        await _repository.initializeWallet();
      } on Object {
        walletInitializationFailed = true;
      }
      if (!ref.mounted) return;
      final loaded = await _repository.loadSection(sectionIndex);
      if (!ref.mounted) return;
      BigInt? walletBalance;
      try {
        walletBalance = await _repository.loadWalletBalance();
      } on Object {
        walletInitializationFailed = true;
      }
      if (!ref.mounted) return;
      final snapshot = loaded == null
          ? GameSnapshot.empty(
              gameIndex: state.snapshot.gameIndex,
              sectionIndex: state.snapshot.section.index,
            )
          : _reconcileSnapshot(state.snapshot, loaded);
      final retainQueue =
          loaded != null &&
          snapshot.section.lifecycle == SectionLifecycle.active;
      state = state.copyWith(
        snapshot: snapshot,
        queued: retainQueue ? state.queued : const {},
        clearCursor: !retainQueue,
        isBusy: false,
        walletAddress: _repository.walletAddress,
        walletBalanceLamports: walletBalance,
        clearWalletBalance: walletBalance == null,
        activity: walletInitializationFailed
            ? const GameActivity(GameNotice.walletIssue)
            : state.activity,
        loadStatus: loaded == null
            ? GameLoadStatus.unavailable
            : GameLoadStatus.ready,
      );
    } on Object {
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
        loadStatus: GameLoadStatus.error,
      );
    }
  }

  GameSnapshot _reconcileSnapshot(GameSnapshot current, GameSnapshot loaded) {
    if (current.isDemo ||
        current.section.index != loaded.section.index ||
        loaded.section.revision >= current.section.revision) {
      return loaded;
    }
    return current;
  }

  Future<void> connectWallet([String? walletId]) async {
    if (state.isBusy || !state.isWalletSupported) return;
    state = state.copyWith(isBusy: true);
    try {
      final address = await _repository.connectWallet(walletId);
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        walletAddress: address,
        activity: const GameActivity(GameNotice.connected),
      );
    } on Object {
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> fundWithMobileWallet(BigInt lamports) async {
    if (state.isBusy ||
        state.walletKind != BitflipWalletKind.embedded ||
        !state.canFundWithMobileWallet ||
        lamports <= BigInt.zero) {
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final transactionSignature = await _repository.fundWithMobileWallet(
        lamports,
      );
      if (!ref.mounted) return;
      var walletBalance = state.walletBalanceLamports;
      try {
        walletBalance = await _repository.loadWalletBalance();
      } on Object {
        // The transfer is already confirmed; a balance refresh failure must
        // not be reported as though the funding transaction failed.
      }
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        walletBalanceLamports: walletBalance,
        activity: GameActivity(
          GameNotice.funded,
          transactionSignature: transactionSignature,
        ),
      );
    } on Object {
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.walletIssue),
      );
    }
  }

  void togglePixel(PixelCoordinate coordinate) {
    if (!state.snapshot.section.isEditable ||
        state.isBusy ||
        (!state.snapshot.isDemo && !state.isWalletSupported)) {
      return;
    }
    final queued = {...state.queued};
    if (!queued.remove(coordinate)) {
      if (queued.length == maxFlipBatch) {
        state = state.copyWith(
          cursor: coordinate,
          activity: const GameActivity(GameNotice.batchFull),
        );
        return;
      }
      queued.add(coordinate);
    }
    state = state.copyWith(
      queued: queued,
      cursor: coordinate,
      activity: GameActivity(GameNotice.queued, coordinate: coordinate),
    );
  }

  void selectPixel(PixelCoordinate coordinate) {
    if (coordinate.x < 0 ||
        coordinate.x >= sectionSide ||
        coordinate.y < 0 ||
        coordinate.y >= sectionSide) {
      return;
    }
    state = state.copyWith(cursor: coordinate);
  }

  void clearQueue() {
    state = state.copyWith(
      queued: const {},
      clearCursor: true,
      activity: const GameActivity(GameNotice.ready),
    );
  }

  Future<void> commitMoves() async {
    if (state.queued.isEmpty || state.isBusy || !state.canTransact) return;
    final coordinates = state.queued.toList()..sort();
    if (state.snapshot.isDemo) {
      _commitLocally(coordinates);
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final transactionSignature = await _repository.flipPixels(
        state.snapshot,
        coordinates,
      );
      _commitLocally(coordinates, transactionSignature: transactionSignature);
      unawaited(refresh());
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> claimSection() async {
    final now = BigInt.from(DateTime.now().millisecondsSinceEpoch ~/ 1000);
    if (state.isBusy ||
        state.snapshot.section.isClaimed ||
        !state.snapshot.canClaimSectionAt(now) ||
        !state.canTransact) {
      return;
    }
    if (state.snapshot.isDemo) {
      final section = state.snapshot.section.copyWith(
        lifecycle: SectionLifecycle.active,
        owner: 'YOU…DEMO',
      );
      state = state.copyWith(
        snapshot: state.snapshot.copyWith(section: section),
      );
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final transactionSignature = await _repository.claimSection(
        state.snapshot,
      );
      state = state.copyWith(
        isBusy: false,
        activity: GameActivity(
          GameNotice.claimed,
          transactionSignature: transactionSignature,
        ),
      );
      await refresh();
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> listSection(BigInt priceLamports) async {
    final section = state.snapshot.section;
    if (state.isBusy ||
        !state.canTransact ||
        priceLamports <= BigInt.zero ||
        section.isProtocolOwned ||
        section.lifecycle == SectionLifecycle.minted ||
        state.walletAddress != section.owner) {
      return;
    }
    if (state.snapshot.isDemo) {
      state = state.copyWith(
        snapshot: state.snapshot.copyWith(
          section: section.copyWith(salePriceLamports: priceLamports),
        ),
        activity: const GameActivity(GameNotice.listed),
      );
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final transactionSignature = await _repository.listSection(
        state.snapshot,
        priceLamports,
      );
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: GameActivity(
          GameNotice.listed,
          transactionSignature: transactionSignature,
        ),
      );
      await refresh();
    } on Object {
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> cancelSectionListing() async {
    final section = state.snapshot.section;
    if (state.isBusy ||
        !state.canTransact ||
        !section.isListed ||
        state.walletAddress != section.owner) {
      return;
    }
    if (state.snapshot.isDemo) {
      state = state.copyWith(
        snapshot: state.snapshot.copyWith(
          section: section.copyWith(salePriceLamports: BigInt.zero),
        ),
        activity: const GameActivity(GameNotice.listingCancelled),
      );
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final transactionSignature = await _repository.cancelSectionListing(
        state.snapshot,
      );
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: GameActivity(
          GameNotice.listingCancelled,
          transactionSignature: transactionSignature,
        ),
      );
      await refresh();
    } on Object {
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> purchaseSection() async {
    final section = state.snapshot.section;
    if (state.isBusy ||
        !state.canTransact ||
        !section.isListed ||
        state.walletAddress == section.owner) {
      return;
    }
    if (state.snapshot.isDemo) {
      state = state.copyWith(
        snapshot: state.snapshot.copyWith(
          section: section.copyWith(
            owner: state.walletAddress ?? 'YOU…DEMO',
            salePriceLamports: BigInt.zero,
          ),
        ),
        activity: const GameActivity(GameNotice.purchased),
      );
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final transactionSignature = await _repository.purchaseSection(
        state.snapshot,
      );
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: GameActivity(
          GameNotice.purchased,
          transactionSignature: transactionSignature,
        ),
      );
      await refresh();
    } on Object {
      if (!ref.mounted) return;
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> sealSection() async {
    if (state.isBusy ||
        !state.snapshot.section.isEditable ||
        !state.canTransact) {
      return;
    }
    if (!state.snapshot.isDemo) {
      state = state.copyWith(isBusy: true);
      try {
        final transactionSignature = await _repository.sealSection(
          state.snapshot,
        );
        state = state.copyWith(
          activity: GameActivity(
            GameNotice.sealed,
            transactionSignature: transactionSignature,
          ),
        );
      } on Object {
        state = state.copyWith(
          isBusy: false,
          activity: const GameActivity(GameNotice.connectionIssue),
        );
        return;
      }
    }
    final section = state.snapshot.section.copyWith(
      lifecycle: SectionLifecycle.sealed,
    );
    state = state.copyWith(
      snapshot: state.snapshot.copyWith(section: section),
      queued: const {},
      isBusy: false,
      activity: state.snapshot.isDemo
          ? const GameActivity(GameNotice.sealed)
          : state.activity,
    );
    if (!state.snapshot.isDemo) unawaited(refresh());
  }

  Future<void> mintSection() async {
    if (state.isBusy ||
        state.snapshot.section.lifecycle != SectionLifecycle.sealed ||
        !state.canTransact) {
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final result = state.snapshot.isDemo
          ? BitflipMintResult(
              assetId: 'cnft:demo:${state.snapshot.section.index}',
              alreadyMinted: false,
            )
          : await _repository.mintSection(state.snapshot);
      final section = state.snapshot.section.copyWith(
        lifecycle: SectionLifecycle.minted,
        assetId: result.assetId,
      );
      state = state.copyWith(
        snapshot: state.snapshot.copyWith(
          mintedSections: state.snapshot.mintedSections + 1,
          section: section,
        ),
        isBusy: false,
        activity: GameActivity(
          GameNotice.minted,
          transactionSignature: result.transactionSignature,
          assetId: result.assetId,
        ),
      );
      if (!state.snapshot.isDemo) unawaited(refresh());
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> selectSection(int index) async {
    if (index < 0 || index >= sectionCount || state.isBusy) return;
    final demoMode = _repository.isDemoMode;
    state = state.copyWith(
      snapshot: demoMode
          ? GameSnapshot.demo(sectionIndex: index)
          : GameSnapshot.empty(
              gameIndex: state.snapshot.gameIndex,
              sectionIndex: index,
            ),
      queued: const {},
      clearCursor: true,
      activity: GameActivity(GameNotice.sectionChanged, sectionIndex: index),
      loadStatus: demoMode ? GameLoadStatus.demo : GameLoadStatus.loading,
    );
    if (!demoMode) await refresh();
  }

  void _commitLocally(
    List<PixelCoordinate> coordinates, {
    String? transactionSignature,
  }) {
    final current = state.snapshot;
    final section = current.section.copyWith(
      bitmap: current.section.bitmap.toggled(coordinates),
      flipCount: current.section.flipCount + BigInt.from(coordinates.length),
      revision: current.section.revision + BigInt.one,
    );
    state = state.copyWith(
      snapshot: current.copyWith(
        totalFlips: current.totalFlips + BigInt.from(coordinates.length),
        section: section,
      ),
      queued: const {},
      clearCursor: true,
      isBusy: false,
      activity: GameActivity(
        GameNotice.committed,
        transactionSignature: transactionSignature,
      ),
    );
  }
}
