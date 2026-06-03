<script lang="ts">
  import type { HTMLImgAttributes } from 'svelte/elements';

  type Props = HTMLImgAttributes;

  let { class: className = '', alt = 'ObjectID', ...rest }: Props = $props();
</script>

<img class={className} src="/brand/objectid-icon.png" {alt} {...rest} />

<style>
  img {
    display: block;
    height: auto;
    max-width: 100%;
  }
</style>
