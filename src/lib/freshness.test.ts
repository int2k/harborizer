import { describe, expect, it } from 'vitest';
import { classifyFreshness } from './freshness';

const validSummary = (generatedEpoch: number, overrides: Record<string, unknown> = {}) => ({
  schema: 'fm-secondmate-home-summary.v1',
  hold_classifier_schema: 'fm-captain-hold-buckets.v1',
  generated: new Date(generatedEpoch * 1000).toISOString(),
  generated_epoch: generatedEpoch,
  home: '/example/firstmate',
  valid: true,
  omitted: [],
  ...overrides,
});

describe('classifyFreshness', () => {
  it('marks a recent valid summary fresh', () => {
    expect(classifyFreshness(validSummary(1_000) as never, 1_030)).toBe('fresh');
  });

  it('marks an older summary stale', () => {
    expect(classifyFreshness(validSummary(1_000) as never, 1_200, 90)).toBe('stale');
  });

  it('distinguishes invalid and unsupported summaries', () => {
    expect(classifyFreshness(validSummary(1_000, { valid: false }) as never, 1_010)).toBe('invalid');
    expect(classifyFreshness(validSummary(1_000, { schema: 'fm-secondmate-home-summary.v9' }) as never, 1_010)).toBe('unsupported');
  });
});
