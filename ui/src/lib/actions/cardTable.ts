/// Tabla que en pantallas angostas se muestra como tarjetas (una por fila): copia el
/// texto de cada encabezado al atributo `data-label` de las celdas de su columna y el
/// CSS (`.tg-card-table` en app.css) lo muestra como etiqueta. En escritorio no cambia
/// nada. Se vuelve a aplicar cuando cambian las filas.
export function cardTable(node: HTMLTableElement) {
  const apply = () => {
    const heads = Array.from(node.querySelectorAll('thead th')).map((th) => th.textContent?.trim() ?? '');
    node.querySelectorAll('tbody tr').forEach((tr) => {
      Array.from(tr.children).forEach((cell, i) => {
        if (heads[i]) cell.setAttribute('data-label', heads[i]);
      });
    });
  };
  apply();
  const observer = new MutationObserver(apply);
  observer.observe(node, { childList: true, subtree: true });
  return {
    destroy() {
      observer.disconnect();
    }
  };
}
