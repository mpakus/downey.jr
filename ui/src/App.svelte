<script lang="ts">
  import { onMount } from 'svelte'

  let lastAction = $state('')

  onMount(() => {
    if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
      return
    }

    let stop = () => {}
    void import('@tauri-apps/api/event').then(({ listen }) =>
      listen('menu://action', (event) => {
        lastAction = String(event.payload)
      }).then((unlisten) => {
        stop = unlisten
      }),
    )
    return () => stop()
  })
</script>

<svelte:head>
  <title>1537paperstreet</title>
</svelte:head>

<main>
  <h1>1537paperstreet</h1>
  <p>Your Markdown projects will appear here.</p>
  {#if lastAction}
    <p>Last menu action: {lastAction}</p>
  {/if}
</main>
