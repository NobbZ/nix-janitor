import 'dart:io';

import 'package:logging/logging.dart';

import 'package:nix_janitor/nix_janitor.dart' as nix_janitor;

final List<String> profiles = [
  "/nix/var/nix/profiles/system",
  "/nix/var/nix/profiles/per-user/nmelzer/profile",
  "/home/nmelzer/.local/state/nix/profiles/home-manager",
];

final int numberOfGenerationsToKeep = 7;
final Duration deleteOlderThan = Duration(days: 7);

final log = Logger('main');

void main(List<String> arguments) async {
  Logger.root.level = Level.ALL;
  Logger.root.onRecord.listen((record) {
    print('${record.level.name}: ${record.time}: ${record.message}');
  });

  DateTime date = DateTime.now().subtract(deleteOlderThan);

  log.info(
      "Deleting generations older than $date, keeping at least $numberOfGenerationsToKeep");

  Future.wait(profiles.map((profile) => Future.sync(() => profile)
      .then(_getGenerations)
      .then(_maybeParseGenerations)
      .then((generations) =>
          _maybeDeleteGenerations(generations, numberOfGenerationsToKeep, date))
      .then((generations) => generations?.map((g) => g.id))
      .then((generations) => _deleteGenerations(generations, profile))));

  log.info("finished deleting generations");
}

Future<String?> _getGenerations(String profile) async {
  log.info("getting profiles generations for $profile");

  String cmd = 'nix-env';
  List<String> args = ['--list-generations', '-p', profile];

  final result = await Process.run(cmd, args);

  log.info("`$cmd ${args.join(' ')}` returned ${result.exitCode}");

  if (result.exitCode != 0) {
    log.warning('nix-env failed: ${result.stderr}');
    return null;
  }

  return result.stdout;
}

Future<List<nix_janitor.Generation>?> _maybeParseGenerations(
    String? generationsString) async {
  if (generationsString == null) {
    return null;
  }

  return nix_janitor.parseGenerations(generationsString);
}

Future<List<nix_janitor.Generation>?> _maybeDeleteGenerations(
    List<nix_janitor.Generation>? generations, int n, DateTime date) async {
  if (generations == null) {
    return null;
  }

  return nix_janitor.generationsToDelete(generations, n, date);
}

Future<String?> _deleteGenerations(
    Iterable<int>? generations, String profile) async {
  if (generations == null) {
    return null;
  }

  if (generations.isEmpty) {
    log.info("Nothing to delete in $profile");
    return null;
  }

  List<String> generationsString =
      generations.map((e) => e.toString()).toList();

  log.info("deleting generations $generationsString for $profile");

  final cmd = 'nix-env';
  final args = [
        '--profile',
        profile,
        '--delete-generations',
      ] +
      generationsString;

  final result = await Process.run(cmd, args);
  if (result.exitCode != 0) {
    throw Exception('nix-env failed: ${result.stderr}');
  }

  log.info("`$cmd ${args.join(' ')}` returned ${result.exitCode}");
  log.finest("$profile: ${result.stderr}");

  return result.stdout;
}
