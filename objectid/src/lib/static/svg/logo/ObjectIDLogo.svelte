<script lang="ts">
  let { class: className = '', alt = 'ObjectID' }: { class?: string; alt?: string } = $props();
</script>

<img class={className} src="/brand/objectid-icon.png" {alt} />

<style>
  img {
    display: block;
    height: auto;
    max-width: 100%;
  }
</style>
