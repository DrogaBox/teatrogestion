/// Comportamiento compartido de ventanas y diálogos de toda la app.
/// Se usa como acción Svelte sobre el fondo del modal (el `fixed inset-0 z-50`):
///   <div class="fixed inset-0 z-50 ..." use:modalBehavior={{ onclose: cerrar, backdrop: false }}>
///
/// - Escape cierra siempre.
/// - Clic en el fondo cierra, salvo que `backdrop: false` (formularios con datos sin guardar).
/// - Atrapa el foco con Tab dentro del diálogo.
/// - Al cerrar, devuelve el foco al elemento que abrió la ventana.
export interface ModalBehaviorOptions {
  onclose: () => void;
  /** false: el clic en el fondo no cierra. Por defecto true. */
  backdrop?: boolean;
}

const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])'
].join(', ');

export function modalBehavior(node: HTMLElement, options: ModalBehaviorOptions) {
  const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  node.setAttribute('tabindex', '-1');

  function focusables(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (el) => el.offsetParent !== null
    );
  }

  function focusFirstField() {
    const candidates = focusables().filter((el) => !el.classList.contains('tg-modal-close'));
    (candidates[0] ?? node).focus({ preventScroll: true });
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      options.onclose();
      return;
    }
    if (event.key === 'Tab') {
      const items = focusables();
      if (items.length === 0) {
        event.preventDefault();
        return;
      }
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;
      if (event.shiftKey) {
        if (active === first || active === node) {
          event.preventDefault();
          last.focus();
        }
      } else if (active === last || active === node) {
        event.preventDefault();
        first.focus();
      }
    }
  }

  function onClick(event: MouseEvent) {
    if (options.backdrop !== false && event.target === node) {
      options.onclose();
    }
  }

  node.addEventListener('keydown', onKeydown);
  node.addEventListener('mousedown', onClick);
  focusFirstField();

  return {
    destroy() {
      node.removeEventListener('keydown', onKeydown);
      node.removeEventListener('mousedown', onClick);
      opener?.focus({ preventScroll: true });
    }
  };
}
