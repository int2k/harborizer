import { describe, expect, it } from 'vitest';
import { mapSummaryToVessels, stageFromResolver } from './stages';

const summary = (overrides: Record<string, unknown> = {}) => ({
  schema: 'fm-secondmate-home-summary.v1',
  hold_classifier_schema: 'fm-captain-hold-buckets.v1',
  generated: '2026-09-30T12:00:00Z',
  generated_epoch: 1790770_000,
  home: '/example/firstmate',
  valid: true,
  reason: null,
  invalidity: { kind: null, ids: [] },
  state: 'active_child_work',
  active_children: [{ id: 'sail-17', kind: 'ship', state: 'working', repo: 'sample/repo', name: 'Map the inlet', doing: 'Inspecting chart data' }],
  decisions_open: [{ id: 'sail-17', key: 'scope', verb: 'needs-decision', summary: 'Choose the inlet boundary', reason: null, source: 'status' }],
  holds: [],
  queued: [{ id: 'sail-18', title: 'Sound the channel', kind: 'ship', repo: 'sample/repo', since: '2026-09-30' }],
  landed: [{ id: 'sail-19', title: 'Chart the north cove', kind: 'scout', report_path: 'data/sail-19/report.md', pr_url: 'https://example.com/int2k/chart/pull/9', completion: { date: '2026-09-29' } }],
  endpoints: [{ id: 'sail-17', state: 'working', source: 'run-step', endpoint: { exists: true } }],
  counts: { active_children: 1, decisions_open: 1, holds: 0, queued: 1, landed: 1, endpoints: 1 },
  omitted: [],
  ...overrides,
});

describe('mapSummaryToVessels', () => {
  it('maps active, queued and landed rows to their known harbor stages', () => {
    const vessels = mapSummaryToVessels(summary() as never);

    expect(vessels.map(({ id, stage }) => [id, stage])).toEqual([
      ['sail-17', 'inspection'],
      ['sail-18', 'setting-out'],
      ['sail-19', 'arrived'],
    ]);
  });

  it('keeps a held queued item in inspection instead of resetting it to setting out', () => {
    const vessels = mapSummaryToVessels(summary({
      active_children: [], decisions_open: [],
      holds: [{ id: 'held-1', title: 'Review the breakwater', reason: 'Waiting on review', source: 'child-state' }],
      queued: [{ id: 'held-1', title: 'Review the breakwater', kind: 'ship', repo: 'demo/tideglass' }],
      landed: [], endpoints: [],
    }) as never);

    expect(vessels[0]?.stage).toBe('inspection');
  });

  it('marks a scout report without inventing a current state', () => {
    const scout = mapSummaryToVessels(summary() as never).find(({ id }) => id === 'sail-19');

    expect(scout).toMatchObject({ kind: 'scout', hasReport: true, stage: 'arrived' });
  });

  it('does not expose PR links containing credentials', () => {
    const vessels = mapSummaryToVessels(summary({
      active_children: [], decisions_open: [], holds: [], queued: [], endpoints: [],
      landed: [{ id: 'unsafe-pr', title: 'Synthetic row', kind: 'ship', pr_url: 'https://captain:secret@example.com/demo/pull/1' }],
    }) as never);

    expect(vessels[0]?.prUrl).toBeNull();
  });

  it('keeps an unrecognized row state unknown', () => {
    const vessels = mapSummaryToVessels(summary({
      active_children: [{ id: 'mystery-1', kind: 'ship', state: 'drifting', repo: null, name: 'Uncharted work', doing: '' }],
      decisions_open: [],
      queued: [],
      landed: [],
    }) as never);

    expect(vessels[0]?.stage).toBe('unknown');
  });
});


describe('stageFromResolver', () => {
  it('keeps a checks-green open pull request at the quay', () => {
    expect(stageFromResolver('done', 'checks green: PR held for merge (ci monitor ended): https://example.com/demo/repo/pull/4')).toBe('quay');
  });

  it('only calls completed run-step work arrived when merge evidence is positive', () => {
    expect(stageFromResolver('done', 'run passed: PR state unknown (no PR identity)', 'run-step')).toBe('unknown');
    expect(stageFromResolver('done', 'verified completion', 'status-log')).toBe('arrived');
  });

  it('leaves unknown resolver states unknown', () => {
    expect(stageFromResolver('mystery', 'unrecognized result')).toBe('unknown');
  });
});
