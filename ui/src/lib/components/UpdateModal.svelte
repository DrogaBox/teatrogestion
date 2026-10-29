<script lang="ts">
  import { api, type UpdateInfo, type UpdateStatus } from '../ipc/client';
  import ModalHeader from './shared/ModalHeader.svelte';
  import { modalBehavior } from '../actions/modalBehavior';

  let { isOpen, initialInfo = null, onClose } = $props<{
    isOpen: boolean;
    initialInfo?: UpdateInfo | null;
    onClose: () => void;
  }>();

  type Phase = 'idle' | 'checking' | 'downloading' | 'done';
  let status = $state<UpdateStatus | null>(null);
  let info = $state<UpdateInfo | null>(null);
  let phase = $state<Phase>('idle');
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);
  let tokenInput = $state('');
  let savingToken = $state(false);

  async function loadStatus() {
    try {
      status = await api.getUpdateStatus();
    } catch (e: any) {
      error = String(e?.message ?? e);
    }
  }

  $effect(() => {
    if (isOpen) {
      info = initialInfo;
      message = null;
      error = null;
      phase = 'idle';
      loadStatus();
    }
  });

  async function saveToken() {
    error = null;
    savingToken = true;
    try {
      await api.setUpdateToken(tokenInput);
      tokenInput = '';
      await loadStatus();
      await check();
    } catch (e: any) {
      error = String(e?.message ?? e);
    } finally {
      savingToken = false;
    }
  }

  async function forgetToken() {
    await api.clearUpdateToken();
    info = null;
    message = null;
    await loadStatus();
  }

  async function check() {
    error = null;
    message = null;
    phase = 'checking';
    try {
      info = await api.checkForUpdate();
      if (!info) message = 'Ya tiene la última versión.';
      try { localStorage.setItem('tg_last_update_check', String(Date.now())); } catch { /* sin almacenamiento */ }
    } catch (e: any) {
      error = String(e?.message ?? e);
    } finally {
      phase = 'idle';
    }
  }

  async function install() {
    if (!info) return;
    error = null;
    phase = 'downloading';
    try {
      const out = await api.installUpdate(info);
      if (out.mode === 'windows') {
        message = 'Se abrió el actualizador. TeatroGestión se cierra ahora: siga las indicaciones de la ventana negra.';
        phase = 'done';
      } else if (out.mode === 'linux') {
        message = 'Versión nueva instalada. La aplicación se reinicia.';
        phase = 'done';
      } else {
        const bridge = (window as any).TGUpdate;
        const r: string = bridge?.installApk ? bridge.installApk(out.path) : 'sin-puente';
        if (r === 'ok') message = 'Confirme la instalación en la ventana de Android.';
        else if (r === 'permiso-pendiente') message = 'Android pide permiso para instalar desde esta app. Actívelo y vuelva a tocar «Descargar e instalar».';
        else error = 'No se pudo abrir el instalador de Android (' + r + ').';
        phase = 'idle';
      }
    } catch (e: any) {
      error = String(e?.message ?? e);
      phase = 'idle';
    }
  }

  const mb = (n: number) => (n / 1048576).toFixed(1).replace('.', ',') + ' MB';
  const dateOf = (iso: string) => (iso ? new Date(iso).toLocaleDateString('es-AR') : '');
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6" role="dialog" aria-modal="true" aria-label="Actualizaciones" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-xl text-ink]">
      <ModalHeader title="Actualizaciones" subtitle="Versión instalada: {status?.version ?? '…'}" onclose={onClose} />

      <div class="flex-1 overflow-y-auto p-5 space-y-4">
        {#if status && !status.platform}
          <p class="text-sm text-dim]">La actualización automática solo está disponible en Windows, Linux y en el celular Android.</p>
        {:else if status && !status.has_token}
          <div class="space-y-2">
            <p class="text-sm">Para buscar versiones nuevas hace falta el <strong>token de actualización</strong>. Se escribe una sola vez en este equipo.</p>
            <input
              type="password"
              bind:value={tokenInput}
              placeholder="Token de actualización"
              autocomplete="off"
              class="w-full h-11 px-3 bg-void] border border-grid] focus:border-amber] rounded text-sm outline-none"
            />
            <button type="button" disabled={savingToken || !tokenInput.trim()} onclick={saveToken} class="px-4 py-2.5 rounded bg-amber] hover:bg-amber] disabled:opacity-50 text-void] text-sm font-bold">
              {savingToken ? 'Comprobando…' : 'Guardar y buscar'}
            </button>
          </div>
        {:else if status}
          {#if info}
            <div class="space-y-3">
              <div class="flex items-baseline gap-2 flex-wrap">
                <span class="px-2 py-0.5 rounded bg-green] text-void] text-xs font-bold font-mono">Versión nueva {info.version}</span>
                <span class="text-xs text-dim] font-mono">{dateOf(info.published_at)} · {mb(info.asset_size)}</span>
              </div>
              <h3 class="text-sm font-bold text-white">{info.name}</h3>
              <div class="text-[13px] leading-relaxed whitespace-pre-wrap bg-void] border border-grid] rounded p-3 max-h-72 overflow-y-auto">{info.notes || 'Sin notas.'}</div>
              <button type="button" disabled={phase === 'downloading' || phase === 'done'} onclick={install} class="w-full px-4 py-3 rounded bg-amber] hover:bg-amber] disabled:opacity-60 text-void] text-sm font-bold">
                {phase === 'downloading' ? 'Descargando y verificando…' : 'Descargar e instalar'}
              </button>
              <p class="text-xs text-dim]">Sus datos y su licencia no se modifican. Antes de reemplazar el programa se guarda una copia de seguridad.</p>
            </div>
          {:else}
            <button type="button" disabled={phase === 'checking'} onclick={check} class="px-4 py-2.5 rounded bg-raised] hover:bg-raised-hover] border border-grid] text-sm font-bold disabled:opacity-60">
              {phase === 'checking' ? 'Buscando…' : 'Buscar actualizaciones'}
            </button>
          {/if}
        {/if}

        {#if message}<p class="text-sm text-green]">{message}</p>{/if}
        {#if error}<p class="text-sm text-red]">{error}</p>{/if}
      </div>

      {#if status?.has_token}
        <div class="tg-modal-foot">
          <button type="button" onclick={forgetToken} class="text-xs text-dim] hover:text-white underline">Quitar el token de este equipo</button>
          <button type="button" onclick={onClose} class="tg-btn">Cerrar</button>
        </div>
      {/if}
    </div>
  </div>
{/if}
