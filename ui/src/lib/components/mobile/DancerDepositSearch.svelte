<script lang="ts">
  import { api, type PersonnelWithQuotaDto } from '../../ipc/client';
  import VoucherDeliveryModal from './VoucherDeliveryModal.svelte';

  let dancers = $state<PersonnelWithQuotaDto[]>([]);
  let searchQuery = $state('');
  let selectedDancer = $state<PersonnelWithQuotaDto | null>(null);
  let isLoading = $state(false);

  const quickNicks = ['Popi', 'Brenda', 'Álvarez', 'Marcato', 'Morchio C', 'Gil'];
  let copiedCode = $state<string | null>(null);

  function copyPointeCode(code: string) {
    if (typeof navigator !== 'undefined' && navigator.clipboard) {
      navigator.clipboard.writeText(code).then(() => {
        copiedCode = code;
        setTimeout(() => (copiedCode = null), 2500);
      });
    }
  }

  async function search(q: string) {
    searchQuery = q;
    isLoading = true;
    try {
      dancers = await api.searchDancerQuick(q);
    } catch (e) {
      console.error(e);
    } finally {
      isLoading = false;
    }
  }

  // Búsqueda inicial
  search('');

  function openDelivery(dancer: PersonnelWithQuotaDto) {
    selectedDancer = dancer;
  }
</script>

<div class="space-y-3">
  <!-- Barra de búsqueda con icono integrado -->
  <div class="relative">
    <div class="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none text-[#ffb31a]">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
    </div>
    <input
      type="text"
      bind:value={searchQuery}
      oninput={(e) => search((e.target as HTMLInputElement).value)}
      placeholder="Buscar bailarín o apodo (ej: Popi, Brenda)..."
      class="w-full h-10 pl-10 pr-10 bg-[#121214] border border-[#34343c] focus:border-[#ffb31a] rounded text-xs font-medium text-[#d8d8dc] placeholder-[#82828a] outline-none transition-colors"
    />
    {#if searchQuery}
      <button
        onclick={() => search('')}
        class="absolute right-1 top-1 h-8 w-8 flex items-center justify-center text-[#82828a] hover:text-white font-mono text-xs font-bold"
      >
        ✕
      </button>
    {/if}
  </div>

  <!-- Chips de acceso rápido por apodo -->
  <div class="flex items-center gap-1.5 overflow-x-auto pb-0.5 no-scrollbar text-xs">
    {#each quickNicks as nick}
      {@const isActive = searchQuery.toLowerCase() === nick.toLowerCase()}
      <button
        onclick={() => search(isActive ? '' : nick)}
        class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors flex items-center whitespace-nowrap {isActive ? 'bg-[#ffb31a] text-black font-semibold' : 'bg-[#24242a] hover:bg-[#34343c] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
      >
        <span>{nick}</span>
      </button>
    {/each}
  </div>

  <!-- Lista de Resultados de Bailarines -->
  <div class="space-y-2">
    {#if isLoading}
      <div class="text-center py-6 text-[#82828a] text-xs">
        Consultando padrón local SQLite...
      </div>
    {:else if dancers.length === 0}
      <div class="text-center py-8 bg-[#1a1a1e] rounded border border-[#34343c] text-[#82828a] text-xs">
        <p>No se encontraron bailarines para "{searchQuery}"</p>
      </div>
    {:else}
      {#each dancers as d}
        {@const pointePct = Math.round((d.remaining_pointe_qty / (d.effective_pointe_quota || 1)) * 100)}
        {@const initials = `${d.first_name?.[0] || ''}${d.last_name?.[0] || ''}`.toUpperCase()}
        <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md p-3.5 space-y-2 text-[#d8d8dc]">
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2.5 min-w-0">
              <!-- Avatar con iniciales -->
              <div class="w-9 h-9 rounded bg-[#24242a] border border-[#34343c] flex items-center justify-center font-bold text-[#ffb31a] text-xs shrink-0">
                {initials}
              </div>
              <div class="min-w-0">
                <div class="text-xs font-semibold text-[#d8d8dc] flex items-center gap-1.5 truncate">
                  <span class="truncate">{d.last_name}, {d.first_name}</span>
                  {#if d.nickname}
                    <span class="bg-[#ffb31a]/15 text-[#ffb31a] border border-[#ffb31a]/30 text-[9px] px-1 rounded font-medium shrink-0">
                      "{d.nickname}"
                    </span>
                  {/if}
                </div>
                <div class="text-[11px] text-[#82828a] flex items-center gap-1.5 mt-0.5 truncate">
                  <span class="text-[#82828a]">{d.category_name}</span>
                  <span>•</span>
                  <span class="font-mono text-[#82828a]">DNI {d.legajo || '—'}</span>
                </div>
              </div>
            </div>

            <!-- Botón de Entrega de Calzado -->
            <button
              onclick={() => openDelivery(d)}
              class="h-8 px-3 bg-[#ffb31a] hover:bg-[#e6a117] text-black font-semibold text-xs rounded transition-colors shrink-0"
            >
              Entregar
            </button>
          </div>

          <!-- Código Habitual de Zapatilla si existe -->
          {#if d.default_pointe_raw_code}
            <div class="bg-[#121214] px-2.5 py-1.5 rounded border border-[#34343c] text-[11px] font-mono flex items-center justify-between">
              <span class="text-[#82828a] text-[9px] uppercase font-semibold tracking-wider">Punta Habitual</span>
              <button
                type="button"
                onclick={() => copyPointeCode(d.default_pointe_raw_code!)}
                class="text-[#ffb31a] hover:text-amber-300 font-bold flex items-center gap-1.5 active:scale-95 transition-transform"
                title="Tocar para copiar código"
              >
                <span>{d.default_pointe_raw_code}</span>
                {#if copiedCode === d.default_pointe_raw_code}
                  <span class="text-[9px] text-[#3de06a] bg-[#1e2a22] px-1 rounded font-sans">✓ Copiado</span>
                {:else}
                  <span class="text-[9px] text-[#82828a] bg-[#24242a] border border-[#34343c] px-1 rounded font-sans">Copiar</span>
                {/if}
              </button>
            </div>
          {/if}

          <!-- Barra de Cupo Anual de Puntas -->
          <div>
            <div class="flex justify-between items-center text-[10px] mb-1">
              <span class="text-[#82828a] font-medium">Cupo Anual de Puntas:</span>
              <span class="font-mono font-bold {pointePct < 25 ? 'text-[#ff4a4a]' : pointePct < 50 ? 'text-[#ffb31a]' : 'text-[#3de06a]'}">
                {d.remaining_pointe_qty} / {d.effective_pointe_quota} pares ({pointePct}%)
              </span>
            </div>
            <div class="w-full bg-[#121214] h-1.5 rounded overflow-hidden border border-[#34343c]">
              <div
                class="h-full rounded transition-all {pointePct < 25 ? 'bg-[#ff4a4a]' : pointePct < 50 ? 'bg-[#ffb31a]' : 'bg-[#3de06a]'}"
                style="width: {Math.max(0, Math.min(100, pointePct))}%"
              ></div>
            </div>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- Modal de Emisión de Valera -->
{#if selectedDancer}
  <VoucherDeliveryModal
    dancer={selectedDancer}
    onClose={() => selectedDancer = null}
    onDelivered={() => {
      search(searchQuery);
    }}
  />
{/if}
