<script lang="ts">
  import { state } from '$lib/stores';
</script>

<p class="text-3xl font-semibold tracking-tight text-slate-700 dark:text-grey">
  Profile:
  <span class="text-primary">{$state?.profile_settings.profile?.name}</span>.
</p>
