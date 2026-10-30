<script lang="ts">
  import type { Snippet } from 'svelte';

  /// Cabecera única para todas las ventanas y diálogos de la app:
  /// título a la izquierda, contenido extra opcional (pestañas, botones) y el
  /// botón de cierre de 28×28 siempre en la misma esquina superior derecha.
  /// El Escape y el clic en el fondo los maneja la acción `modalBehavior`.
  let {
    title,
    subtitle = '',
    onclose,
    children,
    class: klass = ''
  }: {
    title: string;
    subtitle?: string;
    onclose: () => void;
    children?: Snippet;
    class?: string;
  } = $props();
</script>

<div class="tg-modal-head {klass}">
  <!-- La cruz queda fuera de este bloque: aunque los controles pasen a otra fila, sigue en la esquina -->
  <div class="flex flex-1 min-w-0 min-h-7 flex-wrap items-center gap-x-3 gap-y-2">
    <div class="min-w-[14rem] flex-1">
      <h2 class="tg-modal-title">{title}</h2>
      {#if subtitle}
        <p class="tg-modal-sub">{subtitle}</p>
      {/if}
    </div>
    {@render children?.()}
  </div>
  <button type="button" class="tg-modal-close" aria-label="Cerrar" title="Cerrar" onclick={onclose}>
    <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
      <path d="M2 2l10 10M12 2L2 12" />
    </svg>
  </button>
</div>
