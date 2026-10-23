<script lang="ts">
  import { roleLabel, roomLabel } from '../../labels';
  import type { ScheduleConflict } from '../../ipc/client';

  let { conflicts } = $props<{ conflicts: ScheduleConflict[] }>();

  function formatMinutes(minutes?: number): string {
    if (minutes === undefined) return '0h';
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return m > 0 ? `${h}h ${m}m` : `${h}h`;
  }
</script>

<div class="bg-[#1a1a1e] border border-[#34343c] rounded p-3 shadow-md mb-3 text-xs">
  <div class="flex items-center justify-between mb-2 border-b border-[#34343c] pb-2">
    <div>
      <h3 class="text-xs font-bold text-[#d8d8dc] tracking-wider uppercase">
        Auditoría Anti-Cruces y Descansos
      </h3>
    </div>
    <div>
      {#if conflicts.length === 0}
        <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-[#1e2a22] text-[#3de06a] border border-[#3de06a]/40">
          <span class="w-1.5 h-1.5 rounded-full bg-[#3de06a]"></span>
          Sin colisiones de horario ni sala
        </span>
      {:else}
        <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-[#2a1616] text-[#ff4a4a] border border-[#ff4a4a]/40">
          <span class="w-1.5 h-1.5 rounded-full bg-[#ff4a4a]"></span>
          {conflicts.length} {conflicts.length === 1 ? 'conflicto detectado' : 'conflictos detectados'}
        </span>
      {/if}
    </div>
  </div>

  {#if conflicts.length === 0}
    <div class="py-2 px-3 rounded bg-[#141416] border border-[#34343c] text-[11px] text-[#82828a] flex items-center gap-2">
      <span class="text-[#3de06a] font-bold font-mono">✓</span>
      <div>
        <span class="font-bold text-[#d8d8dc]">Cronograma validado:</span>
        No se registran superposiciones de bailarines, pianistas o maestros, ni excesos de franja, descansos inter-jornada (12h), pausas de clase ni francos compensatorios.
      </div>
    </div>
  {:else}
    <div class="space-y-1.5">
      {#each conflicts as c}
        {#if c.conflict_type === 'PERSON_DOUBLE_BOOKED'}
          <div class="p-2.5 rounded bg-[#2a1616] border border-[#ff4a4a]/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-[#ff4a4a] bg-[#121214] px-1.5 py-0.5 rounded border border-[#ff4a4a]/30 shrink-0">
              DOBLE CITACIÓN
            </span>
            <div class="flex-1">
              <div class="font-bold text-[#ffd0d0] flex items-center justify-between">
                <span>{c.person_name} ({roleLabel(c.role)})</span>
                <span class="font-mono text-[11px] text-[#ff4a4a]">
                  {c.overlap_start} - {c.overlap_end}
                </span>
              </div>
              <p class="text-[#d8d8dc] text-[11px] mt-0.5">
                Superposición entre <strong class="text-[#ffd0d0]">{c.first_room}</strong> ("{c.first_passage}") y <strong class="text-[#ffd0d0]">{c.second_room}</strong> ("{c.second_passage}").
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'ROOM_DOUBLE_BOOKED'}
          <div class="p-2.5 rounded bg-[#2a2218] border border-[#ffb31a]/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-[#ffb31a] bg-[#121214] px-1.5 py-0.5 rounded border border-[#ffb31a]/30 shrink-0">
              SALA SOLAPADA
            </span>
            <div class="flex-1">
              <div class="font-bold text-[#ffe2a8] flex items-center justify-between">
                <span>Colisión en {roomLabel(c.room)}</span>
                <span class="font-mono text-[11px] text-[#ffb31a]">
                  {c.overlap_start} - {c.overlap_end}
                </span>
              </div>
              <p class="text-[#d8d8dc] text-[11px] mt-0.5">
                Solapamiento entre pasajes "{c.first_passage}" y "{c.second_passage}" sin permiso de sala compartida habilitado.
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'SHIFT_LIMIT_EXCEEDED'}
          <div class="p-2.5 rounded bg-[#2a1616] border border-[#ff4a4a]/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-[#ff4a4a] bg-[#121214] px-1.5 py-0.5 rounded border border-[#ff4a4a]/30 shrink-0">
              EXCESO HORARIO
            </span>
            <div class="flex-1">
              <div class="font-bold text-[#ffd0d0] flex items-center justify-between">
                <span>Límite de Régimen Excedido (Tope {c.allowed_limit_time} h)</span>
                <span class="font-mono text-[11px] text-[#ff4a4a]">
                  Fin: {c.block_end_time} h
                </span>
              </div>
              <p class="text-[#d8d8dc] text-[11px] mt-0.5">
                El ensayo "{c.passage_name}" excede el límite del régimen actual. {c.suggestion}
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'MINIMUM_INTERSHIFT_REST_VIOLATION'}
          <div class="p-2.5 rounded bg-[#22182a] border border-[#9d72ff]/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-[#9d72ff] bg-[#121214] px-1.5 py-0.5 rounded border border-[#9d72ff]/30 shrink-0">
              DESCANSO INSUFICIENTE
            </span>
            <div class="flex-1">
              <div class="font-bold text-[#e0d0ff] flex items-center justify-between">
                <span>Descanso Mínimo Inter-Jornada menor a 12h</span>
                <span class="font-mono text-[11px] text-[#9d72ff]">
                  {formatMinutes(c.actual_rest_minutes)} descanso
                </span>
              </div>
              <p class="text-[#d8d8dc] text-[11px] mt-0.5">
                Fin anterior: {c.previous_end_time} ({c.previous_date}) → Fichaje actual: {c.current_checkin_time} ({c.current_date}).
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'CLASS_TO_REHEARSAL_REST_VIOLATION'}
          <div class="p-2.5 rounded bg-[#2a2218] border border-[#ffb31a]/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-[#ffb31a] bg-[#121214] px-1.5 py-0.5 rounded border border-[#ffb31a]/30 shrink-0">
              PAUSA CLASE INSUFICIENTE
            </span>
            <div class="flex-1">
              <div class="font-bold text-[#ffe2a8] flex items-center justify-between">
                <span>Pausa reglamentaria menor a {c.required_gap_minutes} min</span>
                <span class="font-mono text-[11px] text-[#ffb31a]">
                  Pausa: {c.actual_gap_minutes}m (Mín: {c.required_gap_minutes}m)
                </span>
              </div>
              <p class="text-[#d8d8dc] text-[11px] mt-0.5">
                Clase finaliza {c.class_end_time} ({c.class_room}) y el ensayo "{c.rehearsal_passage}" inicia a las {c.rehearsal_start_time}. Debe respetarse el tiempo de hidratación y cambio de calzado.
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'COMPENSATORY_REST_DAY_VIOLATION'}
          <div class="p-2.5 rounded bg-[#2a1616] border border-[#ff4a4a]/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-[#ff4a4a] bg-[#121214] px-1.5 py-0.5 rounded border border-[#ff4a4a]/30 shrink-0">
              FRANCO COMPENSATORIO
            </span>
            <div class="flex-1">
              <div class="font-bold text-[#ffd0d0] flex items-center justify-between">
                <span>Día de Franco Compensatorio tras Función</span>
                <span class="font-mono text-[11px] text-[#ff4a4a]">
                  {c.rehearsal_blocks_count} {c.rehearsal_blocks_count === 1 ? 'bloque citado' : 'bloques citados'}
                </span>
              </div>
              <p class="text-[#d8d8dc] text-[11px] mt-0.5">
                {c.suggestion}
              </p>
            </div>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>
