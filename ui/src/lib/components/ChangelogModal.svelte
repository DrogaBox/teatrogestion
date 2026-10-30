<script lang="ts">
  import { CHANGELOG } from '../changelog';
  import ModalHeader from './shared/ModalHeader.svelte';
  import { modalBehavior } from '../actions/modalBehavior';
  let { isOpen, onClose } = $props<{ isOpen: boolean; onClose: () => void }>();
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6" role="dialog" aria-modal="true" aria-label="Novedades" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-2xl text-ink">
      <ModalHeader title="Novedades de TeatroGestión" subtitle="Qué se agregó y qué se arregló en cada versión" onclose={onClose} />

      <div class="flex-1 overflow-y-auto p-5 space-y-6">
        {#each CHANGELOG as v}
          <section>
            <div class="flex items-baseline gap-2 flex-wrap">
              <span class="px-2 py-0.5 rounded bg-amber text-void text-xs font-bold font-mono">Versión {v.version}</span>
              <span class="text-xs text-dim font-mono">{v.date}</span>
            </div>
            <h3 class="mt-1.5 text-sm font-bold text-white">{v.title}</h3>

            <h4 class="mt-3 text-xs font-bold uppercase tracking-wider text-green">Nuevo</h4>
            <ul class="mt-1.5 space-y-1.5 text-[13px] leading-snug list-disc pl-5 marker:text-green">
              {#each v.added as t}<li>{t}</li>{/each}
            </ul>

            <h4 class="mt-3 text-xs font-bold uppercase tracking-wider text-cyan">Arreglado y mejorado</h4>
            <ul class="mt-1.5 space-y-1.5 text-[13px] leading-snug list-disc pl-5 marker:text-cyan">
              {#each v.fixed as t}<li>{t}</li>{/each}
            </ul>
          </section>
        {/each}
      </div>

      <div class="tg-modal-foot">
        <button type="button" onclick={onClose} class="tg-btn tg-btn-primary px-5">Entendido</button>
      </div>
    </div>
  </div>
{/if}
