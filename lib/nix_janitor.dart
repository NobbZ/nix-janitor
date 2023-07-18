class Generation {
  int id;
  DateTime date;
  Generation(this.id, this.date);

  @override
  String toString() {
    return 'Generation{id: $id, date: $date}';
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Generation &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          date == other.date;

  @override
  int get hashCode => Object.hash(id, date);
}

List<Generation> parseGenerations(String generationList) {
  final RegExp generationPattern = RegExp(
      r'^\s*(\d+)\s+(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\s*(\(current\))?\s*$');
  return generationList
      .split('\n')
      .map((String line) {
        final Match? match = generationPattern.firstMatch(line);
        final String? idStr = match?.group(1);
        final String? dateStr = match?.group(2);

        if (idStr == null || dateStr == null) {
          return null;
        }

        return Generation(int.parse(idStr), DateTime.parse(dateStr));
      })
      .whereType<Generation>()
      .toList();
}

List<Generation> getLastNGenerations(List<Generation> generations, int n) {
  generations.sort((l, r) => l.date.compareTo(r.date));
  return generations.reversed.take(n).toList().reversed.toList();
}

List<Generation> getActiveOnOrAfter(
    List<Generation> generations, DateTime date) {
  generations.sort((l, r) => l.date.compareTo(r.date));

  List<Generation> older =
      generations.where((Generation g) => g.date.compareTo(date) <= 0).toList();
  List<Generation> newer =
      generations.where((Generation g) => g.date.compareTo(date) > 0).toList();

  return (older.isNotEmpty ? [older.last] : <Generation>[]) + newer;
}

List<Generation> generationsToDelete(List<Generation> generations, int n, DateTime date) {
  Set<int> byDate = Set.of(getActiveOnOrAfter(generations, date).map((g) => g.id));
  Set<int> byN = Set.of(getLastNGenerations(generations, n).map((g) => g.id));

  Set<int> toKeep = byDate.union(byN);

  return generations.where((g) => !toKeep.contains(g.id)).toList();
}