import 'package:test/test.dart';

import 'package:nix_janitor/nix_janitor.dart' as janitor;

final String generationString = """ 661   2023-06-01 08:10:47   
 662   2023-06-05 21:35:55   
 663   2023-06-06 13:17:20   
 664   2023-06-06 18:29:49   
 665   2023-06-07 07:57:08   
 666   2023-06-08 07:42:25   
 667   2023-06-13 22:13:13   
 668   2023-06-14 09:03:01   
 669   2023-06-15 12:21:00   
 670   2023-06-16 09:59:25   
 671   2023-06-19 18:54:32   
 672   2023-06-20 07:09:24   
 673   2023-07-03 08:56:50   
 674   2023-07-05 18:26:11   
 675   2023-07-10 08:56:27   
 676   2023-07-12 23:32:24   
 677   2023-07-13 12:55:34   
 678   2023-07-14 11:46:59   
 679   2023-07-15 10:32:58   
 680   2023-07-15 22:40:41   
 681   2023-07-16 11:35:46
""";

final String generationStringWithCurrent = "$generationString   (current)";

final List<janitor.Generation> generationList = [
  janitor.Generation(661, DateTime(2023, 6, 1, 8, 10, 47)),
  janitor.Generation(662, DateTime(2023, 6, 5, 21, 35, 55)),
  janitor.Generation(663, DateTime(2023, 6, 6, 13, 17, 20)),
  janitor.Generation(664, DateTime(2023, 6, 6, 18, 29, 49)),
  janitor.Generation(665, DateTime(2023, 6, 7, 7, 57, 8)),
  janitor.Generation(666, DateTime(2023, 6, 8, 7, 42, 25)),
  janitor.Generation(667, DateTime(2023, 6, 13, 22, 13, 13)),
  janitor.Generation(668, DateTime(2023, 6, 14, 9, 3, 1)),
  janitor.Generation(669, DateTime(2023, 6, 15, 12, 21, 0)),
  janitor.Generation(670, DateTime(2023, 6, 16, 9, 59, 25)),
  janitor.Generation(671, DateTime(2023, 6, 19, 18, 54, 32)),
  janitor.Generation(672, DateTime(2023, 6, 20, 7, 9, 24)),
  janitor.Generation(673, DateTime(2023, 7, 3, 8, 56, 50)),
  janitor.Generation(674, DateTime(2023, 7, 5, 18, 26, 11)),
  janitor.Generation(675, DateTime(2023, 7, 10, 8, 56, 27)),
  janitor.Generation(676, DateTime(2023, 7, 12, 23, 32, 24)),
  janitor.Generation(677, DateTime(2023, 7, 13, 12, 55, 34)),
  janitor.Generation(678, DateTime(2023, 7, 14, 11, 46, 59)),
  janitor.Generation(679, DateTime(2023, 7, 15, 10, 32, 58)),
  janitor.Generation(680, DateTime(2023, 7, 15, 22, 40, 41)),
  janitor.Generation(681, DateTime(2023, 7, 16, 11, 35, 46))
];

void main() {
  group('parseGenerations', () {
    test('empty', () {
      expect(janitor.parseGenerations(''), []);
    });

    test('single', () {
      expect(janitor.parseGenerations(' 681   2023-07-16 11:35:46'),
          [janitor.Generation(681, DateTime(2023, 7, 16, 11, 35, 46))]);
    });

    test('multiple', () {
      expect(janitor.parseGenerations(generationString), generationList);
    });

    test('single with current', () {
      expect(janitor.parseGenerations(' 681   2023-07-16 11:35:46   (current)'),
          [janitor.Generation(681, DateTime(2023, 7, 16, 11, 35, 46))]);
    });

    test('multiple with current', () {
      expect(janitor.parseGenerations(generationStringWithCurrent),
          generationList);
    });
  });

  group('getLastNGenerations', () {
    int l = generationList.length;
    for (int n in [1, 5, 10, l, l + 1, l + 10]) {
      test('N = $n', () {
        List<janitor.Generation> expected =
            generationList.reversed.take(n).toList().reversed.toList();

        expect(janitor.getLastNGenerations(generationList, n), expected);
      });
    }
  });

  group('getActiveOnOrAfter', () {
    test('June 1st', () {
      expect(janitor.getActiveOnOrAfter(generationList, DateTime(2023, 6, 1)),
          generationList);
    });

    test('June 10th', () {
      expect(janitor.getActiveOnOrAfter(generationList, DateTime(2023, 6, 10)),
          generationList.sublist(5));
    });

    test('June 20th', () {
      expect(janitor.getActiveOnOrAfter(generationList, DateTime(2023, 6, 20)),
          generationList.sublist(10));
    });

    test('July 1st', () {
      expect(janitor.getActiveOnOrAfter(generationList, DateTime(2023, 7, 1)),
          generationList.sublist(11));
    });

    test('July 15th, Noon', () {
      expect(
          janitor.getActiveOnOrAfter(
              generationList, DateTime(2023, 7, 15, 12, 0)),
          generationList.sublist(18));
    });
  });

  group('generationsToDelete', () {
    test('N = 1; June 1st', () {
      expect(
          janitor.generationsToDelete(generationList, 1, DateTime(2023, 6, 1)),
          <janitor.Generation>[]);
    });

    int l = generationList.length;

    Map<int, Map<DateTime, List<janitor.Generation>>> genLists = {
      1: {
        DateTime(2023, 6, 1): <janitor.Generation>[],
        DateTime(2023, 7, 1): generationList.sublist(0,11),
        DateTime(2023, 7, 15, 12, 0): generationList.sublist(0,18)
      },
      5: {
        DateTime(2023, 6, 1): <janitor.Generation>[],
        DateTime(2023, 7, 1): generationList.sublist(0,11),
        DateTime(2023, 7, 15, 12, 0): generationList.sublist(0,16)
      },
      10: {
        DateTime(2023, 6, 1): <janitor.Generation>[],
        DateTime(2023, 7, 1): generationList.sublist(0, 11),
        DateTime(2023, 7, 15, 12, 0): generationList.sublist(0,11)
      },
      l: {
        DateTime(2023, 6, 1): <janitor.Generation>[],
        DateTime(2023, 7, 1): <janitor.Generation>[],
        DateTime(2023, 7, 15, 12, 0): <janitor.Generation>[]
      },
      l + 1: {
        DateTime(2023, 6, 1): <janitor.Generation>[],
        DateTime(2023, 7, 1): <janitor.Generation>[],
        DateTime(2023, 7, 15, 12, 0): <janitor.Generation>[]
      },
      l + 10: {
        DateTime(2023, 6, 1): <janitor.Generation>[],
        DateTime(2023, 7, 1): <janitor.Generation>[],
        DateTime(2023, 7, 15, 12, 0): <janitor.Generation>[]
      },
    };

    for (int n in [1, 5, 10, l, l + 1, l + 10]) {
      for (DateTime date in [
        DateTime(2023, 6, 1),
        DateTime(2023, 7, 1),
        DateTime(2023, 7, 15, 12, 0)
      ]) {
        test("N = $n; date = $date", () {
          List<janitor.Generation> expected = genLists[n]![date]!;

          expect(janitor.generationsToDelete(generationList, n, date), expected);
        });
      }
    }
  });
}
