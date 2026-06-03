<script lang="ts">
  import ObjectIDLogo from '$lib/static/svg/logo/ObjectIDLogo.svelte';
</script>

<div class="content-height flex items-center justify-center bg-silver dark:bg-navy">
  <ObjectIDLogo class="text-blue dark:text-silver" />
</div>

<style>
  .content-height {
    height: calc(100vh - var(--safe-area-inset-top) - var(--safe-area-inset-bottom));
  }
</style>
