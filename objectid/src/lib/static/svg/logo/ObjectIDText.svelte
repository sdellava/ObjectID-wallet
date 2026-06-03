<script lang="ts">
  let {
    class: className = '',
    alt = 'ObjectID',
    'aria-hidden': ariaHidden,
  }: { class?: string; alt?: string; 'aria-hidden'?: 'true' | 'false' | boolean } = $props();
</script>

<img class={className} src="/brand/objectid-logo-large.png" {alt} aria-hidden={ariaHidden} />

<style>
  img {
    display: block;
    height: auto;
    max-width: 100%;
  }
</style>
