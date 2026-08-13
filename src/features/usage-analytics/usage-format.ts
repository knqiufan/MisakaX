export function parseTokenDecimal(value: string): bigint {
  if (!/^\d+$/.test(value)) throw new Error("Token totals must be unsigned decimal strings");
  return BigInt(value);
}

export function formatCompactTokens(value: string): string {
  const tokens = parseTokenDecimal(value);
  const units = [
    { threshold: 1_000_000_000n, suffix: "B" },
    { threshold: 1_000_000n, suffix: "M" },
    { threshold: 1_000n, suffix: "K" },
  ] as const;
  const unit = units.find(({ threshold }) => tokens >= threshold);
  if (!unit) return tokens.toString();

  const tenths = (tokens * 10n) / unit.threshold;
  const whole = tenths / 10n;
  const decimal = tenths % 10n;
  return decimal === 0n
    ? `${whole}${unit.suffix}`
    : `${whole}.${decimal}${unit.suffix}`;
}

export function formatFullTokens(value: string, locale: string): string {
  return new Intl.NumberFormat(locale).format(parseTokenDecimal(value));
}
