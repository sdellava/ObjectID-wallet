<script lang="ts">
  import { onMount, tick } from 'svelte';

  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import jsQR from 'jsqr';

  import type { Action } from '@bindings/actions/Action';

  import { TopNavBar } from '$lib/components';
  import { dispatch } from '$lib/dispatcher';
  import { parseObjectIDDIDShareQr } from '$lib/objectid-did-share';
  import {
    formatProductFieldLabel,
    loadObjectIDProduct,
    normalizeProductImageUrl,
    prepareObjectGeolocationUpdate,
    prepareObjectOwnerDidUpdate,
    type ObjectIDProduct,
  } from '$lib/objectid-items';
  import { state as appState } from '$lib/stores';

  let loading = true;
  let error = '';
  let product: ObjectIDProduct | null = null;
  let updatingGeolocation = false;
  let updateError = '';
  let updateSuccess = '';
  let ownerTransferOpen = false;
  let ownerDidFileInput: HTMLInputElement | null = null;
  let newOwnerDid = '';
  let ownerTransferError = '';
  let ownerTransferStatus = '';
  let importingOwnerDidImage = false;
  let transferringOwner = false;

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

  const getDeviceGeolocation = () =>
    new Promise<string>((resolve, reject) => {
      if (!navigator.geolocation) {
        reject(new Error('Device geolocation is not available.'));
        return;
      }

      navigator.geolocation.getCurrentPosition(
        (position) => {
          const latitude = position.coords.latitude.toFixed(7);
          const longitude = position.coords.longitude.toFixed(7);
          resolve(`${latitude},${longitude}`);
        },
        () => reject(new Error('Unable to read the device location.')),
        {
          enableHighAccuracy: true,
          maximumAge: 30_000,
          timeout: 20_000,
        },
      );
    });

  const updateGeolocation = async () => {
    if (!product || updatingGeolocation) return;

    updatingGeolocation = true;
    updateError = '';
    updateSuccess = '';

    try {
      const wallet = $appState.iota_wallet;
      const geolocation = await getDeviceGeolocation();

      const payload = await prepareObjectGeolocationUpdate({
        did: wallet.did ?? '',
        seed: wallet.seed_phrase ?? '',
        network: wallet.network ?? product.network,
        objectId: product.id,
        objectType: product.typeRepr,
        geolocation,
      });
      await dispatch({
        type: '[IOTA Wallet] Sign prepared transaction',
        payload,
      } as Action);
      await tick();

      const walletError = $appState.iota_wallet.last_error;
      if (walletError) throw new Error(walletError);

      updateSuccess = 'Object geolocation updated.';
      await loadProduct();
    } catch (err) {
      updateError = err instanceof Error ? err.message : String(err);
    } finally {
      updatingGeolocation = false;
    }
  };

  const openOwnerTransferDialog = () => {
    newOwnerDid = '';
    ownerTransferError = '';
    ownerTransferStatus = '';
    ownerTransferOpen = true;
  };

  const scanNewOwnerDid = async () => {
    if (transferringOwner) return;

    const returnSearch = Array.from(page.url.searchParams.entries())
      .filter(([key]) => key !== 'ownerDid')
      .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
      .join('&');
    const returnTo = `${page.url.pathname}${returnSearch ? `?${returnSearch}` : ''}`;

    await goto(`/scan?ownerTransfer=1&return=${encodeURIComponent(returnTo)}`);
  };

  const decodeDidFromImage = (file: File) =>
    new Promise<string>((resolve, reject) => {
      const url = URL.createObjectURL(file);
      const image = new Image();
      image.onload = () => {
        try {
          const width = image.naturalWidth || image.width;
          const height = image.naturalHeight || image.height;
          const canvas = document.createElement('canvas');
          canvas.width = width;
          canvas.height = height;

          const context = canvas.getContext('2d', { willReadFrequently: true });
          if (!context) throw new Error('Unable to read the selected image.');

          context.drawImage(image, 0, 0, width, height);
          const imageData = context.getImageData(0, 0, width, height);
          const content = jsQR(imageData.data, width, height)?.data ?? '';
          const did = parseObjectIDDIDShareQr(content);
          if (!did) throw new Error('The selected image does not contain a valid ObjectID DID QR code.');
          resolve(did);
        } catch (err) {
          reject(err);
        } finally {
          URL.revokeObjectURL(url);
        }
      };
      image.onerror = () => {
        URL.revokeObjectURL(url);
        reject(new Error('Unable to read the selected image.'));
      };
      image.src = url;
    });

  const importNewOwnerDidImage = async (event: Event) => {
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file || importingOwnerDidImage || transferringOwner) return;

    try {
      importingOwnerDidImage = true;
      ownerTransferError = '';
      ownerTransferStatus = 'Reading QR image...';
      newOwnerDid = await decodeDidFromImage(file);
      ownerTransferStatus = 'New owner DID loaded.';
    } catch (err) {
      ownerTransferError = err instanceof Error ? err.message : String(err);
      ownerTransferStatus = '';
    } finally {
      importingOwnerDidImage = false;
      if (ownerDidFileInput) ownerDidFileInput.value = '';
    }
  };

  const transferObjectOwner = async () => {
    if (!product || transferringOwner) return;

    try {
      transferringOwner = true;
      ownerTransferError = '';
      ownerTransferStatus = 'Submitting sponsored transfer transaction...';
      const wallet = $appState.iota_wallet;

      const payload = await prepareObjectOwnerDidUpdate({
        did: wallet.did ?? '',
        seed: wallet.seed_phrase ?? '',
        network: wallet.network ?? product.network,
        objectId: product.id,
        objectType: product.typeRepr,
        newOwnerDid,
      });
      await dispatch({
        type: '[IOTA Wallet] Sign prepared transaction',
        payload,
      } as Action);
      await tick();

      const walletError = $appState.iota_wallet.last_error;
      if (walletError) throw new Error(walletError);

      ownerTransferStatus = 'Ownership transferred.';
      ownerTransferOpen = false;
      await goto('/me?refreshObjects=1');
    } catch (err) {
      ownerTransferError = err instanceof Error ? err.message : String(err);
      ownerTransferStatus = '';
    } finally {
      transferringOwner = false;
    }
  };

  onMount(() => {
    const scannedOwnerDid = page.url.searchParams.get('ownerDid');
    if (scannedOwnerDid) {
      newOwnerDid = scannedOwnerDid;
      ownerTransferOpen = true;
      ownerTransferStatus = 'New owner DID loaded.';
    }

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

        <button
          class="mt-4 w-full rounded-lg bg-primary px-4 py-3 text-[13px]/[18px] font-semibold text-white disabled:opacity-60 dark:text-dark"
          disabled={updatingGeolocation}
          onclick={updateGeolocation}
        >
          Update Object Geolocation
        </button>
        {#if updateError}
          <p class="mt-2 text-[12px]/[18px] font-medium text-rose-500">{updateError}</p>
        {:else if updateSuccess}
          <p class="mt-2 text-[12px]/[18px] font-medium text-primary">{updateSuccess}</p>
        {/if}

        <button
          class="mt-3 w-full rounded-lg border border-slate-200 bg-white px-4 py-3 text-[13px]/[18px] font-semibold text-slate-800 disabled:opacity-60 dark:border-slate-600 dark:bg-dark dark:text-grey"
          disabled={updatingGeolocation || transferringOwner}
          onclick={openOwnerTransferDialog}
        >
          Transfer Object Ownership
        </button>
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

{#if updatingGeolocation}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-6">
    <section class="w-full max-w-[320px] rounded-xl bg-white p-5 shadow-xl dark:bg-dark">
      <p class="text-base font-semibold text-slate-800 dark:text-grey">Updating object geolocation</p>
      <div class="mt-4 h-2 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
        <div class="h-full w-1/2 animate-pulse rounded-full bg-primary"></div>
      </div>
    </section>
  </div>
{/if}

{#if ownerTransferOpen}
  <div
    class="fixed inset-0 z-50 flex items-end justify-center bg-black/40 px-4 pb-[calc(16px+var(--safe-area-inset-bottom))]"
  >
    <section class="max-h-[90vh] w-full max-w-[420px] overflow-y-auto rounded-2xl bg-white p-5 shadow-xl dark:bg-dark">
      <div class="flex items-start justify-between gap-3">
        <div>
          <p class="text-base font-semibold text-slate-800 dark:text-grey">Transfer object ownership</p>
          <p class="mt-1 text-[12px]/[18px] font-medium text-slate-500 dark:text-slate-300">
            Scan or import the new owner's Distributed Identity DID.
          </p>
        </div>
        <button
          class="rounded-lg px-2 py-1 text-[12px]/[16px] font-semibold text-slate-500 disabled:opacity-50"
          disabled={transferringOwner}
          onclick={() => (ownerTransferOpen = false)}
        >
          Close
        </button>
      </div>

      <div class="mt-5 grid grid-cols-2 gap-3">
        <button
          class="h-12 rounded-xl bg-primary px-3 py-2 text-[12px]/[18px] font-semibold text-white disabled:opacity-60 dark:text-dark"
          disabled={transferringOwner}
          onclick={scanNewOwnerDid}
        >
          Scan DID QR
        </button>
        <button
          class="h-12 rounded-xl border border-slate-200 bg-white px-3 py-2 text-[12px]/[18px] font-semibold text-slate-800 disabled:opacity-60 dark:border-slate-600 dark:bg-dark dark:text-grey"
          disabled={importingOwnerDidImage || transferringOwner}
          onclick={() => ownerDidFileInput?.click()}
        >
          {importingOwnerDidImage ? 'Importing' : 'Import image'}
        </button>
      </div>

      <input
        bind:this={ownerDidFileInput}
        class="hidden"
        type="file"
        accept="image/*"
        onchange={importNewOwnerDidImage}
      />

      {#if newOwnerDid}
        <div class="mt-5 rounded-xl bg-silver p-3 dark:bg-navy">
          <p class="text-[11px]/[16px] font-semibold text-slate-500 dark:text-slate-300">New owner DID</p>
          <p class="mt-1 font-mono text-[11px]/[16px] break-all text-slate-800 dark:text-grey">{newOwnerDid}</p>
        </div>

        <div class="mt-4 rounded-xl border border-rose-200 bg-rose-50 p-3 dark:border-rose-900/60">
          <p class="text-[12px]/[18px] font-semibold text-rose-600">
            Confirming this transaction transfers ownership and control of this object. After it is confirmed, this
            object will no longer appear in your owned objects list.
          </p>
        </div>
      {/if}

      {#if ownerTransferStatus}
        <div class="mt-5 space-y-3">
          <div class="h-2 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
            <div class="h-full w-1/2 animate-pulse rounded-full bg-primary"></div>
          </div>
          <p class="text-center text-[12px]/[18px] font-semibold text-primary">{ownerTransferStatus}</p>
        </div>
      {/if}

      {#if ownerTransferError}
        <p class="mt-4 rounded-xl bg-rose-50 p-3 text-[12px]/[18px] font-semibold text-rose-500">
          {ownerTransferError}
        </p>
      {/if}

      <button
        class="mt-5 h-12 w-full rounded-xl bg-rose-500 px-4 py-2 text-[13px]/[24px] font-semibold text-white disabled:opacity-50"
        disabled={!newOwnerDid || transferringOwner}
        onclick={transferObjectOwner}
      >
        {transferringOwner ? 'Transferring ownership' : 'Confirm transfer'}
      </button>
    </section>
  </div>
{/if}
