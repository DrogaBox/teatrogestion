<script lang="ts">
  import {
    api,
    type PersonnelWithQuotaDto,
    type StockItemAvailabilityDto,
    type VoucherReceiptDto,
    type QuotaBucket
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let {
    dancer,
    onClose,
    onDelivered
  } = $props<{
    dancer: PersonnelWithQuotaDto;
    onClose: () => void;
    onDelivered: () => void;
  }>();

  let catalog = $state<StockItemAvailabilityDto[]>([]);
  let selectedItem = $state<StockItemAvailabilityDto | null>(null);
  let quantity = $state<number>(1);
  let isException = $state<boolean>(false);
  let exceptionReason = $state<string>('');
  let isSubmitting = $state<boolean>(false);
  let receipt = $state<VoucherReceiptDto | null>(null);
  let errorMsg = $state<string | null>(null);

  // Cargar catálogo de stock disponible
  async function loadStock() {
    try {
      catalog = await api.listStockCatalog({ low_stock_only: false });
      // Pre-seleccionar si coincide con su código habitual
      if (dancer.default_pointe_raw_code) {
        const found = catalog.find(i => i.raw_factory_code === dancer.default_pointe_raw_code);
        if (found) selectedItem = found;
      }
      if (!selectedItem && catalog.length > 0) {
        selectedItem = catalog[0];
      }
    } catch (e: any) {
      errorMsg = e?.message || 'Error cargando stock';
    }
  }

  loadStock();

  // Detectar si el bailarín tiene una reserva nominativa especial por apodo
  let reservationPreset = $derived.by(() => {
    if (dancer.nickname === 'Popi') {
      return {
        id: 'RES_INIT_POPI_01',
        label: 'Reserva Activa: Popi (2 pares Gaynor Sculpted 7.5W4PDH)',
        stock_sku: 'GM-EUR-SC-7.5W4PDH',
        qty: 2,
        bucket: 'POINTE' as QuotaBucket,
      };
    } else if (dancer.nickname === 'Brenda') {
      return {
        id: 'RES_INIT_BRENDA_01',
        label: 'Reserva Activa: Brenda (2 pares Gaynor Sleek 8.5M4HDL)',
        stock_sku: 'GM-LYR-SK-8.5M4HDL',
        qty: 2,
        bucket: 'POINTE' as QuotaBucket,
      };
    }
    return null;
  });

  async function executeDirectReservation() {
    if (!reservationPreset) return;
    isSubmitting = true;
    errorMsg = null;
    try {
      const item = catalog.find(i => i.sku_code === reservationPreset?.stock_sku) || catalog[0];
      const res = await api.executeVoucherDelivery({
        personnel_id: dancer.id,
        season_year: 2026,
        delivered_by_inspector_id: 'PER_INSPECTOR_MAIN',
        device_id: 'GALAXY-A53',
        signature_svg_strokes: '<svg>touch-sign</svg>',
        is_quota_exception: false,
        exception_reason: null,
        notes: `Entrega rápida de reserva nominativa (${reservationPreset.label})`,
        items: [
          {
            stock_item_id: item.id,
            reservation_id: reservationPreset.id,
            quantity: reservationPreset.qty,
            quota_bucket: reservationPreset.bucket,
          }
        ]
      });
      receipt = res;
      onDelivered();
    } catch (e: any) {
      errorMsg = e?.message || 'Error ejecutando valera';
    } finally {
      isSubmitting = false;
    }
  }

  async function executeDelivery() {
    if (!selectedItem) return;
    isSubmitting = true;
    errorMsg = null;
    try {
      let bucket: QuotaBucket = 'EXEMPT';
      if (selectedItem.family === 'POINTE') {
        bucket = isException ? 'EXEMPT' : 'POINTE';
      } else if (selectedItem.family === 'MENS_FLAT' || selectedItem.family === 'WOMENS_FLAT') {
        bucket = isException ? 'EXEMPT' : 'FLAT';
      } else {
        bucket = 'EXEMPT';
      }

      const res = await api.executeVoucherDelivery({
        personnel_id: dancer.id,
        season_year: 2026,
        delivered_by_inspector_id: 'PER_INSPECTOR_MAIN',
        device_id: 'GALAXY-A53',
        signature_svg_strokes: '<svg>touch-sign</svg>',
        is_quota_exception: isException,
        exception_reason: isException ? exceptionReason : null,
        notes: `Entrega de depósito (${selectedItem.subcategory})`,
        items: [
          {
            stock_item_id: selectedItem.id,
            reservation_id: null,
            quantity,
            quota_bucket: bucket,
          }
        ]
      });
      receipt = res;
      onDelivered();
    } catch (e: any) {
      errorMsg = e?.message || 'Error ejecutando valera';
    } finally {
      isSubmitting = false;
    }
  }
</script>

<div class="fixed inset-0 z-50 bg-black/80 flex items-end sm:items-center justify-center p-0 sm:p-4" use:modalBehavior={{ onclose: onClose, backdrop: false }}>
  <div class="tg-modal w-full sm:max-w-lg rounded-t-md sm:rounded-md text-ink]">
    <!-- Header -->
    <ModalHeader title={dancer.last_name + ', ' + dancer.first_name + (dancer.nickname ? ` ("${dancer.nickname})` : '')} subtitle="Entrega y Asignación de Inventario — {dancer.category_name}" onclose={onClose} />

    <!-- Contenido desplazable -->
    <div class="tg-modal-body">

    <!-- Saldo de Cupos en Vivo -->
    <div class="bg-void] p-3 rounded border border-grid] mb-3">
      <div class="text-[10px] font-semibold text-dim] mb-2 uppercase tracking-wider">Saldo de Cupo Anual 2026</div>
      <div class="space-y-2">
        <div>
          <div class="flex justify-between text-xs mb-1 font-medium">
            <span>Puntas Clásicas / Gaynor:</span>
            <span class="text-green] font-mono font-bold">{dancer.remaining_pointe_qty} de {dancer.effective_pointe_quota} restantes</span>
          </div>
          <div class="w-full bg-panel] h-1.5 rounded overflow-hidden border border-grid]">
            <div
              class="bg-green] h-full rounded transition-all"
              style="width: {(dancer.remaining_pointe_qty / (dancer.effective_pointe_quota || 1)) * 100}%"
            ></div>
          </div>
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1 font-medium">
            <span>Media Puntas:</span>
            <span class="text-amber] font-mono font-bold">{dancer.remaining_flat_qty} de {dancer.effective_flat_quota} restantes</span>
          </div>
          <div class="w-full bg-panel] h-1.5 rounded overflow-hidden border border-grid]">
            <div
              class="bg-amber] h-full rounded transition-all"
              style="width: {(dancer.remaining_flat_qty / (dancer.effective_flat_quota || 1)) * 100}%"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <!-- ÉXITO: Comprobante de Entrega -->
    {#if receipt}
      <div class="bg-green]/10 border border-green]/40 rounded p-4 mb-4 text-center">
        <div class="text-xs font-semibold uppercase tracking-wider text-green]">Entrega Exitosa Registrada</div>
        <div class="text-xl font-mono font-bold text-white my-1">{receipt.voucher_number}</div>
        <div class="text-xs text-ink]">{receipt.message}</div>
        <div class="text-[10px] text-dim] mt-2 font-mono">Dispositivo: GALAXY-A53 • SQLite WAL</div>
        <button
          onclick={onClose}
          class="w-full mt-3 h-10 bg-green] hover:bg-green] text-black font-semibold text-xs rounded transition-colors"
        >
          Cerrar & Volver al Depósito
        </button>
      </div>
    {:else}
      <!-- ATALLO 1 TOQUE: Si tiene Reserva Nominativa Previa -->
      {#if reservationPreset}
        <div class="bg-amber]/10 border border-amber]/40 rounded p-3 mb-3">
          <div class="text-[10px] font-semibold uppercase tracking-wider text-amber] mb-1">Reserva Nominativa Detectada</div>
          <div class="text-xs font-medium text-ink] mb-2">{reservationPreset.label}</div>
          <button
            disabled={isSubmitting}
            onclick={executeDirectReservation}
            class="w-full h-10 bg-amber] hover:bg-amber] text-black font-semibold text-xs rounded transition-colors flex items-center justify-center disabled:opacity-50 uppercase tracking-wider"
          >
            {#if isSubmitting}
              <span>Registrando...</span>
            {:else}
              <span>Entregar Reserva (1 Toque)</span>
            {/if}
          </button>
        </div>
      {/if}

      <!-- O Entrega Estándar desde Catálogo -->
      <div class="space-y-3 mb-3">
        <div>
          <label for="voucher-catalog-select" class="block text-[10px] font-semibold uppercase tracking-wider text-dim] mb-1">Ítem del Depósito</label>
          <select
            id="voucher-catalog-select"
            bind:value={selectedItem}
            class="w-full h-10 px-3 bg-void] border border-grid] rounded text-xs font-medium text-ink] outline-none focus:border-amber]"
          >
            {#each catalog as item}
              <option value={item}>
                {item.brand} {item.model_name} {item.raw_factory_code ? `(${item.raw_factory_code})` : ''} — Talle {item.size_value} {item.width_value || ''} [Libres: {item.free_available_qty}]
              </option>
            {/each}
          </select>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="voucher-qty-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim] mb-1">Cantidad</label>
            <input
              id="voucher-qty-input"
              type="number"
              min="1"
              max="5"
              bind:value={quantity}
              class="w-full h-10 px-3 bg-void] border border-grid] rounded text-sm font-mono font-bold text-ink] text-center outline-none focus:border-amber]"
            />
          </div>

          <div>
            <span class="block text-[10px] font-semibold uppercase tracking-wider text-dim] mb-1">Disponibilidad</span>
            <div class="w-full h-10 bg-void] border border-grid] rounded flex items-center justify-center font-mono font-medium text-xs text-dim]">
              {selectedItem?.free_available_qty || 0} pares libres
            </div>
          </div>
        </div>

        <!-- Excepción de Cupo -->
        <div class="pt-1">
          <label class="flex items-center gap-2 cursor-pointer text-xs font-medium text-dim]">
            <input type="checkbox" bind:checked={isException} class="h-3.5 w-3.5 rounded accent-amber]" />
            <span>Autorizar como excepción de cupo anual</span>
          </label>
          {#if isException}
            <input
              type="text"
              bind:value={exceptionReason}
              placeholder="Motivo de la excepción (mínimo 5 letras, ej. Reposición de función)"
              class="w-full mt-2 p-2.5 bg-void] border border-amber]/50 rounded text-xs text-ink] outline-none focus:border-amber]"
            />
          {/if}
        </div>
      </div>

      {#if errorMsg}
        <div class="bg-red]/10 border border-red]/40 text-red] p-2.5 rounded text-xs font-medium mb-3">
          {errorMsg}
        </div>
      {/if}

      <!-- Botón de Entrega Directa -->
      <button
        disabled={isSubmitting || !selectedItem}
        onclick={executeDelivery}
        class="w-full h-10 bg-raised] hover:bg-grid] active:scale-95 text-ink] font-semibold text-xs rounded transition-colors flex items-center justify-center border border-grid] disabled:opacity-50"
      >
        {#if isSubmitting}
          <span>Procesando...</span>
        {:else}
          <span>Confirmar Entrega de Calzado</span>
        {/if}
      </button>
    {/if}
    </div>
  </div>
</div>
