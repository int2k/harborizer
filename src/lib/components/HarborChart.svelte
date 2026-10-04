<script lang="ts">
  import type { HarborStage, Vessel } from '../model';

  interface Props {
    vessels: Vessel[];
    selectedId: string | null;
    onSelect: (id: string) => void;
  }

  let { vessels, selectedId, onSelect }: Props = $props();
  const lanes: { stage: HarborStage; label: string; x: number }[] = [
    { stage: 'setting-out', label: 'SETTING OUT', x: 240 },
    { stage: 'under-way', label: 'UNDER WAY', x: 410 },
    { stage: 'inspection', label: 'IN INSPECTION', x: 580 },
    { stage: 'quay', label: 'AT THE QUAY', x: 750 },
    { stage: 'arrived', label: 'ARRIVED', x: 920 },
    { stage: 'unknown', label: 'UNKNOWN', x: 1090 },
  ];
  let ports = $derived([...new Set(vessels.map((vessel) => vessel.project))]);
  let height = $derived(Math.max(310, 155 + ports.length * 112));

  function laneX(stage: HarborStage): number {
    return lanes.find((lane) => lane.stage === stage)?.x ?? 1090;
  }

  function portY(portIndex: number): number {
    return 130 + portIndex * 112;
  }

  function offsetFor(vessel: Vessel, port: string): number {
    const inLane = vessels.filter((item) => item.project === port && item.stage === vessel.stage);
    const slot = inLane.findIndex((item) => item.id === vessel.id);
    return (slot - (inLane.length - 1) / 2) * 31;
  }
</script>

<div class="harbor-scroll" role="region" aria-label="Harbor chart; use the arrow keys to move between vessels">
  <svg class="harbor" viewBox={`0 0 1190 ${height}`} role="group" aria-label="Fleet harbor chart with projects as ports and vessels arranged by stage">
    <defs>
      <pattern id="water-lines" width="74" height="44" patternUnits="userSpaceOnUse">
        <path d="M4 12c9-5 18-5 27 0s18 5 27 0M32 33c8-4 16-4 24 0" fill="none" stroke="currentColor" stroke-opacity=".075" stroke-width="1.2" />
      </pattern>
      <filter id="boat-shadow" x="-30%" y="-40%" width="160%" height="190%">
        <feDropShadow dx="0" dy="3" stdDeviation="3" flood-color="#17394a" flood-opacity=".18" />
      </filter>
    </defs>

    <rect x="0" y="0" width="1190" height={height} rx="16" class="water-base" />
    <rect x="0" y="0" width="1190" height={height} rx="16" fill="url(#water-lines)" class="water-texture" />
    <path d={`M190 70H1120 M190 70V${height - 30}`} class="chart-axis" />

    {#each lanes as lane (lane.stage)}
      <g aria-hidden="true">
        <text x={lane.x} y="48" text-anchor="middle" class="lane-label">{lane.label}</text>
        <path d={`M${lane.x} 70V${height - 30}`} class="lane-line" />
        <circle cx={lane.x} cy="70" r="4" class="lane-dot" />
      </g>
    {/each}

    {#each ports as port, portIndex (port)}
      {@const y = portY(portIndex)}
      <g class="port-group" aria-label={`Port ${port}`}>
        <rect x="18" y={y - 37} width="153" height="74" rx="11" class="port-card" />
        <path d={`M38 ${y + 7}h37l-8 13H45z M49 ${y - 4}v-20h11v20 M72 ${y - 19}h11v23`} class="port-mark" />
        <text x="93" y={y - 4} class="port-kicker">PORT</text>
        <text x="93" y={y + 15} class="port-name">{port}</text>
        <path d={`M180 ${y}H1120`} class="port-route" />
      </g>
      {#each vessels.filter((vessel) => vessel.project === port) as vessel (vessel.id)}
        {@const x = laneX(vessel.stage)}
        {@const vesselY = y + offsetFor(vessel, port)}
        <g
          class="vessel-hit"
          class:selected={selectedId === vessel.id}
          class:scout={vessel.kind === 'scout'}
          role="button"
          tabindex="0"
          aria-label={`${vessel.title}, ${port}, ${vessel.stage.replaceAll('-', ' ')}`}
          aria-pressed={selectedId === vessel.id}
          onclick={() => onSelect(vessel.id)}
          onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onSelect(vessel.id); } }}
        >
          <title>{vessel.title} — {vessel.stage.replaceAll('-', ' ')}</title>
          <circle cx={x} cy={vesselY} r="31" class="vessel-focus-ring" />
          <g filter="url(#boat-shadow)" class="boat-shape">
            {#if vessel.kind === 'scout'}
              <path d={`M${x - 25} ${vesselY + 4}h45l-7 11h-31z`} class="boat-hull scout-hull" />
              <path d={`M${x - 4} ${vesselY + 2}v-26l17 26z`} class="boat-sail scout-sail" />
              <path d={`M${x - 4} ${vesselY - 23}v25`} class="mast" />
            {:else}
              <path d={`M${x - 30} ${vesselY + 3}h56l-9 14h-39z`} class="boat-hull" />
              <path d={`M${x - 8} ${vesselY + 1}v-33l24 33z`} class="boat-sail sail-main" />
              <path d={`M${x - 7} ${vesselY - 23}l-17 24h17z`} class="boat-sail sail-small" />
              <path d={`M${x - 8} ${vesselY - 31}v33`} class="mast" />
            {/if}
          </g>
          {#if vessel.hasReport}
            <g class="report-mark" aria-label="Report available">
              <circle cx={x + 25} cy={vesselY - 22} r="9" />
              <path d={`M${x + 25} ${vesselY - 27}v10m-4-5h8`} />
            </g>
          {/if}
          {#if vessel.decision}
            <g class="decision-buoy" aria-label="Captain decision waiting">
              <path d={`M${x + 33} ${vesselY - 28}v18`} />
              <path d={`M${x + 33} ${vesselY - 28}h19l-5 6 5 6h-19z`} />
              <circle cx={x + 33} cy={vesselY - 8} r="4" />
            </g>
          {/if}
          {#if vessel.wait}<text x={x} y={vesselY + 50} text-anchor="middle" class="vessel-label">{vessel.wait}</text>{/if}
          <text x={x} y={vesselY + 36} text-anchor="middle" class="vessel-label">{vessel.title.length > 19 ? `${vessel.title.slice(0, 18)}…` : vessel.title}</text>
        </g>
      {/each}
    {/each}

    {#if ports.length === 0}
      <g aria-hidden="true" class="empty-chart">
        <circle cx="610" cy="176" r="39" class="empty-sun" />
        <path d="M565 222h90l-14 18h-62zM610 222v-52l29 52zM608 177l-23 45h23z" class="empty-boat" />
        <text x="610" y="270" text-anchor="middle">Clear water. No vessels in this snapshot.</text>
      </g>
    {/if}
  </svg>
</div>
