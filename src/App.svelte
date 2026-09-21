<script lang="ts">
  import { appName } from '$lib/app';
</script>

<main>
  <p class="placeholder">{appName}</p>
</main>

<style>
  main {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
  }
  .placeholder {
    color: #888;
    font-family: ui-monospace, monospace;
  }
</style>
