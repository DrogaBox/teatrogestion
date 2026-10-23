<script lang="ts">
  import { api, type LicenseInfo } from '../../ipc/client';

  let {
    info,
    onActivated
  }: {
    info: LicenseInfo;
    onActivated: () => void;
  } = $props();

  let activationInput = $state('');
  let activationError = $state<string | null>(null);
  let activationSuccess = $state<string | null>(null);
  let isActivating = $state(false);
  let copiedId = $state(false);
  let copiedReq = $state(false);

  async function handleCopyMachineId() {
    if (!info.machine_id) return;
    try {
      await navigator.clipboard.writeText(info.machine_id);
      copiedId = true;
      setTimeout(() => (copiedId = false), 2500);
    } catch (e) {
      console.error('Error copiando Machine ID:', e);
    }
  }

  async function handleCopyChallengeCode() {
    if (!info.challenge_code) return;
    try {
      await navigator.clipboard.writeText(info.challenge_code);
      copiedReq = true;
      setTimeout(() => (copiedReq = false), 2500);
    } catch (e) {
      console.error('Error copiando Código de Solicitud:', e);
    }
  }

  async function handleActivate() {
    if (!activationInput.trim()) return;
    isActivating = true;
    activationError = null;
    activationSuccess = null;
    try {
      const updated = await api.applyEmergencyActivation(activationInput.trim());
      if (updated.is_valid && !updated.is_locked) {
        activationSuccess = 'Licencia activada con éxito. Reiniciando sesión...';
        setTimeout(() => {
          onActivated();
        }, 1200);
      } else {
        activationError = updated.reason || 'La clave no pudo validar la licencia.';
      }
    } catch (err: any) {
      console.error('Error aplicando reactivación:', err);
      activationError = err?.message || String(err);
    } finally {
      isActivating = false;
    }
  }
</script>

<div class="fixed inset-0 z-50 bg-[#121214] text-[#d8d8dc] flex items-center justify-center p-4 select-none overflow-y-auto">
  <div class="w-full max-w-2xl bg-[#1a1a1e] border border-[#ff4a4a]/40 rounded shadow-2xl p-6 sm:p-8 flex flex-col items-center text-center my-auto">
    <!-- Header de la Aplicación -->
    <div class="text-[#ffb31a] text-3xl font-black tracking-widest uppercase mb-1">
      TeatroGestión
    </div>
    <div class="text-[#82828a] text-xs uppercase tracking-wider mb-4 font-mono">
      Ballet Oficial de Córdoba — Teatro del Libertador San Martín
    </div>

    <!-- Indicador de Bloqueo -->
    <div class="inline-flex items-center gap-2 px-3 py-1 bg-[#ff4a4a]/15 text-[#ff4a4a] border border-[#ff4a4a]/30 rounded text-sm font-bold tracking-wide uppercase mb-3">
      Licencia no válida en esta PC
    </div>

    <!-- Motivo técnico -->
    <div class="text-[#d8d8dc] text-sm max-w-lg mb-6 leading-relaxed bg-[#17171b] border border-[#34343c] px-4 py-3 rounded text-left">
      <span class="text-[#82828a] text-xs uppercase block font-semibold mb-1">Diagnóstico del Sistema:</span>
      <span class="font-mono text-xs text-[#ff4a4a]">{info.reason || 'No se encontró un archivo license.key válido para este equipo.'}</span>
    </div>

    <!-- Identificador de Máquina -->
    <div class="w-full bg-[#121214] border border-[#34343c] rounded p-4 mb-6 flex flex-col items-center">
      <span class="text-[#82828a] text-xs uppercase tracking-wider font-semibold mb-2">
        Machine ID de esta PC
      </span>
      <div class="font-mono text-2xl sm:text-3xl font-bold text-[#3de06a] tracking-wider mb-3">
        {info.machine_id || '(no disponible)'}
      </div>
      <button
        type="button"
        onclick={handleCopyMachineId}
        disabled={!info.machine_id}
        class="px-4 py-2 bg-[#24242a] hover:bg-[#34343c] border border-[#34343c] text-[#ffb31a] rounded text-xs font-semibold tracking-wide uppercase transition-colors"
      >
        {copiedId ? '[ID COPIADO]' : 'Copiar Machine ID'}
      </button>
    </div>

    <!-- Instrucciones operativas -->
    <p class="text-[#82828a] text-xs leading-relaxed max-w-lg mb-6">
      Envíe este Machine ID al desarrollador para recibir su archivo <span class="font-mono text-[#d8d8dc]">license.key</span>.<br />
      Cópielo en la carpeta de instalación (<span class="font-mono text-[#d8d8dc]">C:\TeatroGestion</span>) junto al ejecutable y reinicie la aplicación.
    </p>

    <!-- Módulo de Reactivación de Emergencia (Desafío/Respuesta) -->
    {#if info.challenge_code || info.reactivable}
      <div class="w-full bg-[#17171b] border border-[#ffb31a]/40 rounded p-5 text-left mb-4">
        <div class="flex items-center justify-between mb-2">
          <span class="text-[#ffb31a] text-sm font-bold uppercase tracking-wider">
            Reactivación de Emergencia
          </span>
          <span class="text-[10px] text-[#82828a] uppercase font-mono">100% Offline</span>
        </div>
        <p class="text-[#82828a] text-xs leading-relaxed mb-3">
          ¿Sin el pendrive? Envíe este código de solicitud al desarrollador (por WhatsApp) y pegue abajo la clave de activación generada.
        </p>

        {#if info.challenge_code}
          <div class="flex items-center justify-between bg-[#121214] border border-[#34343c] rounded px-3 py-2 mb-3">
            <span class="font-mono text-sm text-[#3de06a] font-bold tracking-wider">{info.challenge_code}</span>
            <button
              type="button"
              onclick={handleCopyChallengeCode}
              class="px-2 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#ffb31a] rounded text-[11px] font-semibold uppercase"
            >
              {copiedReq ? '[COPIADO]' : 'Copiar'}
            </button>
          </div>
        {/if}

        <div class="space-y-2">
          <label for="activation-input" class="text-[#82828a] text-xs uppercase block font-semibold">
            Pegar clave de activación (RESP-...)
          </label>
          <textarea
            id="activation-input"
            bind:value={activationInput}
            placeholder="Pegue aquí la clave recibida..."
            rows={2}
            class="w-full bg-[#121214] border border-[#34343c] rounded p-2 text-xs font-mono text-[#d8d8dc] placeholder-[#82828a] focus:outline-none focus:border-[#ffb31a]"
          ></textarea>
        </div>

        {#if activationError}
          <div class="mt-2 text-xs text-[#ff4a4a] bg-[#ff4a4a]/10 border border-[#ff4a4a]/30 p-2 rounded">
            {activationError}
          </div>
        {/if}

        {#if activationSuccess}
          <div class="mt-2 text-xs text-[#3de06a] bg-[#3de06a]/10 border border-[#3de06a]/30 p-2 rounded">
            {activationSuccess}
          </div>
        {/if}

        <div class="mt-3 flex justify-end">
          <button
            type="button"
            onclick={handleActivate}
            disabled={isActivating || !activationInput.trim()}
            class="px-5 py-2 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] rounded text-xs font-bold uppercase tracking-wider transition-colors disabled:opacity-40"
          >
            {isActivating ? 'Verificando...' : 'Activar Licencia'}
          </button>
        </div>
      </div>
    {/if}

    <div class="text-[11px] text-[#82828a] font-mono mt-2">
      ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
    </div>
  </div>
</div>
