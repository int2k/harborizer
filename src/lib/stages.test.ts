import { describe, expect, it } from 'vitest';
import { applyResolverState, mapSummaryToVessels, stageFromResolver } from './stages';

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
      ['sail-17', 'under-way'],
      ['sail-18', 'setting-out'],
      ['sail-19', 'arrived'],
    ]);
  });

  it('keeps a held queued item at setting out with a held badge', () => {
    const vessels = mapSummaryToVessels(summary({
      active_children: [], decisions_open: [],
      holds: [{ id: 'held-1', title: 'Review the breakwater', reason: 'Waiting on review', source: 'child-state' }],
      queued: [{ id: 'held-1', title: 'Review the breakwater', kind: 'ship', repo: 'demo/tideglass' }],
      landed: [], endpoints: [],
    }) as never);

    expect(vessels[0]).toMatchObject({ stage: 'setting-out', wait: 'held' });
  });

  it('leaves a hold with no other evidence unknown', () => {
    const vessels = mapSummaryToVessels(summary({
      active_children: [], decisions_open: [], queued: [], landed: [], endpoints: [],
      holds: [{ id: 'held-2', title: 'Idle', reason: 'Waiting' }],
    }) as never);

    expect(vessels[0]).toMatchObject({ stage: 'unknown', wait: 'held' });
  });

  it('only maps validation states to inspection', () => {
    const stageOf = (state: string, extra: Record<string, unknown> = {}) => mapSummaryToVessels(summary({
      active_children: [{ id: 'v', kind: 'ship', state, ...extra }], decisions_open: [], queued: [], landed: [], holds: [],
    }) as never)[0];

    expect(stageOf('validating').stage).toBe('inspection');
    expect(stageOf('inspection').stage).toBe('inspection');
    expect(stageOf('paused')).toMatchObject({ stage: 'under-way', wait: 'paused' });
    expect(stageOf('blocked')).toMatchObject({ stage: 'under-way', wait: 'blocked' });
    expect(stageOf('parked')).toMatchObject({ stage: 'under-way', wait: 'held' });
    expect(stageOf('blocked', { pr_url: 'https://example.com/demo/pull/2' })).toMatchObject({ stage: 'quay', wait: 'blocked' });
  });

  it('flags a decision without changing the stage', () => {
    const vessel = mapSummaryToVessels(summary() as never).find(({ id }) => id === 'sail-17');

    expect(vessel).toMatchObject({ stage: 'under-way', decision: 'Choose the inlet boundary' });
  });

  it('shows an orphan decision as unknown', () => {
    const vessels = mapSummaryToVessels(summary({ active_children: [], queued: [], landed: [] }) as never);

    expect(vessels[0]).toMatchObject({ stage: 'unknown', decision: 'Choose the inlet boundary' });
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
    expect(stageFromResolver('blocked', '')).toBe('unknown');
  });
});

describe('applyResolverState', () => {
  const pausedVessel = () => mapSummaryToVessels(summary({
    active_children: [{ id: 'v', kind: 'ship', state: 'paused' }], decisions_open: [], queued: [], landed: [], holds: [],
  }) as never)[0];

  it('starts from a stale paused badge', () => {
    expect(pausedVessel()).toMatchObject({ wait: 'paused', stage: 'under-way' });
  });

  it('clears the stale wait badge and moves to inspection when the resolver says validating', () => {
    expect(applyResolverState(pausedVessel(), { state: 'validating', source: 'run-step' }))
      .toMatchObject({ wait: null, stage: 'inspection', state: 'validating' });
  });

  it('keeps the stage and shows the wait when the resolver reports blocked', () => {
    expect(applyResolverState(pausedVessel(), { state: 'blocked' }))
      .toMatchObject({ wait: 'blocked', stage: 'under-way' });
  });

  it('shows unknown when the resolver state has no stage evidence', () => {
    expect(applyResolverState(pausedVessel(), { state: 'mystery' }))
      .toMatchObject({ wait: null, stage: 'unknown' });
  });
});
