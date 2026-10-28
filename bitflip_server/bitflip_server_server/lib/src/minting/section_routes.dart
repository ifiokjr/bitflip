import 'dart:collection';
import 'dart:convert';

import 'package:bitflip_program/bitflip_program_constraints.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:crypto/crypto.dart';
import 'package:serverpod/serverpod.dart';

const _svgMimeType = MimeType('image', 'svg+xml');

/// Sealed artwork is permanent, so clients and CDNs may keep it for a year
/// without revalidating.
final sealedAssetCacheControl = CacheControlHeader(
  publicCache: true,
  maxAge: 31536000,
  immutable: true,
);

/// One rendered, immutable asset of a sealed section.
final class SectionAsset {
  SectionAsset({required this.body, required this.mimeType})
    : etag = ETagHeader(
        value: sha256.convert(utf8.encode(body)).toString().substring(0, 32),
      );

  final String body;
  final MimeType mimeType;
  final ETagHeader etag;
}

/// Shares bounded artwork and negative caches between both public routes.
///
/// New chain loads have a process-wide concurrency and rolling-window budget
/// within this shared instance. Saturation rejects work without queueing it;
/// cached assets, cached misses, and coalesced loads need no admission slot.
final class SealedSectionArtwork {
  SealedSectionArtwork({
    required this.mintService,
    required String publicBaseUrl,
    this.unsealedRetryAfter = const Duration(seconds: 30),
    this.maximumCachedSections = 256,
    this.maximumConcurrentLoads = 4,
    this.maximumLoadsPerWindow = 32,
    this.loadWindow = const Duration(seconds: 30),
    DateTime Function()? clock,
  }) : _publicBaseUrl = publicBaseUrl.replaceFirst(RegExp(r'/+$'), ''),
       _clock = clock ?? DateTime.now {
    if (maximumCachedSections < 1 ||
        maximumConcurrentLoads < 1 ||
        maximumLoadsPerWindow < 1 ||
        unsealedRetryAfter <= Duration.zero ||
        loadWindow <= Duration.zero) {
      throw ArgumentError('Artwork cache and load limits must be positive.');
    }
  }

  final BitflipMintService mintService;
  final String _publicBaseUrl;
  final Duration unsealedRetryAfter;

  /// Each of the positive and negative caches retains at most this many keys.
  final int maximumCachedSections;
  final int maximumConcurrentLoads;
  final int maximumLoadsPerWindow;
  final Duration loadWindow;
  final DateTime Function() _clock;
  final Map<(int, int), _RenderedSection> _rendered = {};
  final Map<(int, int), Future<_RenderedSection>> _loading = {};
  final Map<(int, int), DateTime> _unavailableUntil = {};
  final Queue<DateTime> _loadStarts = Queue();

  Future<SectionAsset> metadata(int gameIndex, int sectionIndex) async {
    return (await _section(gameIndex, sectionIndex)).metadata;
  }

  Future<SectionAsset> art(int gameIndex, int sectionIndex) async {
    return (await _section(gameIndex, sectionIndex)).art;
  }

  Future<_RenderedSection> _section(int gameIndex, int sectionIndex) async {
    _validateIndices(gameIndex, sectionIndex);
    final key = (gameIndex, sectionIndex);
    final rendered = _rendered.remove(key);

    if (rendered != null) {
      _rendered[key] = rendered;

      return rendered;
    }

    final now = _clock();
    final retryAt = _unavailableUntil[key];

    if (retryAt != null && now.isBefore(retryAt)) {
      throw StateError('The section has no permanent artwork yet.');
    }

    _unavailableUntil.remove(key);
    final loading = _loading[key];

    if (loading != null) return loading;

    while (_loadStarts.isNotEmpty &&
        !now.isBefore(_loadStarts.first.add(loadWindow))) {
      _loadStarts.removeFirst();
    }

    // Security: distinct coordinates must not bypass coalescing and consume
    // the RPC capacity used by minting, indexing, and readiness. Failed loads
    // also spend the budget; there is no queue of attacker-controlled keys.
    if (_loading.length >= maximumConcurrentLoads ||
        _loadStarts.length >= maximumLoadsPerWindow) {
      throw SectionArtworkBusy(loadWindow);
    }

    _loadStarts.addLast(now);

    return _loading[key] = _load(key).whenComplete(() {
      _loading.remove(key);
    });
  }

  void _rememberUnavailable((int, int) key) {
    final now = _clock();
    _unavailableUntil.removeWhere((_, until) => !now.isBefore(until));

    if (_unavailableUntil.length >= maximumCachedSections) {
      _unavailableUntil.remove(_unavailableUntil.keys.first);
    }

    _unavailableUntil[key] = now.add(unsealedRetryAfter);
  }

  Future<_RenderedSection> _load((int, int) key) async {
    final MintableSection section;

    try {
      section = await mintService.loadSection(key.$1, key.$2);
    } on StateError {
      // loadSection uses StateError for absent/invalid on-chain accounts,
      // already exposed as 404 by these routes. Transport failures propagate
      // without becoming cached 404s.
      _rememberUnavailable(key);
      rethrow;
    }

    if (!section.isSealed && !section.isMinted) {
      _rememberUnavailable(key);
      throw StateError('Only sealed sections have permanent artwork.');
    }

    if (section.bitmap.length != 512) {
      throw StateError('The section bitmap has an invalid length.');
    }

    final rendered = _RenderedSection(
      metadata: SectionAsset(
        body: renderSectionMetadata(section, publicBaseUrl: _publicBaseUrl),
        mimeType: MimeType.json,
      ),
      art: SectionAsset(
        body: renderSectionSvg(section),
        mimeType: _svgMimeType,
      ),
    );
    if (_rendered.length >= maximumCachedSections) {
      _rendered.remove(_rendered.keys.first);
    }

    _rendered[key] = rendered;
    _unavailableUntil.remove(key);

    return rendered;
  }
}

/// The public artwork RPC budget is exhausted; callers may retry later.
final class SectionArtworkBusy implements Exception {
  const SectionArtworkBusy(this.retryAfter);

  final Duration retryAfter;
}

final class _RenderedSection {
  const _RenderedSection({required this.metadata, required this.art});

  final SectionAsset metadata;
  final SectionAsset art;
}

/// Serves [asset] with immutable caching headers, answering a matching
/// `If-None-Match` revalidation with `304 Not Modified` instead of the body.
Response sectionAssetResponse(
  SectionAsset asset, {
  IfNoneMatchHeader? ifNoneMatch,
}) {
  final headers = Headers.build((mutable) {
    mutable
      ..cacheControl = sealedAssetCacheControl
      ..etag = asset.etag;
  });
  final matches =
      ifNoneMatch != null &&
      (ifNoneMatch.isWildcard ||
          ifNoneMatch.etags.any((etag) => etag.value == asset.etag.value));

  if (matches) return Response.notModified(headers: headers);

  return Response.ok(
    body: Body.fromString(asset.body, mimeType: asset.mimeType),
    headers: headers,
  );
}

/// Maps asset lookup failures onto HTTP statuses: malformed paths are client
/// errors, while unknown, unsealed, or out-of-range sections do not exist.
Future<Response> serveSectionAsset(
  Future<SectionAsset> Function() load, {
  IfNoneMatchHeader? ifNoneMatch,
}) async {
  try {
    return sectionAssetResponse(await load(), ifNoneMatch: ifNoneMatch);
  } on FormatException {
    return Response.badRequest();
  } on RangeError {
    return Response.notFound();
  } on SectionArtworkBusy catch (error) {
    return Response(
      503,
      headers: Headers.build((headers) {
        headers
          ..cacheControl = CacheControlHeader(noStore: true)
          ..retryAfter = RetryAfterHeader(
            delay: (error.retryAfter.inMicroseconds + 999999) ~/ 1000000,
          );
      }),
    );
  } on StateError {
    return Response.notFound();
  }
}

final class SectionMetadataRoute extends Route {
  SectionMetadataRoute(this._artwork);

  static const _gameParam = IntPathParam(#game);
  static const _sectionParam = PathParam<int>(#section, _parseJsonIndex);

  final SealedSectionArtwork _artwork;

  @override
  Future<Result> handleCall(Session session, Request request) {
    return serveSectionAsset(
      () => _artwork.metadata(
        request.pathParameters.get(_gameParam),
        request.pathParameters.get(_sectionParam),
      ),
      ifNoneMatch: request.headers.ifNoneMatch,
    );
  }
}

final class SectionArtRoute extends Route {
  SectionArtRoute(this._artwork);

  static const _gameParam = IntPathParam(#game);
  static const _sectionParam = PathParam<int>(#section, _parseSvgIndex);

  final SealedSectionArtwork _artwork;

  @override
  Future<Result> handleCall(Session session, Request request) {
    return serveSectionAsset(
      () => _artwork.art(
        request.pathParameters.get(_gameParam),
        request.pathParameters.get(_sectionParam),
      ),
      ifNoneMatch: request.headers.ifNoneMatch,
    );
  }
}

String renderSectionMetadata(
  MintableSection section, {
  required String publicBaseUrl,
}) {
  final gameIndex = section.gameIndex;
  final sectionIndex = section.sectionIndex;
  final paddedSection = sectionIndex.toString().padLeft(3, '0');
  final imageUri = '$publicBaseUrl/art/$gameIndex/$sectionIndex.svg';

  return jsonEncode({
    'name': 'Bitflip $gameIndex:$paddedSection',
    'symbol': 'BITFLIP',
    'description':
        'A sealed 64 × 64 fragment of Bitflip’s collaborative on-chain canvas.',
    'image': imageUri,
    'external_url': '$publicBaseUrl/?game=$gameIndex&section=$sectionIndex',
    'attributes': [
      {'trait_type': 'Game', 'value': gameIndex},
      {'trait_type': 'Section', 'value': sectionIndex},
      {'trait_type': 'Canvas', 'value': '1024 × 1024'},
      {'trait_type': 'Storage', 'value': 'Compressed NFT'},
      {'trait_type': 'Pixel source', 'value': 'Solana account'},
      {'trait_type': 'Pixels on', 'value': _countEnabledPixels(section.bitmap)},
    ],
    'properties': {
      'category': 'image',
      'files': [
        {'uri': imageUri, 'type': 'image/svg+xml'},
      ],
    },
  });
}

/// Lit pixels are emitted as horizontal runs of a single path, which renders
/// identically to one rectangle per pixel under `crispEdges` at a fraction of
/// the bytes.
String renderSectionSvg(MintableSection section) {
  final label =
      'Bitflip ${section.gameIndex}:${section.sectionIndex.toString().padLeft(3, '0')}';
  final runs = StringBuffer();

  for (var y = 0; y < 64; y++) {
    var x = 0;

    while (x < 64) {
      if (!_isEnabled(section.bitmap, x, y)) {
        x++;
        continue;
      }

      final start = x;

      while (x < 64 && _isEnabled(section.bitmap, x, y)) {
        x++;
      }

      final length = x - start;
      runs.write('M$start ${y}h${length}v1h-${length}z');
    }
  }

  final pixels = runs.isEmpty ? '' : '<path fill="#b8ff2c" d="$runs"/>';

  return '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="1024" height="1024" shape-rendering="crispEdges" role="img" aria-label="$label">
<title>$label</title>
<rect width="64" height="64" fill="#050b0a"/>
$pixels<path d="M.5.5h63v63H.5z" fill="none" stroke="#eaf4e5" stroke-opacity=".18"/>
</svg>''';
}

bool _isEnabled(List<int> bitmap, int x, int y) {
  final offset = y * 64 + x;

  return bitmap[offset >> 3] & (1 << (offset & 7)) != 0;
}

int _countEnabledPixels(List<int> bitmap) {
  var count = 0;

  for (final byte in bitmap) {
    var value = byte;

    while (value != 0) {
      value &= value - 1;
      count++;
    }
  }

  return count;
}

void _validateIndices(int gameIndex, int sectionIndex) {
  if (gameIndex < 0 || gameIndex > bitflipMaximumGameIndex) {
    throw RangeError.range(
      gameIndex,
      0,
      bitflipMaximumGameIndex,
      'gameIndex',
    );
  }

  if (sectionIndex < 0 || sectionIndex > bitflipMaximumSectionIndex) {
    throw RangeError.range(
      sectionIndex,
      0,
      bitflipMaximumSectionIndex,
      'sectionIndex',
    );
  }
}

int _parseJsonIndex(String value) => _parseIndex(value, '.json');

int _parseSvgIndex(String value) => _parseIndex(value, '.svg');

int _parseIndex(String value, String extension) {
  if (!value.endsWith(extension)) {
    throw const FormatException('Invalid section asset path.');
  }

  return int.parse(value.substring(0, value.length - extension.length));
}
