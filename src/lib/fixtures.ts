import type { HomeSummary } from './model';

const now = Math.floor(Date.now() / 1000);

function buildSummary(overrides: Partial<HomeSummary> = {}): HomeSummary {
  const active_children = [
    { id: 'demo-sail-17', kind: 'ship', state: 'working', repo: 'demo/tideglass', name: 'Map the inlet', source: 'run-step', doing: 'Checking the western channel' },
    { id: 'demo-scout-4', kind: 'scout', state: 'working', repo: 'demo/bluewater', name: 'Survey the east shoal', source: 'pane', doing: 'Comparing two sample routes' },
  ];
  const decisions_open = [{ id: 'demo-sail-22', key: 'scope', verb: 'needs-decision', summary: 'Choose the harbor boundary', reason: 'Two safe approaches remain', source: 'status' }];
  const holds = [{ id: 'demo-sail-22', title: 'Set the breakwater line', reason: 'Captain review requested', source: 'child-state' }];
  const queued = [{ id: 'demo-sail-23', title: 'Sound the south channel', kind: 'ship', repo: 'demo/tideglass', since: '2026-09-30' }];
  const landed = [{ id: 'demo-scout-2', title: 'Chart the north cove', kind: 'scout', repo: 'demo/bluewater', report_path: 'data/demo-scout-2/report.md', pr_url: 'https://example.com/demo/bluewater/pull/12', completion: { date: '2026-09-29' } }];
  return {
    schema: 'fm-secondmate-home-summary.v1',
    hold_classifier_schema: 'fm-captain-hold-buckets.v1',
    generated: new Date(now * 1000).toISOString(),
    generated_epoch: now,
    home: '/example/firstmate-home',
    valid: true,
    reason: null,
    invalidity: { kind: null, ids: [] },
    state: 'active_child_work',
    active_children,
    decisions_open,
    holds,
    queued,
    landed,
    endpoints: [
      { id: 'demo-sail-17', state: 'working', source: 'run-step', endpoint: { exists: true } },
      { id: 'demo-scout-4', state: 'working', source: 'pane', endpoint: { exists: true } },
    ],
    counts: { active_children: 2, decisions_open: 1, holds: 1, queued: 1, landed: 1, endpoints: 4 },
    omitted: [{ surface: 'endpoints', count: 2 }],
    ...overrides,
  };
}

export type FixtureName = 'busy' | 'empty' | 'stale' | 'invalid' | 'decision';

export const fixtures: Record<FixtureName, HomeSummary> = {
  busy: buildSummary(),
  empty: buildSummary({
    state: 'no_active_work',
    active_children: [], decisions_open: [], holds: [], queued: [], landed: [], endpoints: [],
    counts: { active_children: 0, decisions_open: 0, holds: 0, queued: 0, landed: 0, endpoints: 0 },
    omitted: [],
  }),
  stale: buildSummary({ generated: '2026-09-30T10:00:00Z', generated_epoch: now - 7200 }),
  invalid: buildSummary({ valid: false, state: 'unknown', reason: 'Synthetic inventory mismatch in this fixture' }),
  decision: buildSummary({
    active_children: [],
    queued: [],
    landed: [],
    decisions_open: [{ id: 'demo-sail-22', key: 'scope', verb: 'needs-decision', summary: 'Choose the harbor boundary', reason: 'The east jetty conflicts with the safe channel', source: 'status' }],
    holds: [{ id: 'demo-sail-22', title: 'Set the breakwater line', reason: 'Captain review requested', source: 'child-state' }],
    counts: { active_children: 0, decisions_open: 1, holds: 1, queued: 0, landed: 0, endpoints: 0 },
    omitted: [],
    state: 'captain_decision',
  }),
};
