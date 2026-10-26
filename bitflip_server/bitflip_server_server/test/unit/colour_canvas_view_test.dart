import 'dart:typed_data';

import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:test/test.dart';

void main() {
  test('colour canvas view round-trips with and without the colour bytes', () {
    final full = ColourCanvasView(
      gameIndex: 0,
      sectionIndex: 1,
      policyVersion: 2,
      highestRevision: 3,
      colours: ByteData(4)..setUint8(0, 9),
    );
    final unchanged = full.copyWith(colours: null);
    final cloned = full.copyWith(highestRevision: 4);

    final decodedFull = ColourCanvasView.fromJson(full.toJson());
    final decodedUnchanged = ColourCanvasView.fromJson(
      unchanged.toJsonForProtocol(),
    );

    expect(decodedFull.colours?.getUint8(0), 9);
    expect(decodedUnchanged.colours, isNull);
    expect(decodedUnchanged.highestRevision, 3);
    expect(cloned.colours?.getUint8(0), 9);
    expect(identical(cloned.colours, full.colours), isFalse);
    expect(unchanged.toJson()['colours'], isNull);
    expect(unchanged.toString(), contains('ColourCanvasView'));
  });
}
