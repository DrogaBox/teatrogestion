<script lang="ts">
  import ModalHeader from './ModalHeader.svelte';
  import ZoomControls from './ZoomControls.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';
  import { zoomable, defaultDocumentZoom } from '../../actions/zoomable';
  import { printDocument } from '../../nativeOutput';

  /// Vista previa de impresión editable: clona el documento imprimible del llamador
  /// (`source`, el contenedor que la ventana marca con `bind:this`) y deja su texto
  /// editable al vuelo (contenteditable), para corregir un error antes de mandar a
  /// papel sin salir de la pantalla.
  ///
  /// - Los cambios valen SOLO para la copia impresa: se aplican sobre el clon y no
  ///   tocan los datos guardados.
  /// - «Restablecer» vuelve a clonar el documento generado (descarta las ediciones).
  /// - Si el llamador pasa `onsave`, se muestra la casilla «Guardar cambios en el
  ///   registro»: al imprimir con la casilla marcada, se llama a ese guardado (el
  ///   mismo IPC que ya usa el documento) ANTES de imprimir; si falla, no se imprime
  ///   y el error se muestra en pantalla.
  /// - Al imprimir, el resto de la app se oculta por CSS (body.tg-print-preview-open)
  ///   y sale solo el documento editado, con el mismo layout de @page de siempre.
  let {
    open,
    title,
    subtitle = '',
    source,
    onclose,
    onsave = null
  }: {
    open: boolean;
    title: string;
    subtitle?: string;
    source: HTMLElement | null;
    onclose: () => void;
    onsave?: ((doc: HTMLElement) => Promise<void>) | null;
  } = $props();

  let docHtml: string | null = $state(null);
  let docEl: HTMLElement | null = $state(null);
  let isEditing = $state(false);
  let dirty = $state(false);
  let saveChecked = $state(false);
  let saving = $state(false);
  let errorMsg = $state('');
  let zoom = $state(1);
  $effect(() => {
    if (open) {
      zoom = defaultDocumentZoom();
      isEditing = false;
    }
  });

  // La vista previa se monta como hija directa de <body> (portal) para que, al
  // imprimir, el CSS pueda ocultar todo lo demás y dejar solo el documento.
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      }
    };
  }

  function buildSnapshot(src: HTMLElement): string {
    const clone = src.cloneNode(true) as HTMLElement;
    // Controles fuera de la copia impresa: botones desaparecen y los campos se
    // vuelven texto plano con su valor.
    // También sale lo que el documento ya oculta al imprimir (`print:hidden`).
    clone.querySelectorAll('button, [data-print-skip], .print\\:hidden').forEach((el) => el.remove());
    clone.querySelectorAll('input, textarea').forEach((el) => {
      const span = document.createElement('span');
      span.textContent = (el as HTMLInputElement | HTMLTextAreaElement).value;
      el.replaceWith(span);
    });
    clone.querySelectorAll('select').forEach((el) => {
      const span = document.createElement('span');
      span.textContent = el.selectedOptions[0]?.textContent ?? '';
      el.replaceWith(span);
    });
    clone.querySelectorAll('.tg-card-table').forEach((el) => el.classList.remove('tg-card-table'));
    // Hacer visibles en la vista previa los encabezados/pies que en pantalla llevan `hidden print:block` (o `print:flex`, etc.)
    const printDisplayMap: [string, string][] = [
      ['print:block', 'block'],
      ['print:flex', 'flex'],
      ['print:grid', 'grid'],
      ['print:inline-block', 'inline-block'],
      ['print:inline', 'inline'],
      ['print:table', 'table'],
      ['print:table-row', 'table-row'],
    ];
    clone.querySelectorAll('.hidden').forEach((el) => {
      for (const [printCls, screenCls] of printDisplayMap) {
        if (el.classList.contains(printCls)) {
          el.classList.remove('hidden');
          el.classList.add(screenCls);
          break;
        }
      }
    });
    return clone.innerHTML;
  }

  function rebuild() {
    if (source) {
      docHtml = buildSnapshot(source);
    } else {
      docHtml = null;
    }
    dirty = false;
    saveChecked = false;
    errorMsg = '';
  }

  $effect(() => {
    if (open) {
      document.body.classList.add('tg-print-preview-open');
      return () => document.body.classList.remove('tg-print-preview-open');
    }
  });

  $effect(() => {
    if (open) {
      rebuild();
    }
  });

  async function doPrint() {
    if (saving) return;
    errorMsg = '';
    if (saveChecked && onsave && docEl) {
      saving = true;
      try {
        await onsave(docEl);
      } catch (e: any) {
        errorMsg = `No se pudieron guardar los cambios en el registro: ${e?.message ?? e}`;
        saving = false;
        return;
      }
      saving = false;
    }
    printDocument(title);
  }
</script>

{#if open}
  <div
    class="tg-print-preview fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6"
    use:portal
    use:modalBehavior={{ onclose, backdrop: false }}
  >
    <div class="tg-modal max-w-5xl w-full max-h-[92vh]">
      <div class="tg-print-preview-chrome">
        <ModalHeader title={title} subtitle={subtitle} onclose={onclose}>
          {#if dirty}
            <span
              class="h-6 px-2 inline-flex items-center rounded bg-amber/15 border border-amber/50 text-amber text-[10px] font-bold uppercase tracking-wider"
            >
              Editado
            </span>
          {/if}
        </ModalHeader>
        <div class="flex flex-wrap items-center justify-between gap-2 px-4 py-2 border-b border-grid bg-panel">
          <p class="text-[11px] text-dim font-mono min-w-0">
            {#if isEditing}
              Modo edición activo: tocá el texto para corregirlo solo en esta copia impresa.
            {:else}
              Vista previa de lectura. Activá «Editar texto» si necesitás retocar algo antes de imprimir.
            {/if}
          </p>
          <div class="flex flex-wrap items-center gap-2">
            <ZoomControls bind:zoom />
            <button
              type="button"
              class="tg-btn {isEditing ? 'border-amber text-amber bg-amber/15' : ''}"
              onclick={() => (isEditing = !isEditing)}
              title="Activar o desactivar edición directa sobre el documento"
            >
              {isEditing ? '✓ Editando texto' : 'Editar texto'}
            </button>
            <button
              type="button"
              class="tg-btn"
              onclick={() => rebuild()}
              disabled={!dirty}
              title="Volver al documento generado (descarta las ediciones)"
            >
              Restablecer
            </button>
            {#if onsave}
              <label class="flex items-center gap-1.5 text-[11px] text-dim font-mono select-none" title="Los cambios se aplican además al registro con el guardado habitual de este documento">
                <input type="checkbox" bind:checked={saveChecked} class="accent-amber" />
                Guardar cambios en el registro
              </label>
            {/if}
            <button
              type="button"
              class="tg-btn tg-btn-primary"
              onclick={doPrint}
              disabled={saving || docHtml === null}
            >
              {saving ? 'Guardando…' : 'Imprimir'}
            </button>
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-auto bg-white bg-local min-h-0" use:zoomable={{ zoom, onzoom: (z) => (zoom = z) }}>
        {#if docHtml !== null}
          <div
            class="tg-print-preview-doc tg-zoom-doc"
            style:zoom={zoom}
            bind:this={docEl}
            contenteditable={isEditing ? 'true' : 'false'}
            spellcheck="false"
            oninput={() => (dirty = true)}
          >
            {@html docHtml}
          </div>
        {:else}
          <div class="p-10 text-center text-xs text-dim font-mono">No hay documento para mostrar.</div>
        {/if}
      </div>

      {#if errorMsg}
        <div class="px-4 py-2 bg-red/10 border-t border-red/40 text-red text-xs font-mono">{errorMsg}</div>
      {/if}
    </div>
  </div>
{/if}
