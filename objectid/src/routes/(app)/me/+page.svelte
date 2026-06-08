<script lang="ts">
  import { beforeNavigate, goto, replaceState } from '$app/navigation';
  import { page } from '$app/state';
  import { fly } from 'svelte/transition';
  import QRCode from 'qrcode';

  import { ActionSheet } from '$lib/components';

  import '@lottiefiles/lottie-player';

  import { onMount } from 'svelte';

  import LL from '$i18n/i18n-svelte';
  import { writable, type Writable } from 'svelte/store';

  import { Button, CredentialList, Favorites, IconMessage, PaddedIcon, Tabs } from '$lib/components';
  import { dispatch } from '$lib/dispatcher';
  import { createObjectIDDIDSharePayload } from '$lib/objectid-did-share';
  import { GhostFillIcon, MagnifyingGlassIcon, PlusCircleIcon, RocketLaunchFillIcon } from '$lib/icons';
  import { onboarding_state, state } from '$lib/stores';

  import ObjectProducts from './ObjectProducts.svelte';
  import SortingSheet from './SortingSheet.svelte';
  import UserJourney from './UserJourney.svelte';
  import WelcomeMessage from './WelcomeMessage.svelte';

  let triggers = [$LL.ME.CREDENTIAL_TABS.ALL(), $LL.ME.CREDENTIAL_TABS.DATA(), $LL.ME.CREDENTIAL_TABS.BADGES()];
  let activeTab: Writable<string> = writable(page.state.tab || triggers[0]);
  const identityQrOpen = writable(false);
  let identityQrDataUrl = '';
  let identityShareStatus = '';

  beforeNavigate(async ({ type, cancel }) => {
    replaceState('', { tab: $activeTab });
    // Reset to first tab when navigating back again
    if (page.url.pathname === '/me') {
      activeTab.set(triggers[0]);
    }
    if (type === 'popstate') {
      cancel();
    }
  });

  onMount(() => {
    dispatch({ type: '[Credential] Refresh all statuses' });
  });

  const shortDid = (did: string) => {
    const [prefix, address] = did.match(/^(did:iota:[^:]+:)(.+)$/)?.slice(1) ?? ['', did];
    return address.length > 8 ? `${prefix}${address.slice(0, 8)}...` : did;
  };

  const openIdentityQr = async () => {
    const did = $state.iota_wallet.did ?? '';
    if (!did) return;

    identityShareStatus = '';
    identityQrOpen.set(true);
    identityQrDataUrl = await QRCode.toDataURL(createObjectIDDIDSharePayload(did), {
      errorCorrectionLevel: 'M',
      margin: 2,
      width: 260,
      color: {
        dark: '#000000',
        light: '#ffffff',
      },
    });
  };

  const shareIdentityDid = async () => {
    const did = $state.iota_wallet.did ?? '';
    if (!did) return;

    try {
      if (navigator.share) {
        await navigator.share({
          title: 'ObjectID Distributed Identity',
          text: did,
        });
        identityShareStatus = 'Shared.';
        return;
      }

      await navigator.clipboard.writeText(did);
      identityShareStatus = 'DID copied.';
    } catch (err) {
      identityShareStatus = err instanceof Error ? err.message : String(err);
    }
  };

  const copyIdentityDid = async () => {
    const did = $state.iota_wallet.did ?? '';
    if (!did) return;

    try {
      await navigator.clipboard.writeText(did);
      identityShareStatus = 'DID copied.';
    } catch (err) {
      identityShareStatus = err instanceof Error ? err.message : String(err);
    }
  };

  // security: clear onboarding state after successful creation
  // TODO: move somewhere else
  onboarding_state.set({});
</script>

<!-- Isolate stacking context to avoid z-index conflicts. -->
<div class="relative isolate flex flex-col bg-white dark:bg-dark">
  <div class="sticky top-0 z-10 w-full bg-white px-[20px] py-4 dark:bg-dark">
    <!-- Top Bar -->
    <div class="flex items-center justify-between gap-4">
      <img class="h-12 max-w-[260px] object-contain object-left" src="/brand/objectid-logo-large.png" alt="OID:ObjectID" />
      <button
        onclick={() => goto('/me/search')}
        class="-mr-3 flex h-11 w-11 items-center justify-center rounded-2xl text-black dark:text-white"
      >
        <MagnifyingGlassIcon class="h-6 w-6" />
      </button>
    </div>
  </div>

  <div class="p-5 pt-0">
    <WelcomeMessage />
    {#if $state.iota_wallet.did}
      <button
        class="mt-4 w-full rounded-xl border border-slate-200 bg-silver p-4 text-left dark:border-slate-600 dark:bg-navy"
        onclick={openIdentityQr}
      >
        <div class="flex items-center justify-between gap-3">
          <p class="text-[13px]/[18px] font-semibold text-slate-800 dark:text-grey">Distributed Identity</p>
          <p class="rounded-md bg-primary px-2 py-1 text-[10px]/[14px] font-semibold text-white dark:text-dark">
            {$state.iota_wallet.network}
          </p>
        </div>
        <p class="mt-2 font-mono text-[11px]/[16px] break-all text-slate-600 dark:text-slate-300">
          {shortDid($state.iota_wallet.did)}
        </p>
      </button>
      <ObjectProducts />
    {/if}
    {#if $state?.user_journey}
      <div class="pt-4">
        <UserJourney />
      </div>
    {/if}
  </div>

  <!-- should have min height: full screen - smallest possible welcome header - bottom nav - safe areas (top, bottom) -->
  <div
    in:fly={{ y: 18, duration: 200, opacity: 1 }}
    class="flex grow flex-col items-stretch justify-start rounded-t-[20px] bg-silver p-[18px] dark:bg-navy"
  >
    {#if $state?.credentials && $state?.credentials.length > 0}
      <div class="relative">
        <div>
          <Tabs class="mr-[50px]" value={activeTab} {triggers}>
            <!-- All -->
            <div slot="0" class="h-full pt-5">
              <Favorites />
              <CredentialList />
            </div>

            <!-- Data -->
            <div slot="1" class="h-full pt-5">
              <Favorites credentialType="data" />
              <CredentialList credentialType="data" />
            </div>

            <!-- Badges -->
            <div slot="2" class="h-full pt-5">
              <Favorites credentialType="badges" />
              <CredentialList credentialType="badges" />
            </div>
          </Tabs>
        </div>

        <div class="absolute top-0 right-0">
          <SortingSheet />
        </div>
      </div>
    {:else if $state?.user_journey}
      <!-- With active onboarding journey -->
      <div class="flex h-max grow flex-col items-center justify-center text-center">
        <div class="relative">
          <!-- TODO: extract icon component? -->
          <div class="relative z-10">
            <!-- z-index only applies to elements with explicit position, therefore also "relative" -->
            <PaddedIcon icon={RocketLaunchFillIcon} />
          </div>

          <!-- Confetti -->
          <div class="absolute top-1/2 left-1/2 z-0 -translate-x-1/2 -translate-y-1/2">
            <lottie-player
              src="/lottiefiles/bubble-burst-confetti-ajgRKUnNJ7.json"
              autoplay
              loop
              speed={0.25}
              mode="normal"
              style="width: 320px"
            ></lottie-player>
          </div>
        </div>

        <div class="pt-[15px]">
          <p class="pb-[15px] text-[22px]/[30px] font-semibold tracking-tight text-slate-800 dark:text-grey">
            Shall we get started?
          </p>
          <p class="w-[240px] text-[13px]/[24px] font-normal text-slate-500 dark:text-slate-300">
            Start your first steps to add some credentials to your "Me".
          </p>
        </div>
      </div>

      <ActionSheet
        titleText="Complete new goals"
        descriptionText="Start your mission here! Goals will lead you through important features and possibilities of ObjectID app."
      >
        <!-- TODO: bug: properly $close the drawer with melt-ui (otherwise two clicks necessary) -->
        <Button slot="trigger" let:trigger {trigger} label="Let's go" />
        <div slot="content" class="flex w-full flex-col pt-[20px]">
          <!-- TODO: add multiple steps inline in drawer -->
          <Button label={$LL.CONTINUE()} on:click={() => goto('/goals')} />
        </div>
      </ActionSheet>
    {:else if !$state.iota_wallet.did}
      <!-- Skipped onboarding journey -->
      <div class="flex grow flex-col items-center justify-center">
        <IconMessage icon={GhostFillIcon} title={$LL.ME.EMPTY_CREDENTIALS.TITLE()} />
        <div class="w-[280px] pt-[15px] text-center text-[13px]/[24px] font-normal text-slate-500 dark:text-slate-300">
          {$LL.ME.EMPTY_CREDENTIALS.SUBTITLE()}
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- "Add" button -->
<!-- <div in:fly={{ y: 12, delay: 0, opacity: 1, duration: 200 }} class="absolute bottom-5 right-4"> -->
<div
  in:fly={{ y: 8, opacity: 1, duration: 200 }}
  class="fixed right-4 bottom-[calc(64px+16px+var(--safe-area-inset-bottom))]"
>
  <button
    class="flex w-fit justify-center rounded-full bg-primary px-4 py-3 text-white dark:text-dark"
    onclick={() => goto('/me/add')}
  >
    <PlusCircleIcon class="mr-2 size-6" />
    <div class="text-[13px]/[24px] font-medium">{$LL.ADD_CREDENTIALS.BUTTON()}</div>
  </button>
</div>

<ActionSheet titleText="Share Distributed Identity" descriptionText="Share this DID with another ObjectID wallet." open={identityQrOpen}>
  <div slot="content" class="flex w-full flex-col gap-4 pt-5">
    {#if identityQrDataUrl}
      <div class="flex justify-center">
        <img class="h-[260px] w-[260px] rounded-xl bg-white p-3" src={identityQrDataUrl} alt="Distributed Identity QR code" />
      </div>
    {/if}
    <p class="rounded-xl bg-silver p-3 font-mono text-[11px]/[16px] break-all text-slate-800 dark:bg-navy dark:text-grey">
      {$state.iota_wallet.did}
    </p>
    {#if identityShareStatus}
      <p class="text-center text-[12px]/[18px] font-semibold text-primary">{identityShareStatus}</p>
    {/if}
    <div class="grid grid-cols-2 gap-3">
      <button
        class="h-12 rounded-xl bg-primary px-4 py-2 text-[13px]/[24px] font-semibold text-white dark:text-dark"
        onclick={shareIdentityDid}
      >
        Share
      </button>
      <button
        class="h-12 rounded-xl border border-slate-200 bg-white px-4 py-2 text-[13px]/[24px] font-semibold text-slate-800 dark:border-slate-600 dark:bg-dark dark:text-grey"
        onclick={copyIdentityDid}
      >
        Copy DID
      </button>
    </div>
    <button
      class="h-12 w-full rounded-xl border border-slate-200 bg-white px-4 py-2 text-[13px]/[24px] font-semibold text-slate-800 dark:border-slate-600 dark:bg-dark dark:text-grey"
      onclick={() => goto('/me/iota-identity')}
    >
      Manage identity
    </button>
  </div>
</ActionSheet>
