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
      {open ? 'bg-raised-hover] border-amber]/60 text-white' : primary ? 'bg-amber] border-amber] text-void] hover:bg-amber]' : 'bg-raised] border-grid] text-ink] hover:bg-raised-hover]'}"
  >
    {label}
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true" class="opacity-70"><path d="M2 3.5 5 6.5 8 3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
  </button>

  {#if open}
    <div
      role="menu"
      class="absolute top-full mt-1.5 z-50 min-w-60 max-w-[88vw] py-1.5 rounded border border-grid] bg-panel]  {align === 'right' ? 'right-0' : 'left-0'}"
    >
      {#each items as it}
        {#if it.separatorBefore}<div class="my-1 border-t border-grid]"></div>{/if}
        <button
          type="button"
          role="menuitem"
          onclick={() => choose(it)}
          class="w-full text-left px-3.5 py-2 hover:bg-raised] focus:bg-raised] outline-none"
        >
          <div class="text-[13px] font-semibold text-ink]">{it.label}</div>
          {#if it.hint}<div class="text-xs text-dim] leading-snug">{it.hint}</div>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
