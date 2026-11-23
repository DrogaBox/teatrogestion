<script lang="ts">
  import {
    api,
    type ActiveReservationDto,
    type PersonnelWithQuotaDto,
    type StockItemAvailabilityDto,
    type VoucherReceiptDto,
    type QuotaBucket
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let {
    dancer,
    initialStockItem = null,
    onClose,
    onDelivered
  } = $props<{
    dancer: PersonnelWithQuotaDto;
    initialStockItem?: StockItemAvailabilityDto | null;
    onClose: () => void;
    onDelivered: () => void;
  }>();

  let catalog = $state<StockItemAvailabilityDto[]>([]);
  let activeReservations = $state<ActiveReservationDto[]>([]);
  let selectedItem = $state<StockItemAvailabilityDto | null>(null);
  let quantity = $state<number>(1);
  let isException = $state<boolean>(false);
  let exceptionReason = $state<string>('');
  let isSubmitting = $state<boolean>(false);
  let receipt = $state<VoucherReceiptDto | null>(null);
  let errorMsg = $state<string | null>(null);

  // Cargar catálogo de stock disponible y reservas activas reales desde SQLite
  async function loadStock() {
    try {
      const [items, reservations] = await Promise.all([
        api.listStockCatalog({ low_stock_only: false }),
        api.listActiveReservations(dancer.id),
      ]);
      catalog = items;
      activeReservations = reservations;
      // 1. Pre-seleccionar ítem escaneado / pasado explícitamente si existe
      if (initialStockItem) {
        const byId = catalog.find(i => i.id === initialStockItem.id);
        if (byId) selectedItem = byId;
      }
      // 2. Si no, pre-seleccionar si coincide con su código habitual
      if (!selectedItem && dancer.default_pointe_raw_code) {
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

  async function executeDirectReservation(reservation: ActiveReservationDto) {
    isSubmitting = true;
    errorMsg = null;
    try {
      const label = `${reservation.quantity} × ${reservation.brand} ${reservation.model_name} (Talle ${reservation.size_value})`;
      const res = await api.executeVoucherDelivery({
        personnel_id: dancer.id,
        season_year: null, // la temporada en curso la pone el backend
        delivered_by_inspector_id: 'PER_INSPECTOR_MAIN',
        device_id: 'GALAXY-A53',
        signature_svg_strokes: '<svg>touch-sign</svg>',
        is_quota_exception: false,
        exception_reason: null,
        notes: `Entrega rápida de reserva nominativa (${label})`,
        items: [
          {
            stock_item_id: reservation.stock_item_id,
            reservation_id: reservation.id,
            quantity: reservation.quantity,
            quota_bucket: reservation.quota_bucket,
          }
        ]
      });
      receipt = res;
      activeReservations = activeReservations.filter(r => r.id !== reservation.id);
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
        season_year: null, // la temporada en curso la pone el backend
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
  <div class="tg-modal w-full sm:max-w-lg rounded-t-md sm:rounded-md text-ink">
    <!-- Header -->
    <ModalHeader title={dancer.last_name + ', ' + dancer.first_name + (dancer.nickname ? ` ("${dancer.nickname}")` : '')} subtitle="Entrega y Asignación de Inventario — {dancer.category_name}" onclose={onClose} />

    <!-- Contenido desplazable -->
    <div class="tg-modal-body">

    <!-- Saldo de Cupos en Vivo -->
    <div class="bg-void p-3 rounded border border-grid mb-3">
      <div class="text-[10px] font-semibold text-dim mb-2 uppercase tracking-wider">Saldo de Cupo Anual {new Date().getFullYear()}</div>
      <div class="space-y-2">
        <div>
          <div class="flex justify-between text-xs mb-1 font-medium">
            <span>Puntas Clásicas / Gaynor:</span>
            <span class="text-green font-mono font-bold">{dancer.remaining_pointe_qty} de {dancer.effective_pointe_quota} restantes</span>
          </div>
          <div class="w-full bg-panel h-1.5 rounded overflow-hidden border border-grid">
            <div
              class="bg-green h-full rounded transition-all"
              style="width: {(dancer.remaining_pointe_qty / (dancer.effective_pointe_quota || 1)) * 100}%"
            ></div>
          </div>
        </div>

        <div>
          <div class="flex justify-between text-xs mb-1 font-medium">
            <span>Media Puntas:</span>
            <span class="text-amber font-mono font-bold">{dancer.remaining_flat_qty} de {dancer.effective_flat_quota} restantes</span>
          </div>
          <div class="w-full bg-panel h-1.5 rounded overflow-hidden border border-grid">
            <div
              class="bg-amber h-full rounded transition-all"
              style="width: {(dancer.remaining_flat_qty / (dancer.effective_flat_quota || 1)) * 100}%"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <!-- ÉXITO: Comprobante de Entrega -->
    {#if receipt}
      <div class="bg-green/10 border border-green/40 rounded p-4 mb-4 text-center">
        <div class="text-xs font-semibold uppercase tracking-wider text-green">Entrega Exitosa Registrada</div>
        <div class="text-xl font-mono font-bold text-white my-1">{receipt.voucher_number}</div>
        <div class="text-xs text-ink">{receipt.message}</div>
        <button
          onclick={onClose}
          class="w-full mt-3 h-10 bg-green hover:bg-green text-black font-semibold text-xs rounded transition-colors"
        >
          Cerrar & Volver al Depósito
        </button>
      </div>
    {:else}
      <!-- ATAJO 1 TOQUE: Reservas Nominativas Activas desde SQLite -->
      {#if activeReservations.length > 0}
        <div class="space-y-2 mb-3">
          {#each activeReservations as res (res.id)}
            <div class="bg-amber/10 border border-amber/40 rounded p-3">
              <div class="text-[10px] font-semibold uppercase tracking-wider text-amber mb-1">Reserva Nominativa Activa</div>
              <div class="text-xs font-medium text-ink mb-1">
                {res.quantity} {res.quantity === 1 ? 'par' : 'pares'} · {res.brand} {res.model_name}
                {res.raw_factory_code ? ` (${res.raw_factory_code})` : ''} — Talle {res.size_value} {res.width_value || ''}
              </div>
              {#if res.notes}
                <div class="text-[11px] text-dim mb-2">{res.notes}</div>
              {/if}
              <button
                disabled={isSubmitting}
                onclick={() => executeDirectReservation(res)}
                class="w-full h-10 bg-amber hover:bg-amber text-black font-semibold text-xs rounded transition-colors flex items-center justify-center disabled:opacity-50 uppercase tracking-wider"
              >
                {#if isSubmitting}
                  <span>Registrando...</span>
                {:else}
                  <span>Entregar Reserva (1 Toque)</span>
                {/if}
              </button>
            </div>
          {/each}
        </div>
      {/if}

      <!-- O Entrega Estándar desde Catálogo -->
      <div class="space-y-3 mb-3">
        <div>
          <label for="voucher-catalog-select" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Ítem del Depósito</label>
          <select
            id="voucher-catalog-select"
            bind:value={selectedItem}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-amber"
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
            <label for="voucher-qty-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Cantidad</label>
            <input
              id="voucher-qty-input"
              type="number"
              min="1"
              max="5"
              bind:value={quantity}
              class="w-full h-10 px-3 bg-void border border-grid rounded text-sm font-mono font-bold text-ink text-center outline-none focus:border-amber"
            />
          </div>

          <div>
            <span class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Disponibilidad</span>
            <div class="w-full h-10 bg-void border border-grid rounded flex items-center justify-center font-mono font-medium text-xs text-dim">
              {selectedItem?.free_available_qty || 0} pares libres
            </div>
          </div>
        </div>

        <!-- Excepción de Cupo -->
        <div class="pt-1">
          <label class="flex items-center gap-2 cursor-pointer text-xs font-medium text-dim">
            <input type="checkbox" bind:checked={isException} class="h-3.5 w-3.5 rounded accent-amber" />
            <span>Autorizar como excepción de cupo anual</span>
          </label>
          {#if isException}
            <input
              type="text"
              bind:value={exceptionReason}
              placeholder="Motivo de la excepción (mínimo 5 letras, ej. Reposición de función)"
              class="w-full mt-2 p-2.5 bg-void border border-amber/50 rounded text-xs text-ink outline-none focus:border-amber"
            />
          {/if}
        </div>
      </div>

      {#if errorMsg}
        <div class="bg-red/10 border border-red/40 text-red p-2.5 rounded text-xs font-medium mb-3">
          {errorMsg}
        </div>
      {/if}

      <!-- Botón de Entrega Directa -->
      <button
        disabled={isSubmitting || !selectedItem}
        onclick={executeDelivery}
        class="w-full h-10 bg-raised hover:bg-grid active:scale-95 text-ink font-semibold text-xs rounded transition-colors flex items-center justify-center border border-grid disabled:opacity-50"
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
