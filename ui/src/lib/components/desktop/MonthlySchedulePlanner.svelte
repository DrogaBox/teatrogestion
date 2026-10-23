<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type DailyScheduleSummaryDto,
    type WorkRegime
  } from '../../ipc/client';

  let {
    currentDate,
    onSelectDate,
    onSwitchToDailyView,
    onSwitchToWeeklyView
  } = $props<{
    currentDate: string;
    onSelectDate: (date: string) => void;
    onSwitchToDailyView: (date: string) => void;
    onSwitchToWeeklyView: (date: string) => void;
  }>();

  interface CalendarMonthDay {
    date: string;
    dayNumber: number;
    isCurrentMonth: boolean;
    isToday: boolean;
    isSelected: boolean;
    summary?: DailyScheduleSummaryDto;
  }

  let year = $state(2026);
  let month = $state(10); // 1-indexed (1..12)
  let monthName = $state('');
  let calendarDays = $state<CalendarMonthDay[]>([]);
  let allSummaries = $state<DailyScheduleSummaryDto[]>([]);
  let isLoading = $state(true);

  // Estadísticas del mes
  let totalBlocksInMonth = $state(0);
  let daysWithRehearsals = $state(0);
  let daysDoubleTurn = $state(0);
  let daysPerformance = $state(0);

  const MONTH_NAMES = [
    'ENERO', 'FEBRERO', 'MARZO', 'ABRIL', 'MAYO', 'JUNIO',
    'JULIO', 'AGOSTO', 'SEPTIEMBRE', 'OCTUBRE', 'NOVIEMBRE', 'DICIEMBRE'
  ];

  const WEEK_HEADERS = ['LUN', 'MAR', 'MIÉ', 'JUE', 'VIE', 'SÁB', 'DOM'];

  function initFromDate(dStr: string) {
    const [y, m] = dStr.split('-').map(Number);
    year = y;
    month = m;
    monthName = MONTH_NAMES[month - 1];
  }

  async function loadMonthSummaries() {
    isLoading = true;
    try {
      allSummaries = await api.listDailySchedules();
      buildCalendarGrid();
    } catch (e) {
      console.error('Error cargando resúmenes de cronograma:', e);
    } finally {
      isLoading = false;
    }
  }

  function buildCalendarGrid() {
    monthName = MONTH_NAMES[month - 1];
    const todayStr = getTodayIso();

    const firstDayOfMonth = new Date(year, month - 1, 1);
    const lastDayOfMonth = new Date(year, month, 0);
    const totalDays = lastDayOfMonth.getDate();

    // 0 = Dom, 1 = Lun, ..., 6 = Sáb -> convertir a 0 = Lun, ..., 6 = Dom
    let startDayOfWeek = firstDayOfMonth.getDay() - 1;
    if (startDayOfWeek === -1) startDayOfWeek = 6;

    const days: CalendarMonthDay[] = [];

    // Días del mes anterior para rellenar
    const prevMonthLastDay = new Date(year, month - 1, 0).getDate();
    for (let i = startDayOfWeek - 1; i >= 0; i--) {
      const dNum = prevMonthLastDay - i;
      const prevM = month === 1 ? 12 : month - 1;
      const prevY = month === 1 ? year - 1 : year;
      const iso = `${prevY}-${String(prevM).padStart(2, '0')}-${String(dNum).padStart(2, '0')}`;
      days.push({
        date: iso,
        dayNumber: dNum,
        isCurrentMonth: false,
        isToday: iso === todayStr,
        isSelected: iso === currentDate,
        summary: allSummaries.find(s => s.schedule_date === iso)
      });
    }

    // Días del mes actual
    let bCount = 0;
    let dRehearsalCount = 0;
    let dDoubleCount = 0;
    let dPerfCount = 0;

    for (let d = 1; d <= totalDays; d++) {
      const iso = `${year}-${String(month).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
      const sum = allSummaries.find(s => s.schedule_date === iso);
      if (sum) {
        if (sum.rehearsal_blocks_count > 0) {
          bCount += sum.rehearsal_blocks_count;
          dRehearsalCount++;
        }
        if (sum.regime === 'EXTRAORDINARY') dDoubleCount++;
        if (sum.regime === 'PRE_FUNCTION') dPerfCount++;
      }

      days.push({
        date: iso,
        dayNumber: d,
        isCurrentMonth: true,
        isToday: iso === todayStr,
        isSelected: iso === currentDate,
        summary: sum
      });
    }

    totalBlocksInMonth = bCount;
    daysWithRehearsals = dRehearsalCount;
    daysDoubleTurn = dDoubleCount;
    daysPerformance = dPerfCount;

    // Rellenar hasta completar semanas completas (múltiplo de 7)
    const remaining = 7 - (days.length % 7);
    if (remaining < 7) {
      for (let i = 1; i <= remaining; i++) {
        const nextM = month === 12 ? 1 : month + 1;
        const nextY = month === 12 ? year + 1 : year;
        const iso = `${nextY}-${String(nextM).padStart(2, '0')}-${String(i).padStart(2, '0')}`;
        days.push({
          date: iso,
          dayNumber: i,
          isCurrentMonth: false,
          isToday: iso === todayStr,
          isSelected: iso === currentDate,
          summary: allSummaries.find(s => s.schedule_date === iso)
        });
      }
    }

    calendarDays = days;
  }

  function getTodayIso(): string {
    const d = new Date();
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  function shiftMonth(offset: number) {
    let nextM = month + offset;
    let nextY = year;
    if (nextM > 12) {
      nextM = 1;
      nextY++;
    } else if (nextM < 1) {
      nextM = 12;
      nextY--;
    }
    year = nextY;
    month = nextM;
    buildCalendarGrid();
  }

  function goToCurrentMonth() {
    initFromDate(getTodayIso());
    buildCalendarGrid();
  }

  function handleSelectDay(day: CalendarMonthDay) {
    onSelectDate(day.date);
    buildCalendarGrid();
  }

  function getRegimeTag(regime?: WorkRegime): { label: string; color: string } {
    switch (regime) {
      case 'PRE_FUNCTION': return { label: 'Función', color: 'text-[#ff7070]' };
      case 'EXTRAORDINARY': return { label: 'Doble T.', color: 'text-[#ffb31a]' };
      default: return { label: 'Ordinario', color: 'text-[#82828a]' };
    }
  }

  $effect(() => {
    if (currentDate) {
      initFromDate(currentDate);
      loadMonthSummaries();
    }
  });
</script>

<div class="space-y-3">
  <!-- Barra de Herramientas y Navegación de Mes -->
  <div class="bg-[#1a1a1e] border border-[#34343c] rounded p-2.5 flex flex-wrap items-center justify-between gap-3 text-xs">
    
    <!-- Navegación del Mes -->
    <div class="flex items-center gap-1.5">
      <button
        onclick={() => shiftMonth(-1)}
        class="h-8 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] border border-[#34343c] rounded font-mono active:scale-95"
        title="Mes anterior"
      >
        ◀
      </button>
      <button
        onclick={goToCurrentMonth}
        class="h-8 px-3 bg-[#24242a] hover:bg-[#34343e] text-[#ffb31a] border border-[#34343c] rounded font-bold"
      >
        Mes Actual
      </button>
      <button
        onclick={() => shiftMonth(1)}
        class="h-8 px-2.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] border border-[#34343c] rounded font-mono active:scale-95"
        title="Mes siguiente"
      >
        ▶
      </button>
      <span class="ml-2 font-mono text-[#d8d8dc] text-sm font-black tracking-wide">
        {monthName} {year}
      </span>
    </div>

    <!-- Métricas del Mes (Estilo TeatroCue) -->
    <div class="flex items-center gap-4 text-[11px] font-mono">
      <div>
        <span class="text-[#82828a]">Ensayos:</span>
        <span class="font-bold text-[#ffb31a] ml-1">{totalBlocksInMonth}</span>
      </div>
      <div>
        <span class="text-[#82828a]">Días c/ensayos:</span>
        <span class="font-bold text-[#3de06a] ml-1">{daysWithRehearsals}</span>
      </div>
      <div>
        <span class="text-[#82828a]">Doble Turno:</span>
        <span class="font-bold text-[#2ed6e8] ml-1">{daysDoubleTurn}</span>
      </div>
      <div>
        <span class="text-[#82828a]">Funciones:</span>
        <span class="font-bold text-[#ff7070] ml-1">{daysPerformance}</span>
      </div>
    </div>
  </div>

  <!-- Rejilla Calendario: 7 Columnas -->
  <div class="bg-[#1a1a1e] border border-[#34343c] rounded overflow-hidden shadow-sm">
    
    <!-- Encabezados de Días de la Semana -->
    <div class="grid grid-cols-7 bg-[#17171b] border-b border-[#34343c] text-center text-[10px] font-mono font-bold text-[#82828a] py-1.5">
      {#each WEEK_HEADERS as wh}
        <div>{wh}</div>
      {/each}
    </div>

    <!-- Celdas del Calendario -->
    <div class="grid grid-cols-7 gap-px bg-[#34343c]">
      {#each calendarDays as day}
        {@const rTag = getRegimeTag(day.summary?.regime)}
        <div
          role="button"
          tabindex="0"
          onclick={() => handleSelectDay(day)}
          onkeydown={(e) => { if (e.key === 'Enter') handleSelectDay(day); }}
          class="min-h-[90px] md:min-h-[110px] p-2 flex flex-col justify-between transition-colors cursor-pointer text-left {day.isCurrentMonth ? 'bg-[#1a1a1e] hover:bg-[#24242a]' : 'bg-[#141416] text-[#82828a]'} {day.isSelected ? 'ring-2 ring-[#ffb31a] ring-inset' : ''}"
        >
          <!-- Fila Superior: Número de Día y Régimen -->
          <div class="flex items-start justify-between">
            <span class="text-xs font-mono font-bold {day.isToday ? 'px-1.5 py-0.5 rounded bg-[#ffb31a] text-[#121214] font-black' : day.isCurrentMonth ? 'text-[#d8d8dc]' : 'text-[#82828a]'}">
              {day.dayNumber}
            </span>

            {#if day.summary}
              <span class="text-[9px] font-mono {rTag.color}">
                {rTag.label}
              </span>
            {/if}
          </div>

          <!-- Información de Actividad -->
          <div class="my-1 space-y-1">
            {#if day.summary && day.summary.rehearsal_blocks_count > 0}
              <div class="text-[10px] font-mono bg-[#24242a] border border-[#34343c] px-1.5 py-0.5 rounded text-[#ffb31a] truncate">
                {day.summary.rehearsal_blocks_count} {day.summary.rehearsal_blocks_count === 1 ? 'ensayo' : 'ensayos'}
              </div>
            {/if}

            {#if day.summary && day.summary.morning_classes_count > 0}
              <div class="text-[9px] font-mono text-[#3de06a] truncate">
                ✓ Clases 09:00
              </div>
            {/if}
          </div>

          <!-- Acciones Rápidas al Clic -->
          <div class="flex items-center justify-between pt-1 border-t border-[#34343c]/60 text-[9px] font-mono">
            <button
              onclick={(e) => { e.stopPropagation(); onSwitchToDailyView(day.date); }}
              class="text-[#2ed6e8] hover:underline"
            >
              Diaria ➔
            </button>
            <button
              onclick={(e) => { e.stopPropagation(); onSwitchToWeeklyView(day.date); }}
              class="text-[#82828a] hover:text-[#d8d8dc]"
            >
              Semana
            </button>
          </div>
        </div>
      {/each}
    </div>

  </div>
</div>
