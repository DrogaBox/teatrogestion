<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type StockItemAvailabilityDto,
    type UpsertStockItemPayload
  } from '../../ipc/client';
  import NewStockItemModal from '../desktop/NewStockItemModal.svelte';

  let items = $state<StockItemAvailabilityDto[]>([]);
  let searchQuery = $state('');
  let categoryFilter = $state<'ALL' | 'POINTE' | 'FLAT' | 'APPAREL' | 'LOW'>('ALL');
  let isLoading = $state(false);
  let showNewItemModal = $state(false);
  let updatingItemId = $state<string | null>(null);

  async function loadStock() {
    isLoading = true;
    try {
      const results = await api.listStockCatalog({
        query: searchQuery.trim() || undefined,
        low_stock_only: categoryFilter === 'LOW' ? true : undefined,
      });
      items = results;
    } catch (e) {
      console.error('Error cargando catálogo de depósito:', e);
    } finally {
      isLoading = false;
    }
  }

  // Filtrado reactivo en memoria para categorías específicas
  const filteredItems = $derived(
    items.filter((item) => {
      if (categoryFilter === 'POINTE') {
        return (
          item.subcategory.includes('PUNTA') &&
          !item.subcategory.includes('MEDIA')
        );
      }
      if (categoryFilter === 'FLAT') {
        return item.subcategory.includes('MEDIA_PUNTA');
      }
      if (categoryFilter === 'APPAREL') {
        return item.family === 'APPAREL' || item.subcategory.includes('MALLA');
      }
      if (categoryFilter === 'LOW') {
        return item.physical_qty < 3 || item.is_low_stock;
      }
      return true;
    })
  );

  async function adjustStock(item: StockItemAvailabilityDto, delta: number) {
    updatingItemId = item.id;
    try {
      const newQty = Math.max(0, Number(item.physical_qty) + delta);
      const payload: UpsertStockItemPayload = {
        id: item.id,
        sku_code: item.sku_code,
        family: item.family,
        subcategory: item.subcategory,
        quota_bucket: 'EXEMPT',
        brand: item.brand,
        model_name: item.model_name,
        raw_factory_code: item.raw_factory_code,
        color: item.color,
        material: item.material,
        size_system: item.size_system,
        size_value: item.size_value,
        width_value: item.width_value,
        physical_qty: newQty,
        min_alert_qty: 2,
        storage_bin_location: item.storage_bin_location,
      };

      const updated = await api.upsertStockItem(payload);
      items = items.map((i) => (i.id === updated.id ? updated : i));
    } catch (e: any) {
      alert(`Error al ajustar stock: ${e?.message || e}`);
    } finally {
      updatingItemId = null;
    }
  }

  onMount(() => {
    loadStock();
  });
</script>

<div class="space-y-3">
  <!-- Cabecera de Depósito con Botón de Alta -->
  <div class="flex items-center justify-between gap-2">
    <div>
      <h3 class="text-xs font-bold uppercase tracking-wider text-[#d8d8dc]">
        Depósito de Calzado & Stock
      </h3>
      <p class="text-[10px] text-[#82828a] font-mono">
        {filteredItems.length} {filteredItems.length === 1 ? 'ítem disponible' : 'ítems en catálogo'}
      </p>
    </div>
    <button
      onclick={() => (showNewItemModal = true)}
      class="h-7 px-2.5 bg-[#ffb31a] hover:bg-[#e6a117] text-[#121214] font-bold text-xs rounded transition-colors flex items-center gap-1 shrink-0"
    >
      <span>+</span>
      <span>Nuevo Producto</span>
    </button>
  </div>

  <!-- Buscador Táctil con Limpieza Rápida -->
  <div class="relative">
    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-[#ffb31a]">
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
    </div>
    <input
      type="text"
      bind:value={searchQuery}
      oninput={() => loadStock()}
      placeholder="Filtrar por marca, modelo o código..."
      class="w-full h-9 pl-9 pr-9 bg-[#121214] border border-[#34343c] focus:border-[#ffb31a] rounded text-xs font-medium text-[#d8d8dc] placeholder-[#82828a] outline-none transition-colors"
    />
    {#if searchQuery}
      <button
        onclick={() => { searchQuery = ''; loadStock(); }}
        class="absolute right-1 top-0.5 h-8 w-8 flex items-center justify-center text-[#82828a] hover:text-white font-mono text-xs font-bold"
      >
        ✕
      </button>
    {/if}
  </div>

  <!-- Píldoras de Filtro por Categoría -->
  <div class="flex items-center gap-1.5 overflow-x-auto pb-0.5 no-scrollbar text-xs">
    <button
      onclick={() => { categoryFilter = 'ALL'; loadStock(); }}
      class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors whitespace-nowrap {categoryFilter === 'ALL' ? 'bg-[#ffb31a] text-black font-semibold' : 'bg-[#24242a] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
    >
      Todas
    </button>
    <button
      onclick={() => { categoryFilter = 'POINTE'; loadStock(); }}
      class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors whitespace-nowrap {categoryFilter === 'POINTE' ? 'bg-[#ffb31a] text-black font-semibold' : 'bg-[#24242a] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
    >
      Puntas
    </button>
    <button
      onclick={() => { categoryFilter = 'FLAT'; loadStock(); }}
      class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors whitespace-nowrap {categoryFilter === 'FLAT' ? 'bg-[#ffb31a] text-black font-semibold' : 'bg-[#24242a] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
    >
      Media Punta
    </button>
    <button
      onclick={() => { categoryFilter = 'APPAREL'; loadStock(); }}
      class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors whitespace-nowrap {categoryFilter === 'APPAREL' ? 'bg-[#ffb31a] text-black font-semibold' : 'bg-[#24242a] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
    >
      Indumentaria
    </button>
    <button
      onclick={() => { categoryFilter = 'LOW'; loadStock(); }}
      class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors whitespace-nowrap {categoryFilter === 'LOW' ? 'bg-[#ff4a4a] text-white font-semibold' : 'bg-[#24242a] text-[#ff4a4a] hover:bg-[#ff4a4a]/20 border border-[#ff4a4a]/30'}"
    >
      Stock Bajo
    </button>
  </div>

  <!-- Lista de Productos en Tarjetas Ergonómicas One UI -->
  <div class="space-y-2">
    {#if isLoading}
      <div class="text-center py-6 text-[#82828a] text-xs font-mono">
        Consultando catálogo SQLite WAL...
      </div>
    {:else if filteredItems.length === 0}
      <div class="text-center py-8 bg-[#1a1a1e] rounded border border-[#34343c] text-[#82828a] text-xs space-y-2">
        <p>No se encontraron productos para los filtros seleccionados.</p>
        <button
          onclick={() => (showNewItemModal = true)}
          class="px-3 py-1 bg-[#24242a] hover:bg-[#34343e] text-[#ffb31a] border border-[#ffb31a]/40 rounded text-xs font-bold"
        >
          + Dar de alta este producto
        </button>
      </div>
    {:else}
      {#each filteredItems as item}
        {@const isCritical = item.physical_qty < 2}
        {@const isLow = item.physical_qty >= 2 && item.physical_qty < 5}
        {@const isBusy = updatingItemId === item.id}
        <div class="bg-[#1a1a1e] border {isCritical ? 'border-[#ff4a4a]/50' : isLow ? 'border-amber-500/40' : 'border-[#34343c]'} rounded-md p-3 space-y-2 text-[#d8d8dc] shadow-sm">
          
          <!-- Encabezado de Producto -->
          <div class="flex items-start justify-between gap-2">
            <div class="min-w-0">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="px-1.5 py-0.5 rounded text-[9px] font-mono font-bold uppercase bg-[#24242a] text-[#ffb31a] border border-[#34343c]">
                  {item.brand}
                </span>
                <span class="text-xs font-bold text-[#d8d8dc] truncate">
                  {item.model_name}
                </span>
              </div>
              <div class="text-[11px] font-mono text-[#82828a] mt-0.5 flex items-center gap-1.5">
                <span>{item.raw_factory_code || item.sku_code}</span>
                {#if item.storage_bin_location}
                  <span>•</span>
                  <span class="text-[#2ed6e8]">Loc: {item.storage_bin_location}</span>
                {/if}
              </div>
            </div>

            <!-- Cantidad en Stock -->
            <div class="text-right font-mono shrink-0">
              <div class="text-[9px] text-[#82828a] uppercase font-bold">Físico / Libre</div>
              <div class="text-base font-black {isCritical ? 'text-[#ff4a4a]' : isLow ? 'text-amber-400' : 'text-[#3de06a]'}">
                {item.physical_qty} <span class="text-xs font-normal text-[#82828a]">({item.free_available_qty})</span>
              </div>
            </div>
          </div>

          <!-- Detalles de Talle, Ancho y Material -->
          <div class="grid grid-cols-3 gap-1.5 bg-[#141416] p-1.5 rounded border border-[#34343c] text-[10px] font-mono">
            <div>
              <span class="text-[#82828a] block text-[8px] uppercase">Talle</span>
              <span class="font-bold text-[#d8d8dc]">{item.size_value} {item.width_value ? `/${item.width_value}` : ''}</span>
            </div>
            <div>
              <span class="text-[#82828a] block text-[8px] uppercase">Color / Mat.</span>
              <span class="text-[#d8d8dc] truncate block">{item.color}</span>
            </div>
            <div>
              <span class="text-[#82828a] block text-[8px] uppercase">Estado</span>
              {#if isCritical}
                <span class="text-[#ff4a4a] font-bold">CRÍTICO</span>
              {:else if isLow}
                <span class="text-amber-400 font-bold">BAJO</span>
              {:else}
                <span class="text-[#3de06a] font-bold">OK</span>
              {/if}
            </div>
          </div>

          <!-- Botones de Acción Rápida -->
          <div class="flex items-center justify-between gap-1.5 pt-1 border-t border-[#34343c]/60">
            <span class="text-[10px] text-[#82828a] font-mono">Sumar:</span>
            <div class="flex items-center gap-1">
              <button
                disabled={isBusy}
                onclick={() => adjustStock(item, 1)}
                class="h-7 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] rounded text-[11px] font-mono font-bold border border-[#34343c] active:scale-95 transition-transform"
                title="Sumar 1 par al stock físico"
              >
                +1
              </button>
              <button
                disabled={isBusy}
                onclick={() => adjustStock(item, 5)}
                class="h-7 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] rounded text-[11px] font-mono font-bold border border-[#34343c] active:scale-95 transition-transform"
                title="Sumar 5 pares al stock físico"
              >
                +5
              </button>
              <button
                disabled={isBusy}
                onclick={() => adjustStock(item, 10)}
                class="h-7 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] rounded text-[11px] font-mono font-bold border border-[#34343c] active:scale-95 transition-transform"
                title="Sumar 10 pares al stock físico"
              >
                +10
              </button>
            </div>
          </div>

        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- Modal para Alta de Producto -->
<NewStockItemModal
  isOpen={showNewItemModal}
  onClose={() => (showNewItemModal = false)}
  onSaved={() => {
    showNewItemModal = false;
    loadStock();
  }}
/>
