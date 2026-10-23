<script lang="ts">
  import { CHANGELOG } from '../changelog';
  let { isOpen, onClose } = $props<{ isOpen: boolean; onClose: () => void }>();
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6" role="dialog" aria-modal="true" aria-label="Novedades">
    <div class="w-full max-w-2xl max-h-[90vh] flex flex-col bg-[#1a1a1e] border border-[#34343c] rounded-lg shadow-2xl overflow-hidden text-[#d8d8dc]">
      <div class="px-5 py-4 border-b border-[#34343c] flex items-center justify-between gap-3 bg-[#24242a]">
        <div>
          <h2 class="text-base font-bold">Novedades de TeatroGestión</h2>
          <p class="text-xs text-[#82828a]">Qué se agregó y qué se arregló en cada versión</p>
        </div>
        <button type="button" onclick={onClose} class="w-9 h-9 rounded bg-[#1a1a1e] border border-[#34343c] text-[#82828a] hover:text-white" title="Cerrar">✕</button>
      </div>

      <div class="flex-1 overflow-y-auto p-5 space-y-6">
        {#each CHANGELOG as v}
          <section>
            <div class="flex items-baseline gap-2 flex-wrap">
              <span class="px-2 py-0.5 rounded bg-[#ffb31a] text-[#121214] text-xs font-bold font-mono">Versión {v.version}</span>
              <span class="text-xs text-[#82828a] font-mono">{v.date}</span>
            </div>
            <h3 class="mt-1.5 text-sm font-bold text-white">{v.title}</h3>

            <h4 class="mt-3 text-xs font-bold uppercase tracking-wider text-[#3de06a]">Nuevo</h4>
            <ul class="mt-1.5 space-y-1.5 text-[13px] leading-snug list-disc pl-5 marker:text-[#3de06a]">
              {#each v.added as t}<li>{t}</li>{/each}
            </ul>

            <h4 class="mt-3 text-xs font-bold uppercase tracking-wider text-[#2ed6e8]">Arreglado y mejorado</h4>
            <ul class="mt-1.5 space-y-1.5 text-[13px] leading-snug list-disc pl-5 marker:text-[#2ed6e8]">
              {#each v.fixed as t}<li>{t}</li>{/each}
            </ul>
          </section>
        {/each}
      </div>

      <div class="px-5 py-3 border-t border-[#34343c] flex justify-end">
        <button type="button" onclick={onClose} class="px-5 py-2 rounded bg-[#ffb31a] hover:bg-[#e6a117] text-[#121214] text-sm font-bold">Entendido</button>
      </div>
    </div>
  </div>
{/if}
