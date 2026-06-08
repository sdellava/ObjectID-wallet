<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

  import { beforeNavigate, goto } from '$app/navigation';
  import { page } from '$app/state';
  import LL from '$i18n/i18n-svelte';

  import type { Action } from '@bindings/actions/Action';
  import {
    cancel,
    checkPermissions,
    Format,
    openAppSettings,
    requestPermissions,
    scan,
    type PermissionState,
    type Scanned,
  } from '@tauri-apps/plugin-barcode-scanner';
  import { debug, info, warn } from '@tauri-apps/plugin-log';

  import { BottomNavBar, Button, LoadingSpinner, ProgressBar } from '$lib/components';
  import { dispatch } from '$lib/dispatcher';
  import { CameraSlashRegularIcon } from '$lib/icons';
  import { parseObjectIDDIDShareQr } from '$lib/objectid-did-share';
  import {
    decryptObjectIDSeedShare,
    parseObjectIDSeedShareQr,
    type ObjectIDSeedSharePayload,
  } from '$lib/objectid-seed-share';
  import { state } from '$lib/stores';

  let scanning = false;
  let loading = false;
  let seedSharePayload: ObjectIDSeedSharePayload | null = null;
  let seedSharePassword = '';
  let seedShareError = '';
  let seedShareProgress = 0;
  let seedShareStatus = '';
  let importingSeedShare = false;

  type ImportSeedIdentityAction = Action & {
    type: '[IOTA Wallet] Import seed identity';
    payload: {
      seed_hex: string;
      did: string;
      network: ObjectIDSeedSharePayload['network'];
      expected_address: string | null;
    };
  };

  // We temporarily introduce this type that extends `PermissionState` to handle a possible error when checking for permissions.
  let permissions_nullable: PermissionState | null;

  let mockQrCodeValue = '';

  const isOwnerTransferScan = () => page.url.searchParams.get('ownerTransfer') === '1';

  const ownerTransferReturnPath = (did: string) => {
    const fallback = '/me';
    const returnTo = page.url.searchParams.get('return') || fallback;
    const target = new URL(returnTo.startsWith('/') ? returnTo : fallback, window.location.origin);
    target.searchParams.set('ownerDid', did);
    return `${target.pathname}${target.search}${target.hash}`;
  };

  function onMessage(scanned: Scanned) {
    debug(`Scanned: ${scanned.content}`);

    if (isOwnerTransferScan()) {
      const did = parseObjectIDDIDShareQr(scanned.content);
      if (did) {
        loading = true;
        goto(ownerTransferReturnPath(did));
        return;
      }

      seedShareError = 'The QR code does not contain a valid ObjectID DID.';
      loading = false;
      return;
    }

    try {
      const payload = parseObjectIDSeedShareQr(scanned.content);
      if (payload) {
        seedSharePayload = payload;
        seedSharePassword = '';
        seedShareError = '';
        seedShareProgress = 0;
        seedShareStatus = '';
        loading = false;
        return;
      }
    } catch (error) {
      seedShareError = String((error as Error)?.message ?? error);
      loading = false;
      return;
    }

    loading = true;
    dispatch({ type: '[QR Code] Scanned', payload: { form_urlencoded: scanned.content } });
  }

  async function importSeedShare() {
    if (!seedSharePayload || importingSeedShare) return;

    try {
      seedShareError = '';
      importingSeedShare = true;
      seedShareProgress = 20;
      seedShareStatus = 'Decrypting ObjectID seed...';
      const seedHex = await decryptObjectIDSeedShare(seedSharePayload, seedSharePassword);

      seedShareProgress = 60;
      seedShareStatus = 'Saving wallet and Distributed Identity...';
      await dispatch({
        type: '[IOTA Wallet] Import seed identity',
        payload: {
          seed_hex: seedHex,
          did: seedSharePayload.did,
          network: seedSharePayload.network,
          expected_address: seedSharePayload.address ?? null,
        },
      } as ImportSeedIdentityAction);

      seedShareProgress = 100;
      seedShareStatus = 'Opening your wallet home...';
      await goto('/me');
    } catch (error) {
      seedShareError = String((error as Error)?.message ?? error ?? 'Failed to import ObjectID wallet.');
    } finally {
      importingSeedShare = false;
    }
  }

  function resetSeedShare() {
    seedSharePayload = null;
    seedSharePassword = '';
    seedShareError = '';
    seedShareProgress = 0;
    seedShareStatus = '';
  }

  // from example in plugin-barcode-scanner repo
  async function startScan() {
    let permissions = await checkPermissions()
      .then((permissions) => {
        info(`Permissions to use the camera: ${permissions}`);
        return permissions;
      })
      .catch((error) => {
        warn(`Error checking for permissions to use the camera: ${error}`);
        return null; // possibly return "denied"? or does that imply that the check has been successful, but was actively denied?
      });

    // TODO: handle receiving "prompt-with-rationale" (issue: https://github.com/tauri-apps/plugins-workspace/issues/979)
    if (permissions === 'prompt') {
      info('Requesting camera permissions');
      permissions = await requestPermissions(); // handle in more detail?
      info(`Permissions to use the camera: ${permissions}`);
    }

    permissions_nullable = permissions;

    if (permissions === 'granted') {
      // Scanning parameters
      const formats = [Format.QRCode];
      const windowed = true;

      info(`Starting scan with parameters: { formats: ${formats}, windowed: ${windowed} }`);
      scanning = true;
      scan({ formats, windowed })
        .then((res) => {
          onMessage(res);
        })
        .catch((error) => {
          // TODO: display error to user
          warn(error);
        })
        .finally(() => {
          scanning = false;
        });
    }
  }

  async function cancelScan() {
    await cancel();
    scanning = false;
    // TODO: non-scanning view is visible before redirecting to /me
    // goto('/me');
  }

  onDestroy(async () => {
    await cancelScan();
  });

  onMount(async () => {
    // TODO find a good way to test if not dev_mode. This will have to be checked after $state is loaded.
    startScan();
  });

  beforeNavigate(async ({ type, cancel }) => {
    if (type === 'popstate') {
      cancel();
      goto('/me');
    }
  });
</script>

<div class="content-height isolate flex flex-col items-stretch">
  <div class="hide-scrollbar grow overflow-x-hidden overflow-y-scroll">
    <div class="flex h-full w-full flex-col">
      {#if seedSharePayload}
        <div class="flex h-full flex-col justify-center space-y-5 bg-silver p-6 dark:bg-navy">
          <div class="rounded-3xl bg-white p-5 shadow-sm dark:bg-dark">
            <p class="text-[22px]/[30px] font-semibold text-primary">Import ObjectID wallet</p>
            <p class="mt-3 text-[13px]/[20px] font-medium text-slate-500 dark:text-slate-300">
              Enter the password used on dapp.objectid.io to decrypt the SEED and configure this wallet.
            </p>

            <div class="mt-5 space-y-2">
              <p class="text-[12px]/[18px] font-semibold text-slate-500 dark:text-slate-300">DID</p>
              <p
                class="rounded-2xl bg-slate-50 p-3 text-[12px]/[18px] font-medium break-all text-slate-700 dark:bg-navy dark:text-grey"
              >
                {seedSharePayload.did}
              </p>
            </div>

            <div class="mt-4 space-y-2">
              <p class="text-[12px]/[18px] font-semibold text-slate-500 dark:text-slate-300">Network</p>
              <p
                class="inline-flex rounded-full bg-slate-100 px-3 py-1 text-[12px]/[18px] font-semibold text-slate-700 dark:bg-navy dark:text-grey"
              >
                {seedSharePayload.network}
              </p>
            </div>

            <input
              bind:value={seedSharePassword}
              disabled={importingSeedShare}
              type="password"
              inputmode="text"
              autocomplete="current-password"
              class="mt-5 h-12 w-full rounded-xl border border-slate-200 px-3 text-[15px]/[22px] text-secondary dark:border-slate-600 dark:bg-dark"
              placeholder="Password"
              on:keydown={(event) => {
                if (event.key === 'Enter') importSeedShare();
              }}
            />

            {#if seedShareStatus}
              <div class="mt-5 space-y-3">
                <ProgressBar value={seedShareProgress} />
                <p class="text-center text-[12px]/[18px] font-semibold text-primary">{seedShareStatus}</p>
              </div>
            {/if}

            {#if seedShareError}
              <p class="mt-4 rounded-2xl bg-rose-50 p-3 text-[13px]/[20px] font-semibold text-rose-500">
                {seedShareError}
              </p>
            {/if}

            <div class="mt-6 flex flex-col gap-3">
              <Button
                label="Import wallet"
                on:click={importSeedShare}
                loading={importingSeedShare}
                disabled={!seedSharePassword || importingSeedShare}
              />
              <Button
                label="Scan another QR"
                variant="secondary"
                on:click={() => {
                  resetSeedShare();
                  startScan();
                }}
                disabled={importingSeedShare}
              />
            </div>
          </div>
        </div>
      {:else if !scanning && !loading}
        <!-- This part is only visible when no scanning or loading is happening.
          Only visible when user has not granted permissions to the camera. -->
        <div class="relative flex h-full flex-col items-center justify-center space-y-4 bg-silver p-8 dark:bg-navy">
          <!-- Ask for permissions (only if not given) -->
          {#if permissions_nullable && permissions_nullable !== 'granted'}
            <div class="flex w-3/4 flex-col space-y-4">
              <div class="flex flex-col items-center rounded-lg bg-rose-100 px-8 py-4 text-rose-500">
                <CameraSlashRegularIcon class="m-2 h-8 w-8" />
                <p class="text-center text-[13px]/[24px] font-semibold">{$LL.SCAN.PERMISSION_DENIED()}</p>
              </div>
              <Button label={$LL.SCAN.OPEN_SETTINGS()} on:click={openAppSettings} />
            </div>
          {/if}

          <!-- Dev mode -->
          {#if $state?.dev_mode !== 'Off' && !loading}
            <div class="flex w-3/4 flex-col space-y-4">
              <div class="flex flex-col space-y-2 rounded-[20px] border border-slate-200 p-2 dark:border-slate-600">
                <input
                  bind:value={mockQrCodeValue}
                  class="h-12 w-full rounded-xl border border-slate-200 px-3 text-[13px]/[24px] text-secondary dark:border-slate-600 dark:bg-dark"
                  placeholder="Paste QR code value"
                />
                <Button
                  variant="secondary"
                  on:click={() =>
                    dispatch({ type: '[QR Code] Scanned', payload: { form_urlencoded: mockQrCodeValue } })}
                  label="Process QR code"
                />
              </div>
              <Button variant="primary" on:click={startScan} label="Start new scan" />
            </div>
          {/if}
        </div>
      {:else}
        <!-- Scanning or loading/processing -->
        <div class="flex grow flex-col">
          <div class="bg-white p-5 dark:bg-dark">
            <p class="text-3xl font-semibold text-slate-700 dark:text-grey">
              {$LL.SCAN.TITLE_1()} <span class="text-primary">{$LL.SCAN.TITLE_2()}</span>
            </p>
            <p class="mt-4 text-sm font-medium text-slate-500 dark:text-slate-300">
              {$LL.SCAN.SUBTITLE()}
            </p>
          </div>
          <div class="my-container relative grow">
            {#if loading}
              <div class="absolute z-10 h-full w-full bg-silver dark:bg-navy"></div>
            {/if}
            <div class="barcode-scanner--area--container">
              <div class="square surround-cover">
                <div class="barcode-scanner--area--outer surround-cover">
                  <div
                    class="barcode-scanner--area--inner surround-cover flex items-center justify-center border-2 border-white"
                  >
                    {#if loading}
                      <LoadingSpinner class="z-20 h-12 w-12" />
                    {/if}
                  </div>
                </div>
              </div>
            </div>
            {#if $state?.dev_mode !== 'Off'}
              <div class="fixed bottom-[128px] z-10 flex w-full justify-center">
                <button class="rounded-lg bg-rose-100 px-4 py-3 font-medium text-rose-500" on:click={cancelScan}
                  >{$LL.CANCEL()}</button
                >
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
  <div class="z-10 shrink-0">
    {#if loading}
      <!-- Disable the BottomNavBar by overlaying a transparent element -->
      <div class="absolute z-10 h-full w-full bg-white opacity-60 dark:bg-dark"></div>
    {/if}
    <div class="fixed bottom-(--safe-area-inset-bottom) w-full shadow-[0_-4px_20px_0px_rgba(0,0,0,0.03)]">
      <BottomNavBar
        active={'scan'}
        on:me={() => goto('/me')}
        on:scan={() => goto('/scan')}
        on:activity={() => goto('/activity')}
      />
    </div>
  </div>
</div>

<style>
  .content-height {
    /* bottom-navigation: 64px */
    height: calc(100vh - var(--safe-area-inset-top) - var(--safe-area-inset-bottom) - 64px);
  }

  .my-container {
    width: 100%;
    overflow: hidden;
  }
  .my-container {
    display: flex;
  }

  .square {
    width: 100%;
    position: relative;
    overflow: hidden;
    transition: 0.3s;
  }
  .square:after {
    content: '';
    top: 0;
    display: block;
    padding-bottom: 100%;
  }
  .square > div {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
  }

  .surround-cover {
    box-shadow: 0 0 0 99999px rgba(0, 0, 0, 0.5);
  }

  .barcode-scanner--area--container {
    width: 75%;
    max-width: min(500px, 80vh);
    margin: auto;
  }
  .barcode-scanner--area--outer {
    display: flex;
  }
  .barcode-scanner--area--inner {
    width: 100%;
    border-radius: 20px;
  }
</style>
