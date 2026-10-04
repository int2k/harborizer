<script lang="ts">
  import { onMount } from 'svelte';
  import HarborChart from '../lib/components/HarborChart.svelte';
  import VesselList from '../lib/components/VesselList.svelte';
  import DetailPanel from '../lib/components/DetailPanel.svelte';
  import ConnectionSettings from '../lib/components/ConnectionSettings.svelte';
  import { classifyFreshness } from '../lib/freshness';
  import { fixtures, type FixtureName } from '../lib/fixtures';
  import { applyResolverState, mapSummaryToVessels } from '../lib/stages';
  import type { Freshness, HomeSummary, Vessel } from '../lib/model';
  import {
    chooseFolder, desktopRuntime, listenForSummaryChanges, openPullRequest, readSummary,
    readTaskDetail, runtimeDefaults, startSummaryWatch, type LastWakeEvent, type SummaryReadResult,
    type TaskDetail,
  } from '../lib/runtime';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  type DataSource = FixtureName | 'live';
  const isDevelopment = import.meta.env.DEV;
  const isDesktop = desktopRuntime();
  let dataSource = $state<DataSource>('busy');
  let home = $state('');
  let resolverRoot = $state('');
  let liveSummary = $state<HomeSummary | null>(null);
  let readResult = $state<SummaryReadResult | null>(null);
  let connectionError = $state('');
  let watchMode = $state<'watching' | 'polling' | 'offline'>('offline');
  let selectedId = $state<string | null>(null);
  let selectedTaskDetail = $state<{ id: string; data: TaskDetail } | null>(null);
  let detailLoading = $state(false);
  let view = $state<'chart' | 'list'>('chart');
  let darkTheme = $state(false);
  let toast = $state('');
  let unlisten: UnlistenFn | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  let summary = $derived(dataSource === 'live' ? liveSummary : isDevelopment ? fixtures[dataSource] : null);
  let baseVessels = $derived(summary ? mapSummaryToVessels(summary) : []);
  let selectedDetail = $derived(selectedTaskDetail?.id === selectedId ? selectedTaskDetail.data : null);
  let vessels: Vessel[] = $derived(baseVessels.map((vessel) => {
    if (vessel.id !== selectedId || !selectedDetail) return vessel;
    return applyResolverState(vessel, selectedDetail.current_state);
  }));
  let selectedVessel = $derived(vessels.find((vessel) => vessel.id === selectedId) ?? null);
  let freshness: Freshness = $derived(summary
    ? classifyFreshness(summary)
    : readResult?.kind === 'unsupported' ? 'unsupported'
      : readResult?.kind === 'invalid' ? 'invalid'
        : 'missing');
  let omittedCount = $derived(summary?.omitted.reduce((sum, item) => sum + item.count, 0) ?? 0);
  let portsCount = $derived(new Set(vessels.map((vessel) => vessel.project)).size);
  let decisionCount = $derived(vessels.filter((vessel) => vessel.decision).length);
  let updatedLabel = $derived(summary && Number.isFinite(summary.generated_epoch)
    ? new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(summary.generated_epoch * 1000))
    : 'Not available');

  $effect(() => {
    if (selectedId && vessels.some((vessel) => vessel.id === selectedId)) return;
    selectedId = vessels[0]?.id ?? null;
  });

  $effect(() => {
    const taskId = selectedId;
    if (!isDesktop || dataSource !== 'live' || !home || !taskId) {
      selectedTaskDetail = null;
      detailLoading = false;
      return;
    }
    let cancelled = false;
    detailLoading = true;
    selectedTaskDetail = null;
    readTaskDetail(home, resolverRoot, taskId).then((data) => {
      if (!cancelled) selectedTaskDetail = { id: taskId, data };
    }).catch((error: unknown) => {
      if (!cancelled) {
        selectedTaskDetail = {
          id: taskId,
          data: { current_state: { state: 'unknown', source: 'none', detail: String(error), pr_url: null, unknown: true }, last_event: null },
        };
      }
    }).finally(() => { if (!cancelled) detailLoading = false; });
    return () => { cancelled = true; };
  });

  async function refreshLive() {
    if (!isDesktop || !home.trim()) return;
    try {
      readResult = await readSummary(home.trim());
      if (readResult.kind === 'ready') {
        liveSummary = readResult.summary;
        connectionError = '';
      } else {
        liveSummary = null;
        connectionError = readResult.kind === 'unsupported'
          ? `Unsupported summary version: ${readResult.schema ?? 'unknown'} / ${readResult.hold_schema ?? 'unknown classifier'}`
          : readResult.reason;
      }
    } catch (error) {
      liveSummary = null;
      readResult = null;
      connectionError = `Could not read the selected home: ${String(error)}`;
    }
  }

  async function beginWatching() {
    if (!isDesktop || !home.trim()) return;
    if (unlisten) { unlisten(); unlisten = null; }
    try {
      await startSummaryWatch(home.trim());
      watchMode = 'watching';
    } catch {
      watchMode = 'polling';
    }
    try {
      unlisten = await listenForSummaryChanges(() => { void refreshLive(); });
    } catch {
      watchMode = 'polling';
    }
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(() => { void refreshLive(); }, 4000);
  }

  async function connectHome() {
    home = home.trim();
    resolverRoot = resolverRoot.trim();
    if (!home) { connectionError = 'Choose a Firstmate home folder first.'; return; }
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('harborizer.home', home);
      localStorage.setItem('harborizer.resolverRoot', resolverRoot);
    }
    dataSource = 'live';
    await refreshLive();
    await beginWatching();
  }

  async function selectSource(value: string) {
    if (value === 'live') {
      dataSource = 'live';
      await connectHome();
      return;
    }
    dataSource = value as FixtureName;
    if (unlisten) { unlisten(); unlisten = null; }
    if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
    watchMode = 'offline';
    liveSummary = null;
    readResult = null;
    connectionError = '';
    selectedTaskDetail = null;
  }

  async function chooseHome() {
    if (!isDesktop) return;
    const selected = await chooseFolder('Choose a Firstmate home folder');
    if (selected) home = selected;
  }

  async function chooseRoot() {
    if (!isDesktop) return;
    const selected = await chooseFolder('Choose the Firstmate code root');
    if (selected) resolverRoot = selected;
  }

  function handleKeys(event: KeyboardEvent) {
    if (!['ArrowDown', 'ArrowUp'].includes(event.key) || !vessels.length) return;
    const target = event.target as HTMLElement | null;
    if (target && ['INPUT', 'SELECT', 'TEXTAREA', 'BUTTON'].includes(target.tagName) && !target.closest('.harbor')) return;
    event.preventDefault();
    const current = vessels.findIndex((vessel) => vessel.id === selectedId);
    const delta = event.key === 'ArrowDown' ? 1 : -1;
    selectedId = vessels[(current + delta + vessels.length) % vessels.length]?.id ?? null;
  }

  async function openPr(url: string) {
    try {
      await openPullRequest(url);
    } catch (error) {
      toast = `Could not open pull request: ${String(error)}`;
      setTimeout(() => { toast = ''; }, 5000);
    }
  }

  function toggleTheme() {
    darkTheme = !darkTheme;
    document.documentElement.dataset.theme = darkTheme ? 'dark' : 'light';
    localStorage.setItem('harborizer.theme', darkTheme ? 'dark' : 'light');
  }

  onMount(() => {
    darkTheme = localStorage.getItem('harborizer.theme') === 'dark';
    document.documentElement.dataset.theme = darkTheme ? 'dark' : 'light';
    home = localStorage.getItem('harborizer.home') ?? '';
    resolverRoot = localStorage.getItem('harborizer.resolverRoot') ?? '';

    if (isDesktop) {
      void runtimeDefaults().then((defaults) => {
        if (!home) home = defaults.home ?? '';
        if (!resolverRoot) resolverRoot = defaults.resolver_root ?? '';
        if (!isDevelopment && home) {
          dataSource = 'live';
          void connectHome();
        }
      }).catch(() => {});
    }

    return () => {
      if (unlisten) unlisten();
      if (pollTimer) clearInterval(pollTimer);
    };
  });
</script>

<svelte:head>
  <title>Harborizer — Firstmate fleet chart</title>
  <meta name="description" content="A read-only harbor chart for a local Firstmate fleet." />
</svelte:head>

<svelte:window onkeydown={handleKeys} />

<div class="app-shell">
  <header class="topbar">
    <a class="brand" href="#main" aria-label="Harborizer home">
      <span class="brand-mark" aria-hidden="true">
        <svg viewBox="0 0 40 40" fill="none"><path d="M5 24h30l-4 8H11l-6-8Z" /><path d="M19 22V7l11 15M17 12 9 23M19 7v16" /><path d="M7 35c4-2 7-2 11 0s7 2 11 0 4-2 6-1" /></svg>
      </span>
      <span><strong>Harborizer</strong><small>FIRSTMATE FLEET CHART</small></span>
    </a>
    <div class="topbar-right">
      {#if isDevelopment}
        <label class="fixture-control" for="fixture-picker"><span class="live-dot" aria-hidden="true"></span> Preview
          <select id="fixture-picker" value={dataSource} onchange={(event) => void selectSource(event.currentTarget.value)}>
            <option value="busy">Busy fleet</option>
            <option value="empty">Empty harbor</option>
            <option value="stale">Stale summary</option>
            <option value="invalid">Invalid summary</option>
            <option value="decision">Captain’s call</option>
            {#if isDesktop}<option value="live">Live local home</option>{/if}
          </select>
        </label>
      {/if}
      <span class="read-only-chip"><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M4 10h12l-2 4H7l-3-4Zm5-1V3l5 8M8 6 5 10" /></svg> Read only</span>
      <button class="theme-toggle" type="button" onclick={toggleTheme} aria-label={darkTheme ? 'Switch to light theme' : 'Switch to dark theme'}>
        {#if darkTheme}
          <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4" /><path d="M12 2v2m0 16v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" /></svg>
        {:else}
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20.8 14.1A8.5 8.5 0 0 1 9.9 3.2 8.6 8.6 0 1 0 20.8 14.1Z" /></svg>
        {/if}
      </button>
    </div>
  </header>

  <main id="main" class="main-content">
    <section class="page-heading">
      <div>
        <p class="eyebrow heading-kicker"><span class="heading-rule"></span> YOUR LOCAL FLEET AT A GLANCE</p>
        <h1>See the work.<br /><em>Know the harbor.</em></h1>
        <p class="heading-copy">A calm chart of what’s setting out, underway, waiting, and home.</p>
      </div>
      <div class="snapshot-card" aria-live="polite">
        <span class="eyebrow">LAST SNAPSHOT</span>
        <strong>{updatedLabel}</strong>
        <span class={`freshness-badge freshness-${freshness}`}><span class="freshness-dot" aria-hidden="true"></span>{freshness.replace('-', ' ')}</span>
        {#if dataSource === 'live'}<small class="watch-note">{watchMode === 'watching' ? 'Watching state folder' : watchMode === 'polling' ? 'Polling every 4 seconds' : 'Waiting for a local home'}</small>{/if}
      </div>
    </section>

    {#if isDesktop && (!home || dataSource === 'live')}
      <details class="connection-drawer" open={!home || (dataSource === 'live' && !liveSummary)}>
        <summary>{home ? 'Home connection' : 'Connect a Firstmate home'} <span aria-hidden="true">⌄</span></summary>
        <ConnectionSettings {home} {resolverRoot} error={connectionError} onHomeChange={(value) => home = value} onRootChange={(value) => resolverRoot = value} onChooseHome={chooseHome} onChooseRoot={chooseRoot} onConnect={connectHome} />
      </details>
    {/if}

    {#if freshness === 'stale'}
      <div class="status-banner banner-stale" role="status"><span class="banner-icon" aria-hidden="true">!</span><p><strong>This snapshot is stale.</strong> It was generated more than 90 seconds ago; Harborizer will keep checking for a fresh file.</p></div>
    {:else if freshness === 'invalid'}
      <div class="status-banner banner-invalid" role="alert"><span class="banner-icon" aria-hidden="true">!</span><p><strong>Inventory marked invalid.</strong> {summary?.reason ?? 'The summary does not report a valid fleet inventory.'}</p></div>
    {:else if freshness === 'unsupported'}
      <div class="status-banner banner-invalid" role="alert"><span class="banner-icon" aria-hidden="true">?</span><p><strong>Unsupported summary version.</strong> Harborizer will not guess how to read this file. {connectionError}</p></div>
    {:else if freshness === 'missing' && dataSource === 'live'}
      <div class="status-banner banner-invalid" role="status"><span class="banner-icon" aria-hidden="true">·</span><p><strong>No readable summary yet.</strong> {connectionError || 'Choose the Firstmate home containing state/home-summary.json.'}</p></div>
    {/if}

    <section class="metrics-row" aria-label="Fleet summary">
      <div class="metric-card"><span class="metric-icon icon-vessels" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M3 14h18l-3 5H7l-4-5Zm8-2V4l7 10M10 7 6 14" /></svg></span><span><strong>{vessels.length}</strong><small>vessels</small></span></div>
      <div class="metric-card"><span class="metric-icon icon-ports" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M3 18h18M5 17v-7h5v7m2 0V5h6v12m-4-9h2m-2 3h2" /></svg></span><span><strong>{portsCount}</strong><small>ports</small></span></div>
      <div class="metric-card metric-decision"><span class="metric-icon icon-buoy" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 3v12m0-11h7l-2 3 2 3h-7M8 21h8m-7-6 1 6m5-6-1 6" /></svg></span><span><strong>{decisionCount}</strong><small>captain’s call{decisionCount === 1 ? '' : 's'}</small></span></div>
      <div class="metrics-spacer"></div>
      <div class="data-note"><span class="data-note-icon" aria-hidden="true"><svg viewBox="0 0 20 20"><circle cx="10" cy="10" r="7" /><path d="M10 6v4l3 2" /></svg></span> Snapshot view · {dataSource === 'live' ? watchMode : 'synthetic fixture'}</div>
    </section>

    {#if omittedCount > 0}
      <p class="omitted-note"><span aria-hidden="true">＋</span> {omittedCount} more {omittedCount === 1 ? 'row' : 'rows'} omitted by Firstmate’s bounded summary.</p>
    {/if}

    <section class="fleet-section" aria-labelledby="fleet-heading">
      <div class="fleet-heading-row">
        <div><p class="eyebrow">THE FLEET</p><h2 id="fleet-heading">Harbor overview</h2></div>
        <div class="view-switch" role="group" aria-label="Choose fleet view">
          <button type="button" class:active={view === 'chart'} aria-pressed={view === 'chart'} onclick={() => view = 'chart'}><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M2 10h16M5 10l3-4m1 4 3 4m0-4 3-3" /></svg> Chart</button>
          <button type="button" class:active={view === 'list'} aria-pressed={view === 'list'} onclick={() => view = 'list'}><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M7 4h11M7 10h11M7 16h11M2 4h1m-1 6h1m-1 6h1" /></svg> List</button>
        </div>
      </div>

      <div class="workspace-grid">
        <div class="view-card">
          {#if view === 'chart'}
            <HarborChart {vessels} {selectedId} onSelect={(id) => selectedId = id} />
            <div class="chart-legend" aria-label="Chart legend">
              <span><i class="legend-ship" aria-hidden="true"></i> Work vessel</span>
              <span><i class="legend-scout" aria-hidden="true"></i> Scout</span>
              <span><i class="legend-buoy" aria-hidden="true"></i> Captain decision</span>
              <span><i class="legend-report" aria-hidden="true"></i> Report</span>
            </div>
          {:else}
            <VesselList {vessels} {selectedId} onSelect={(id) => selectedId = id} />
          {/if}
        </div>
        <DetailPanel vessel={selectedVessel} detail={selectedDetail} loading={detailLoading} onOpenPr={openPr} />
      </div>
    </section>

    <footer class="footer-note"><span><svg viewBox="0 0 18 18"><path d="M2 10h14l-2 4H5l-3-4Zm7-1V3l5 8M8 5 5 10" /></svg></span> Harborizer observes. Firstmate remains at the helm.</footer>
  </main>

  {#if toast}<div class="toast" role="alert">{toast}</div>{/if}
</div>
