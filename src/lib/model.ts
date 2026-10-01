export interface HomeSummary {
  schema: string;
  hold_classifier_schema: string;
  generated: string;
  generated_epoch: number;
  home: string;
  valid: boolean;
  state: string;
  reason?: string | null;
  invalidity?: { kind?: string | null; ids?: string[]; reason?: string | null };
  active_children: Record<string, unknown>[];
  decisions_open: Record<string, unknown>[];
  holds: Record<string, unknown>[];
  queued: Record<string, unknown>[];
  landed: Record<string, unknown>[];
  endpoints: Record<string, unknown>[];
  counts: Record<string, number>;
  omitted: { surface: string; count: number }[];
}

export type HarborStage = 'setting-out' | 'under-way' | 'inspection' | 'quay' | 'arrived' | 'unknown';

export interface Vessel {
  id: string;
  title: string;
  project: string;
  kind: string;
  stage: HarborStage;
  state: string;
  detail: string;
  decision: string | null;
  since: string | null;
  prUrl: string | null;
  hasReport: boolean;
}

export type Freshness = 'fresh' | 'stale' | 'invalid' | 'unsupported' | 'clock-skew' | 'missing';
