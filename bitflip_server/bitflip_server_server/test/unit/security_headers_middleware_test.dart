import 'package:bitflip_server_server/src/web/security_headers_middleware.dart';
import 'package:serverpod/serverpod.dart';
import 'package:test/test.dart';

void main() {
  test('sets restrictive browser security headers', () {
    final headers = withSecurityHeaders(Headers.empty());

    expect(
      headers['Content-Security-Policy']?.single,
      allOf(
        contains("default-src 'self'"),
        contains("frame-ancestors 'none'"),
        contains("script-src 'self' 'wasm-unsafe-eval'"),
      ),
    );
    expect(headers['Permissions-Policy']?.single, contains('camera=()'));
    expect(headers['Referrer-Policy']?.single, 'no-referrer');
    expect(headers['X-Content-Type-Options']?.single, 'nosniff');
    expect(headers['X-Frame-Options']?.single, 'DENY');
  });

  test('preserves response headers set by the route', () {
    final headers = withSecurityHeaders(
      Headers.fromMap({
        'Content-Type': const ['application/json'],
        'X-Bitflip-Revision': const ['7'],
      }),
    );

    expect(headers['Content-Type']?.single, 'application/json');
    expect(headers['X-Bitflip-Revision']?.single, '7');
  });
}
