import type { HarborStage, HomeSummary, Vessel, VesselWait } from './model';

function stringValue(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function stageForState(value: unknown): HarborStage {
  switch (value) {
    case 'queued':
      return 'setting-out';
    case 'working':
      return 'under-way';
    case 'validating':
    case 'inspection':
      return 'inspection';
    case 'pr_open':
    case 'at_quay':
      return 'quay';
    case 'done':
    case 'merged':
    case 'arrived':
      return 'arrived';
    default:
      return 'unknown';
  }
}

export function waitForState(value: unknown): VesselWait | null {
  switch (value) {
    case 'paused':
    case 'blocked':
      return value;
    case 'parked':
      return 'held';
    default:
      return null;
  }
}

function safeHttpUrl(value: unknown): string | null {
  const candidate = stringValue(value);
  if (!candidate) return null;

  try {
    const url = new URL(candidate);
    return (url.protocol === 'https:' || url.protocol === 'http:') && !url.username && !url.password ? url.toString() : null;
  } catch {
    return null;
  }
}

export function stageFromResolver(state: string, detail: string, source = ''): HarborStage {
  const note = detail.trim();
  if (/^run passed: PR open(?:$|\b)/i.test(note) || /^checks green: PR held for merge \(ci monitor ended\)(?:$|:)/i.test(note)) return 'quay';
  if (/^run passed: PR merged(?:$|\b)/i.test(note) || /^checks green: PR merged \(ci monitor ended\)(?:$|:)/i.test(note)) return 'arrived';
  if (state === 'done') return source === 'status-log' ? 'arrived' : 'unknown';
  return stageForState(state);
}

export function mapSummaryToVessels(summary: HomeSummary): Vessel[] {
  const vessels = new Map<string, Vessel>();
  const decisions = new Map<string, string[]>();
  const sourcePriority = new Map<string, number>();
  const priorityFor = { queued: 1, active: 2, hold: 3, landed: 4 } as const;

  for (const decision of summary.decisions_open ?? []) {
    const id = stringValue(decision.id);
    const text = [stringValue(decision.summary), stringValue(decision.reason)].filter(Boolean).join(' — ');
    if (id && text) decisions.set(id, [...(decisions.get(id) ?? []), text]);
  }

  const upsert = (row: Record<string, unknown>, source: 'active' | 'hold' | 'queued' | 'landed') => {
    const id = stringValue(row.id);
    if (!id) return;

    const existing = vessels.get(id);
    const currentPriority = sourcePriority.get(id) ?? 0;
    const nextPriority = priorityFor[source];
    const preferRow = nextPriority >= currentPriority;
    const rowState = stringValue(row.state) ?? (source === 'queued' ? 'queued' : source === 'landed' ? 'arrived' : 'unknown');
    const state = preferRow ? rowState : existing?.state ?? rowState;
    const decision = decisions.get(id)?.join('\n') ?? existing?.decision ?? null;
    const wait = source === 'landed' ? null : waitForState(rowState) ?? (source === 'hold' ? 'held' : null);
    const hasPr = Boolean(safeHttpUrl(row.pr_url) ?? existing?.prUrl);
    const evidenceStage: HarborStage = source === 'landed'
      ? 'arrived'
      : source === 'queued'
        ? 'setting-out'
        : wait
          ? hasPr ? 'quay' : source === 'active' ? 'under-way' : 'unknown'
          : stageForState(rowState);
    const candidateStage = evidenceStage === 'unknown' && wait ? existing?.stage ?? 'unknown' : evidenceStage;
    const stage = preferRow || existing?.stage === 'unknown' ? candidateStage : existing?.stage ?? candidateStage;
    sourcePriority.set(id, Math.max(currentPriority, nextPriority));

    const prUrl = safeHttpUrl(row.pr_url) ?? existing?.prUrl ?? null;
    vessels.set(id, {
      id,
      title: stringValue(row.name) ?? stringValue(row.title) ?? existing?.title ?? id,
      project: stringValue(row.repo) ?? existing?.project ?? 'Unknown project',
      kind: stringValue(row.kind) ?? existing?.kind ?? 'unknown',
      stage,
      state,
      wait: wait ?? existing?.wait ?? null,
      detail: (preferRow ? stringValue(row.doing) ?? stringValue(row.reason) : null) ?? existing?.detail ?? stringValue(row.doing) ?? stringValue(row.reason) ?? '',
      decision,
      since: stringValue(row.since) ?? stringValue((row.completion as Record<string, unknown> | undefined)?.date) ?? existing?.since ?? null,
      prUrl,
      hasReport: Boolean(stringValue(row.report_path) ?? stringValue(row.report_url) ?? existing?.hasReport),
    });
  };

  for (const row of summary.active_children ?? []) upsert(row, 'active');
  for (const row of summary.holds ?? []) upsert(row, 'hold');
  for (const row of summary.queued ?? []) upsert(row, 'queued');
  for (const row of summary.landed ?? []) upsert(row, 'landed');

  for (const [id, decisionText] of decisions) {
    if (vessels.has(id)) continue;
    vessels.set(id, {
      id,
      title: id,
      project: 'Unknown project',
      kind: 'unknown',
      stage: 'unknown',
      state: 'unknown',
      wait: null,
      detail: '',
      decision: decisionText.join('\n'),
      since: null,
      prUrl: null,
      hasReport: false,
    });
  }

  return [...vessels.values()].sort((a, b) => a.id.localeCompare(b.id));
}
