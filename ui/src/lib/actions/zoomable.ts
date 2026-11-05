/// Zoom de un documento en pantalla: pellizco en el celular y Ctrl + rueda en la PC.
/// Se usa sobre el contenedor con scroll del documento:
///   <div use:zoomable={{ zoom, onzoom: (z) => (zoom = z) }}><div style:zoom={zoom}>...</div></div>
///
/// La app tiene el zoom del navegador apagado (toda la interfaz es de tamaño fijo): el pellizco se
/// atiende acá y solo agranda el documento. Un dedo sigue desplazando. Al imprimir no cuenta
/// (`.tg-zoom-doc` en app.css).
export const ZOOM_MIN = 0.3;
export const ZOOM_MAX = 3;
export const ZOOM_STEP = 0.1;

export function clampZoom(value: number): number {
  return Math.round(Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, value)) * 100) / 100;
}

/// Ancho de la hoja A4 en pantalla (`.tg-zoom-doc`): el documento conserva su formato y el zoom lo agranda
/// entero, como una lupa, en lugar de reacomodar el texto.
const PAGE_WIDTH = 794;

/// Zoom inicial: en la computadora, tamaño real; en el celular, la hoja entera a lo ancho de la pantalla.
export function defaultDocumentZoom(): number {
  const available = typeof window !== 'undefined' ? window.innerWidth - 32 : PAGE_WIDTH;
  return available >= PAGE_WIDTH ? 1 : clampZoom(available / PAGE_WIDTH);
}

export interface ZoomableOptions {
  zoom: number;
  onzoom: (value: number) => void;
}

export function zoomable(node: HTMLElement, options: ZoomableOptions) {
  let pinch: { distance: number; zoom: number } | null = null;

  function fingers(event: TouchEvent) {
    const [a, b] = [event.touches[0], event.touches[1]];
    return {
      distance: Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY),
      x: (a.clientX + b.clientX) / 2,
      y: (a.clientY + b.clientY) / 2,
    };
  }

  /// Cambia el zoom dejando quieto el punto del documento que está bajo los dedos (o el puntero).
  function apply(next: number, clientX: number, clientY: number) {
    const ratio = next / options.zoom;
    if (ratio === 1) return;
    const box = node.getBoundingClientRect();
    const [x, y] = [clientX - box.left, clientY - box.top];
    const [left, top] = [node.scrollLeft, node.scrollTop];
    options.onzoom(next);
    requestAnimationFrame(() => {
      node.scrollLeft = (left + x) * ratio - x;
      node.scrollTop = (top + y) * ratio - y;
    });
  }

  function onTouchStart(event: TouchEvent) {
    if (event.touches.length === 2) {
      pinch = { distance: fingers(event).distance, zoom: options.zoom };
    }
  }

  function onTouchMove(event: TouchEvent) {
    if (!pinch || event.touches.length !== 2) return;
    event.preventDefault();
    const now = fingers(event);
    apply(clampZoom((pinch.zoom * now.distance) / pinch.distance), now.x, now.y);
  }

  function onTouchEnd(event: TouchEvent) {
    if (event.touches.length < 2) pinch = null;
  }

  function onWheel(event: WheelEvent) {
    if (!event.ctrlKey) return;
    event.preventDefault();
    apply(clampZoom(options.zoom + (event.deltaY < 0 ? ZOOM_STEP : -ZOOM_STEP)), event.clientX, event.clientY);
  }

  // Al girar el celular, si el documento estaba a lo ancho de la pantalla, sigue a lo ancho.
  let fitted = defaultDocumentZoom();
  function onResize() {
    const next = defaultDocumentZoom();
    if (options.zoom === fitted) options.onzoom(next);
    fitted = next;
  }

  node.style.touchAction = 'pan-x pan-y';
  window.addEventListener('resize', onResize);
  node.addEventListener('touchstart', onTouchStart, { passive: true });
  node.addEventListener('touchmove', onTouchMove, { passive: false });
  node.addEventListener('touchend', onTouchEnd, { passive: true });
  node.addEventListener('touchcancel', onTouchEnd, { passive: true });
  node.addEventListener('wheel', onWheel, { passive: false });

  return {
    update(next: ZoomableOptions) {
      options = next;
    },
    destroy() {
      node.removeEventListener('touchstart', onTouchStart);
      node.removeEventListener('touchmove', onTouchMove);
      node.removeEventListener('touchend', onTouchEnd);
      node.removeEventListener('touchcancel', onTouchEnd);
      node.removeEventListener('wheel', onWheel);
      window.removeEventListener('resize', onResize);
    },
  };
}
