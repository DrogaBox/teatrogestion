<script lang="ts">
  import { clampZoom, defaultDocumentZoom, ZOOM_MAX, ZOOM_MIN, ZOOM_STEP } from '../../actions/zoomable';

  let { zoom = $bindable(1) }: { zoom: number } = $props();

  const buttonClass = 'h-8 w-8 rounded bg-raised hover:bg-grid text-ink border border-grid text-sm font-bold disabled:opacity-40';
</script>

<div class="inline-flex items-center gap-1 print:hidden" data-print-skip role="group" aria-label="Zoom del documento">
  <button type="button" class={buttonClass} onclick={() => (zoom = clampZoom(zoom - ZOOM_STEP))} disabled={zoom <= ZOOM_MIN} aria-label="Achicar" title="Achicar">−</button>
  <button
    type="button"
    class="h-8 min-w-12 px-1 rounded bg-void hover:bg-raised text-ink border border-grid text-xs font-mono font-bold"
    onclick={() => (zoom = defaultDocumentZoom())}
    title="Volver al tamaño normal. También se puede pellizcar con dos dedos o usar Ctrl + rueda del mouse"
  >{Math.round(zoom * 100)}%</button>
  <button type="button" class={buttonClass} onclick={() => (zoom = clampZoom(zoom + ZOOM_STEP))} disabled={zoom >= ZOOM_MAX} aria-label="Agrandar" title="Agrandar">+</button>
</div>
