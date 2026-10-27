import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bitflip_server_server/src/minting/section_routes.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

void main() {
  withServerpod('Sealed section routes over HTTP', (sessionBuilder, endpoints) {
    test('serve immutable, revalidatable artwork and metadata', () async {
      final session = sessionBuilder.build();
      final pod = session.serverpod;
      final service = _FakeMintService();
      final artwork = SealedSectionArtwork(
        mintService: service,
        publicBaseUrl: 'https://bitflip.art',
      );
      pod.webServer
        ..addRoute(SectionMetadataRoute(artwork), '/metadata/:game/:section')
        ..addRoute(SectionArtRoute(artwork), '/art/:game/:section');
      expect(await pod.webServer.start(), isTrue);
      final client = HttpClient();
      addTearDown(() async {
        client.close(force: true);
        await session.close();
      });
      Uri url(String path) => Uri.http('localhost:${pod.webServer.port}', path);

      final art = await client
          .getUrl(url('/art/0/5.svg'))
          .then((r) => r.close());
      final svg = await utf8.decodeStream(art);
      final etag = art.headers.value(HttpHeaders.etagHeader);

      expect(art.statusCode, 200);
      expect(art.headers.contentType?.mimeType, 'image/svg+xml');
      expect(
        art.headers.value(HttpHeaders.cacheControlHeader),
        allOf(contains('max-age=31536000'), contains('immutable')),
      );
      expect(etag, isNotNull);
      expect(svg, contains('<path fill="#b8ff2c" d="M0 0h3v1h-3z"/>'));

      final revalidation = await client.getUrl(url('/art/0/5.svg')).then((
        request,
      ) {
        request.headers.set(HttpHeaders.ifNoneMatchHeader, etag!);

        return request.close();
      });

      expect(revalidation.statusCode, 304);
      expect(await revalidation.length, 0);

      final metadata = await client
          .getUrl(url('/metadata/0/5.json'))
          .then((r) => r.close());
      final json =
          jsonDecode(await utf8.decodeStream(metadata)) as Map<String, Object?>;

      expect(metadata.statusCode, 200);
      expect(metadata.headers.contentType?.mimeType, 'application/json');
      expect(json['image'], 'https://bitflip.art/art/0/5.svg');
      expect(service.loads, 1, reason: 'both assets come from one chain read');

      final unsealed = await client
          .getUrl(url('/art/0/7.svg'))
          .then((r) => r.close());
      final malformed = await client
          .getUrl(url('/art/0/5.png'))
          .then((r) => r.close());
      final outOfRange = await client
          .getUrl(url('/metadata/0/999.json'))
          .then((r) => r.close());

      expect(unsealed.statusCode, 404);
      expect(malformed.statusCode, 400);
      expect(outOfRange.statusCode, 404);
      await Future.wait([
        unsealed.drain<void>(),
        malformed.drain<void>(),
        outOfRange.drain<void>(),
      ]);
    });
  });
}

final class _FakeMintService implements BitflipMintService {
  var loads = 0;

  @override
  Future<MintableSection> loadSection(int gameIndex, int sectionIndex) async {
    loads++;
    final bitmap = Uint8List(512);
    bitmap[0] = 7;

    return MintableSection(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      owner: systemProgramAddress,
      status: sectionIndex == 5 ? 2 : 1,
      collectionAuthority: systemProgramAddress,
      configAddress: systemProgramAddress,
      gameAddress: systemProgramAddress,
      sectionAddress: systemProgramAddress,
      bitmap: bitmap,
    );
  }

  @override
  Future<MintSubmission> mint(MintableSection section) {
    throw UnsupportedError('Artwork never mints.');
  }
}
