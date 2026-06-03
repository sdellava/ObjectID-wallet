<script lang="ts">
  import { goto } from '$app/navigation';
  import { fade } from 'svelte/transition';

  import '@lottiefiles/lottie-player';

  import LL from '$i18n/i18n-svelte';

  import { Button, ProgressBar, TopNavBar } from '$lib/components';
  import { dispatch } from '$lib/dispatcher';
  import { ShieldFillIcon } from '$lib/icons';
  import { onboarding_state } from '$lib/stores';
  import { calculateInitials } from '$lib/utils';

  let loadingIdentity = false;
  let loadingImport = false;
  let statusMessage = '';
  let progress = 0;

  const createProfile = () =>
    dispatch({
      type: '[DID] Create new',
      payload: {
        name: $onboarding_state.name ?? '',
        picture: '',
        theme: 'system',
        password: $onboarding_state.password ?? '',
        biometrics_enabled: $onboarding_state.biometrics_enabled ?? false,
      },
    });

  const createDistributedIdentity = async () => {
    if (loadingIdentity || loadingImport) return;

    loadingIdentity = true;
    progress = 10;
    statusMessage = 'Creating your profile...';
    await createProfile();

    progress = 35;
    statusMessage = 'Configuring your IOTA wallet...';
    await dispatch({
      type: '[IOTA Wallet] Create or load',
      payload: { network: 'testnet' },
    });

    progress = 65;
    statusMessage = 'Creating your Distributed Identity...';
    await dispatch({ type: '[IOTA Wallet] Create identity', payload: {} });

    progress = 100;
    statusMessage = 'Opening your wallet home...';
    await goto('/me');
  };

  const importSeed = async () => {
    if (loadingIdentity || loadingImport) return;

    loadingImport = true;
    progress = 35;
    statusMessage = 'Creating your profile...';
    await createProfile();

    progress = 100;
    statusMessage = 'Opening the scanner...';
    setTimeout(() => goto('/scan'), 100);
  };
</script>

<!-- TODO: should we show this screen AFTER a successful creation of a stronghold? -->
<TopNavBar on:back={() => history.back()} title={$LL.ONBOARDING.PASSWORD.COMPLETED.NAVBAR_TITLE()} disabled />
<!-- Content -->
<div class="mt-8 grow p-4" in:fade={{ delay: 200 }}>
  <div class="pt-4 pb-8">
    <p class="pb-8 text-3xl font-semibold text-slate-700 dark:text-grey">
      {$LL.ONBOARDING.PASSWORD.COMPLETED.TITLE_1()}
      <span class="text-primary">{$LL.ONBOARDING.PASSWORD.COMPLETED.TITLE_2()}</span>
    </p>
  </div>
  <div class="flex flex-col items-center justify-center space-y-6 rounded-3xl bg-white p-5 dark:bg-dark">
    <p class="text-[22px]/[30px] font-semibold text-primary">{$LL.ONBOARDING.PASSWORD.COMPLETED.MESSAGE_1()}</p>
    <div class="relative">
      <div class="relative z-10">
        <div class="text-[100px]/[100px]"><ShieldFillIcon class="text-primary" /></div>
        <span class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-[36px]/[36px]">
          <p class="font-semibold tracking-tight text-white dark:text-dark">
            {calculateInitials($onboarding_state.name ?? '')}
          </p>
        </span>
      </div>
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
    <p class="text-[22px]/[30px] font-semibold text-primary">
      {$LL.ONBOARDING.PASSWORD.COMPLETED.MESSAGE_2()}, {$onboarding_state.name}!
    </p>
    <p class="text-center text-[13px]/[20px] font-medium text-slate-500 dark:text-slate-300">
      Choose how you want to set up your ObjectID wallet.
    </p>
    {#if statusMessage}
      <div class="w-full space-y-3">
        <ProgressBar value={progress} />
        <p class="text-center text-[12px]/[18px] font-semibold text-primary">{statusMessage}</p>
      </div>
    {/if}
    <!-- Hint: backup -->
    <!-- <div class="bg-slate-100 p-4 rounded-2xl w-full">
      <p class="text-sm text-slate-800">Let's create a quick backup.</p>
    </div> -->
  </div>
</div>

<div class="rounded-t-3xl bg-white p-6 dark:bg-dark" in:fade={{ delay: 200 }}>
  <div class="flex flex-col gap-3">
    <Button
      label="Create a new distributed identity"
      on:click={createDistributedIdentity}
      loading={loadingIdentity}
      disabled={loadingImport}
    />
    <Button
      label="Import SEED from dapp.objectid.io"
      variant="secondary"
      on:click={importSeed}
      loading={loadingImport}
      disabled={loadingIdentity}
    />
  </div>
</div>
