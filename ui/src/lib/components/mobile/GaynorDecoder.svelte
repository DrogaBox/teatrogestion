<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type GaynorParsedSpec } from '../../ipc/client';

  let rawCode = $state('SC - 7.5W4PDH +6mm');
  let decoded = $state<GaynorParsedSpec | null>(null);
  let errorMsg = $state<string | null>(null);

  const presets = [
    'SC - 7.5W4PDH +6mm',
    'SK - 8.5M4HDL',
    'CL - 8M3XDH',
    'SC - 8W4PDH (+1/2")',
    'CL - 7.5M3SDH'
  ];

  async function decode(code: string) {
    rawCode = code;
    errorMsg = null;
    try {
      decoded = await api.parseGaynorCodePreview(code);
    } catch (e: any) {
      errorMsg = e?.message || 'Código Gaynor inválido';
      decoded = null;
    }
  }

  onMount(() => {
    decode(rawCode);
  });

  function shankColor(shank: string) {
    switch (shank) {
      case 'EXTRAFLEX': return { bg: 'bg-yellow-500/20', text: 'text-yellow-400', border: 'border-yellow-500', name: 'Bolsa Amarilla (Extraflex - X)' };
      case 'SUPPLE': return { bg: 'bg-pink-500/20', text: 'text-pink-400', border: 'border-pink-500', name: 'Bolsa Rosa (Supple - S)' };
      case 'FEATHER': return { bg: 'bg-blue-500/20', text: 'text-blue-400', border: 'border-blue-500', name: 'Bolsa Azul (Feather - F)' };
      case 'PIANISSIMO': return { bg: 'bg-purple-500/20', text: 'text-purple-300', border: 'border-purple-400', name: 'Bolsa Lavanda (Pianissimo - P)' };
      case 'HARD': return { bg: 'bg-emerald-500/20', text: 'text-emerald-400', border: 'border-emerald-500', name: 'Bolsa Verde (Hard - H)' };
      default: return { bg: 'bg-slate-800', text: 'text-slate-300', border: 'border-slate-700', name: shank };
    }
  }
</script>

<div class="bg-panel] border border-grid] rounded p-4 shadow-md">
  <div class="flex items-center justify-between mb-3">
    <div class="flex items-center gap-2">
      <span class="text-amber] text-xs font-mono font-bold">[GAYNOR]</span>
      <h3 class="text-xs font-bold uppercase tracking-wider text-ink]">Decodificador Técnico de Calzado</h3>
    </div>
    <span class="text-[10px] bg-void] text-dim] px-2 py-0.5 rounded border border-grid] font-mono">Tauri v2 IPC</span>
  </div>

  <!-- Input Táctil -->
  <div class="relative mb-3">
    <input
      type="text"
      bind:value={rawCode}
      oninput={(e) => decode((e.target as HTMLInputElement).value)}
      placeholder="Ej: SC - 7.5W4PDH +6mm"
      class="w-full h-11 pl-4 pr-10 bg-void] border border-grid] focus:border-amber] rounded text-sm font-mono font-bold text-ink] outline-none transition-colors"
    />
    {#if rawCode}
      <button
        onclick={() => decode('')}
        class="absolute right-2 top-2 h-7 w-7 flex items-center justify-center text-dim] hover:text-white font-mono text-xs"
      >
        ✕
      </button>
    {/if}
  </div>

  <!-- Presets rápidos para un toque -->
  <div class="flex gap-2 overflow-x-auto pb-2 mb-3 no-scrollbar">
    {#each presets as p}
      <button
        onclick={() => decode(p)}
        class="whitespace-nowrap px-3 py-1.5 bg-raised] hover:bg-grid] border border-grid] rounded text-xs font-mono font-medium text-dim] hover:text-ink] transition-colors"
      >
        {p}
      </button>
    {/each}
  </div>

  <!-- Resultado Decodificado -->
  {#if decoded}
    {@const shankInfo = shankColor(decoded.shank)}
    <div class="bg-panel] border border-grid] rounded-md p-4 text-ink]">
      <div class="flex items-center justify-between pb-3 mb-3 border-b border-grid]">
        <div>
          <div class="text-[10px] uppercase text-dim] font-semibold tracking-wider">Código Normalizado</div>
          <div class="text-base font-mono font-bold text-amber]">{decoded.raw_code}</div>
        </div>
        <div class="text-right">
          <span class="inline-block px-2.5 py-0.5 rounded text-xs font-semibold {shankInfo.bg} {shankInfo.text} border {shankInfo.border}">
            {shankInfo.name}
          </span>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-2 text-xs">
        <div class="bg-void] p-2.5 rounded border border-grid]">
          <div class="text-[10px] text-dim] font-semibold uppercase tracking-wider">Línea & Fit</div>
          <div class="font-medium text-ink] mt-0.5">
            {#if decoded.fit === 'SCULPTED'}
              Sculpted (SC) — Empeine Medio/Alto
            {:else if decoded.fit === 'SLEEK'}
              Sleek (SK) — Empeine Plano
            {:else}
              Classic (CL) — Empeine Ancho
            {/if}
          </div>
        </div>

        <div class="bg-void] p-2.5 rounded border border-grid]">
          <div class="text-[10px] text-dim] font-semibold uppercase tracking-wider">Talle & Ancho</div>
          <div class="font-medium text-ink] mt-0.5 font-mono">
            Talle {decoded.size} — Ancho {decoded.width === 'WIDE' ? 'W (Wide)' : decoded.width === 'MEDIUM' ? 'M (Medium)' : 'N (Narrow)'}
          </div>
        </div>

        <div class="bg-void] p-2.5 rounded border border-grid]">
          <div class="text-[10px] text-dim] font-semibold uppercase tracking-wider">Caja de Apoyo</div>
          <div class="font-medium text-ink] mt-0.5 font-mono">
            Caja #{decoded.box_shape}
            <span class="text-xs text-dim] font-normal">
              {decoded.box_shape === 2 ? '(Estrecha)' : decoded.box_shape === 3 ? '(Media)' : decoded.box_shape === 4 ? '(Cuadrada)' : '(Extra Ancha)'}
            </span>
          </div>
        </div>

        <div class="bg-void] p-2.5 rounded border border-grid]">
          <div class="text-[10px] text-dim] font-semibold uppercase tracking-wider">Capellada & Talón</div>
          <div class="font-medium text-ink] mt-0.5 flex items-center gap-1.5 flex-wrap">
            <span>{decoded.vamp === 'LOW' ? 'Low' : 'Deep'}</span>
            {#if decoded.vamp === 'DEEP_PLUS_6MM'}
              <span class="bg-green]/15 text-green] border border-green]/30 text-[9px] px-1 py-0.5 rounded font-mono font-semibold">+6mm</span>
            {:else if decoded.vamp === 'DEEP_PLUS_HALF_INCH'}
              <span class="bg-green]/15 text-green] border border-green]/30 text-[9px] px-1 py-0.5 rounded font-mono font-semibold">+1/2"</span>
            {/if}
            <span class="text-dim]">•</span>
            <span>Talón {decoded.heel === 'HIGH' ? 'Alto (H)' : 'Bajo (L)'}</span>
          </div>
        </div>
      </div>
    </div>
  {:else if errorMsg}
    <div class="bg-red]/10 border border-red]/40 text-red] p-3 rounded text-xs font-medium">
      {errorMsg}
    </div>
  {/if}
</div>
