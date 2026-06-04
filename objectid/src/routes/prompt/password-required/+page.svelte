<script lang="ts">
  import { onMount } from 'svelte';

  import LL from '$i18n/i18n-svelte';

  import { retrieve } from '@impierce/tauri-plugin-keystore';
  import { melt } from '@melt-ui/svelte';
  import { warn } from '@tauri-apps/plugin-log';

  import { ActionSheet, Button } from '$lib/components';
  import { dispatch } from '$lib/dispatcher';
  import { EyeClosedRegularIcon, EyeRegularIcon } from '$lib/icons';
  import ObjectIDLogo from '$lib/static/svg/logo/ObjectIDLogo.svelte';
  import { state } from '$lib/stores';

  let showPassword = false;
  let biometricsUnlocking = false;
  let biometricsError: string | undefined;
  let biometricsAttempted = false;

  let password: string;

  const SERVICE = 'com.impierce.identity-wallet';
  const USER = 'objectid'; // TODO: rename to "ACCOUNT" to reflect Keychain Access item?

  const withTimeout = <T,>(promise: Promise<T>, timeoutMs: number) =>
    Promise.race<T>([
      promise,
      new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error('Biometric authentication timed out')), timeoutMs);
      }),
    ]);

  const unlockWithBiometrics = async () => {
    if (biometricsUnlocking) {
      return;
    }

    biometricsAttempted = true;
    biometricsUnlocking = true;
    biometricsError = undefined;

    try {
      const password = await withTimeout(retrieve(SERVICE, USER), 20_000);
      if (password) {
        await dispatch({ type: '[Storage] Unlock', payload: { password } });
      } else {
        biometricsError = 'No biometric secret was found. Enter your password to continue.';
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : `${error}`;
      biometricsError = message;
      warn(message);
    } finally {
      biometricsUnlocking = false;
    }
  };

  // TODO move to the backend
  onMount(async () => {
    // When developer mode is enabled, a static password is injected automatically.
    if ($state?.dev_mode === 'OnWithAutologin') {
      warn('Developer mode - Injecting password automatically ...');
      setTimeout(() => {
        dispatch({ type: '[Storage] Unlock', payload: { password: 'sup3rSecr3t' } });
      }, 500);
    }

    if ($state?.profile_settings.biometrics_enabled && !biometricsAttempted) {
      setTimeout(() => {
        void unlockWithBiometrics();
      }, 300);
    }
  });
</script>

<div class="content-height flex items-center justify-center bg-silver dark:bg-navy">
  <!-- Placeholder -->
  <!-- <div class="aspect-square w-1/4 rounded-3xl border border-slate-200 bg-slate-100" /> -->
  <div class="flex flex-col items-center justify-center">
    <ObjectIDLogo class="text-blue dark:text-silver" />

    {#if biometricsUnlocking}
      <p class="mt-8 text-center text-[13px]/[20px] font-medium text-slate-500 dark:text-slate-300">
        Unlocking with biometrics...
      </p>
    {:else if biometricsError}
      <p class="mt-8 max-w-[260px] text-center text-[13px]/[20px] font-medium text-slate-500 dark:text-slate-300">
        {biometricsError}
      </p>
      <button
        class="mt-3 rounded-xl px-4 py-2 text-[13px]/[24px] font-medium text-blue active:bg-grey dark:text-silver dark:active:bg-dark"
        on:click={unlockWithBiometrics}>Try fingerprint again</button
      >
    {:else if $state?.profile_settings.biometrics_enabled}
      <button
        class="mt-8 rounded-xl px-4 py-2 text-[13px]/[24px] font-medium text-blue active:bg-grey dark:text-silver dark:active:bg-dark"
        on:click={unlockWithBiometrics}>Unlock with fingerprint</button
      >
    {/if}

    <!-- Manual password entry -->
    <div class="relative mt-6 mb-4 w-[240px]">
      <input
        type={showPassword ? 'text' : 'password'}
        class="h-12 w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-[13px]/[24px] text-slate-500 dark:border-slate-600 dark:bg-dark dark:text-slate-300"
        placeholder={$LL.LOCK_SCREEN.PASSWORD_INPUT_PLACEHOLDER()}
        on:input={(e: Event) => (password = (e.target as HTMLInputElement).value)}
      />
      <div class="absolute top-0 right-3 flex h-full items-center">
        <button class="rounded-full p-2" on:click={() => (showPassword = !showPassword)}>
          {#if showPassword}
            <EyeRegularIcon class="text-slate-700 dark:text-grey" />
          {:else}
            <EyeClosedRegularIcon class="text-slate-700 dark:text-grey" />
          {/if}
        </button>
      </div>
    </div>
    <Button
      label={$LL.LOCK_SCREEN.BUTTON_TEXT()}
      on:click={() => dispatch({ type: '[Storage] Unlock', payload: { password } })}
      disabled={!password}
    />

    <!-- Forgot password? Reset app -->
    <div class="mt-8">
      <ActionSheet titleText={$LL.SETTINGS.RESET_APP.TITLE()} descriptionText={$LL.SETTINGS.RESET_APP.DESCRIPTION()}>
        <button
          slot="trigger"
          let:trigger
          use:melt={trigger}
          class="rounded-xl px-4 py-2 text-[13px]/[24px] font-medium text-slate-400 opacity-50 active:bg-grey dark:active:bg-dark"
          >{$LL.LOCK_SCREEN.FORGOT_PASSWORD()}</button
        >

        <!-- TODO: bug: after resetting (closing the drawer, main UI is not clickable anymore) -->
        <div slot="content" class="w-full pt-[20px] pb-[10px]">
          <button
            class="h-[48px] w-full rounded-xl bg-rose-100 px-4 py-2 text-[14px]/[24px] font-medium text-rose-500"
            on:click={() => dispatch({ type: '[App] Reset' })}>{$LL.SETTINGS.RESET_APP.CONFIRM()}</button
          >
        </div>

        <Button variant="secondary" slot="close" let:close trigger={close} label={$LL.SETTINGS.RESET_APP.CANCEL()} />
      </ActionSheet>
    </div>
  </div>
</div>

<!-- Overwrite colors from template -->
<div class="safe-area-bottom z-10 bg-silver dark:bg-navy"></div>

<style>
  .content-height {
    height: calc(100vh - var(--safe-area-inset-top) - var(--safe-area-inset-bottom));
  }

  :global(body) {
    /* Fixes a UI problem on iOS where there is a white bar at the bottom when the emoji drawer is open */
    position: unset !important;
  }
</style>
