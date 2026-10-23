<script lang="ts">
  type Item = { label: string; hint?: string; onSelect: () => void; separatorBefore?: boolean };
  let { label, items, align = 'left', primary = false } = $props<{
    label: string;
    items: Item[];
    align?: 'left' | 'right';
    primary?: boolean;
  }>();

  let open = $state(false);
  let root: HTMLElement | undefined = $state();

  function onWindowClick(e: MouseEvent) {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
  function choose(it: Item) {
    open = false;
    it.onSelect();
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={onKey} />

<div class="relative" bind:this={root}>
  <button
    type="button"
    onclick={() => (open = !open)}
    aria-haspopup="menu"
    aria-expanded={open}
    class="h-9 px-3 rounded-md text-[13px] font-semibold flex items-center gap-1.5 whitespace-nowrap border transition-colors
      {open ? 'bg-[#34343e] border-[#ffb31a]/60 text-white' : primary ? 'bg-[#ffb31a] border-[#ffb31a] text-[#121214] hover:bg-[#e6a117]' : 'bg-[#24242a] border-[#34343c] text-[#d8d8dc] hover:bg-[#2e2e36]'}"
  >
    {label}
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true" class="opacity-70"><path d="M2 3.5 5 6.5 8 3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
  </button>

  {#if open}
    <div
      role="menu"
      class="absolute top-full mt-1.5 z-50 min-w-60 max-w-[88vw] py-1.5 rounded-lg border border-[#34343c] bg-[#1a1a1e] shadow-2xl {align === 'right' ? 'right-0' : 'left-0'}"
    >
      {#each items as it}
        {#if it.separatorBefore}<div class="my-1 border-t border-[#34343c]"></div>{/if}
        <button
          type="button"
          role="menuitem"
          onclick={() => choose(it)}
          class="w-full text-left px-3.5 py-2 hover:bg-[#2a2a32] focus:bg-[#2a2a32] outline-none"
        >
          <div class="text-[13px] font-semibold text-[#d8d8dc]">{it.label}</div>
          {#if it.hint}<div class="text-xs text-[#82828a] leading-snug">{it.hint}</div>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
