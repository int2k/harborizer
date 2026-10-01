<script lang="ts">
  import type { Vessel } from '../model';
  import type { TaskDetail } from '../runtime';
  interface Props { vessel: Vessel | null; detail: TaskDetail | null; loading: boolean; onOpenPr: (url: string) => void; }
  let { vessel, detail, loading, onOpenPr }: Props = $props();
  const stageNames: Record<string, string> = {
    'setting-out': 'Setting out', 'under-way': 'Under way', inspection: 'In inspection', quay: 'At the quay', arrived: 'Arrived', unknown: 'Unknown',
  };
  function formatTime(epoch: number | null | undefined): string | null {
    if (typeof epoch !== 'number' || !Number.isFinite(epoch)) return null;
    return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(epoch * 1000));
  }
</script>

<aside class="detail-panel" aria-label="Selected vessel details">
  {#if vessel}
    <div class="detail-topline"><span class="eyebrow">VESSEL DETAIL</span><span class="detail-id">{vessel.id}</span></div>
    <h2>{vessel.title}</h2>
    <div class="detail-stage"><span class="stage-lamp" aria-hidden="true"></span>{stageNames[vessel.stage] ?? 'Unknown'}</div>
    <dl class="detail-facts">
      <div><dt>Port / project</dt><dd>{vessel.project}</dd></div>
      <div><dt>Kind</dt><dd>{vessel.kind}{#if vessel.hasReport}<span class="report-chip">Survey report</span>{/if}</dd></div>
      <div><dt>Current state</dt><dd>{detail?.current_state.state ?? vessel.state}{#if detail?.current_state.source}<span class="detail-source">via {detail.current_state.source}</span>{/if}</dd></div>
      <div><dt>Latest summary activity</dt><dd>{vessel.detail || 'No activity detail in this summary'}{#if vessel.since}<small>{vessel.since}</small>{/if}</dd></div>
      <div><dt>Last wake event · history only</dt><dd>{detail?.last_event?.line ?? 'Not present in the selected task summary'}{#if formatTime(detail?.last_event?.epoch)}<small>{formatTime(detail?.last_event?.epoch)}</small>{/if}</dd></div>
    </dl>
    {#if vessel.decision}
      <section class="decision-detail" aria-labelledby="decision-heading">
        <p class="eyebrow" id="decision-heading">CAPTAIN’S CALL</p>
        <p>{vessel.decision}</p>
      </section>
    {/if}
    {#if detail?.current_state.detail}
      <section class="resolver-detail">
        <p class="eyebrow">FIRSTMATE RESOLVER</p>
        <p>{detail.current_state.detail}</p>
        {#if detail.current_state.unknown}<span class="unknown-note">Current state could not be confirmed.</span>{/if}
      </section>
    {:else if loading}
      <p class="detail-loading" role="status">Reading current state…</p>
    {/if}
    {#if vessel.prUrl}
      <button class="pr-button" type="button" onclick={() => onOpenPr(vessel.prUrl!)}>
        Open pull request <span aria-hidden="true">↗</span>
      </button>
    {/if}
  {:else}
    <div class="detail-empty">
      <span class="detail-empty-mark" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M3 15h18l-3 5H7l-4-5Zm8-2V5l7 10M10 8l-4 7" /></svg></span>
      <h2>Choose a vessel</h2>
      <p>Select any boat on the chart or a row in the fleet list to inspect its details.</p>
    </div>
  {/if}
</aside>
