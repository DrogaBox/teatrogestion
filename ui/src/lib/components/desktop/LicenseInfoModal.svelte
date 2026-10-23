<script lang="ts">
  import { api, type LicenseInfo } from '../../ipc/client';

  let {
    info,
    onClose,
    onUpdated
  }: {
    info: LicenseInfo;
    onClose: () => void;
    onUpdated: () => void;
  } = $props();

  let activationInput = $state('');
  let activationError = $state<string | null>(null);
  let activationSuccess = $state<string | null>(null);
  let isActivating = $state(false);
  let copiedId = $state(false);
  let copiedReq = $state(false);
  let showReactivation = $state(false);

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
        activationSuccess = 'Licencia renovada con éxito.';
        setTimeout(() => {
          onUpdated();
        }, 1200);
      } else {
        activationError = updated.reason || 'No se pudo aplicar la reactivación.';
      }
    } catch (err: any) {
      console.error('Error aplicando reactivación:', err);
      activationError = err?.message || String(err);
    } finally {
      isActivating = false;
    }
  }
</script>

<div
  class="fixed inset-0 z-50 bg-black/75 flex items-center justify-center p-4 select-none backdrop-blur-sm"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
  onkeydown={(e) => {
    if (e.key === 'Escape') onClose();
  }}
  role="presentation"
>
  <div
    class="w-full max-w-lg bg-[#1a1a1e] border border-[#34343c] rounded shadow-2xl flex flex-col max-h-[92vh] overflow-hidden"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="license-modal-title"
    tabindex="-1"
  >
    <!-- Cabecera -->
    <div class="px-5 py-4 border-b border-[#34343c] bg-[#24242a] flex items-center justify-between flex-shrink-0">
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full bg-[#ffb31a]"></span>
        <h2 id="license-modal-title" class="text-sm font-bold text-[#d8d8dc] uppercase tracking-wider">
          Licencia del Sistema
        </h2>
      </div>
      <button
        type="button"
        onclick={onClose}
        class="w-9 h-9 flex items-center justify-center text-[#82828a] hover:text-[#d8d8dc] hover:bg-[#34343c] rounded transition-colors text-lg"
        aria-label="Cerrar modal"
      >
        &times;
      </button>
    </div>

    <!-- Cuerpo -->
    <div class="p-5 space-y-4 overflow-y-auto flex-1 text-xs">
      <!-- Estado de la Licencia -->
      <div class="bg-[#121214] border border-[#34343c] rounded p-3 flex items-center justify-between">
        <span class="text-[#82828a] uppercase font-semibold">Estado de Autorización:</span>
        {#if info.state === 'Valid'}
          <span class="px-2 py-0.5 bg-[#3de06a]/15 text-[#3de06a] border border-[#3de06a]/30 rounded font-bold uppercase">
            Activa / Válida
          </span>
        {:else if info.state === 'NotEnforced'}
          <span class="px-2 py-0.5 bg-[#2ed6e8]/15 text-[#2ed6e8] border border-[#2ed6e8]/30 rounded font-bold uppercase">
            Modo Desarrollo
          </span>
        {:else}
          <span class="px-2 py-0.5 bg-[#ff4a4a]/15 text-[#ff4a4a] border border-[#ff4a4a]/30 rounded font-bold uppercase">
            No Válida
          </span>
        {/if}
      </div>

      <!-- Datos de Titularidad y Equipo -->
      <div class="bg-[#17171b] border border-[#34343c] rounded p-4 space-y-2.5">
        <div class="flex justify-between items-center pb-2 border-b border-[#34343c]/60">
          <span class="text-[#82828a] uppercase">Titular / Institución:</span>
          <span class="text-[#d8d8dc] font-semibold">{info.licensee || 'Ballet Oficial de Córdoba'}</span>
        </div>

        <div class="flex justify-between items-center pb-2 border-b border-[#34343c]/60">
          <span class="text-[#82828a] uppercase">Machine ID:</span>
          <div class="flex items-center gap-2">
            <span class="font-mono text-[#3de06a] font-bold">{info.machine_id || '(no disponible)'}</span>
            <button
              type="button"
              onclick={handleCopyMachineId}
              class="px-2 py-0.5 bg-[#24242a] hover:bg-[#34343c] text-[#ffb31a] rounded text-[10px] font-semibold uppercase"
            >
              {copiedId ? '[COPIADO]' : 'Copiar'}
            </button>
          </div>
        </div>

        <div class="flex justify-between items-center pb-2 border-b border-[#34343c]/60">
          <span class="text-[#82828a] uppercase">Fecha de Emisión:</span>
          <span class="text-[#d8d8dc] font-mono">{info.issued_date || 'Permanente'}</span>
        </div>

        <div class="flex justify-between items-center">
          <span class="text-[#82828a] uppercase">Vencimiento:</span>
          <span class="text-[#d8d8dc] font-mono">
            {#if info.expires_date}
              {info.expires_date} {info.days_left !== null ? `(${info.days_left} días)` : ''}
            {:else}
              <span class="text-[#3de06a] font-semibold">Sin vencimiento (Perpetua)</span>
            {/if}
          </span>
        </div>
      </div>

      <!-- Preaviso de vencimiento si aplica -->
      {#if info.warning}
        <div class="bg-[#ffb31a]/10 border border-[#ffb31a]/40 text-[#ffb31a] p-3 rounded text-xs leading-relaxed">
          {info.warning}
        </div>
      {/if}

      <!-- Plegable de Reactivación de Emergencia -->
      <div class="pt-2">
        <button
          type="button"
          onclick={() => (showReactivation = !showReactivation)}
          class="w-full py-2 bg-[#24242a] hover:bg-[#34343c] border border-[#34343c] text-[#d8d8dc] rounded text-xs font-semibold uppercase tracking-wider flex items-center justify-between px-3"
        >
          <span>Reactivación de Emergencia (Desafío/Respuesta)</span>
          <span class="text-[#ffb31a]">{showReactivation ? '▲' : '▼'}</span>
        </button>

        {#if showReactivation}
          <div class="mt-2 bg-[#121214] border border-[#ffb31a]/30 rounded p-4 space-y-3">
            <p class="text-[#82828a] text-[11px] leading-relaxed">
              Si no cuenta con el pendrive físico de instalación, proporcione el siguiente código al desarrollador para extender la vigencia.
            </p>

            {#if info.challenge_code}
              <div class="flex items-center justify-between bg-[#17171b] border border-[#34343c] rounded px-3 py-2">
                <span class="font-mono text-xs text-[#3de06a] font-bold">{info.challenge_code}</span>
                <button
                  type="button"
                  onclick={handleCopyChallengeCode}
                  class="px-2 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#ffb31a] rounded text-[10px] font-semibold uppercase"
                >
                  {copiedReq ? '[COPIADO]' : 'Copiar'}
                </button>
              </div>
            {/if}

            <div class="space-y-1">
              <label for="modal-activation-key" class="text-[#82828a] text-[11px] uppercase block font-semibold">
                Clave de activación (RESP-...):
              </label>
              <textarea
                id="modal-activation-key"
                bind:value={activationInput}
                placeholder="Pegar código RESP aquí..."
                rows={2}
                class="w-full bg-[#17171b] border border-[#34343c] rounded p-2 text-xs font-mono text-[#d8d8dc] focus:outline-none focus:border-[#ffb31a]"
              ></textarea>
            </div>

            {#if activationError}
              <div class="text-[11px] text-[#ff4a4a] bg-[#ff4a4a]/10 border border-[#ff4a4a]/30 p-2 rounded">
                {activationError}
              </div>
            {/if}

            {#if activationSuccess}
              <div class="text-[11px] text-[#3de06a] bg-[#3de06a]/10 border border-[#3de06a]/30 p-2 rounded">
                {activationSuccess}
              </div>
            {/if}

            <div class="flex justify-end">
              <button
                type="button"
                onclick={handleActivate}
                disabled={isActivating || !activationInput.trim()}
                class="px-4 py-1.5 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] rounded text-xs font-bold uppercase transition-colors disabled:opacity-40"
              >
                {isActivating ? 'Verificando...' : 'Aplicar Código'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Pie -->
    <div class="px-5 py-3 border-t border-[#34343c] bg-[#24242a] flex justify-end flex-shrink-0">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-1.5 bg-[#1a1a1e] hover:bg-[#34343c] border border-[#34343c] text-[#d8d8dc] rounded text-xs font-semibold uppercase transition-colors"
      >
        Cerrar
      </button>
    </div>
  </div>
</div>
