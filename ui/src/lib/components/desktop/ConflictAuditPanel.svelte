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

<div class="bg-panel border border-grid rounded p-3 shadow-md mb-3 text-xs">
  <div class="flex items-center justify-between mb-2 border-b border-grid pb-2">
    <div>
      <h3 class="text-xs font-bold text-ink tracking-wider uppercase">
        Auditoría Anti-Cruces y Descansos
      </h3>
    </div>
    <div>
      {#if conflicts.length === 0}
        <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-void text-green border border-green/40">
          <span class="w-1.5 h-1.5 rounded-full bg-green"></span>
          Sin colisiones de horario ni sala
        </span>
      {:else}
        <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-void text-red border border-red/40">
          <span class="w-1.5 h-1.5 rounded-full bg-red"></span>
          {conflicts.length} {conflicts.length === 1 ? 'conflicto detectado' : 'conflictos detectados'}
        </span>
      {/if}
    </div>
  </div>

  {#if conflicts.length === 0}
    <div class="py-2 px-3 rounded bg-void border border-grid text-[11px] text-dim flex items-center gap-2">
      <span class="text-green font-bold font-mono">✓</span>
      <div>
        <span class="font-bold text-ink">Cronograma validado:</span>
        No se registran superposiciones de bailarines, pianistas o maestros, ni excesos de franja, descansos inter-jornada (12h), pausas de clase ni francos compensatorios.
      </div>
    </div>
  {:else}
    <div class="space-y-1.5">
      {#each conflicts as c}
        {#if c.conflict_type === 'PERSON_DOUBLE_BOOKED'}
          <div class="p-2.5 rounded bg-void border border-red/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-red bg-void px-1.5 py-0.5 rounded border border-red/30 shrink-0">
              DOBLE CITACIÓN
            </span>
            <div class="flex-1">
              <div class="font-bold text-red flex items-center justify-between">
                <span>{c.person_name} ({roleLabel(c.role)})</span>
                <span class="font-mono text-[11px] text-red">
                  {c.overlap_start} - {c.overlap_end}
                </span>
              </div>
              <p class="text-ink text-[11px] mt-0.5">
                Superposición entre <strong class="text-red">{c.first_room}</strong> ("{c.first_passage}") y <strong class="text-red">{c.second_room}</strong> ("{c.second_passage}").
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'ROOM_DOUBLE_BOOKED'}
          <div class="p-2.5 rounded bg-raised border border-amber/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-amber bg-void px-1.5 py-0.5 rounded border border-amber/30 shrink-0">
              SALA SOLAPADA
            </span>
            <div class="flex-1">
              <div class="font-bold text-amber flex items-center justify-between">
                <span>Colisión en {roomLabel(c.room)}</span>
                <span class="font-mono text-[11px] text-amber">
                  {c.overlap_start} - {c.overlap_end}
                </span>
              </div>
              <p class="text-ink text-[11px] mt-0.5">
                Solapamiento entre pasajes "{c.first_passage}" y "{c.second_passage}" sin permiso de sala compartida habilitado.
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'SHIFT_LIMIT_EXCEEDED'}
          <div class="p-2.5 rounded bg-void border border-red/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-red bg-void px-1.5 py-0.5 rounded border border-red/30 shrink-0">
              EXCESO HORARIO
            </span>
            <div class="flex-1">
              <div class="font-bold text-red flex items-center justify-between">
                <span>Límite de Régimen Excedido (Tope {c.allowed_limit_time} h)</span>
                <span class="font-mono text-[11px] text-red">
                  Fin: {c.block_end_time} h
                </span>
              </div>
              <p class="text-ink text-[11px] mt-0.5">
                El ensayo "{c.passage_name}" excede el límite del régimen actual. {c.suggestion}
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'MINIMUM_INTERSHIFT_REST_VIOLATION'}
          <div class="p-2.5 rounded bg-void border border-violet/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-violet bg-void px-1.5 py-0.5 rounded border border-violet/30 shrink-0">
              DESCANSO INSUFICIENTE
            </span>
            <div class="flex-1">
              <div class="font-bold text-violet flex items-center justify-between">
                <span>Descanso Mínimo Inter-Jornada menor a 12h</span>
                <span class="font-mono text-[11px] text-violet">
                  {formatMinutes(c.actual_rest_minutes)} descanso
                </span>
              </div>
              <p class="text-ink text-[11px] mt-0.5">
                Fin anterior: {c.previous_end_time} ({c.previous_date}) → Fichaje actual: {c.current_checkin_time} ({c.current_date}).
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'CLASS_TO_REHEARSAL_REST_VIOLATION'}
          <div class="p-2.5 rounded bg-raised border border-amber/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-amber bg-void px-1.5 py-0.5 rounded border border-amber/30 shrink-0">
              PAUSA CLASE INSUFICIENTE
            </span>
            <div class="flex-1">
              <div class="font-bold text-amber flex items-center justify-between">
                <span>Pausa reglamentaria menor a {c.required_gap_minutes} min</span>
                <span class="font-mono text-[11px] text-amber">
                  Pausa: {c.actual_gap_minutes}m (Mín: {c.required_gap_minutes}m)
                </span>
              </div>
              <p class="text-ink text-[11px] mt-0.5">
                Clase finaliza {c.class_end_time} ({c.class_room}) y el ensayo "{c.rehearsal_passage}" inicia a las {c.rehearsal_start_time}. Debe respetarse el tiempo de hidratación y cambio de calzado.
              </p>
            </div>
          </div>
        {:else if c.conflict_type === 'COMPENSATORY_REST_DAY_VIOLATION'}
          <div class="p-2.5 rounded bg-void border border-red/50 text-xs flex items-start gap-2.5">
            <span class="font-mono text-[10px] uppercase font-bold text-red bg-void px-1.5 py-0.5 rounded border border-red/30 shrink-0">
              FRANCO COMPENSATORIO
            </span>
            <div class="flex-1">
              <div class="font-bold text-red flex items-center justify-between">
                <span>Día de Franco Compensatorio tras Función</span>
                <span class="font-mono text-[11px] text-red">
                  {c.rehearsal_blocks_count} {c.rehearsal_blocks_count === 1 ? 'bloque citado' : 'bloques citados'}
                </span>
              </div>
              <p class="text-ink text-[11px] mt-0.5">
                {c.suggestion}
              </p>
            </div>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>
