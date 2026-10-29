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
  <div class="min-w-0 flex-1">
    <h2 class="tg-modal-title">{title}</h2>
    {#if subtitle}
      <p class="tg-modal-sub">{subtitle}</p>
    {/if}
  </div>
  {@render children?.()}
  <button type="button" class="tg-modal-close" aria-label="Cerrar" title="Cerrar" onclick={onclose}>
    <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
      <path d="M2 2l10 10M12 2L2 12" />
    </svg>
  </button>
</div>
