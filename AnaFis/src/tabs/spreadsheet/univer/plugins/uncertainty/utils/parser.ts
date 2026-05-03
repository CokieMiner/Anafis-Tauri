import type { ParsedUncertainty } from '@/tabs/spreadsheet/univer/plugins/uncertainty/types';

// Non-capturing group for numbers to simplify outer capture groups
const NUM_PART = /[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?/;

const DELIMITERS = /(?:\+-|\+\/-|-\+|-?\/\+|±)/;

export function parseUncertaintyInput(
  input: string,
  fallbackNominal?: number
): ParsedUncertainty | null {
  const str = normalizeUncertaintyInput(input);

  // 1. Asymmetric Bounds (+upper/-lower OR -lower/+upper)
  const asymMatch1 = new RegExp(
    `^(${NUM_PART.source})\\s*\\+\\s*(${NUM_PART.source})(%?)\\s*\\/\\s*-\\s*(${NUM_PART.source})(%?)$`
  ).exec(str);
  const asymMatch2 = new RegExp(
    `^(${NUM_PART.source})\\s*-\\s*(${NUM_PART.source})(%?)\\s*\\/\\s*\\+\\s*(${NUM_PART.source})(%?)$`
  ).exec(str);
  const adjacentAsymMatch1 = new RegExp(
    `^(${NUM_PART.source})\\s*\\+\\s*(${NUM_PART.source})(%?)\\s*-\\s*(${NUM_PART.source})(%?)$`
  ).exec(str);
  const adjacentAsymMatch2 = new RegExp(
    `^(${NUM_PART.source})\\s*-\\s*(${NUM_PART.source})(%?)\\s*\\+\\s*(${NUM_PART.source})(%?)$`
  ).exec(str);

  if (asymMatch1 || asymMatch2 || adjacentAsymMatch1 || adjacentAsymMatch2) {
    const match = (asymMatch1 ||
      asymMatch2 ||
      adjacentAsymMatch1 ||
      adjacentAsymMatch2) as RegExpExecArray;
    const nominal = parseFloat(match[1] as string);

    const isFirstFormat = !!(asymMatch1 || adjacentAsymMatch1);
    // Bound 1: upper in format 1 (+u/-l), lower in format 2 (-l/+u)
    const b1Val = Math.abs(parseFloat(match[2] as string));
    const b1IsPercent = match[3] === '%';

    // Bound 2: lower in format 1, upper in format 2
    const b2Val = Math.abs(parseFloat(match[4] as string));
    const b2IsPercent = match[5] === '%';

    const upperBound = isFirstFormat ? b1Val : b2Val;
    const upperType = (isFirstFormat ? b1IsPercent : b2IsPercent)
      ? 'rel'
      : 'abs';

    const lowerBound = isFirstFormat ? b2Val : b1Val;
    const lowerType = (isFirstFormat ? b2IsPercent : b1IsPercent)
      ? 'rel'
      : 'abs';

    if (
      !Number.isNaN(nominal) &&
      !Number.isNaN(upperBound) &&
      !Number.isNaN(lowerBound)
    ) {
      return {
        nominal,
        metadata: {
          upperBound,
          lowerBound,
          upperType,
          lowerType,
          upperSource: 'manual',
          lowerSource: 'manual',
        },
      };
    }
  }

  const boundsOnlyAsymMatch1 =
    fallbackNominal !== undefined
      ? new RegExp(
          `^\\+\\s*(${NUM_PART.source})(%?)\\s*\\/\\s*-\\s*(${NUM_PART.source})(%?)$`
        ).exec(str)
      : null;
  const boundsOnlyAsymMatch2 =
    fallbackNominal !== undefined
      ? new RegExp(
          `^-\\s*(${NUM_PART.source})(%?)\\s*\\/\\s*\\+\\s*(${NUM_PART.source})(%?)$`
        ).exec(str)
      : null;
  const boundsOnlyAdjacentAsymMatch1 =
    fallbackNominal !== undefined
      ? new RegExp(
          `^\\+\\s*(${NUM_PART.source})(%?)\\s*-\\s*(${NUM_PART.source})(%?)$`
        ).exec(str)
      : null;
  const boundsOnlyAdjacentAsymMatch2 =
    fallbackNominal !== undefined
      ? new RegExp(
          `^-\\s*(${NUM_PART.source})(%?)\\s*\\+\\s*(${NUM_PART.source})(%?)$`
        ).exec(str)
      : null;

  if (
    boundsOnlyAsymMatch1 ||
    boundsOnlyAsymMatch2 ||
    boundsOnlyAdjacentAsymMatch1 ||
    boundsOnlyAdjacentAsymMatch2
  ) {
    const match = (boundsOnlyAsymMatch1 ||
      boundsOnlyAsymMatch2 ||
      boundsOnlyAdjacentAsymMatch1 ||
      boundsOnlyAdjacentAsymMatch2) as RegExpExecArray;
    const isFirstFormat = !!(
      boundsOnlyAsymMatch1 || boundsOnlyAdjacentAsymMatch1
    );
    const b1Val = Math.abs(parseFloat(match[1] as string));
    const b1IsPercent = match[2] === '%';
    const b2Val = Math.abs(parseFloat(match[3] as string));
    const b2IsPercent = match[4] === '%';

    const upperBound = isFirstFormat ? b1Val : b2Val;
    const upperType = (isFirstFormat ? b1IsPercent : b2IsPercent)
      ? 'rel'
      : 'abs';
    const lowerBound = isFirstFormat ? b2Val : b1Val;
    const lowerType = (isFirstFormat ? b2IsPercent : b1IsPercent)
      ? 'rel'
      : 'abs';

    if (!Number.isNaN(upperBound) && !Number.isNaN(lowerBound)) {
      return {
        nominal: fallbackNominal as number,
        metadata: {
          upperBound,
          lowerBound,
          upperType,
          lowerType,
          upperSource: 'manual',
          lowerSource: 'manual',
        },
      };
    }
  }

  // 2. Standard Delimiters (e.g., 5 ± 0.1 or 5 ± 2%)
  const stdMatch = new RegExp(
    `^(${NUM_PART.source})\\s*${DELIMITERS.source}\\s*(${NUM_PART.source})(%?)$`
  ).exec(str);

  if (stdMatch?.[1] && stdMatch[2]) {
    const nominal = parseFloat(stdMatch[1]);
    const isPercent = stdMatch[3] === '%';
    const rawErr = parseFloat(stdMatch[2]);

    if (!Number.isNaN(nominal) && !Number.isNaN(rawErr)) {
      return {
        nominal,
        metadata: {
          upperBound: Math.abs(rawErr),
          upperType: isPercent ? 'rel' : 'abs',
          upperSource: 'manual',
        },
      };
    }
  }

  const boundsOnlyStdMatch =
    fallbackNominal !== undefined
      ? new RegExp(`^${DELIMITERS.source}\\s*(${NUM_PART.source})(%?)$`).exec(
          str
        )
      : null;

  if (boundsOnlyStdMatch?.[1]) {
    const rawErr = parseFloat(boundsOnlyStdMatch[1]);

    if (!Number.isNaN(rawErr)) {
      return {
        nominal: fallbackNominal as number,
        metadata: {
          upperBound: Math.abs(rawErr),
          upperType: boundsOnlyStdMatch[2] === '%' ? 'rel' : 'abs',
          upperSource: 'manual',
        },
      };
    }
  }

  // 3. Concise Scientific Notation (e.g. 5.00(12) or 5(1))
  const conciseMatch = /^([-+]?\d+(?:\.\d+)?)\s*\((\d+)\)$/.exec(str);
  if (conciseMatch?.[1] && conciseMatch[2]) {
    const nominalStr = conciseMatch[1];
    const errStr = conciseMatch[2];

    const nominal = parseFloat(nominalStr);

    const decimalPlaces = nominalStr.split('.')[1]?.length ?? 0;
    const errVal = parseFloat(errStr) * 10 ** -decimalPlaces;

    if (!Number.isNaN(nominal) && !Number.isNaN(errVal)) {
      return {
        nominal,
        metadata: {
          upperBound: errVal,
          upperType: 'abs',
          upperSource: 'manual',
        },
      };
    }
  }

  return null;
}

function normalizeUncertaintyInput(input: string): string {
  return input
    .trim()
    .replace(/\u2212/g, '-')
    .replace(/\u2013/g, '-')
    .replace(/\u2014/g, '-')
    .replace(/\uFF0B/g, '+')
    .replace(/\uFF05/g, '%');
}
