<script lang="ts">
  import { onMount } from 'svelte';

  import { goto } from '$app/navigation';

  import { loadOwnedObjectProducts, type ObjectIDProduct } from '$lib/objectid-items';
  import { state } from '$lib/stores';

  let loading = false;
  let error = '';
  let products: ObjectIDProduct[] = [];
  let lastDid = '';
  let lastAddress = '';
  let lastNetwork = '';

  const truncate = (value: string, left = 10, right = 6) => {
    if (value.length <= left + right + 3) return value;
    return `${value.slice(0, left)}...${value.slice(-right)}`;
  };

  const refreshProducts = async () => {
    const did = $state.iota_wallet.did ?? '';
    const address = $state.iota_wallet.address ?? '';
    const network = $state.iota_wallet.network ?? 'testnet';

    if (!did) {
      products = [];
      error = '';
      lastDid = '';
      lastAddress = '';
      lastNetwork = '';
      return;
    }

    if (did === lastDid && address === lastAddress && network === lastNetwork && products.length) return;

    loading = true;
    error = '';
    lastDid = did;
    lastAddress = address;
    lastNetwork = network;

    try {
      products = await loadOwnedObjectProducts({ did, ownerAddress: address, network });
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      products = [];
    } finally {
      loading = false;
    }
  };

  onMount(() => {
    void refreshProducts();
  });

  $: if ($state.iota_wallet.did) {
    void refreshProducts();
  }
</script>

{#if $state.iota_wallet.did}
  <section class="mt-5">
    <div class="mb-3 flex items-center justify-between gap-3">
      <div>
        <p class="text-[16px]/[22px] font-semibold text-slate-800 dark:text-grey">Owned objects</p>
        <p class="text-[11px]/[16px] font-medium text-slate-500 dark:text-slate-300">
          Objects where owner contains your DID
        </p>
      </div>
      <button
        class="rounded-lg border border-slate-200 px-3 py-2 text-[11px]/[14px] font-semibold text-slate-700 disabled:opacity-50 dark:border-slate-600 dark:text-grey"
        onclick={refreshProducts}
        disabled={loading}
      >
        {loading ? 'Loading' : 'Refresh'}
      </button>
    </div>

    {#if error}
      <div
        class="rounded-xl border border-rose-200 bg-white p-3 text-[12px]/[18px] font-medium text-rose-600 dark:border-rose-900/60 dark:bg-dark"
      >
        {error}
      </div>
    {:else if loading && products.length === 0}
      <div class="h-1 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
        <div class="h-full w-1/2 animate-pulse rounded-full bg-primary"></div>
      </div>
    {:else if products.length > 0}
      <div class="flex flex-col gap-3">
        {#each products as product (product.id)}
          <button
            class="flex w-full gap-3 rounded-xl border border-slate-200 bg-white p-3 text-left dark:border-slate-600 dark:bg-dark"
            onclick={() =>
              goto(
                `/me/objectid-products/${encodeURIComponent(product.id)}?n=${encodeURIComponent(product.network)}${
                  product.imageUrl ? `&img=${encodeURIComponent(product.imageUrl)}` : ''
                }`,
              )}
          >
            {#if product.imageUrl}
              <div class="h-20 w-20 shrink-0 overflow-hidden rounded-lg bg-silver dark:bg-navy" data-product-image>
                <img
                  class="h-full w-full object-cover"
                  src={product.imageUrl}
                  alt=""
                  loading="lazy"
                  onerror={(event) => {
                    (event.currentTarget.closest('[data-product-image]') as HTMLElement | null)?.classList.add(
                      'hidden',
                    );
                  }}
                />
              </div>
            {/if}
            <div class="min-w-0 flex-1">
              <p class="truncate text-[14px]/[20px] font-semibold text-slate-800 dark:text-grey">{product.title}</p>
              <p class="mt-1 line-clamp-3 text-[12px]/[18px] font-medium text-slate-500 dark:text-slate-300">
                {product.description}
              </p>
              <p class="mt-2 font-mono text-[10px]/[14px] text-slate-400">{truncate(product.id)}</p>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-600 dark:bg-dark">
        <p class="text-[13px]/[18px] font-semibold text-slate-800 dark:text-grey">No owned objects found</p>
        <p class="mt-1 text-[12px]/[18px] font-medium text-slate-500 dark:text-slate-300">
          We did not find objects whose owner field contains this DID.
        </p>
      </div>
    {/if}
  </section>
{/if}
