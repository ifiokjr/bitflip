import 'package:serverpod/serverpod.dart';

const _contentSecurityPolicy =
    "default-src 'self'; "
    "base-uri 'self'; "
    "connect-src 'self' https: wss:; "
    "font-src 'self' data:; "
    "form-action 'self'; "
    "frame-ancestors 'none'; "
    "img-src 'self' data: blob:; "
    "manifest-src 'self'; "
    "object-src 'none'; "
    "script-src 'self' 'wasm-unsafe-eval' "
    "'sha256-tecZABc+RVGVG9QvlzKN61d19xR30pC0YmMtO8K74C8='; "
    "style-src 'self' 'unsafe-inline'; "
    "worker-src 'self' blob:";

Headers withSecurityHeaders(Headers headers) {
  return headers.transform((mutable) {
    mutable
      ..['Content-Security-Policy'] = const [_contentSecurityPolicy]
      ..['Permissions-Policy'] = const [
        'camera=(), geolocation=(), microphone=(), usb=()',
      ]
      ..['Referrer-Policy'] = const ['no-referrer']
      ..['X-Content-Type-Options'] = const ['nosniff']
      ..['X-Frame-Options'] = const ['DENY'];
  });
}

final class SecurityHeadersMiddleware extends MiddlewareObject {
  const SecurityHeadersMiddleware();

  @override
  Handler call(Handler next) {
    return (request) async {
      final result = await next(request);
      if (result is! Response) return result;
      return result.copyWith(headers: withSecurityHeaders(result.headers));
    };
  }
}
