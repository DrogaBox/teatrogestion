<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type StockItemAvailabilityDto,
    type PersonnelWithQuotaDto,
    type QuotaBucket
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';
  import { cardTable } from '../../actions/cardTable';
  import NewStockItemModal from './NewStockItemModal.svelte';
  import PrintPreviewModal from '../shared/PrintPreviewModal.svelte';

  let printPreviewOpen = $state(false);
  let printableEl: HTMLElement | null = $state(null);

  let {
    isOpen,
    onClose
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let stockItems = $state<StockItemAvailabilityDto[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let selectedCategory = $state<string>('ALL');
  let lowStockOnly = $state(false);
  let showNewItemModal = $state(false);
  let editingItem = $state<StockItemAvailabilityDto | null>(null);

  // Modal de Entrega Directa de Inventario
  let showDeliverModal = $state(false);
  let deliverItem = $state<StockItemAvailabilityDto | null>(null);
  let personnelList = $state<PersonnelWithQuotaDto[]>([]);
  let deliverPersonnelId = $state<string>('');
  let deliverQty = $state<number>(1);
  let deliverNotes = $state<string>('Entrega de depósito');
  let isDelivering = $state(false);
  let deliverSuccessToast = $state(false);

  const categoryOptions = [
    { value: 'ALL', label: 'Todos los Rubros' },
    { value: 'CALZA_MASCULINA', label: 'Calzas de Varón' },
    { value: 'SUSPENSOR_MASCULINO', label: 'Suspensores' },
    { value: 'CANCAN', label: 'Cancán / Medias Mallas' },
    { value: 'MEDIAS_DANZA', label: 'Medias de Danza / Soquetes' },
    { value: 'MEDIA_PUNTA_CUERO', label: 'Media Punta Cuero' },
    { value: 'MEDIA_PUNTA_TELA', label: 'Media Punta Tela' },
    { value: 'MEDIA_PUNTA_VARON', label: 'Media Punta Varón (Sansha)' },
    { value: 'MEDIA_PUNTA_MUJER_UNISEX', label: 'Media Punta Bloch Canvas' },
    { value: 'PUNTA_GAYNOR_MINDEN', label: 'Puntas Gaynor Minden' },
    { value: 'PUNTA_BLOCH', label: 'Puntas Bloch' },
    { value: 'PUNTA_SANSHA_DUVAL', label: 'Puntas Sansha Duval' },
    { value: 'ZAPATOS_JAZZ', label: 'Zapatos de Jazz' },
    { value: 'ZAPATOS_CARACTER', label: 'Zapatos de Carácter' },
    { value: 'BOMBACHA_ESCENA', label: 'Bombachas de Escena' },
  ];

  async function loadCatalog() {
    isLoading = true;
    try {
      const items = await api.listStockCatalog({
        query: searchQuery.trim() || null,
        subcategory_str: selectedCategory === 'ALL' ? null : selectedCategory,
        low_stock_only: lowStockOnly,
      });
      stockItems = items;
    } catch (err) {
      console.error('Error al cargar inventario:', err);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      loadCatalog();
    }
  });

  function handleFilterChange() {
    loadCatalog();
  }

  function openNewItem() {
    editingItem = null;
    showNewItemModal = true;
  }

  function openEditItem(item: StockItemAvailabilityDto) {
    editingItem = item;
    showNewItemModal = true;
  }

  function handleItemSaved() {
    loadCatalog();
  }

  async function openDirectDeliver(item: StockItemAvailabilityDto) {
    deliverItem = item;
    deliverQty = 1;
    deliverNotes = `Entrega de depósito (${item.subcategory})`;
    if (personnelList.length === 0) {
      try {
        personnelList = await api.listPersonnel(null, true);
        if (personnelList.length > 0 && !deliverPersonnelId) {
          deliverPersonnelId = personnelList[0].id;
        }
      } catch (e) {
        console.error(e);
      }
    }
    showDeliverModal = true;
  }

  async function submitDirectDeliver() {
    if (!deliverItem || !deliverPersonnelId) return;
    isDelivering = true;
    try {
      let bucket: QuotaBucket = 'EXEMPT';
      if (deliverItem.family === 'POINTE') {
        bucket = 'POINTE';
      } else if (deliverItem.family === 'MENS_FLAT' || deliverItem.family === 'WOMENS_FLAT') {
        bucket = 'FLAT';
      }

      await api.executeVoucherDelivery({
        personnel_id: deliverPersonnelId,
        season_year: 2026,
        delivered_by_inspector_id: 'PER_INSPECTOR_MAIN',
        device_id: 'DESKTOP-PC',
        signature_svg_strokes: '<svg>desktop-assign</svg>',
        is_quota_exception: true,
        exception_reason: 'Asignación directa desde reporte de inventario',
        notes: deliverNotes,
        items: [
          {
            stock_item_id: deliverItem.id,
            reservation_id: null,
            quantity: deliverQty,
            quota_bucket: bucket,
          }
        ]
      });

      await loadCatalog();
      deliverSuccessToast = true;
      setTimeout(() => deliverSuccessToast = false, 3000);
      showDeliverModal = false;
    } catch (e: any) {
      alert(`Error al entregar ítem: ${e?.message || e}`);
    } finally {
      isDelivering = false;
    }
  }

  // Exportar Excel en formato compatible con Windows 11 Excel (BOM UTF-8 y delimitador ;)
  function exportExcel() {
    let csv = '\uFEFF'; // BOM UTF-8
    csv += 'TEATRO DEL LIBERTADOR SAN MARTIN - BALLET OFICIAL DE CORDOBA\n';
    csv += 'REPORTE OFICIAL DE INVENTARIO Y CONTROL DE STOCK DE DEPOSITO\n';
    csv += `FECHA DE EMISION:;${new Date().toLocaleDateString('es-AR')} ${new Date().toLocaleTimeString('es-AR')}\n\n`;
    csv += 'RUBRO;MARCA;MODELO;SKU_CODIGO;COLOR;MATERIAL;SISTEMA;TALLE;ANCHO;STOCK_FISICO;RESERVADO;LIBRE_DISPONIBLE;UBICACION;ESTADO\n';

    stockItems.forEach((it) => {
      const estado = it.is_low_stock ? 'ALERTA_STOCK_BAJO' : 'OPTIMO';
      csv += `"${it.subcategory}";"${it.brand}";"${it.model_name}";"${it.sku_code}";"${it.color}";"${it.material}";"${it.size_system}";"${it.size_value}";"${it.width_value || ''}";${it.physical_qty};${it.reserved_qty};${it.free_available_qty};"${it.storage_bin_location || ''}";"${estado}"\n`;
    });

    csv += '\n"★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★"\n';

    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', `Inventario_Oficial_Ballet_Oficial_Cordoba_${new Date().toISOString().split('T')[0]}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  // Totales calculados
  let totalFisico = $derived(stockItems.reduce((acc, it) => acc + Number(it.physical_qty || 0), 0));
  let totalReservado = $derived(stockItems.reduce((acc, it) => acc + Number(it.reserved_qty || 0), 0));
  let totalDisponible = $derived(stockItems.reduce((acc, it) => acc + Number(it.free_available_qty || 0), 0));
  let totalCritico = $derived(stockItems.filter((it) => it.is_low_stock).length);
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-2 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-6xl relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">
      
      <!-- Barra Superior (Oculta en Impresión) -->
      <div class="bg-raised border-b border-grid px-4 py-2.5 shrink-0 flex print:hidden items-start gap-3">
        <!-- cierre fijo en la esquina: el resto de la barra puede partirse en filas -->
        <div class="flex flex-1 min-w-0 flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2.5">
            <span class="max-md:hidden text-[10px] font-mono font-bold text-amber bg-void px-2 py-0.5 rounded border border-grid">INVENTARIO</span>
            <div>
              <h2 class="text-xs font-bold text-ink uppercase tracking-wider flex items-center gap-2">
                Inventario Oficial & Control de Depósito
                <span class="text-[10px] font-mono text-dim">
                  ({stockItems.length} ítems)
                </span>
              </h2>
              <p class="text-[11px] text-dim max-md:hidden">
                Ballet Oficial de Córdoba • Teatro del Libertador San Martín
              </p>
            </div>
          </div>

          <!-- Botonera de Exportación y Alta -->
          <div class="tg-bar-scroll flex items-center gap-2 flex-wrap">
            <button
              onclick={openNewItem}
              class="px-2.5 py-1 bg-amber hover:bg-amber text-void rounded text-xs font-bold transition-colors"
            >
              + Nuevo Ítem
            </button>

            <button
              onclick={exportExcel}
              class="px-2.5 py-1 bg-raised hover:bg-raised-hover text-green border border-green/40 rounded text-xs font-bold transition-colors"
              title="Exportar a Microsoft Excel con acentos legibles (BOM UTF-8)"
            >
              Exportar Excel
            </button>

            <button
              onclick={() => (printPreviewOpen = true)}
              class="px-2.5 py-1 bg-raised hover:bg-raised-hover text-ink border border-grid rounded text-xs font-bold transition-colors"
              title="Imprimir o Guardar en PDF con formato oficial"
            >
              Exportar PDF / Imprimir
            </button>

          </div>
        </div>
        <button
          onclick={onClose}
          class="tg-modal-close"
          aria-label="Cerrar"
          title="Cerrar"
        >
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
            <path d="M2 2l10 10M12 2L2 12" />
          </svg>
        </button>
      </div>

      <!-- Filtros Rápidos (Ocultos en Impresión) -->
      <div class="bg-alt px-4 py-2 border-b border-grid print:hidden space-y-2">
        <div class="grid grid-cols-1 md:grid-cols-12 gap-2 items-center">
          
          <!-- Buscador por texto -->
          <div class="md:col-span-5 relative">
            <input
              type="text"
              bind:value={searchQuery}
              oninput={handleFilterChange}
              placeholder="Buscar por SKU, modelo, marca, color o talle..."
              class="w-full h-8 pl-3 pr-8 bg-void border border-grid focus:border-amber rounded text-xs font-bold text-ink outline-none placeholder-dim"
            />
            {#if searchQuery}
              <button
                onclick={() => { searchQuery = ''; handleFilterChange(); }}
                class="absolute right-2 top-1.5 text-dim hover:text-white text-xs"
              >
                ✕
              </button>
            {/if}
          </div>

          <!-- Selector de Rubro / Subcategoría -->
          <div class="md:col-span-4">
            <select
              bind:value={selectedCategory}
              onchange={handleFilterChange}
              class="w-full h-8 px-2.5 bg-void border border-grid focus:border-amber rounded text-xs font-bold text-ink outline-none"
            >
              {#each categoryOptions as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- Filtro de Stock Bajo / Alerta -->
          <div class="md:col-span-3 flex items-center">
            <label class="inline-flex items-center gap-2 cursor-pointer text-xs font-medium text-ink">
              <input
                type="checkbox"
                bind:checked={lowStockOnly}
                onchange={handleFilterChange}
                class="w-4 h-4 text-amber rounded bg-void border-grid"
              />
              <span class="text-red font-bold">
                Solo Alerta de Stock Bajo
              </span>
            </label>
          </div>
        </div>

        <!-- Tarjetas de Resumen Rápido -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 pt-1 border-t border-grid text-center">
          <div class="bg-void p-1.5 rounded border border-grid">
            <span class="text-[9px] text-dim uppercase font-bold block">Stock Físico Total</span>
            <span class="text-xs font-bold font-mono text-ink">{totalFisico} un.</span>
          </div>
          <div class="bg-void p-1.5 rounded border border-grid">
            <span class="text-[9px] text-dim uppercase font-bold block">Reservas Nominativas</span>
            <span class="text-xs font-bold font-mono text-amber">{totalReservado} un.</span>
          </div>
          <div class="bg-void p-1.5 rounded border border-grid">
            <span class="text-[9px] text-dim uppercase font-bold block">Libre Disponible</span>
            <span class="text-xs font-bold font-mono text-green">{totalDisponible} un.</span>
          </div>
          <div class="bg-void p-1.5 rounded border border-grid">
            <span class="text-[9px] text-dim uppercase font-bold block">Líneas en Alerta</span>
            <span class="text-xs font-bold font-mono text-red">{totalCritico} ítems</span>
          </div>
        </div>
      </div>

      <!-- ================================================================= -->
      <!-- DOCUMENTO IMPRIMIBLE A4 / TABLA DE GESTIÓN OFICIAL                 -->
      <!-- ================================================================= -->
      <div bind:this={printableEl} class="flex-1 overflow-y-auto p-4 text-ink print:text-black print:overflow-visible">
        
        <!-- Encabezado Institucional Exclusivo de Impresión (PDF / Papel) -->
        <div class="hidden print:block text-center pb-4 mb-4 border-b-2 border-black">
          <h1 class="text-lg font-black tracking-tight uppercase">
            Teatro del Libertador San Martín • Ballet Oficial de Córdoba
          </h1>
          <h2 class="text-sm font-bold uppercase tracking-widest text-slate-700 mt-0.5">
            Libro Oficial de Inventario de Calzado e Indumentaria de Escenario
          </h2>
          <div class="flex justify-between items-center text-xs font-mono mt-2 pt-1 border-t border-slate-400">
            <span>Fecha y Hora de Emisión: {new Date().toLocaleDateString('es-AR')} - {new Date().toLocaleTimeString('es-AR')}</span>
            <span>Total Físico: {totalFisico} | Reservado: {totalReservado} | Disponible: {totalDisponible}</span>
          </div>
        </div>

        {#if isLoading}
          <div class="py-12 text-center text-dim font-mono text-xs flex items-center justify-center gap-2">
            Cargando catálogo de depósito desde SQLite local...
          </div>
        {:else if stockItems.length === 0}
          <div class="py-12 text-center bg-alt rounded border border-grid text-dim text-xs">
            No se encontraron ítems de inventario para el criterio seleccionado.
          </div>
        {:else}
          <div class="overflow-x-auto print:overflow-visible">
            <table use:cardTable class="tg-card-table w-full text-left border-collapse text-xs print:text-[10px] print:border print:border-black">
              <thead class="print:table-header-group">
                <tr class="bg-alt border-b border-grid text-dim uppercase tracking-wider font-mono text-[10px] print:bg-slate-200 print:text-black print:border-black">
                  <th class="py-2 px-3">Rubro</th>
                  <th class="py-2 px-3">Marca / Modelo</th>
                  <th class="py-2 px-3">Color / Mat.</th>
                  <th class="py-2 px-3">Talle</th>
                  <th class="py-2 px-2 text-center">Físico</th>
                  <th class="py-2 px-2 text-center">Res.</th>
                  <th class="py-2 px-2 text-center">Libre</th>
                  <th class="py-2 px-3">Ubicación</th>
                  <th class="py-2 px-2 text-center print:hidden">Acción</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-grid/60 print:divide-black">
                {#each stockItems as it}
                  <tr class="hover:bg-raised transition-colors print:hover:bg-transparent print:break-inside-avoid {it.is_low_stock ? 'bg-red/5 print:bg-slate-100' : 'print:bg-white'}">
                    <!-- Rubro / Subcategoría -->
                    <td class="py-2 px-3 font-bold text-ink print:text-black">
                      <div class="flex items-center gap-1.5">
                        {#if it.is_low_stock}
                          <span class="text-red text-[10px] font-mono font-bold print:hidden">[ALERTA]</span>
                          <span class="hidden print:inline-block font-mono font-black text-[8px] border border-black px-1 rounded bg-white">ALERTA</span>
                        {/if}
                        <span class="text-[11px] font-mono">{it.subcategory}</span>
                      </div>
                    </td>

                    <!-- Marca y Modelo -->
                    <td class="py-2 px-3">
                      <div class="font-bold text-ink print:text-black">{it.brand} {it.model_name}</div>
                      {#if it.raw_factory_code}
                        <div class="text-[10px] font-mono text-amber print:text-slate-700">{it.raw_factory_code}</div>
                      {:else}
                        <div class="text-[10px] font-mono text-dim print:text-slate-600">{it.sku_code}</div>
                      {/if}
                    </td>

                    <!-- Color y Material -->
                    <td class="py-2 px-3 text-dim print:text-black">
                      <div>{it.color}</div>
                      <div class="text-[10px] text-dim print:text-slate-600">{it.material}</div>
                    </td>

                    <!-- Talle y Ancho -->
                    <td class="py-2 px-3 font-mono font-bold text-ink print:text-black">
                      <span>{it.size_value}</span>
                      {#if it.width_value}
                        <span class="text-amber print:text-black">/{it.width_value}</span>
                      {/if}
                      <span class="text-[10px] text-dim block font-normal">({it.size_system})</span>
                    </td>

                    <!-- Stock Físico -->
                    <td class="py-2 px-2 text-center font-mono font-bold text-ink print:text-black">
                      {it.physical_qty}
                    </td>

                    <!-- Stock Reservado -->
                    <td class="py-2 px-2 text-center font-mono font-bold {it.reserved_qty > 0 ? 'text-amber' : 'text-dim'} print:text-black">
                      {it.reserved_qty}
                    </td>

                    <!-- Libre Disponible -->
                    <td class="py-2 px-2 text-center font-mono font-bold {it.free_available_qty > 0 ? 'text-green' : 'text-red'} print:text-black">
                      {it.free_available_qty}
                    </td>

                    <!-- Ubicación en Estante / Gaveta -->
                    <td class="py-2 px-3 text-[11px] text-dim font-mono print:text-black">
                      {it.storage_bin_location || '—'}
                    </td>

                    <!-- Botón de Entrega y Edición Rápida (Oculto en Impresión) -->
                    <td class="py-2 px-2 text-center print:hidden">
                      <div class="flex items-center justify-center gap-1">
                        <button
                          onclick={() => openDirectDeliver(it)}
                          class="px-2 py-0.5 rounded bg-raised hover:bg-green/20 text-green text-xs font-mono border border-grid"
                          title="Entregar o asignar este ítem a un bailarín o agente"
                        >
                          Entregar
                        </button>
                        <button
                          onclick={() => openEditItem(it)}
                          class="px-2 py-0.5 rounded bg-raised hover:bg-grid text-ink text-xs font-mono border border-grid"
                          title="Modificar stock o ubicación"
                        >
                          Editar
                        </button>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}

        <!-- =============================================================== -->
        <!-- PIE DOCUMENTAL INSTITUCIONAL PARA PDF / IMPRESIÓN (FIRMAS)     -->
        <!-- =============================================================== -->
        <div class="hidden print:block mt-12 pt-6 border-t-2 border-black">
          <!-- Cláusula Legal Invariable -->
          <p class="text-center font-mono font-black text-xs tracking-wider uppercase mb-8">
            ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
          </p>

          <!-- Casilleros de Firma y Aclaración Formal -->
          <div class="grid grid-cols-2 gap-12 text-center text-xs pt-8">
            <div class="border-t border-black pt-2">
              <p class="font-extrabold uppercase">Inspectora / Supervisora General</p>
              <p class="text-[10px] text-slate-600">Inspección de Escenario y Depósito — Ballet Oficial de Córdoba</p>
              <p class="text-[9px] text-slate-500 mt-1">Firma y Aclaración</p>
            </div>
            <div class="border-t border-black pt-2">
              <p class="font-extrabold uppercase">Dirección Artística</p>
              <p class="text-[10px] text-slate-600">Teatro del Libertador San Martín</p>
              <p class="text-[9px] text-slate-500 mt-1">Firma y Sello Oficial</p>
            </div>
          </div>
        </div>

      </div>

      <PrintPreviewModal
        open={printPreviewOpen}
        title="Vista previa de impresión"
        subtitle="Inventario — ediciones solo para la copia impresa"
        source={printableEl}
        onclose={() => (printPreviewOpen = false)}
      />
    </div>
  </div>
{/if}

<!-- Sub-Modal para Alta / Edición Dinámica de Stock -->
<NewStockItemModal
  isOpen={showNewItemModal}
  initialItem={editingItem}
  onClose={() => showNewItemModal = false}
  onSaved={handleItemSaved}
/>

<!-- Sub-Modal para Entrega Directa de Cualquier Ítem de Inventario -->
{#if showDeliverModal && deliverItem}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4" use:modalBehavior={{ onclose: () => showDeliverModal = false, backdrop: false }}>
    <div class="tg-modal max-w-md">
      <ModalHeader title="Entregar / Asignar Ítem" subtitle="{deliverItem.brand} {deliverItem.model_name}" onclose={() => showDeliverModal = false} />

      <div class="tg-modal-body space-y-3">
        <div class="bg-void p-3 rounded border border-grid text-xs space-y-1">
          <div class="flex justify-between">
            <span class="text-dim">Rubro:</span>
            <span class="font-medium text-ink">{deliverItem.subcategory}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-dim">Talle / Ancho:</span>
            <span class="font-mono text-ink">{deliverItem.size_value} {deliverItem.width_value || ''}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-dim">Libres en depósito:</span>
            <span class="font-mono font-bold text-green">{deliverItem.free_available_qty} unidades</span>
          </div>
        </div>

        <div>
          <label for="deliver-dest-select" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Destinatario (Bailarín / Agente)</label>
          <select
            id="deliver-dest-select"
            bind:value={deliverPersonnelId}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-amber"
          >
            {#each personnelList as p}
              <option value={p.id}>
                {p.last_name}, {p.first_name} {p.nickname ? `("${p.nickname}")` : ''} — {p.category_name}
              </option>
            {/each}
          </select>
        </div>

        <div>
          <label for="deliver-qty-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Cantidad a Entregar</label>
          <input
            id="deliver-qty-input"
            type="number"
            min="1"
            max={deliverItem.free_available_qty || 1}
            bind:value={deliverQty}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-bold text-ink font-mono text-center outline-none focus:border-amber"
          />
        </div>

        <div>
          <label for="deliver-notes-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Observaciones / Destino</label>
          <input
            id="deliver-notes-input"
            type="text"
            bind:value={deliverNotes}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs text-ink outline-none focus:border-amber"
          />
        </div>
      </div>

      <div class="tg-modal-foot">
        <button onclick={() => showDeliverModal = false} class="tg-btn">Cancelar</button>
        <button
          disabled={isDelivering || deliverItem.free_available_qty < 1}
          onclick={submitDirectDeliver}
          class="tg-btn tg-btn-primary disabled:opacity-40"
        >
          {isDelivering ? 'Registrando...' : 'Confirmar Entrega'}
        </button>
      </div>
    </div>
  </div>
{/if}
