<script lang="ts">
  import { api, type LicenseInfo } from '../../ipc/client';
  import { modalBehavior } from '../../actions/modalBehavior';

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
  class="fixed inset-0 z-50 bg-black/75 flex items-center justify-center p-4 select-none"
  use:modalBehavior={{ onclose: onClose }}
  role="presentation"
>
  <div
    class="tg-modal max-w-lg"
    role="dialog"
    aria-modal="true"
    aria-labelledby="license-modal-title"
  >
    <!-- Cabecera -->
    <div class="tg-modal-head">
      <div class="flex items-center gap-2 min-w-0">
        <span class="w-2.5 h-2.5 rounded-full bg-amber] shrink-0"></span>
        <h2 id="license-modal-title" class="tg-modal-title">
          Licencia del Sistema
        </h2>
      </div>
      <button
        type="button"
        onclick={onClose}
        class="tg-modal-close"
        aria-label="Cerrar"
      >
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
          <path d="M2 2l10 10M12 2L2 12" />
        </svg>
      </button>
    </div>

    <!-- Cuerpo -->
    <div class="p-5 space-y-4 overflow-y-auto flex-1 text-xs">
      <!-- Estado de la Licencia -->
      <div class="bg-void] border border-grid] rounded p-3 flex items-center justify-between">
        <span class="text-dim] uppercase font-semibold">Estado de Autorización:</span>
        {#if info.state === 'Valid'}
          <span class="px-2 py-0.5 bg-green]/15 text-green] border border-green]/30 rounded font-bold uppercase">
            Activa / Válida
          </span>
        {:else if info.state === 'NotEnforced'}
          <span class="px-2 py-0.5 bg-cyan]/15 text-cyan] border border-cyan]/30 rounded font-bold uppercase">
            Modo Desarrollo
          </span>
        {:else}
          <span class="px-2 py-0.5 bg-red]/15 text-red] border border-red]/30 rounded font-bold uppercase">
            No Válida
          </span>
        {/if}
      </div>

      <!-- Datos de Titularidad y Equipo -->
      <div class="bg-alt] border border-grid] rounded p-4 space-y-2.5">
        <div class="flex justify-between items-center pb-2 border-b border-grid]/60">
          <span class="text-dim] uppercase">Titular / Institución:</span>
          <span class="text-ink] font-semibold">{info.licensee || 'Ballet Oficial de Córdoba'}</span>
        </div>

        <div class="flex justify-between items-center pb-2 border-b border-grid]/60">
          <span class="text-dim] uppercase">Machine ID:</span>
          <div class="flex items-center gap-2">
            <span class="font-mono text-green] font-bold">{info.machine_id || '(no disponible)'}</span>
            <button
              type="button"
              onclick={handleCopyMachineId}
              class="px-2 py-0.5 bg-raised] hover:bg-grid] text-amber] rounded text-[10px] font-semibold uppercase"
            >
              {copiedId ? '[COPIADO]' : 'Copiar'}
            </button>
          </div>
        </div>

        <div class="flex justify-between items-center pb-2 border-b border-grid]/60">
          <span class="text-dim] uppercase">Fecha de Emisión:</span>
          <span class="text-ink] font-mono">{info.issued_date || 'Permanente'}</span>
        </div>

        <div class="flex justify-between items-center">
          <span class="text-dim] uppercase">Vencimiento:</span>
          <span class="text-ink] font-mono">
            {#if info.expires_date}
              {info.expires_date} {info.days_left !== null ? `(${info.days_left} días)` : ''}
            {:else}
              <span class="text-green] font-semibold">Sin vencimiento (Perpetua)</span>
            {/if}
          </span>
        </div>
      </div>

      <!-- Preaviso de vencimiento si aplica -->
      {#if info.warning}
        <div class="bg-amber]/10 border border-amber]/40 text-amber] p-3 rounded text-xs leading-relaxed">
          {info.warning}
        </div>
      {/if}

      <!-- Plegable de Reactivación de Emergencia -->
      <div class="pt-2">
        <button
          type="button"
          onclick={() => (showReactivation = !showReactivation)}
          class="w-full py-2 bg-raised] hover:bg-grid] border border-grid] text-ink] rounded text-xs font-semibold uppercase tracking-wider flex items-center justify-between px-3"
        >
          <span>Reactivación de Emergencia (Desafío/Respuesta)</span>
          <span class="text-amber]">{showReactivation ? '▲' : '▼'}</span>
        </button>

        {#if showReactivation}
          <div class="mt-2 bg-void] border border-amber]/30 rounded p-4 space-y-3">
            <p class="text-dim] text-[11px] leading-relaxed">
              Si no cuenta con el pendrive físico de instalación, proporcione el siguiente código al desarrollador para extender la vigencia.
            </p>

            {#if info.challenge_code}
              <div class="flex items-center justify-between bg-alt] border border-grid] rounded px-3 py-2">
                <span class="font-mono text-xs text-green] font-bold">{info.challenge_code}</span>
                <button
                  type="button"
                  onclick={handleCopyChallengeCode}
                  class="px-2 py-1 bg-raised] hover:bg-grid] text-amber] rounded text-[10px] font-semibold uppercase"
                >
                  {copiedReq ? '[COPIADO]' : 'Copiar'}
                </button>
              </div>
            {/if}

            <div class="space-y-1">
              <label for="modal-activation-key" class="text-dim] text-[11px] uppercase block font-semibold">
                Clave de activación (RESP-...):
              </label>
              <textarea
                id="modal-activation-key"
                bind:value={activationInput}
                placeholder="Pegar código RESP aquí..."
                rows={2}
                class="w-full bg-alt] border border-grid] rounded p-2 text-xs font-mono text-ink] focus:outline-none focus:border-amber]"
              ></textarea>
            </div>

            {#if activationError}
              <div class="text-[11px] text-red] bg-red]/10 border border-red]/30 p-2 rounded">
                {activationError}
              </div>
            {/if}

            {#if activationSuccess}
              <div class="text-[11px] text-green] bg-green]/10 border border-green]/30 p-2 rounded">
                {activationSuccess}
              </div>
            {/if}

            <div class="flex justify-end">
              <button
                type="button"
                onclick={handleActivate}
                disabled={isActivating || !activationInput.trim()}
                class="px-4 py-1.5 bg-amber] hover:bg-amber]/90 text-void] rounded text-xs font-bold uppercase transition-colors disabled:opacity-40"
              >
                {isActivating ? 'Verificando...' : 'Aplicar Código'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Pie -->
    <div class="tg-modal-foot">
      <button
        type="button"
        onclick={onClose}
        class="tg-btn uppercase"
      >
        Cerrar
      </button>
    </div>
  </div>
</div>
