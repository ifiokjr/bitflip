// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'game_controller.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(bitflipRepository)
final bitflipRepositoryProvider = BitflipRepositoryProvider._();

final class BitflipRepositoryProvider
    extends
        $FunctionalProvider<
          BitflipRepository,
          BitflipRepository,
          BitflipRepository
        >
    with $Provider<BitflipRepository> {
  BitflipRepositoryProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'bitflipRepositoryProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$bitflipRepositoryHash();

  @$internal
  @override
  $ProviderElement<BitflipRepository> $createElement(
    $ProviderPointer pointer,
  ) => $ProviderElement(pointer);

  @override
  BitflipRepository create(Ref ref) {
    return bitflipRepository(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(BitflipRepository value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<BitflipRepository>(value),
    );
  }
}

String _$bitflipRepositoryHash() => r'dd814ba28ca085ce92d7ac3e61f3c2ba971ce25c';

@ProviderFor(GameController)
final gameControllerProvider = GameControllerProvider._();

final class GameControllerProvider
    extends $NotifierProvider<GameController, GameViewState> {
  GameControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'gameControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$gameControllerHash();

  @$internal
  @override
  GameController create() => GameController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(GameViewState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<GameViewState>(value),
    );
  }
}

String _$gameControllerHash() => r'a8c4b75ef3f8403dc4f9ec10448c4cefa8fe711b';

abstract class _$GameController extends $Notifier<GameViewState> {
  GameViewState build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<GameViewState, GameViewState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<GameViewState, GameViewState>,
              GameViewState,
              Object?,
              Object?
            >;
    return element.handleCreate(ref, build);
  }
}
