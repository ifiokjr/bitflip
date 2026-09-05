import 'dart:async';

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
  sealed,
  minted,
  batchFull,
  connectionIssue,
}

final class GameActivity {
  const GameActivity(this.notice, {this.coordinate, this.sectionIndex});

  final GameNotice notice;
  final PixelCoordinate? coordinate;
  final int? sectionIndex;
}

final class GameViewState {
  const GameViewState({
    required this.snapshot,
    required this.queued,
    required this.activity,
    required this.isBusy,
    required this.isWalletSupported,
    required this.walletAddress,
    this.cursor,
  });

  factory GameViewState.initial({required bool isWalletSupported}) {
    return GameViewState(
      snapshot: GameSnapshot.demo(sectionIndex: 12),
      queued: const {},
      cursor: null,
      activity: const GameActivity(GameNotice.ready),
      isBusy: false,
      isWalletSupported: isWalletSupported,
      walletAddress: null,
    );
  }

  final GameSnapshot snapshot;
  final Set<PixelCoordinate> queued;
  final PixelCoordinate? cursor;
  final GameActivity activity;
  final bool isBusy;
  final bool isWalletSupported;
  final String? walletAddress;

  PixelBitmap get previewBitmap => snapshot.section.bitmap.toggled(queued);

  BigInt get queuedFee => snapshot.flipFeeLamports * BigInt.from(queued.length);

  GameViewState copyWith({
    GameSnapshot? snapshot,
    Set<PixelCoordinate>? queued,
    PixelCoordinate? cursor,
    bool clearCursor = false,
    GameActivity? activity,
    bool? isBusy,
    String? walletAddress,
  }) {
    return GameViewState(
      snapshot: snapshot ?? this.snapshot,
      queued: Set.unmodifiable(queued ?? this.queued),
      cursor: clearCursor ? null : cursor ?? this.cursor,
      activity: activity ?? this.activity,
      isBusy: isBusy ?? this.isBusy,
      isWalletSupported: isWalletSupported,
      walletAddress: walletAddress ?? this.walletAddress,
    );
  }
}

@Riverpod(keepAlive: true)
BitflipRepository bitflipRepository(Ref ref) => SolanaBitflipRepository();

@Riverpod(keepAlive: true)
class GameController extends _$GameController {
  BitflipRepository get _repository => ref.read(bitflipRepositoryProvider);

  List<BitflipWalletOption>? get availableWallets =>
      _repository.availableWallets;

  @override
  GameViewState build() {
    return GameViewState.initial(
      isWalletSupported: _repository.isWalletSupported,
    );
  }

  Future<void> refresh() async {
    if (state.isBusy) return;
    state = state.copyWith(isBusy: true);
    try {
      final loaded = await _repository.loadSection(
        state.snapshot.section.index,
      );
      state = state.copyWith(
        snapshot: loaded ?? state.snapshot,
        isBusy: false,
        walletAddress: _repository.walletAddress,
      );
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> connectWallet([String? walletId]) async {
    if (state.isBusy || !state.isWalletSupported) return;
    state = state.copyWith(isBusy: true);
    try {
      final address = await _repository.connectWallet(walletId);
      state = state.copyWith(
        isBusy: false,
        walletAddress: address,
        activity: const GameActivity(GameNotice.connected),
      );
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  void togglePixel(PixelCoordinate coordinate) {
    if (!state.snapshot.section.isEditable || state.isBusy) return;
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

  void clearQueue() {
    state = state.copyWith(
      queued: const {},
      clearCursor: true,
      activity: const GameActivity(GameNotice.ready),
    );
  }

  Future<void> commitMoves() async {
    if (state.queued.isEmpty || state.isBusy) return;
    final coordinates = state.queued.toList()..sort();
    if (state.snapshot.isDemo) {
      _commitLocally(coordinates);
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      await _repository.flipPixels(state.snapshot, coordinates);
      _commitLocally(coordinates);
      unawaited(refresh());
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> claimSection() async {
    if (state.isBusy || state.snapshot.section.isClaimed) return;
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
      await _repository.claimSection(state.snapshot);
      state = state.copyWith(isBusy: false);
      await refresh();
    } on Object {
      state = state.copyWith(
        isBusy: false,
        activity: const GameActivity(GameNotice.connectionIssue),
      );
    }
  }

  Future<void> sealSection() async {
    if (state.isBusy || !state.snapshot.section.isEditable) return;
    if (!state.snapshot.isDemo) {
      state = state.copyWith(isBusy: true);
      try {
        await _repository.sealSection(state.snapshot);
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
      activity: const GameActivity(GameNotice.sealed),
    );
    if (!state.snapshot.isDemo) unawaited(refresh());
  }

  Future<void> mintSection() async {
    if (state.isBusy ||
        state.snapshot.section.lifecycle != SectionLifecycle.sealed) {
      return;
    }
    state = state.copyWith(isBusy: true);
    try {
      final assetId = state.snapshot.isDemo
          ? 'cnft:demo:${state.snapshot.section.index}'
          : await _repository.mintSection(state.snapshot);
      final section = state.snapshot.section.copyWith(
        lifecycle: SectionLifecycle.minted,
        assetId: assetId,
      );
      state = state.copyWith(
        snapshot: state.snapshot.copyWith(
          mintedSections: state.snapshot.mintedSections + 1,
          section: section,
        ),
        isBusy: false,
        activity: const GameActivity(GameNotice.minted),
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
    state = state.copyWith(
      snapshot: GameSnapshot.demo(sectionIndex: index),
      queued: const {},
      clearCursor: true,
      activity: GameActivity(GameNotice.sectionChanged, sectionIndex: index),
    );
    await refresh();
  }

  void _commitLocally(List<PixelCoordinate> coordinates) {
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
      activity: const GameActivity(GameNotice.committed),
    );
  }
}
