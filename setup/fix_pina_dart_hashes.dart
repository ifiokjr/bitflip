import 'dart:io';

final _hashGetter = RegExp(r'int get hashCode => Object\.hash\(([^;\n]*)\);');

void main(List<String> arguments) {
  if (arguments.length != 1) {
    stderr.writeln(
      'Usage: dart fix_pina_dart_hashes.dart <generated-directory>',
    );
    exitCode = 64;
    return;
  }

  final generatedDirectory = Directory(arguments.single);
  if (!generatedDirectory.existsSync()) {
    stderr.writeln('Generated directory does not exist: ${arguments.single}');
    exitCode = 66;
    return;
  }

  for (final entity in generatedDirectory.listSync(recursive: true)) {
    if (entity is! File || !entity.path.endsWith('.dart')) {
      continue;
    }

    final original = entity.readAsStringSync();
    final fixed = original.replaceAllMapped(
      _hashGetter,
      (match) => 'int get hashCode => Object.hashAll([${match.group(1)}]);',
    );
    if (fixed != original) {
      entity.writeAsStringSync(fixed);
    }
  }
}
