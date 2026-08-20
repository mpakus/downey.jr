<script lang="ts">
  import { onMount } from 'svelte'

  import { errorMessage } from '../lib/ipc'
  import type { UpdateCheck } from '../lib/ipc'

  let {
    version,
    autocheck = false,
    onclose,
    onopen,
    oncheck,
  }: {
    version: string
    autocheck?: boolean
    onclose: () => void
    onopen: (url: string) => void
    oncheck: () => Promise<UpdateCheck>
  } = $props()

  const site = 'https://aomega.co'

  let checking = $state(false)
  let result = $state<UpdateCheck | null>(null)
  let checkError = $state('')

  onMount(() => {
    if (autocheck) {
      void runCheck()
    }
  })

  async function runCheck() {
    checking = true
    checkError = ''
    result = null
    try {
      result = await oncheck()
    } catch (cause) {
      checkError = errorMessage(cause)
    } finally {
      checking = false
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === 'Escape') {
      onclose()
    }
  }}
/>

<div class="scrim" role="presentation" onclick={onclose}>
  <div
    class="sheet"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    aria-label="About 1537paperstreet"
    aria-busy={checking}
    onpointerdown={(event) => event.stopPropagation()}
  >
    <div class="logo-stripe">
      <img class="logo" src="/logo.png" alt="1537paperstreet" />
    </div>
    <p class="version">Version {version}</p>
    <p class="blurb">
      A local Markdown reader for macOS. It works with folders on disk, does not
      require an account, and does not send your documents over the network.
    </p>
    <p class="credit">Made in Austin ✩ Texas</p>
    <a
      href={site}
      onclick={(event) => {
        event.preventDefault()
        onopen(site)
      }}>{site.replace('https://', '')}</a
    >
    {#if checking}
      <p class="status" role="status">Checking for updates…</p>
    {:else if checkError}
      <p class="status" role="status">{checkError}</p>
    {:else if result}
      <p class="status" role="status">{result.message}</p>
    {/if}
    <div class="actions">
      <button type="button" disabled={checking} onclick={() => void runCheck()}>
        Check for Updates
      </button>
      {#if result?.available && result.release_url}
        <button
          type="button"
          onclick={() => {
            const url = result?.release_url
            if (url) {
              onopen(url)
            }
          }}
        >
          Open Download
        </button>
      {/if}
      <button type="button" onclick={onclose}>Close</button>
    </div>
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: color-mix(in srgb, var(--fg) 20%, transparent);
  }

  .sheet {
    display: grid;
    justify-items: stretch;
    gap: var(--space-3);
    width: min(26rem, 100%);
    padding: 0 0 var(--space-5);
    overflow: hidden;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    text-align: center;
  }

  .logo-stripe {
    display: grid;
    place-items: center;
    padding: var(--space-4) var(--space-5);
    background: white;
    border-block-end: 1px solid var(--border);
  }

  .logo {
    width: min(18rem, 100%);
    height: auto;
  }

  .version,
  .blurb,
  .credit,
  a,
  .status,
  .actions {
    justify-self: center;
    padding-inline: var(--space-5);
  }

  .version,
  .credit {
    margin: 0;
    color: var(--fg-muted);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .blurb {
    margin: 0;
    color: var(--fg);
    font-size: 0.875rem;
  }

  a {
    color: var(--accent);
    font-size: 0.875rem;
    font-weight: 600;
  }

  .status {
    margin: 0;
    color: var(--fg-muted);
    font-size: 0.8125rem;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-2);
  }

  button {
    min-height: 28px;
    padding: 0 var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    font-size: 0.75rem;
    font-weight: 600;
  }

  button:hover:not(:disabled) {
    background: var(--selection);
  }

  button:disabled {
    opacity: 0.6;
  }
</style>
