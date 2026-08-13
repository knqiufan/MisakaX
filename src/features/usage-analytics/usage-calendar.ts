import type { DailyUsageV1 } from "@/lib/ipc/types";

export type UsageHeatLevel = 0 | 1 | 2 | 3 | 4;
export type ActivityVisualState =
  | "none"
  | "known"
  | "unknown"
  | "mixed"
  | "future";

export interface ActivityCalendarCell {
  day: DailyUsageV1;
  date: Date;
  slotIndex: number;
  rowIndex: number;
  weekIndex: number;
  heatLevel: UsageHeatLevel;
  visualState: ActivityVisualState;
}

export interface ActivityMonthLabel {
  monthKey: string;
  date: Date;
  weekIndex: number;
}

export interface ActivityCalendarGrid {
  cells: Array<ActivityCalendarCell | null>;
  days: ActivityCalendarCell[];
  monthLabels: ActivityMonthLabel[];
  weekCount: number;
  weekStart: 0 | 1;
}

export function parseLocalDate(value: string): Date {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) throw new Error(`Invalid local date: ${value}`);
  const date = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3])));
  if (date.toISOString().slice(0, 10) !== value) {
    throw new Error(`Invalid local date: ${value}`);
  }
  return date;
}

export function relativeWeekday(date: Date, weekStart: 0 | 1): number {
  return (date.getUTCDay() - weekStart + 7) % 7;
}

function percentile95Cap(values: bigint[]): bigint {
  if (values.length === 0) return 0n;
  const sorted = [...values].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  const index = Math.floor((sorted.length - 1) * 0.95);
  return sorted[index];
}

export function calculateUsageHeatLevels(days: DailyUsageV1[]): Map<string, UsageHeatLevel> {
  const knownActive = days.filter(
    (day) => day.operation_count > 0 && day.total_tokens !== null
  );
  const positiveValues = knownActive
    .map((day) => BigInt(day.total_tokens ?? "0"))
    .filter((value) => value > 0n);
  const cap = percentile95Cap(positiveValues);
  const levels = new Map<string, UsageHeatLevel>();

  for (const day of knownActive) {
    const value = BigInt(day.total_tokens ?? "0");
    if (value === 0n || cap === 0n) {
      levels.set(day.local_date, 1);
      continue;
    }
    const capped = value > cap ? cap : value;
    const ratio = Math.log1p(Number(capped)) / Math.log1p(Number(cap));
    const level = Math.max(1, Math.min(4, Math.ceil(ratio * 4))) as UsageHeatLevel;
    levels.set(day.local_date, level);
  }
  return levels;
}

function compareDates(left: Date, right: Date): number {
  return left.getTime() - right.getTime();
}

export function buildActivityCalendar(
  inputDays: DailyUsageV1[],
  weekStart: 0 | 1,
  today = new Date()
): ActivityCalendarGrid {
  if (inputDays.length === 0) {
    return { cells: [], days: [], monthLabels: [], weekCount: 0, weekStart };
  }

  const normalizedToday = new Date(
    Date.UTC(today.getUTCFullYear(), today.getUTCMonth(), today.getUTCDate())
  );
  const sorted = inputDays
    .map((day) => ({ day, date: parseLocalDate(day.local_date) }))
    .sort((left, right) => compareDates(left.date, right.date));
  const heatLevels = calculateUsageHeatLevels(sorted.map(({ day }) => day));
  const leadingSlots = relativeWeekday(sorted[0].date, weekStart);
  const slotCount = Math.ceil((leadingSlots + sorted.length) / 7) * 7;
  const cells: Array<ActivityCalendarCell | null> = Array.from({ length: slotCount }, () => null);
  const calendarDays: ActivityCalendarCell[] = [];

  sorted.forEach(({ day, date }, dayIndex) => {
    const slotIndex = leadingSlots + dayIndex;
    const isFuture = compareDates(date, normalizedToday) > 0;
    const hasActivity = day.operation_count > 0;
    const hasKnownTotal = day.total_tokens !== null;
    const hasUnknown = day.quality.unknown_operation_count > 0;
    let visualState: ActivityVisualState = "none";
    if (isFuture) visualState = "future";
    else if (hasActivity && !hasKnownTotal) visualState = "unknown";
    else if (hasActivity && hasKnownTotal && hasUnknown) visualState = "mixed";
    else if (hasActivity) visualState = "known";

    const cell: ActivityCalendarCell = {
      day,
      date,
      slotIndex,
      rowIndex: slotIndex % 7,
      weekIndex: Math.floor(slotIndex / 7),
      heatLevel: heatLevels.get(day.local_date) ?? 0,
      visualState,
    };
    cells[slotIndex] = cell;
    calendarDays.push(cell);
  });

  const monthByWeek = new Map<number, ActivityMonthLabel>();
  for (const cell of calendarDays) {
    const monthKey = cell.day.local_date.slice(0, 7);
    const existingMonth = Array.from(monthByWeek.values()).find(
      (label) => label.monthKey === monthKey
    );
    if (!existingMonth) {
      // If a range starts at month end, prefer the new month label sharing its week.
      monthByWeek.set(cell.weekIndex, { monthKey, date: cell.date, weekIndex: cell.weekIndex });
    }
  }

  return {
    cells,
    days: calendarDays,
    monthLabels: Array.from(monthByWeek.values()).sort(
      (left, right) => left.weekIndex - right.weekIndex
    ),
    weekCount: slotCount / 7,
    weekStart,
  };
}
