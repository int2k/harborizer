<script lang="ts">
  interface Props {
    home: string;
    resolverRoot: string;
    error: string;
    onHomeChange: (value: string) => void;
    onRootChange: (value: string) => void;
    onChooseHome: () => void;
    onChooseRoot: () => void;
    onConnect: () => void;
  }
  let { home, resolverRoot, error, onHomeChange, onRootChange, onChooseHome, onChooseRoot, onConnect }: Props = $props();
</script>

<section class="connection-card" aria-labelledby="connection-heading">
  <div class="connection-intro">
    <span class="eyebrow">LOCAL FLEET CONNECTION</span>
    <h2 id="connection-heading">Chart one Firstmate home</h2>
    <p>Harborizer reads the summary snapshot and selected task detail. It will not change your Firstmate home.</p>
  </div>
  <label for="home-path">Firstmate home folder</label>
  <div class="path-row">
    <input id="home-path" value={home} oninput={(event) => onHomeChange(event.currentTarget.value)} placeholder="Choose a local home folder" />
    <button type="button" class="quiet-button" onclick={onChooseHome}>Browse…</button>
  </div>
  <label for="root-path">Firstmate code root <span class="field-hint">optional if the resolver lives in the home</span></label>
  <div class="path-row">
    <input id="root-path" value={resolverRoot} oninput={(event) => onRootChange(event.currentTarget.value)} placeholder="Use FM_ROOT or the home folder" />
    <button type="button" class="quiet-button" onclick={onChooseRoot}>Browse…</button>
  </div>
  {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
  <div class="connection-actions">
    <p><span class="lock-mark" aria-hidden="true">◇</span> Reads only <code>state/home-summary.json</code> and selected detail.</p>
    <button type="button" class="primary-button" onclick={onConnect} disabled={!home.trim()}>Connect home <span aria-hidden="true">→</span></button>
  </div>
</section>
