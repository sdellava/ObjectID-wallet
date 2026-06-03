<script lang="ts">
  import { goto } from '$app/navigation';
  import { fade } from 'svelte/transition';

  import '@lottiefiles/lottie-player';

  import LL from '$i18n/i18n-svelte';

  import { TopNavBar } from '$lib/components';
  import { dispatch } from '$lib/dispatcher';
  import { ShieldFillIcon } from '$lib/icons';
  import { onboarding_state } from '$lib/stores';
  import { calculateInitials } from '$lib/utils';

  let loading = false;
  let loadingMode: 'initialize' | 'import_seed' | null = null;

  async function createProfile(iotaWallet?: { mode: 'initialize' }) {
    loading = true;
    await dispatch({
      type: '[DID] Create new',
      payload: {
        name: $onboarding_state.name ?? '',
        picture: '',
        theme: 'system',
        password: $onboarding_state.password ?? '',
        biometrics_enabled: $onboarding_state.biometrics_enabled ?? false,
        iota_wallet: iotaWallet ?? null,
      },
    });
  }

  async function initializeWallet() {
    loadingMode = 'initialize';
    await createProfile({ mode: 'initialize' });
  }

  async function importSeed() {
    loadingMode = 'import_seed';
    await createProfile();
    await goto('/scan');
  }
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
    <div class="w-full rounded-xl border border-black/10 bg-white p-4 dark:border-white/15 dark:bg-dark">
      <p class="text-[13px]/[20px] font-semibold text-slate-900 dark:text-grey">IOTA wallet</p>
      <p class="pt-1 text-[12px]/[18px] font-medium text-slate-500 dark:text-slate-300">
        Initialize a new secure wallet or import the seed you created on dapp.objectid.io.
      </p>
      <div class="mt-4 grid grid-cols-1 gap-2">
        <button
          class="rounded-lg border border-black bg-black px-3 py-3 text-[12px]/[18px] font-semibold text-white disabled:opacity-50 dark:border-white dark:bg-white dark:text-black"
          on:click={initializeWallet}
          disabled={loading}
        >
          {loadingMode === 'initialize' ? 'Initializing wallet...' : 'Initialize wallet and DID'}
        </button>
        <button
          class="rounded-lg border border-slate-200 bg-white px-3 py-3 text-[12px]/[18px] font-semibold text-slate-800 disabled:opacity-50 dark:border-slate-600 dark:bg-dark dark:text-grey"
          on:click={importSeed}
          disabled={loading}
        >
          {loadingMode === 'import_seed' ? 'Opening scanner...' : 'Import seed from dapp.objectid.io'}
        </button>
      </div>
    </div>
    <!-- Hint: backup -->
    <!-- <div class="bg-slate-100 p-4 rounded-2xl w-full">
      <p class="text-sm text-slate-800">Let's create a quick backup.</p>
    </div> -->
  </div>
</div>
