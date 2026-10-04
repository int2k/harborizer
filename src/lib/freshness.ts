import type { Freshness, HomeSummary } from './model';

export const SUMMARY_SCHEMA = 'fm-secondmate-home-summary.v1';
export const HOLD_CLASSIFIER_SCHEMA = 'fm-captain-hold-buckets.v1';
export const STALE_AFTER_SECONDS = 90;

export function classifyFreshness(
  summary: HomeSummary | null | undefined,
  nowSeconds = Date.now() / 1000,
  staleAfterSeconds = STALE_AFTER_SECONDS,
): Freshness {
  if (!summary) return 'missing';
  if (summary.schema !== SUMMARY_SCHEMA || summary.hold_classifier_schema !== HOLD_CLASSIFIER_SCHEMA) return 'unsupported';
  if (summary.valid !== true) return 'invalid';
  if (!Number.isFinite(summary.generated_epoch) || summary.generated_epoch < 0) return 'invalid';

  const age = nowSeconds - summary.generated_epoch;
  if (age < -30) return 'clock-skew';
  return age > staleAfterSeconds ? 'stale' : 'fresh';
}
