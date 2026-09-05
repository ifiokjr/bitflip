import 'dart:convert';

import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:serverpod/serverpod.dart';

const _svgMimeType = MimeType('image', 'svg+xml');

final class SectionMetadataRoute extends Route {
  SectionMetadataRoute({
    required BitflipMintService mintService,
    required String publicBaseUrl,
  }) : _sections = _SealedSectionCache(mintService),
       _publicBaseUrl = publicBaseUrl.replaceFirst(RegExp(r'/+$'), '');

  static const _gameParam = IntPathParam(#game);
  static const _sectionParam = PathParam<int>(#section, _parseJsonIndex);

  final _SealedSectionCache _sections;
  final String _publicBaseUrl;

  @override
  Future<Result> handleCall(Session session, Request request) async {
    try {
      final gameIndex = request.pathParameters.get(_gameParam);
      final sectionIndex = request.pathParameters.get(_sectionParam);
      final section = await _sections.get(gameIndex, sectionIndex);
      final paddedSection = sectionIndex.toString().padLeft(3, '0');
      final imageUri = '$_publicBaseUrl/art/$gameIndex/$sectionIndex.svg';
      return Response.ok(
        body: Body.fromString(
          jsonEncode({
            'name': 'Bitflip $gameIndex:$paddedSection',
            'symbol': 'BITFLIP',
            'description': 'A sealed 64 × 64 fragment of Bitflip’s collaborative on-chain canvas.',
            'image': imageUri,
            'external_url':
                '$_publicBaseUrl/?game=$gameIndex&section=$sectionIndex',
            'attributes': [
              {'trait_type': 'Game', 'value': gameIndex},
              {'trait_type': 'Section', 'value': sectionIndex},
              {'trait_type': 'Canvas', 'value': '1024 × 1024'},
              {'trait_type': 'Storage', 'value': 'Compressed NFT'},
              {'trait_type': 'Pixel source', 'value': 'Solana account'},
              {
                'trait_type': 'Pixels on',
                'value': _countEnabledPixels(section.bitmap),
              },
            ],
            'properties': {
              'category': 'image',
              'files': [
                {'uri': imageUri, 'type': 'image/svg+xml'},
              ],
            },
          }),
          mimeType: MimeType.json,
        ),
      );
    } on FormatException {
      return Response.badRequest();
    } on RangeError {
      return Response.notFound();
    } on StateError {
      return Response.notFound();
    }
  }
}

final class SectionArtRoute extends Route {
  SectionArtRoute(BitflipMintService mintService)
    : _sections = _SealedSectionCache(mintService);

  static const _gameParam = IntPathParam(#game);
  static const _sectionParam = PathParam<int>(#section, _parseSvgIndex);

  final _SealedSectionCache _sections;

  @override
  Future<Result> handleCall(Session session, Request request) async {
    try {
      final gameIndex = request.pathParameters.get(_gameParam);
      final sectionIndex = request.pathParameters.get(_sectionParam);
      final section = await _sections.get(gameIndex, sectionIndex);
      return Response.ok(
        body: Body.fromString(_renderSvg(section), mimeType: _svgMimeType),
      );
    } on FormatException {
      return Response.badRequest();
    } on RangeError {
      return Response.notFound();
    } on StateError {
      return Response.notFound();
    }
  }
}

final class _SealedSectionCache {
  _SealedSectionCache(this._mintService);

  final BitflipMintService _mintService;
  final Map<String, MintableSection> _cache = {};

  Future<MintableSection> get(int gameIndex, int sectionIndex) async {
    _validateIndices(gameIndex, sectionIndex);
    final key = '$gameIndex:$sectionIndex';
    final cached = _cache[key];
    if (cached != null) return cached;

    final section = await _mintService.loadSection(gameIndex, sectionIndex);
    if (!section.isSealed && !section.isMinted) {
      throw StateError('Only sealed sections have permanent artwork.');
    }
    if (section.bitmap.length != 512) {
      throw StateError('The section bitmap has an invalid length.');
    }
    _cache[key] = section;
    return section;
  }
}

String _renderSvg(MintableSection section) {
  final label =
      'Bitflip ${section.gameIndex}:${section.sectionIndex.toString().padLeft(3, '0')}';
  final pixels = StringBuffer();
  for (var y = 0; y < 64; y++) {
    for (var x = 0; x < 64; x++) {
      final offset = y * 64 + x;
      final enabled = section.bitmap[offset >> 3] & (1 << (offset & 7)) != 0;
      if (enabled) {
        pixels.write('<rect x="$x" y="$y" width="1" height="1"/>');
      }
    }
  }
  return '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="1024" height="1024" shape-rendering="crispEdges" role="img" aria-label="$label">
<title>$label</title>
<rect width="64" height="64" fill="#050b0a"/>
<g fill="#b8ff2c">$pixels</g>
<path d="M.5.5h63v63H.5z" fill="none" stroke="#eaf4e5" stroke-opacity=".18"/>
</svg>''';
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
  if (gameIndex < 0 || gameIndex > 255) {
    throw RangeError.range(gameIndex, 0, 255, 'gameIndex');
  }
  if (sectionIndex < 0 || sectionIndex > 255) {
    throw RangeError.range(sectionIndex, 0, 255, 'sectionIndex');
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
