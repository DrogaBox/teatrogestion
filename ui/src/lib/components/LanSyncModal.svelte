<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type LanSyncInfo, type SyncReportDto } from '../ipc/client';

  let {
    isOpen,
    onClose,
    onSyncCompleted = () => {}
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
    onSyncCompleted?: () => void;
  }>();

  let syncInfo = $state<LanSyncInfo>({
    local_ip: 'Detectando...',
    port: 45870,
    is_server_active: true,
    device_name: 'TeatroGestión',
  });

  let peerIp = $state('192.168.100.32');
  let isLoading = $state(false);
  let syncReport = $state<SyncReportDto | null>(null);
  let errorMessage = $state('');

  // Clave de sincronización: los dos equipos tienen que tener la misma. Sin ella el otro equipo rechaza la conexión.
  let syncKey = $state('');
  let newKey = $state('');
  let keyMessage = $state('');
  let keyIsError = $state(false);

  async function loadKey() {
    try {
      syncKey = await api.getLanSyncKey();
    } catch (e) {
      console.warn('Error leyendo la clave de sincronización:', e);
    }
  }

  async function copyKey() {
    try {
      await navigator.clipboard.writeText(syncKey);
      keyIsError = false;
      keyMessage = 'Clave copiada.';
    } catch {
      keyIsError = true;
      keyMessage = 'No se pudo copiar: selecciónela y cópiela a mano.';
    }
  }

  async function useNewKey() {
    keyMessage = '';
    try {
      syncKey = await api.setLanSyncKey(newKey.trim());
      newKey = '';
      keyIsError = false;
      keyMessage = 'Listo: este equipo ahora usa esa clave.';
    } catch (e: any) {
      keyIsError = true;
      keyMessage = e.message || e.toString() || 'No se pudo cambiar la clave.';
    }
  }

  function loadSavedIp() {
    try {
      const saved = localStorage.getItem('teatro_sync_peer_ip');
      if (saved) {
        peerIp = saved;
      }
    } catch {}
  }

  async function fetchSyncInfo() {
    try {
      syncInfo = await api.getLanSyncInfo();
    } catch (e) {
      console.warn('Error fetching sync info:', e);
    }
  }

  $effect(() => {
    if (isOpen) {
      loadSavedIp();
      fetchSyncInfo();
      loadKey();
      keyMessage = '';
      syncReport = null;
      errorMessage = '';
    }
  });

  onMount(() => {
    loadSavedIp();
    fetchSyncInfo();
  });

  async function handleSync() {
    if (!peerIp.trim()) {
      errorMessage = 'Ingrese la dirección IP del equipo con el que desea sincronizar.';
      return;
    }

    try {
      localStorage.setItem('teatro_sync_peer_ip', peerIp.trim());
    } catch {}

    isLoading = true;
    errorMessage = '';
    syncReport = null;

    try {
      const res = await api.syncWithLanPeer(peerIp.trim());
      syncReport = res;
      onSyncCompleted();
    } catch (e: any) {
      errorMessage = e.message || e.toString() || 'Error al conectar con el equipo remoto.';
    } finally {
      isLoading = false;
    }
  }

  async function handleExportJson() {
    try {
      const bundle = await api.exportLocalSyncBundle();
      const blob = new Blob([JSON.stringify(bundle, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `teatro_gestion_respaldo_${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e: any) {
      errorMessage = 'Error exportando archivo JSON: ' + (e.message || e);
    }
  }
</script>

{#if isOpen}
  <div
    role="presentation"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4"
    onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
  >
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-lg shadow-2xl overflow-hidden flex flex-col max-h-[90vh] text-[#d8d8dc]">
      
      <!-- Encabezado -->
      <div class="p-4 border-b border-[#34343c] bg-[#121214] flex items-center justify-between">
        <div>
          <h2 class="text-sm font-semibold text-[#d8d8dc] tracking-wide uppercase">Sincronización P2P Wi-Fi</h2>
          <p class="text-xs text-[#82828a]">Ballet Oficial de Córdoba — Red Local Sin Nube</p>
        </div>
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); onClose(); }}
          class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#24242a] text-[#82828a] hover:text-white font-mono text-base transition-colors shrink-0"
          title="Cerrar modal"
        >
          ✕
        </button>
      </div>

      <!-- Contenido Principal -->
      <div class="p-5 space-y-4 overflow-y-auto">
        
        <!-- Estado de este equipo -->
        <div class="bg-[#121214] rounded p-3.5 border border-[#34343c] space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold uppercase tracking-wider text-[#82828a]">Este Dispositivo:</span>
            <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-xs font-medium bg-[#3de06a]/15 text-[#3de06a] border border-[#3de06a]/30">
              <span class="w-1.5 h-1.5 rounded-full bg-[#3de06a]"></span>
              Servidor Activo (Puerto {syncInfo.port})
            </span>
          </div>
          <div class="text-sm font-bold text-[#d8d8dc]">{syncInfo.device_name}</div>
          <div class="flex items-center justify-between text-xs text-[#ffb31a] bg-[#1a1a1e] px-3 py-1.5 rounded border border-[#34343c]">
            <span class="text-[#82828a]">IP Local:</span>
            <span class="font-mono font-bold tracking-wider">{syncInfo.local_ip}</span>
          </div>
        </div>

        <!-- Clave de sincronización -->
        <div class="bg-[#121214] rounded p-3.5 border border-[#34343c] space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold uppercase tracking-wider text-[#82828a]">Clave de sincronización</span>
            <button
              onclick={copyKey}
              class="px-2 py-1 bg-[#24242a] hover:bg-[#34343c] border border-[#34343c] text-[#82828a] hover:text-[#d8d8dc] rounded text-[11px] transition-colors"
            >Copiar</button>
          </div>
          <div class="font-mono text-[11px] text-[#ffb31a] bg-[#1a1a1e] px-3 py-1.5 rounded border border-[#34343c] break-all select-all" data-testid="lan-sync-key">{syncKey || '...'}</div>
          <p class="text-[11px] text-[#82828a]">
            Sin esta clave nadie más en la red puede pedir ni enviar datos. En el <b>otro equipo</b> escriba esta misma clave acá abajo (una sola vez).
          </p>
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newKey}
              placeholder="Clave del otro equipo"
              autocomplete="off"
              spellcheck="false"
              class="flex-1 bg-[#1a1a1e] border border-[#34343c] rounded px-3 py-1.5 text-[11px] text-[#d8d8dc] font-mono focus:outline-none focus:border-[#ffb31a] transition-colors"
            />
            <button
              onclick={useNewKey}
              disabled={newKey.trim().length < 8}
              class="px-3 py-1.5 bg-[#ffb31a] hover:bg-[#e6a117] disabled:opacity-50 text-black font-semibold text-[11px] rounded transition-colors shrink-0"
            >Usar esta clave</button>
          </div>
          {#if keyMessage}
            <div class="text-[11px] {keyIsError ? 'text-[#ff4a4a]' : 'text-[#3de06a]'}">{keyMessage}</div>
          {/if}
        </div>

        <!-- Conectar con el otro equipo -->
        <div class="space-y-2">
          <label for="peer-ip-input" class="block text-xs font-semibold uppercase tracking-wider text-[#82828a]">
            IP del Otro Equipo (PC o Celular en la misma red)
          </label>
          <div class="flex gap-2">
            <input
              id="peer-ip-input"
              type="text"
              bind:value={peerIp}
              placeholder="ej. 192.168.100.32"
              class="flex-1 bg-[#121214] border border-[#34343c] rounded px-3 py-2 text-xs text-[#d8d8dc] font-mono focus:outline-none focus:border-[#ffb31a] transition-colors"
            />
            <button
              onclick={handleSync}
              disabled={isLoading || !peerIp.trim()}
              class="px-4 py-2 bg-[#ffb31a] hover:bg-[#e6a117] disabled:opacity-50 text-black font-semibold text-xs rounded transition-colors shrink-0"
            >
              {#if isLoading}
                <span>Sincronizando...</span>
              {:else}
                <span>Sincronizar</span>
              {/if}
            </button>
          </div>
          <p class="text-[11px] text-[#82828a]">
            Conecta ambos dispositivos a la misma red Wi-Fi del teatro o comparte zona Wi-Fi desde el móvil.
          </p>
        </div>

        <!-- Mensajes de Reporte o Error -->
        {#if errorMessage}
          <div class="p-3 bg-[#ff4a4a]/10 border border-[#ff4a4a]/40 rounded text-[#ff4a4a] text-xs">
            <div>{errorMessage}</div>
          </div>
        {/if}

        {#if syncReport}
          <div class="p-3 bg-[#3de06a]/10 border border-[#3de06a]/40 rounded text-[#3de06a] text-xs space-y-1">
            <div class="font-semibold text-sm">
              Sincronización Exitosa
            </div>
            <div>{syncReport.message}</div>
            <div class="text-[11px] text-[#3de06a]/80 pt-1 border-t border-[#3de06a]/20">
              Registros procesados: <b>{syncReport.records_received}</b> | Modificados: <b>{syncReport.records_applied}</b>
            </div>
          </div>
        {/if}

        <!-- Respaldo alternativo -->
        <div class="pt-3 border-t border-[#34343c] flex items-center justify-between">
          <span class="text-xs text-[#82828a]">¿Sin red Wi-Fi disponible?</span>
          <button
            onclick={handleExportJson}
            class="px-3 py-1.5 bg-[#24242a] hover:bg-[#34343c] border border-[#34343c] text-[#82828a] hover:text-[#d8d8dc] rounded text-xs font-medium transition-colors"
          >
            Exportar Respaldo JSON
          </button>
        </div>

      </div>

    </div>
  </div>
{/if}
