import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { HomeSummary } from './model';

export interface ResolverDetail {
  state: string;
  source: string;
  detail: string | null;
  pr_url: string | null;
  unknown: boolean;
}

export interface LastWakeEvent {
  line: string;
  epoch: number | null;
}

export interface TaskDetail {
  current_state: ResolverDetail;
  last_event: LastWakeEvent | null;
}

export type SummaryReadResult =
  | { kind: 'ready'; summary: HomeSummary }
  | { kind: 'invalid'; reason: string }
  | { kind: 'unsupported'; schema: string | null; hold_schema: string | null }
  | { kind: 'missing'; reason: string };

export interface RuntimeDefaults {
  home: string | null;
  resolver_root: string | null;
}

export function desktopRuntime(): boolean {
  return isTauri();
}

export async function runtimeDefaults(): Promise<RuntimeDefaults> {
  return invoke<RuntimeDefaults>('runtime_defaults');
}

export async function chooseFolder(title: string): Promise<string | null> {
  const choice = await open({ directory: true, multiple: false, title });
  if (Array.isArray(choice)) return choice[0] ?? null;
  return choice;
}

export async function readSummary(home: string): Promise<SummaryReadResult> {
  return invoke<SummaryReadResult>('read_summary', { home });
}

export async function startSummaryWatch(home: string): Promise<void> {
  await invoke('watch_summary', { home });
}

export async function listenForSummaryChanges(callback: () => void): Promise<UnlistenFn> {
  return listen('summary-changed', callback);
}

export async function readTaskDetail(home: string, resolverRoot: string, taskId: string): Promise<TaskDetail> {
  return invoke<TaskDetail>('task_detail', { home, resolverRoot, taskId });
}

export async function openPullRequest(url: string): Promise<void> {
  const parsed = new URL(url);
  if (!['http:', 'https:'].includes(parsed.protocol) || parsed.username || parsed.password) throw new Error('Only credential-free HTTP(S) PR links can be opened');
  await openUrl(parsed.toString());
}
