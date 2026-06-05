<script lang="ts">
  import { onMount } from 'svelte';

  import { page } from '$app/state';

  import { TopNavBar } from '$lib/components';
  import {
    formatProductFieldLabel,
    loadObjectIDProduct,
    normalizeProductImageUrl,
    type ObjectIDProduct,
  } from '$lib/objectid-items';

  let loading = true;
  let error = '';
  let product: ObjectIDProduct | null = null;

  const stringifyValue = (value: unknown) => {
    if (value === null || value === undefined || value === '') return '—';
    if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') return String(value);

    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  };

  const loadProduct = async () => {
    loading = true;
    error = '';

    try {
      const id = decodeURIComponent(page.params.id ?? '');
      const network = page.url.searchParams.get('n') ?? page.url.searchParams.get('network') ?? 'testnet';
      const fallbackImageUrl = normalizeProductImageUrl(page.url.searchParams.get('img'));
      const loadedProduct = await loadObjectIDProduct(id, network);
      product = loadedProduct
        ? {
            ...loadedProduct,
            imageUrl: loadedProduct.imageUrl || fallbackImageUrl,
          }
        : null;
      if (!product) error = 'Object not found.';
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      product = null;
    } finally {
      loading = false;
    }
  };

  onMount(() => {
    void loadProduct();
  });

  $: fieldEntries = Object.entries(product?.fields ?? {}).filter(([key]) => {
    return !['id', 'uid'].includes(key.toLowerCase());
  });
</script>

<TopNavBar on:back={() => history.back()} title="Object details" class="sticky top-0 z-10" />

<div class="flex min-h-full flex-col gap-4 bg-silver px-4 py-5 dark:bg-navy">
  {#if loading}
    <div class="h-1 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
      <div class="h-full w-1/2 animate-pulse rounded-full bg-primary"></div>
    </div>
  {:else if error}
    <section class="rounded-xl border border-rose-200 bg-white p-4 dark:border-rose-900/60 dark:bg-dark">
      <p class="text-base font-semibold text-rose-600">Unable to load object</p>
      <p class="mt-1 text-[12px]/[18px] font-medium text-rose-500">{error}</p>
      <button
        class="mt-4 rounded-lg bg-primary px-3 py-2 text-[12px]/[16px] font-semibold text-white dark:text-dark"
        onclick={loadProduct}
      >
        Retry
      </button>
    </section>
  {:else if product}
    <section class="overflow-hidden rounded-xl border border-slate-200 bg-white dark:border-slate-600 dark:bg-dark">
      {#if product.imageUrl}
        <img
          class="max-h-[320px] w-full object-cover"
          src={product.imageUrl}
          alt=""
          onerror={(event) => {
            event.currentTarget.remove();
          }}
        />
      {/if}
      <div class="p-4">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0 flex-1">
            <p class="text-lg/[24px] font-semibold text-slate-800 dark:text-grey">{product.title}</p>
            <p class="mt-1 text-[13px]/[20px] font-medium text-slate-500 dark:text-slate-300">
              {product.description}
            </p>
          </div>
          <span class="rounded-md bg-primary px-2 py-1 text-[10px]/[14px] font-semibold text-white dark:text-dark">
            {product.network}
          </span>
        </div>

        <div class="mt-4 rounded-lg bg-silver p-3 dark:bg-navy">
          <p class="text-[11px]/[16px] font-semibold text-slate-500 dark:text-slate-300">Object ID</p>
          <p class="mt-1 font-mono text-[11px]/[16px] break-all text-slate-800 dark:text-grey">{product.id}</p>
        </div>
      </div>
    </section>

    <section class="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-600 dark:bg-dark">
      <p class="text-base font-semibold text-slate-800 dark:text-grey">Details</p>
      <div class="mt-3 flex flex-col divide-y divide-slate-100 dark:divide-slate-700">
        <div class="py-3">
          <p class="text-[11px]/[16px] font-semibold text-slate-500 dark:text-slate-300">Type</p>
          <p class="mt-1 font-mono text-[11px]/[16px] break-all text-slate-800 dark:text-grey">
            {product.typeRepr || '—'}
          </p>
        </div>
        {#each fieldEntries as [key, value] (key)}
          <div class="py-3">
            <p class="text-[11px]/[16px] font-semibold text-slate-500 dark:text-slate-300">
              {formatProductFieldLabel(key)}
            </p>
            <p class="mt-1 text-[12px]/[18px] font-medium break-all whitespace-pre-wrap text-slate-800 dark:text-grey">
              {stringifyValue(value)}
            </p>
          </div>
        {/each}
      </div>
    </section>
  {/if}
</div>
