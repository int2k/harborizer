<script lang="ts">
  import type { Vessel } from '../model';
  interface Props { vessels: Vessel[]; selectedId: string | null; onSelect: (id: string) => void; }
  let { vessels, selectedId, onSelect }: Props = $props();
  let sortKey = $state<'title' | 'project' | 'stage'>('project');
  let ascending = $state(true);
  let sorted = $derived([...vessels].sort((a, b) => {
    const order = a[sortKey].localeCompare(b[sortKey]);
    return ascending ? order : -order;
  }));

  function sortBy(key: 'title' | 'project' | 'stage') {
    if (sortKey === key) ascending = !ascending;
    else { sortKey = key; ascending = true; }
  }
</script>

<div class="table-wrap">
  <table class="vessel-table">
    <caption class="sr-only">Fleet vessels and their known harbor stages</caption>
    <thead>
      <tr>
        <th><button type="button" onclick={() => sortBy('title')} aria-label="Sort by vessel">Vessel <span aria-hidden="true">{sortKey === 'title' ? ascending ? '↑' : '↓' : '↕'}</span></button></th>
        <th><button type="button" onclick={() => sortBy('project')} aria-label="Sort by project">Port <span aria-hidden="true">{sortKey === 'project' ? ascending ? '↑' : '↓' : '↕'}</span></button></th>
        <th><button type="button" onclick={() => sortBy('stage')} aria-label="Sort by stage">Stage <span aria-hidden="true">{sortKey === 'stage' ? ascending ? '↑' : '↓' : '↕'}</span></button></th>
        <th>Kind</th>
      </tr>
    </thead>
    <tbody>
      {#each sorted as vessel (vessel.id)}
        <tr class:selected={selectedId === vessel.id}>
          <td>
            <button class="vessel-row-select" type="button" aria-pressed={selectedId === vessel.id} onclick={() => onSelect(vessel.id)}>
              <span class="table-vessel-icon" aria-hidden="true">
                {#if vessel.kind === 'scout'}<svg viewBox="0 0 20 20"><path d="M2 12h16l-2.5 4H5.5L2 12Zm7-.7V3l5.5 9M8.5 5.5 5 12" /></svg>
                {:else}<svg viewBox="0 0 20 20"><path d="M2 12h16l-2.5 4H5.5L2 12Zm7-.7V3l5.5 9M8.5 5.5 5 12" /></svg>{/if}
              </span>
              <span>{vessel.title}</span>
              {#if vessel.decision}<span class="row-flag">Decision</span>{/if}{#if vessel.wait}<span class="row-flag">{vessel.wait}</span>{/if}
            </button>
          </td>
          <td>{vessel.project}</td>
          <td><span class={`stage-pill stage-${vessel.stage}`}>{vessel.stage.replaceAll('-', ' ')}</span></td>
          <td>{vessel.kind}{#if vessel.hasReport}<span class="report-chip">Report</span>{/if}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if vessels.length === 0}
    <p class="empty-list">No vessels are present in this summary.</p>
  {/if}
</div>
