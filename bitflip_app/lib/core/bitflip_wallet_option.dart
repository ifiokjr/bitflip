enum BitflipWalletKind { embedded, external, unavailable }

final class BitflipWalletOption {
  const BitflipWalletOption({required this.id, required this.name});

  final String id;
  final String name;
}
