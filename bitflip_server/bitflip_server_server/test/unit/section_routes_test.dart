import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bitflip_server_server/src/minting/section_routes.dart';
import 'package:serverpod/serverpod.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:test/test.dart';

const _sealed = 2;
const _minted = 3;
const _active = 1;

void main() {
  group('SealedSectionArtwork', () {
    test(
      'renders a sealed section once and serves both assets from it',
      () async {
        final service = _FakeMintService({
          (0, 5): _section(0, 5, status: _sealed),
        });
        final artwork = SealedSectionArtwork(
          mintService: service,
          publicBaseUrl: 'https://bitflip.art/',
        );

        final metadata = await artwork.metadata(0, 5);
        final art = await artwork.art(0, 5);
        final artAgain = await artwork.art(0, 5);

        expect(service.loads, 1);
        expect(identical(art, artAgain), isTrue);
        expect(metadata.mimeType, MimeType.json);
        expect(art.mimeType.toString(), contains('svg'));
        expect(metadata.etag.value, hasLength(32));
        expect(metadata.etag.value, isNot(equals(art.etag.value)));

        final json = jsonDecode(metadata.body) as Map<String, Object?>;

        expect(json['name'], 'Bitflip 0:005');
        expect(json['image'], 'https://bitflip.art/art/0/5.svg');
        expect(json['external_url'], 'https://bitflip.art/?game=0&section=5');
        expect(
          (json['attributes'] as List<Object?>).last,
          {'trait_type': 'Pixels on', 'value': 3},
        );
      },
    );

    test('shares one chain read between concurrent misses', () async {
      final gate = Completer<void>();
      final service = _FakeMintService(
        {(0, 1): _section(0, 1, status: _minted)},
        beforeLoad: () => gate.future,
      );
      final artwork = SealedSectionArtwork(
        mintService: service,
        publicBaseUrl: 'https://bitflip.art',
      );

      final first = artwork.metadata(0, 1);
      final second = artwork.art(0, 1);
      gate.complete();
      await Future.wait([first, second]);

      expect(service.loads, 1);
    });

    test(
      'remembers an unsealed section until the retry window passes',
      () async {
        var now = DateTime.utc(2026, 10, 26, 12);
        final service = _FakeMintService({
          (0, 2): _section(0, 2, status: _active),
        });
        final artwork = SealedSectionArtwork(
          mintService: service,
          publicBaseUrl: 'https://bitflip.art',
          unsealedRetryAfter: const Duration(seconds: 30),
          clock: () => now,
        );

        await expectLater(artwork.art(0, 2), throwsStateError);
        await expectLater(artwork.metadata(0, 2), throwsStateError);
        expect(service.loads, 1, reason: 'the miss is remembered');

        now = now.add(const Duration(seconds: 31));
        service.sections[(0, 2)] = _section(0, 2, status: _sealed);

        expect((await artwork.art(0, 2)).body, contains('<svg'));
        expect(service.loads, 2);
      },
    );

    test('rejects malformed bitmaps and out-of-range sections', () async {
      final service = _FakeMintService({
        (0, 3): _section(0, 3, status: _sealed, bitmap: Uint8List(511)),
      });
      final artwork = SealedSectionArtwork(
        mintService: service,
        publicBaseUrl: 'https://bitflip.art',
      );

      await expectLater(artwork.art(0, 3), throwsStateError);
      await expectLater(artwork.art(0, 256), throwsRangeError);
      await expectLater(artwork.metadata(-1, 0), throwsRangeError);
      expect(service.loads, 1);
    });
  });

  group('sectionAssetResponse', () {
    final asset = SectionAsset(body: '<svg/>', mimeType: MimeType.json);

    test('serves the body with immutable caching headers', () async {
      final response = sectionAssetResponse(asset);

      expect(response.statusCode, 200);
      expect(response.headers.cacheControl?.immutable, isTrue);
      expect(response.headers.cacheControl?.publicCache, isTrue);
      expect(response.headers.cacheControl?.maxAge, 31536000);
      expect(response.headers.etag?.value, asset.etag.value);
      expect(await response.readAsString(), '<svg/>');
    });

    test('answers a matching revalidation with 304 and no body', () async {
      final matching = sectionAssetResponse(
        asset,
        ifNoneMatch: IfNoneMatchHeader.etags([asset.etag]),
      );
      final wildcard = sectionAssetResponse(
        asset,
        ifNoneMatch: const IfNoneMatchHeader.wildcard(),
      );
      final stale = sectionAssetResponse(
        asset,
        ifNoneMatch: IfNoneMatchHeader.etags([
          const ETagHeader(value: 'something-else'),
        ]),
      );

      expect(matching.statusCode, 304);
      expect(matching.headers.etag?.value, asset.etag.value);
      expect(wildcard.statusCode, 304);
      expect(stale.statusCode, 200);
    });
  });

  group('serveSectionAsset', () {
    test('maps lookup failures onto HTTP statuses', () async {
      Future<Response> serve(Object error) {
        return serveSectionAsset(() async => throw error);
      }

      expect((await serve(const FormatException())).statusCode, 400);
      expect((await serve(RangeError('range'))).statusCode, 404);
      expect((await serve(StateError('unsealed'))).statusCode, 404);
      expect(
        (await serveSectionAsset(
          () async => SectionAsset(body: '{}', mimeType: MimeType.json),
        )).statusCode,
        200,
      );
    });
  });

  group('renderSectionSvg', () {
    test('encodes exactly the lit pixels as horizontal runs', () {
      final bitmap = Uint8List(512);

      for (var index = 0; index < 4096; index++) {
        final x = index % 64;
        final y = index ~/ 64;
        final lit = (x * 7 + y * 13) % 5 == 0 || (y == 10 && x > 3 && x < 60);

        if (lit) bitmap[index >> 3] |= 1 << (index & 7);
      }

      final svg = renderSectionSvg(
        _section(0, 9, status: _sealed, bitmap: bitmap),
      );

      expect(svg, contains('aria-label="Bitflip 0:009"'));
      expect(_decodeRuns(svg), equals(bitmap));
    });

    test('omits the pixel path for an empty canvas', () {
      final svg = renderSectionSvg(
        _section(0, 0, status: _sealed, bitmap: Uint8List(512)),
      );

      expect(svg, isNot(contains('<path fill')));
      expect(svg, contains('<rect width="64" height="64"'));
    });

    test('stays compact for dense and alternating canvases', () {
      final full = Uint8List(512)..fillRange(0, 512, 0xff);
      final checkerboard = Uint8List(512);

      for (var index = 0; index < 4096; index++) {
        if ((index % 64 + index ~/ 64).isEven) {
          checkerboard[index >> 3] |= 1 << (index & 7);
        }
      }

      final fullSvg = renderSectionSvg(
        _section(0, 0, status: _sealed, bitmap: full),
      );
      final boardSvg = renderSectionSvg(
        _section(0, 0, status: _sealed, bitmap: checkerboard),
      );

      // Every pixel lit collapses to one run per row; the alternating board is
      // the worst case and must still stay far below one rect per pixel.
      expect(utf8.encode(fullSvg).length, lessThan(2_000));
      expect(utf8.encode(boardSvg).length, lessThan(40_000));
      expect(_decodeRuns(fullSvg), equals(full));
      expect(_decodeRuns(boardSvg), equals(checkerboard));
    });
  });
}

/// Replays the SVG path runs back into a bitmap so the encoding can be
/// checked bit for bit against the source.
Uint8List _decodeRuns(String svg) {
  final bitmap = Uint8List(512);
  final path = RegExp(r'<path fill="#b8ff2c" d="([^"]*)"').firstMatch(svg);

  if (path == null) return bitmap;

  for (final run in RegExp(
    r'M(\d+) (\d+)h(\d+)v1h-\d+z',
  ).allMatches(path.group(1)!)) {
    final x = int.parse(run.group(1)!);
    final y = int.parse(run.group(2)!);
    final length = int.parse(run.group(3)!);

    for (var offset = x; offset < x + length; offset++) {
      final index = y * 64 + offset;
      bitmap[index >> 3] |= 1 << (index & 7);
    }
  }

  return bitmap;
}

MintableSection _section(
  int gameIndex,
  int sectionIndex, {
  required int status,
  Uint8List? bitmap,
}) {
  return MintableSection(
    gameIndex: gameIndex,
    sectionIndex: sectionIndex,
    owner: systemProgramAddress,
    status: status,
    collectionAuthority: systemProgramAddress,
    configAddress: systemProgramAddress,
    gameAddress: systemProgramAddress,
    sectionAddress: systemProgramAddress,
    bitmap: bitmap ?? _threePixels(),
  );
}

Uint8List _threePixels() {
  final bitmap = Uint8List(512);
  bitmap[0] = 7;

  return bitmap;
}

final class _FakeMintService implements BitflipMintService {
  _FakeMintService(this.sections, {this.beforeLoad});

  final Map<(int, int), MintableSection> sections;
  final Future<void> Function()? beforeLoad;
  var loads = 0;

  @override
  Future<MintableSection> loadSection(int gameIndex, int sectionIndex) async {
    loads++;
    await beforeLoad?.call();

    return sections[(gameIndex, sectionIndex)] ??
        (throw StateError('No section $gameIndex:$sectionIndex in the fake.'));
  }

  @override
  Future<MintSubmission> mint(MintableSection section) {
    throw UnsupportedError('Artwork never mints.');
  }
}
