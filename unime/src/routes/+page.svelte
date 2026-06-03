<script lang="ts">
  import OIDwalletLogo from '$lib/static/svg/logo/OIDwalletLogo.svelte';
</script>

<div class="content-height flex items-center justify-center bg-silver dark:bg-navy">
  <OIDwalletLogo class="text-blue dark:text-silver" />
</div>

<style>
  .content-height {
    height: calc(100vh - var(--safe-area-inset-top) - var(--safe-area-inset-bottom));
  }
</style>
