<script lang="ts">
  import { roomLabel } from '../../labels';
  import { onMount } from 'svelte';
  import {
    api,
    type DailyScheduleAggregate,
    type RehearsalBlock,
    type WorkRegime,
    type UpsertRehearsalBlockPayload
  } from '../../ipc/client';

  let {
    currentDate,
    onSelectDate,
    onOpenCitationModal,
    onSwitchToDailyView
  } = $props<{
    currentDate: string;
    onSelectDate: (date: string) => void;
    onOpenCitationModal: (date: string, block?: RehearsalBlock) => void;
    onSwitchToDailyView: (date: string) => void;
  }>();

  interface DayScheduleData {
    date: string;
    dayName: string;
    dayNumber: number;
    monthName: string;
    isToday: boolean;
    isSelected: boolean;
    schedule: DailyScheduleAggregate | null;
    isLoading: boolean;
  }

  let weekDays = $state<DayScheduleData[]>([]);
  let weekStartStr = $state('');
  let weekEndStr = $state('');
  let isCloning = $state(false);
  let showCopyModal = $state(false);
  let copySourceDate = $state('');
  let copyTargetDate = $state('');
  let showPrintWeekly = $state(false);

  function getMonday(d: Date): Date {
    const date = new Date(d);
    const day = date.getDay();
    const diff = date.getDate() - day + (day === 0 ? -6 : 1);
    return new Date(date.setDate(diff));
  }

  function formatIso(d: Date): string {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  const DAY_NAMES = ['LUNES', 'MARTES', 'MIÉRCOLES', 'JUEVES', 'VIERNES', 'SÁBADO'];

  async function calculateAndLoadWeek(baseDateStr: string) {
    const [y, m, d] = baseDateStr.split('-').map(Number);
    const baseDate = new Date(y, m - 1, d);
    const monday = getMonday(baseDate);

    const days: DayScheduleData[] = [];
    const todayStr = formatIso(new Date());

    for (let i = 0; i < 6; i++) {
      const currentDay = new Date(monday);
      currentDay.setDate(monday.getDate() + i);
      const iso = formatIso(currentDay);

      days.push({
        date: iso,
        dayName: DAY_NAMES[i],
        dayNumber: currentDay.getDate(),
        monthName: currentDay.toLocaleDateString('es-AR', { month: 'short' }).toUpperCase().replace('.', ''),
        isToday: iso === todayStr,
        isSelected: iso === currentDate,
        schedule: null,
        isLoading: true
      });
    }

    weekDays = days;
    weekStartStr = days[0].date;
    weekEndStr = days[days.length - 1].date;

    copySourceDate = currentDate;
    copyTargetDate = days.find(d => d.date !== currentDate)?.date || days[1].date;

    // Cargar cronogramas en paralelo
    await Promise.all(
      days.map(async (day, index) => {
        try {
          const res = await api.getOrCreateDailySchedule(day.date);
          weekDays[index].schedule = res.schedule;
        } catch (e) {
          console.error(`Error cargando día ${day.date}:`, e);
        } finally {
          weekDays[index].isLoading = false;
        }
      })
    );
  }

  function shiftWeek(offsetWeeks: number) {
    const [y, m, d] = currentDate.split('-').map(Number);
    const current = new Date(y, m - 1, d);
    current.setDate(current.getDate() + (offsetWeeks * 7));
    const nextIso = formatIso(current);
    onSelectDate(nextIso);
  }

  function goToCurrentWeek() {
    const todayIso = formatIso(new Date());
    onSelectDate(todayIso);
  }

  // 1-Click: Duplicar cronograma de un día a otro
  async function executeCopyDay() {
    if (!copySourceDate || !copyTargetDate || copySourceDate === copyTargetDate) {
      alert('Seleccione dos fechas distintas para copiar.');
      return;
    }
    isCloning = true;
    try {
      const sourceRes = await api.getOrCreateDailySchedule(copySourceDate);
      const targetRes = await api.getOrCreateDailySchedule(copyTargetDate);

      const targetScheduleId = targetRes.schedule.id;

      // 1. Copiar clases matutinas si no existen
      for (const mc of sourceRes.schedule.morning_classes) {
        await api.upsertMorningClass({
          schedule_id: targetScheduleId,
          room: mc.room,
          gender_division: mc.gender_division,
          start_time: mc.start_time,
          end_time: mc.end_time,
          master_id: mc.master_id,
          pianist_id: mc.pianist_id
        });
      }

      // 2. Copiar bloques de ensayo
      for (const rb of sourceRes.schedule.rehearsal_blocks) {
        const payload: UpsertRehearsalBlockPayload = {
          schedule_id: targetScheduleId,
          room: rb.room,
          start_time: rb.start_time,
          end_time: rb.end_time,
          activity_type: rb.activity_type,
          work_id: rb.work_id,
          passage_id: rb.passage_id,
          custom_passage_label: rb.passage_name,
          confirmation_state: rb.confirmation_state,
          sound_requirement: rb.sound_requirement,
          teatrocue_cue_ref: rb.teatrocue_cue_ref,
          master_id: rb.master_id,
          pianist_id: rb.pianist_id,
          allow_shared_room: rb.allow_shared_room,
          cast_groups: rb.cast_groups.map(g => ({
            group_order: g.group_order,
            group_label: g.group_label,
            is_to_confirm: g.is_to_confirm,
            members: g.members.map(m => ({
              personnel_id: m.personnel_id,
              character_role: m.character_role,
              partner_personnel_id: m.partner_personnel_id,
              condition: m.condition
            }))
          }))
        };
        await api.upsertRehearsalBlock(payload);
      }

      showCopyModal = false;
      await calculateAndLoadWeek(currentDate);
    } catch (e: any) {
      alert(`Error al copiar cronograma: ${e?.message || e}`);
    } finally {
      isCloning = false;
    }
  }

  function getRoomBadge(room: string): { label: string; bg: string; text: string } {
    switch (room) {
      case 'SALA_1':
        return { label: 'Sala 1', bg: 'bg-[#1e2a22]', text: 'text-[#3de06a]' };
      case 'SALA_2':
        return { label: 'Sala 2', bg: 'bg-[#18262a]', text: 'text-[#2ed6e8]' };
      case 'SALA_3':
        return { label: 'Sala 3', bg: 'bg-[#2a2218]', text: 'text-[#ffb31a]' };
      case 'ESCENARIO':
        return { label: 'Escenario', bg: 'bg-[#2a1a1e]', text: 'text-[#ff7070]' };
      default:
        return { label: 'Gira', bg: 'bg-[#202026]', text: 'text-[#82828a]' };
    }
  }

  function getRegimeLabel(regime?: WorkRegime): string {
    switch (regime) {
      case 'PRE_FUNCTION': return 'Pre-Función';
      case 'EXTRAORDINARY': return 'Doble Turno';
      default: return 'Ordinario';
    }
  }

  $effect(() => {
    if (currentDate) {
      calculateAndLoadWeek(currentDate);
    }
  });
</script>

<div class="space-y-3">
  <!-- Barra de Herramientas de Planificación Semanal (Estilo TeatroCue) -->
  <div class="bg-[#1a1a1e] border border-[#34343c] rounded p-2.5 flex flex-wrap items-center justify-between gap-2 text-xs">
    
    <!-- Navegación de Semana -->
    <div class="flex items-center gap-1">
      <button
        onclick={() => shiftWeek(-1)}
        class="h-8 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] border border-[#34343c] rounded font-mono active:scale-95"
        title="Semana anterior"
      >
        ◀
      </button>
      <button
        onclick={goToCurrentWeek}
        class="h-8 px-3 bg-[#24242a] hover:bg-[#34343e] text-[#ffb31a] border border-[#34343c] rounded font-bold"
      >
        Esta Semana
      </button>
      <button
        onclick={() => shiftWeek(1)}
        class="h-8 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] border border-[#34343c] rounded font-mono active:scale-95"
        title="Semana siguiente"
      >
        ▶
      </button>
      <span class="ml-2 font-mono text-[#d8d8dc] font-bold">
        {weekStartStr} al {weekEndStr}
      </span>
    </div>

    <!-- Acciones Rápidas de Planificación -->
    <div class="flex items-center gap-2">
      <button
        onclick={() => showCopyModal = true}
        class="h-8 px-3 bg-[#24242a] hover:bg-[#34343e] text-[#2ed6e8] border border-[#34343c] rounded font-medium flex items-center gap-1.5"
      >
        <span>Duplicar Día</span>
      </button>

      <button
        onclick={() => showPrintWeekly = true}
        class="h-8 px-3 bg-[#24242a] hover:bg-[#34343e] text-[#ffb31a] border border-[#34343c] rounded font-bold flex items-center gap-1.5"
      >
        <span>Imprimir Plan Semanal</span>
      </button>
    </div>
  </div>

  <!-- Grilla Semanal: 6 Columnas (Lunes a Sábado) -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-2">
    {#each weekDays as day}
      <div class="bg-[#1a1a1e] border {day.isSelected ? 'border-[#ffb31a]' : 'border-[#34343c]'} rounded flex flex-col min-h-[580px] shadow-sm">
        
        <!-- Cabecera del Día -->
        <div class="p-2 border-b border-[#34343c] {day.isToday ? 'bg-[#24242a]' : 'bg-[#17171b]'} flex items-center justify-between">
          <div>
            <div class="flex items-center gap-1.5">
              <span class="text-xs font-black tracking-wider text-[#d8d8dc]">{day.dayName}</span>
              <span class="text-xs font-mono font-bold {day.isToday ? 'text-[#ffb31a]' : 'text-[#82828a]'}">
                {day.dayNumber} {day.monthName}
              </span>
            </div>
            <div class="text-[10px] text-[#82828a] font-mono mt-0.5">
              {getRegimeLabel(day.schedule?.regime)}
            </div>
          </div>

          <button
            onclick={() => onSwitchToDailyView(day.date)}
            class="text-[10px] px-1.5 py-0.5 bg-[#24242a] hover:bg-[#34343e] text-[#2ed6e8] border border-[#34343c] rounded font-mono"
            title="Abrir en vista diaria detallada"
          >
            Ver día ➔
          </button>
        </div>

        <!-- Clases Matutinas Resumen -->
        {#if day.schedule && day.schedule.morning_classes.length > 0}
          <div class="p-1.5 bg-[#141416] border-b border-[#34343c] text-[10px] space-y-1">
            <span class="text-[#82828a] font-mono uppercase block font-bold text-[9px]">Clases 09:00 - 10:15:</span>
            {#each day.schedule.morning_classes as mc}
              <div class="text-[#d8d8dc] truncate">
                <span class="text-[#3de06a] font-mono font-bold">{mc.room === 'SALA_1' ? 'S1' : 'S3'}:</span>
                <span>{mc.master_name?.split(',')[0]}</span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Lista de Bloques de Ensayo -->
        <div class="flex-1 p-1.5 overflow-y-auto space-y-1.5">
          {#if day.isLoading}
            <div class="p-6 text-center text-[#82828a] text-[11px] font-mono">
              Cargando...
            </div>
          {:else if !day.schedule || day.schedule.rehearsal_blocks.length === 0}
            <div class="p-6 text-center text-[#82828a] text-[11px] font-mono italic">
              Sin ensayos citados
            </div>
          {:else}
            {#each day.schedule.rehearsal_blocks as block}
              {@const badge = getRoomBadge(block.room)}
              <button
                type="button"
                onclick={() => onOpenCitationModal(day.date, block)}
                class="w-full text-left p-2 bg-[#24242a] hover:bg-[#34343e] border border-[#34343c] rounded transition-colors group"
              >
                <!-- Horario y Sala -->
                <div class="flex items-center justify-between gap-1 mb-1">
                  <span class="text-[10px] font-mono font-bold text-[#ffb31a]">
                    {block.start_time} - {block.end_time}
                  </span>
                  <span class="text-[9px] px-1 py-0.2 rounded font-mono font-bold {badge.bg} {badge.text}">
                    {badge.label}
                  </span>
                </div>

                <!-- Obra / Pasaje -->
                <div class="text-[11px] font-bold text-[#d8d8dc] truncate group-hover:text-white">
                  {block.passage_name}
                </div>

                <!-- Reparto / Bailarines -->
                {#if block.cast_groups.length > 0 && block.cast_groups[0].formatted_line}
                  <div class="text-[10px] text-[#82828a] truncate mt-0.5 font-mono">
                    {block.cast_groups[0].formatted_line}
                  </div>
                {/if}

                <!-- Maestro / Pianista / Pista -->
                <div class="text-[9px] text-[#82828a] truncate mt-1 flex items-center justify-between">
                  <span>{block.master_name ? block.master_name.split(',')[0] : 'Sin maestro'}</span>
                  <span class="font-mono text-[#2ed6e8]">
                    {block.teatrocue_cue_ref ? `Cue: ${block.teatrocue_cue_ref}` : block.pianist_name ? 'Piano' : ''}
                  </span>
                </div>
              </button>
            {/each}
          {/if}
        </div>

        <!-- Botón para Citar en este Día -->
        <div class="p-1.5 border-t border-[#34343c] bg-[#17171b]">
          <button
            type="button"
            onclick={() => onOpenCitationModal(day.date)}
            class="w-full py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] hover:text-[#ffb31a] border border-[#34343c] rounded text-xs font-bold font-mono transition-colors"
          >
            + Citar Ensayo
          </button>
        </div>

      </div>
    {/each}
  </div>
</div>

<!-- MODAL PARA DUPLICAR/COPIAR DÍA -->
{#if showCopyModal}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3">
    <div class="w-full max-w-md bg-[#1a1a1e] border border-[#34343c] rounded-lg p-5 shadow-2xl text-xs space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-[#34343c]">
        <h3 class="text-sm font-bold text-[#d8d8dc] uppercase tracking-wide">
          Duplicar Cronograma de Ensayo
        </h3>
        <button onclick={() => showCopyModal = false} class="text-[#82828a] hover:text-white">✕</button>
      </div>

      <p class="text-[#82828a]">
        Copia todas las clases matutinas, requerimientos de sonido, maestros y bloques de ensayo desde el día origen al día destino.
      </p>

      <div class="space-y-3">
        <div>
          <label for="weekly-copy-source" class="block text-[10px] uppercase font-bold text-[#82828a] mb-1">Día Origen (Fuente)</label>
          <select id="weekly-copy-source" bind:value={copySourceDate} class="w-full h-10 px-3 bg-[#141416] border border-[#34343c] rounded text-xs text-[#d8d8dc] font-mono">
            {#each weekDays as d}
              <option value={d.date}>{d.dayName} {d.date} ({d.schedule?.rehearsal_blocks.length || 0} ensayos)</option>
            {/each}
          </select>
        </div>

        <div>
          <label for="weekly-copy-target" class="block text-[10px] uppercase font-bold text-[#82828a] mb-1">Día Destino</label>
          <select id="weekly-copy-target" bind:value={copyTargetDate} class="w-full h-10 px-3 bg-[#141416] border border-[#34343c] rounded text-xs text-[#d8d8dc] font-mono">
            {#each weekDays as d}
              <option value={d.date}>{d.dayName} {d.date}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="pt-3 border-t border-[#34343c] flex gap-2">
        <button
          onclick={() => showCopyModal = false}
          class="flex-1 h-9 bg-[#24242a] hover:bg-[#34343e] text-[#82828a] rounded font-bold"
        >
          Cancelar
        </button>
        <button
          disabled={isCloning}
          onclick={executeCopyDay}
          class="flex-1 h-9 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] font-black rounded uppercase tracking-wide"
        >
          {isCloning ? 'Duplicando...' : 'Copiar Día'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL DE IMPRESIÓN DEL PLAN SEMANAL -->
{#if showPrintWeekly}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3 print:p-0 print:bg-white print:fixed print:inset-0">
    <div class="w-full max-w-5xl max-h-[92vh] flex flex-col bg-[#1a1a1e] print:bg-white border border-[#34343c] print:border-none rounded-lg shadow-2xl overflow-hidden print:w-full print:h-auto print:max-h-none">
      
      <!-- Barra superior no imprimible -->
      <div class="p-3 border-b border-[#34343c] flex items-center justify-between print:hidden bg-[#17171b]">
        <span class="text-xs font-bold text-[#d8d8dc] uppercase font-mono">
          Plan Semanal de Ensayos — {weekStartStr} al {weekEndStr}
        </span>
        <div class="flex items-center gap-2">
          <button
            onclick={() => window.print()}
            class="px-4 py-1.5 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] font-bold text-xs rounded"
          >
            Imprimir (A4 Horizontal)
          </button>
          <button
            onclick={() => showPrintWeekly = false}
            class="px-3 py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] text-xs rounded"
          >
            Cerrar
          </button>
        </div>
      </div>

      <!-- Contenido Imprimible -->
      <div class="flex-1 overflow-y-auto p-6 bg-white text-black font-sans text-xs print:p-0">
        <div class="text-center border-b-2 border-black pb-3 mb-4">
          <h1 class="text-base font-black uppercase tracking-wider">TEATRO DEL LIBERTADOR SAN MARTÍN</h1>
          <h2 class="text-sm font-bold uppercase text-slate-800">BALLET OFICIAL DE CÓRDOBA</h2>
          <p class="text-xs font-mono font-bold mt-1">PLAN SEMANAL DE CITACIONES: {weekStartStr} AL {weekEndStr}</p>
        </div>

        <div class="grid grid-cols-6 gap-2">
          {#each weekDays as d}
            <div class="border border-black p-2 flex flex-col min-h-[350px]">
              <div class="border-b border-black pb-1 mb-1.5 text-center">
                <div class="font-black text-xs uppercase">{d.dayName}</div>
                <div class="font-mono text-[11px] font-bold">{d.date}</div>
                <div class="text-[9px] uppercase text-slate-600">{getRegimeLabel(d.schedule?.regime)}</div>
              </div>

              <!-- Clases -->
              {#if d.schedule && d.schedule.morning_classes.length > 0}
                <div class="text-[9px] border-b border-slate-300 pb-1 mb-1">
                  <span class="font-bold block">CLASES 09:00:</span>
                  {#each d.schedule.morning_classes as c}
                    <div>{c.room === 'SALA_1' ? 'Sala 1' : 'Sala 3'}: {c.master_name?.split(',')[0]}</div>
                  {/each}
                </div>
              {/if}

              <!-- Ensayos -->
              <div class="space-y-1.5 flex-1">
                {#if d.schedule}
                  {#each d.schedule.rehearsal_blocks as b}
                    <div class="text-[9px] border border-slate-400 p-1 rounded">
                      <div class="font-bold font-mono">{b.start_time}-{b.end_time} • {roomLabel(b.room)}</div>
                      <div class="font-bold">{b.passage_name}</div>
                      {#if b.cast_groups.length > 0 && b.cast_groups[0].formatted_line}
                        <div class="text-[8px] text-slate-700 italic">{b.cast_groups[0].formatted_line}</div>
                      {/if}
                    </div>
                  {/each}
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <div class="text-center font-bold font-mono text-[10px] mt-6 pt-3 border-t border-black">
          ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
        </div>
      </div>

    </div>
  </div>
{/if}
