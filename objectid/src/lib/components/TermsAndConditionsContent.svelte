<script lang="ts">
  import { onMount } from 'svelte';

  import { invoke } from '@tauri-apps/api/core';

  const termsUrl = 'https://objectid.io/general-terms/';

  let terms = $state('');
  let error = $state('');
  let loading = $state(true);

  onMount(async () => {
    try {
      terms = await invoke<string>('fetch_objectid_terms');
    } catch (caughtError) {
      error = caughtError instanceof Error ? caughtError.message : String(caughtError);
    } finally {
      loading = false;
    }
  });
</script>

<div class="space-y-3 text-xs leading-relaxed text-slate-700 dark:text-slate-200">
  {#if loading}
    <p class="font-medium">Loading the latest ObjectID terms...</p>
  {:else if error}
    <div class="rounded-lg border border-slate-200 bg-white p-3 dark:border-slate-600 dark:bg-dark">
      <p class="font-semibold">The live terms could not be loaded.</p>
      <p class="mt-2 text-slate-500 dark:text-slate-300">
        Open <a class="underline" href={termsUrl} target="_blank" rel="noreferrer">objectid.io/general-terms</a>
        to review the latest version.
      </p>
    </div>
  {:else}
    <p class="whitespace-pre-line">{terms}</p>
    <p class="pt-2 text-[11px] text-slate-500 dark:text-slate-300">
      Loaded from <a class="underline" href={termsUrl} target="_blank" rel="noreferrer">objectid.io/general-terms</a>
    </p>
  {/if}
</div>
