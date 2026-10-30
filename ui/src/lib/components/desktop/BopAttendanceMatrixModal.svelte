<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import {
    api,
    type MonthlyBopMatrixDto,
    type BopMatrixRowDto,
    type BopMatrixCellDto,
    type BopMatrixDayInfo,
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import PrintPreviewModal from '../shared/PrintPreviewModal.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let printPreviewOpen = $state(false);
  let printableEl: HTMLElement | null = $state(null);

  let {
    isOpen,
    onClose,
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let selectedYear = $state<number>(2026);
  let selectedMonth = $state<number>(9); // Septiembre 2026 por defecto (como en la planilla física de Carina)
  let matrix = $state<MonthlyBopMatrixDto | null>(null);
  // Meses vecinos: la semana de lunes a domingo puede empezar en el mes anterior o terminar en el siguiente
  let prevMatrix = $state<MonthlyBopMatrixDto | null>(null);
  let nextMatrix = $state<MonthlyBopMatrixDto | null>(null);
  type WeekDay = BopMatrixDayInfo & { src: 'prev' | 'cur' | 'next' };
  let isLoading = $state(false);
  // Vista de la planilla: una semana, el mes completo o el resumen del año
  type ViewMode = 'week' | 'month' | 'year';
  const viewModes: { key: ViewMode; label: string }[] = [
    { key: 'week', label: 'Semana' },
    { key: 'month', label: 'Mes' },
    { key: 'year', label: 'Año' },
  ];
  let viewMode = $state<ViewMode>('month');
  let weekIdx = $state(0);
  type YearMonth = { p: number; ei: number; sp: number; other: number; loaded: boolean };
  type YearRow = { personnelId: string; name: string; months: YearMonth[] };
  let yearRows = $state<YearRow[]>([]);
  const monthsShort = ['Ene', 'Feb', 'Mar', 'Abr', 'May', 'Jun', 'Jul', 'Ago', 'Sep', 'Oct', 'Nov', 'Dic'];
  type ShownRow = {
    row: BopMatrixRowDto;
    cells: BopMatrixCellDto[];
    total_p: number;
    total_ei: number;
    total_sp: number;
    total_other: number;
  };
  const DAY_LETTERS = ['D', 'L', 'M', 'X', 'J', 'V', 'S'];
  function isoDate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }
  function addDays(dateIso: string, n: number): string {
    const [y, m, d] = dateIso.split('-').map(Number);
    return isoDate(new Date(y, m - 1, d + n));
  }
  // Día de la semana con los datos del mes al que pertenece (el cargado o uno vecino)
  function dayInfoFor(d: Date): WeekDay {
    const y = d.getFullYear();
    const m = d.getMonth() + 1;
    let src: WeekDay['src'] = 'cur';
    let owner = matrix;
    if (matrix && (y !== matrix.year || m !== matrix.month)) {
      const before = y * 12 + m < matrix.year * 12 + matrix.month;
      src = before ? 'prev' : 'next';
      owner = before ? prevMatrix : nextMatrix;
    }
    const known = owner && owner.year === y && owner.month === m ? owner.days_header[d.getDate() - 1] : undefined;
    if (known) return { ...known, src };
    const wd = d.getDay();
    return { day_number: d.getDate(), date: isoDate(d), day_name: DAY_LETTERS[wd], is_weekend: wd === 0 || wd === 6, day_annotation: null, src };
  }
  // Semanas que tocan al mes cargado: siempre de lunes a domingo, con 7 días cada una
  let weeks = $derived.by<WeekDay[][]>(() => {
    if (!matrix) return [];
    const first = new Date(matrix.year, matrix.month - 1, 1);
    const last = new Date(matrix.year, matrix.month, 0);
    const out: WeekDay[][] = [];
    for (let s = new Date(matrix.year, matrix.month - 1, 1 - ((first.getDay() + 6) % 7)); s <= last; s.setDate(s.getDate() + 7)) {
      out.push(Array.from({ length: 7 }, (_, i) => dayInfoFor(new Date(s.getFullYear(), s.getMonth(), s.getDate() + i))));
    }
    return out;
  });
  let safeWeekIdx = $derived(Math.min(weekIdx, Math.max(0, weeks.length - 1)));
  // Días y filas que se muestran según la vista; los totales se recalculan sobre lo visible
  let shownDays = $derived.by<WeekDay[]>(() =>
    viewMode === 'week' ? (weeks[safeWeekIdx] ?? []) : (matrix?.days_header ?? []).map(d => ({ ...d, src: 'cur' as const })),
  );
  function cellFor(row: BopMatrixRowDto, d: WeekDay): BopMatrixCellDto {
    const owner = d.src === 'cur' ? matrix : d.src === 'prev' ? prevMatrix : nextMatrix;
    const r = d.src === 'cur' ? row : owner?.rows.find(x => x.personnel_id === row.personnel_id);
    return r?.cells.find(c => c.date === d.date)
      ?? { day_number: d.day_number, date: d.date, is_weekend: d.is_weekend, code: '', minutes_late: 0, remarks: null };
  }
  let shownRows = $derived.by<ShownRow[]>(() => {
    if (!matrix) return [];
    return matrix.rows.map(row => {
      const cells = shownDays.map(d => cellFor(row, d));
      return {
        row,
        cells,
        total_p: cells.filter(c => c.code === 'P').length,
        total_ei: cells.filter(c => c.code === 'EI').length,
        total_sp: cells.filter(c => c.code === 'SP').length,
        total_other: cells.filter(c => c.code && !['P', 'EI', 'SP', 'S/P'].includes(c.code)).length,
      };
    });
  });
  // "28" si el día es del mes cargado; "31 AGO" si es de un mes vecino
  function dayLabel(d: WeekDay): string {
    return d.src === 'cur' ? String(d.day_number) : `${d.day_number} ${monthsShort[Number(d.date.slice(5, 7)) - 1].toUpperCase()}`;
  }
  let periodLabel = $derived.by(() => {
    if (!matrix) return '';
    if (viewMode === 'year') return `AÑO ${selectedYear}`;
    if (viewMode === 'week' && shownDays.length) {
      return `${matrix.month_name.toUpperCase()} ${matrix.year} · SEMANA ${safeWeekIdx + 1} (${dayLabel(shownDays[0])} AL ${dayLabel(shownDays[shownDays.length - 1])})`;
    }
    return `${matrix.month_name.toUpperCase()} ${matrix.year}`;
  });
  // Menú contextual de edición de celda
  let activeCellPopup = $state<{
    personnelId: string;
    dancerName: string;
    date: string;
    dayNumber: number;
    currentCode: string;
  } | null>(null);

  // Modal para fijar anotación de día (ej. ENSAYO SUSPENDIDO)
  let dayAnnotationPrompt = $state<{
    date: string;
    dayNumber: number;
    currentAnnotation: string;
  } | null>(null);
  let annotationInput = $state('');

  const months = [
    { value: 1, name: 'Enero' },
    { value: 2, name: 'Febrero' },
    { value: 3, name: 'Marzo' },
    { value: 4, name: 'Abril' },
    { value: 5, name: 'Mayo' },
    { value: 6, name: 'Junio' },
    { value: 7, name: 'Julio' },
    { value: 8, name: 'Agosto' },
    { value: 9, name: 'Septiembre' },
    { value: 10, name: 'Octubre' },
    { value: 11, name: 'Noviembre' },
    { value: 12, name: 'Diciembre' },
  ];

  const CODES = [
    { code: 'P', label: 'P - Presente', color: 'text-green font-bold' },
    { code: 'EI', label: 'EI - Enfermedad Inculpable (Carpeta Médica)', color: 'text-cyan font-bold' },
    { code: 'SP', label: 'SP - Sin Presencia / Sin Permiso', color: 'text-red font-bold' },
    { code: 'S/P', label: 'S/P - Sin Parte (Sin convocar)', color: 'text-violet font-bold' },
    { code: 'LC', label: 'LC - Licencia Compensatoria / Franco', color: 'text-amber font-bold' },
    { code: 'ART', label: 'ART - Accidente Laboral', color: 'text-amber font-bold' },
    { code: 'LA', label: 'LA - Licencia Anual / Vacaciones', color: 'text-purple-400 font-bold' },
    { code: 'IJ', label: 'IJ - Inasistencia Justificada (máx. 2 por mes y 10 por año)', color: 'text-blue font-bold' },
    { code: 'AT', label: 'AT - Adecuación de Tareas (trabaja en otra área)', color: 'text-blue font-bold' },
    { code: 'ES', label: 'ES - Ensayo Suspendido', color: 'text-dim' },
    { code: '', label: '— Vacío', color: 'text-dim' },
  ];

  async function loadMatrix() {
    isLoading = true;
    try {
      matrix = await api.getMonthlyBopMatrix(selectedYear, selectedMonth);
      if (viewMode === 'week') await loadNeighbors();
    } catch (e: any) {
      console.error('Error cargando matriz BOP:', e);
    } finally {
      isLoading = false;
    }
  }

  async function loadNeighbors() {
    const prev: [number, number] = selectedMonth === 1 ? [selectedYear - 1, 12] : [selectedYear, selectedMonth - 1];
    const next: [number, number] = selectedMonth === 12 ? [selectedYear + 1, 1] : [selectedYear, selectedMonth + 1];
    [prevMatrix, nextMatrix] = await Promise.all([
      api.getMonthlyBopMatrix(prev[0], prev[1]).catch(() => null),
      api.getMonthlyBopMatrix(next[0], next[1]).catch(() => null),
    ]);
  }
  async function loadYear() {
    isLoading = true;
    try {
      const results = await Promise.all(
        Array.from({ length: 12 }, (_, i) =>
          api.getMonthlyBopMatrix(selectedYear, i + 1).catch(() => null),
        ),
      );
      const byId = new Map<string, YearRow>();
      results.forEach((m, mi) => {
        if (!m) return;
        for (const r of m.rows) {
          let yr = byId.get(r.personnel_id);
          if (!yr) {
            yr = {
              personnelId: r.personnel_id,
              name: r.dancer_name,
              months: Array.from({ length: 12 }, () => ({ p: 0, ei: 0, sp: 0, other: 0, loaded: false })),
            };
            byId.set(r.personnel_id, yr);
          }
          yr.months[mi] = { p: r.total_p, ei: r.total_ei, sp: r.total_sp, other: r.total_other, loaded: true };
        }
      });
      yearRows = [...byId.values()];
    } catch (e: any) {
      console.error('Error cargando el resumen anual BOP:', e);
    } finally {
      isLoading = false;
    }
  }
  async function loadView() {
    if (viewMode === 'year') await loadYear();
    else await loadMatrix();
  }
  function shiftMonth(delta: number) {
    let m = selectedMonth + delta;
    let y = selectedYear;
    if (m < 1) { m = 12; y -= 1; }
    if (m > 12) { m = 1; y += 1; }
    selectedMonth = m;
    selectedYear = y;
  }
  // Semana que contiene a hoy; si no está en el mes cargado, la primera
  function currentWeekIdx(): number {
    const today = isoDate(new Date());
    const i = weeks.findIndex(w => w.some(d => d.date === today));
    return i >= 0 ? i : 0;
  }
  async function setViewMode(mode: ViewMode) {
    if (mode === viewMode) return;
    viewMode = mode;
    if (mode === 'year') {
      await loadYear();
      return;
    }
    if (!matrix) await loadMatrix();
    if (mode === 'week') {
      await loadNeighbors();
      weekIdx = currentWeekIdx();
    }
  }
  async function onMonthChange() {
    weekIdx = 0;
    await loadMatrix();
  }
  // Semana anterior o siguiente: se busca por el lunes de destino y, si cae en otro mes, se cambia de mes
  async function goWeek(delta: number) {
    if (!shownDays.length) return;
    const target = addDays(shownDays[0].date, 7 * delta);
    const idx = weeks.findIndex(w => w[0].date === target);
    if (idx >= 0) {
      weekIdx = idx;
      return;
    }
    shiftMonth(delta);
    await loadMatrix();
    weekIdx = Math.max(0, weeks.findIndex(w => w[0].date === target));
  }
  async function prevPeriod() {
    if (viewMode === 'year') {
      selectedYear -= 1;
      await loadYear();
    } else if (viewMode === 'week') {
      await goWeek(-1);
    } else {
      shiftMonth(-1);
      await loadMatrix();
      weekIdx = 0;
    }
  }
  async function nextPeriod() {
    if (viewMode === 'year') {
      selectedYear += 1;
      await loadYear();
    } else if (viewMode === 'week') {
      await goWeek(1);
    } else {
      shiftMonth(1);
      await loadMatrix();
      weekIdx = 0;
    }
  }
  async function openMonthFromYear(month: number) {
    selectedMonth = month;
    viewMode = 'month';
    weekIdx = 0;
    await loadMatrix();
  }
  function getDailyPresentCount(day: WeekDay): number {
    const owner = day.src === 'cur' ? matrix : day.src === 'prev' ? prevMatrix : nextMatrix;
    if (!owner) return 0;
    return owner.rows.filter(r => r.cells.some(c => c.date === day.date && c.code === 'P')).length;
  }
  async function setCellCode(personnelId: string, date: string, code: string) {
    try {
      await api.updateBopMatrixCell({
        date,
        personnel_id: personnelId,
        code,
      });
      // Actualizar localmente la celda (en el mes cargado o en un vecino) para respuesta inmediata
      let found = false;
      for (const m of [matrix, prevMatrix, nextMatrix]) {
        const row = m?.rows.find(r => r.personnel_id === personnelId);
        const cell = row?.cells.find(c => c.date === date);
        if (!row || !cell) continue;
        found = true;
        cell.code = code;
        row.total_p = row.cells.filter(c => c.code === 'P').length;
        row.total_ei = row.cells.filter(c => c.code === 'EI').length;
        row.total_sp = row.cells.filter(c => c.code === 'SP').length;
        row.total_other = row.cells.filter(c => c.code && !['P', 'EI', 'SP', 'S/P'].includes(c.code)).length;
      }
      if (!found) await loadMatrix();
      activeCellPopup = null;
    } catch (e: any) {
      console.error('Error actualizando celda:', e);
      alert(`Error al guardar celda: ${e?.message || e}`);
    }
  }

  function handleCellClick(row: BopMatrixRowDto, cell: BopMatrixCellDto, event: MouseEvent) {
    // En el celular se toca la celda y se elige el código (no hay clic derecho)
    if (window.matchMedia('(max-width: 767px)').matches) {
      handleCellContextMenu(row, cell, event);
      return;
    }
    // Si se hace clic simple, ciclar código P -> EI -> SP -> S/P -> vacío
    const cycle = ['', 'P', 'EI', 'SP', 'S/P'];
    const currentIdx = cycle.indexOf(cell.code);
    const nextCode = currentIdx >= 0 && currentIdx < cycle.length - 1 ? cycle[currentIdx + 1] : '';
    setCellCode(row.personnel_id, cell.date, nextCode);
  }

  function handleCellContextMenu(row: BopMatrixRowDto, cell: BopMatrixCellDto, event: MouseEvent) {
    event.preventDefault();
    activeCellPopup = {
      personnelId: row.personnel_id,
      dancerName: row.dancer_name,
      date: cell.date,
      dayNumber: cell.day_number,
      currentCode: cell.code,
    };
  }

  function openDayAnnotation(dayNumber: number, date: string, currentAnnot?: string | null) {
    dayAnnotationPrompt = {
      dayNumber,
      date,
      currentAnnotation: currentAnnot || '',
    };
    annotationInput = currentAnnot || '';
  }

  async function saveDayAnnotation() {
    if (!dayAnnotationPrompt) return;
    try {
      await api.setDayScheduleAnnotation({
        date: dayAnnotationPrompt.date,
        annotation: annotationInput.trim() || null,
      });
      dayAnnotationPrompt = null;
      await loadMatrix();
    } catch (e: any) {
      alert(`Error guardando anotación de día: ${e?.message || e}`);
    }
  }

  async function setAllPresentForDay(date: string) {
    if (!matrix) return;
    if (!confirm(`¿Marcar "P" (Presente) a todos los bailarines en la fecha ${date}?`)) return;
    try {
      for (const row of matrix.rows) {
        await api.updateBopMatrixCell({
          date,
          personnel_id: row.personnel_id,
          code: 'P',
        });
      }
      await loadMatrix();
    } catch (e: any) {
      alert(`Error marcando presentes: ${e?.message || e}`);
    }
  }

  function downloadCsv(csv: string, fileName: string) {
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', fileName);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
  function exportYearCsv() {
    let csv = '\uFEFF';
    csv += `AUSENTISMO BOP AÑO ${selectedYear}\n`;
    csv += 'Cada mes: días de ausencia (EI + SP + otras licencias)\n\n';
    csv += `Nro;Apellido y Nombre;${monthsShort.join(';')};Total P;Total EI;Total SP;Total Otros\n`;
    yearRows.forEach((yr, i) => {
      const abs = yr.months.map(m => (m.loaded ? m.ei + m.sp + m.other : ''));
      const sum = (k: 'p' | 'ei' | 'sp' | 'other') => yr.months.reduce((a, m) => a + m[k], 0);
      csv += `${i + 1};"${yr.name}";${abs.join(';')};${sum('p')};${sum('ei')};${sum('sp')};${sum('other')}\n`;
    });
    downloadCsv(csv, `Ausentismo_BOP_${selectedYear}.csv`);
  }
  function exportMatrixCsv() {
    if (viewMode === 'year') {
      exportYearCsv();
      return;
    }
    if (!matrix) return;
    let csv = '\uFEFF';
    csv += `AUSENTISMO BOP ${periodLabel}\n\n`;
    // Encabezado de columnas (solo los días de la vista)
    let headerLine = 'Nro;Apellido y Nombre;';
    for (const d of shownDays) {
      headerLine += `${d.day_number};`;
    }
    headerLine += 'Total P;Total EI;Total SP;Total Otros\n';
    csv += headerLine;
    // Fila de anotaciones de día si existen
    let annotLine = ';;';
    let hasAnnot = false;
    for (const d of shownDays) {
      if (d.day_annotation) {
        annotLine += `"${d.day_annotation}";`;
        hasAnnot = true;
      } else {
        annotLine += ';';
      }
    }
    annotLine += '\n';
    if (hasAnnot) csv += annotLine;
    // Filas de personal
    for (const sr of shownRows) {
      let line = `${sr.row.nro};"${sr.row.dancer_name}";`;
      for (const cell of sr.cells) {
        line += `${cell.code};`;
      }
      line += `${sr.total_p};${sr.total_ei};${sr.total_sp};${sr.total_other}\n`;
      csv += line;
    }
    const suffix = viewMode === 'week' ? `_Semana${safeWeekIdx + 1}` : '';
    downloadCsv(csv, `Ausentismo_BOP_${matrix.month_name}_${matrix.year}${suffix}.csv`);
  }
  $effect(() => {
    if (isOpen) {
      untrack(() => loadView());
    }
  });
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-0 md:p-4 print:p-0 print:bg-white print:fixed print:inset-0" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-[98vw] max-md:max-w-none max-md:h-[100dvh] max-md:max-h-none max-md:rounded-none relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">

      <!-- ================================================================= -->
      <!-- BARRA SUPERIOR (NO IMPRIMIBLE)                                    -->
      <!-- ================================================================= -->
      <ModalHeader
        title="[MATRIZ DE AUSENTISMO BOP]"
        subtitle="Planilla mensual de asistencia histórica (Inspección Carina Lion)"
        onclose={onClose}
      >
        <div class="flex flex-wrap items-center gap-2 print:hidden">
          <!-- Vista: semana, mes o año -->
          <div class="inline-flex bg-void p-0.5 rounded border border-grid">
            {#each viewModes as vm}
              <button
                onclick={() => setViewMode(vm.key)}
                class="h-6 max-md:h-10 px-2.5 max-md:px-3.5 rounded text-xs max-md:text-sm font-semibold transition-colors {viewMode === vm.key ? 'bg-amber text-void' : 'text-dim hover:text-ink'}"
              >
                {vm.label}
              </button>
            {/each}
          </div>
          <!-- Selector de período -->
          <div class="flex items-center gap-2">
            <button
              onclick={prevPeriod}
              title={viewMode === 'year' ? 'Año anterior' : viewMode === 'week' ? 'Semana anterior' : 'Mes anterior'}
              class="w-7 h-7 max-md:w-11 max-md:h-11 max-md:text-base flex items-center justify-center bg-raised hover:bg-raised-hover text-ink border border-grid rounded text-xs font-mono transition-colors"
            >
              ◀
            </button>
            {#if viewMode !== 'year'}
              <select
                bind:value={selectedMonth}
                onchange={onMonthChange}
                class="h-7 max-md:h-11 max-md:text-sm px-2.5 bg-void border border-grid rounded text-xs font-semibold text-ink outline-none"
              >
                {#each months as m}
                  <option value={m.value}>{m.name.toUpperCase()}</option>
                {/each}
              </select>
            {/if}
            <input
              type="number"
              bind:value={selectedYear}
              onchange={loadView}
              class="h-7 max-md:h-11 max-md:text-sm w-20 px-2 bg-void border border-grid rounded text-xs font-mono font-bold text-amber text-center outline-none"
            />
            <button
              onclick={nextPeriod}
              title={viewMode === 'year' ? 'Año siguiente' : viewMode === 'week' ? 'Semana siguiente' : 'Mes siguiente'}
              class="w-7 h-7 max-md:w-11 max-md:h-11 max-md:text-base flex items-center justify-center bg-raised hover:bg-raised-hover text-ink border border-grid rounded text-xs font-mono transition-colors"
            >
              ▶
            </button>
            {#if viewMode === 'week' && shownDays.length}
              <span class="text-xs font-mono text-dim whitespace-nowrap">
                Semana {safeWeekIdx + 1} de {weeks.length} · días {dayLabel(shownDays[0])}–{dayLabel(shownDays[shownDays.length - 1])}
              </span>
            {/if}
          </div>
          <!-- Leyenda de Códigos y Acciones -->
          <div class="flex items-center gap-2">
            <div class="hidden lg:flex items-center gap-2 text-[10px] font-mono text-dim bg-void px-2 py-1 rounded border border-grid">
              <span><strong class="text-green">P</strong>: Presente</span>
              <span><strong class="text-cyan">EI</strong>: Médica</span>
              <span><strong class="text-red">SP</strong>: Sin Presencia</span>
              <span><strong class="text-violet">S/P</strong>: Sin Parte</span>
              <span><strong class="text-amber">LC</strong>: Compensatorio</span>
              <span><strong class="text-blue">AT</strong>: Adec. de tareas</span>
              <span><strong class="text-ink">Otras</strong>: Lic. Justificadas (LC, ART, LA, IJ, AT, ES)</span>
              <span><span class="text-dim italic">(Clic rota, Clic der. elige)</span></span>
            </div>

            <button
              onclick={exportMatrixCsv}
              class="px-2.5 py-1 max-md:py-2.5 max-md:text-sm bg-raised hover:bg-raised-hover text-green border border-grid rounded text-xs font-mono font-semibold transition-colors"
            >
              Exportar Excel
            </button>

            <button
              onclick={() => (printPreviewOpen = true)}
              class="px-2.5 py-1 max-md:py-2.5 max-md:text-sm max-md:hidden bg-raised hover:bg-raised-hover text-ink border border-grid rounded text-xs font-mono font-semibold transition-colors"
            >
              Imprimir A4
            </button>
          </div>
        </div>
      </ModalHeader>

      <!-- ================================================================= -->
      <!-- CUERPO PRINCIPAL: GRILLA MATRICIAL ESTILO EXCEL HISTÓRICO         -->
      <!-- ================================================================= -->
      <div bind:this={printableEl} class="flex-1 overflow-auto border border-grid rounded bg-void p-3 md:p-4 print:border-none print:overflow-visible print:bg-white print:text-black">
        
        {#if isLoading}
          <div class="py-20 text-center text-xs font-mono text-dim">
            Cargando matriz mensual de ausentismo...
          </div>
        {:else if viewMode === 'year'}
          <div class="p-2 text-center border-b border-grid bg-panel print:bg-white print:border-black print:pb-3">
            <h1 class="text-xs md:text-sm font-black font-sans uppercase tracking-widest text-ink print:text-black">
              AUSENTISMO BOP {periodLabel}
            </h1>
          </div>
          <p class="px-3 py-2 border-b border-grid text-xs font-mono text-dim print:hidden">
            Cada mes muestra los días de ausencia (EI + SP + otras licencias); tocá el nombre del mes para abrirlo. Los totales de la derecha son de todo el año.
          </p>
          <table class="w-full max-md:w-max border-collapse text-[10px] max-md:text-xs font-mono text-center select-none">
            <thead>
              <tr class="bg-panel text-ink border-b border-grid print:bg-neutral-200 print:text-black print:border-black">
                <th class="border-r border-grid px-1 py-1.5 w-8 font-bold print:border-black max-md:hidden">Nro</th>
                <th class="border-r border-grid px-2.5 py-1.5 text-left font-bold w-48 sm:w-56 max-md:w-[120px] max-md:min-w-[120px] max-md:max-w-[120px] max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-panel print:border-black">
                  Apellido y Nombre
                </th>
                {#each monthsShort as ms, i}
                  <th
                    class="border-r border-grid px-1 py-1.5 max-md:min-w-[44px] font-bold cursor-pointer hover:bg-raised transition-colors print:border-black"
                    title="Abrir {months[i].name} {selectedYear}"
                    onclick={() => openMonthFromYear(i + 1)}
                  >
                    {ms}
                  </th>
                {/each}
                <th class="border-r border-grid px-1 py-1 text-green font-bold w-9 print:border-black" title="Total Presentes (P) del año">P</th>
                <th class="border-r border-grid px-1 py-1 text-cyan font-bold w-9 print:border-black" title="Total Enfermedad Inculpable (EI) del año">EI</th>
                <th class="border-r border-grid px-1 py-1 text-red font-bold w-9 print:border-black" title="Total Sin Presencia (SP) del año">SP</th>
                <th class="px-1 py-1 text-ink font-bold w-11 print:border-black" title="Otras licencias del año">Otras</th>
              </tr>
            </thead>
            <tbody>
              {#each yearRows as yr, i}
                {@const tp = yr.months.reduce((a, m) => a + m.p, 0)}
                {@const tei = yr.months.reduce((a, m) => a + m.ei, 0)}
                {@const tsp = yr.months.reduce((a, m) => a + m.sp, 0)}
                {@const tot = yr.months.reduce((a, m) => a + m.other, 0)}
                <tr class="border-b border-raised hover:bg-panel transition-colors print:border-black print:hover:bg-transparent">
                  <td class="border-r border-grid py-1 text-dim font-mono text-[9px] print:border-black max-md:hidden">{i + 1}</td>
                  <td class="border-r border-grid py-1 px-2.5 max-md:py-2.5 max-md:w-[120px] max-md:min-w-[120px] max-md:max-w-[120px] max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-panel text-left font-sans text-xs font-semibold text-ink truncate print:text-black print:border-black">
                    {yr.name}
                  </td>
                  {#each yr.months as m, mi}
                    {@const aus = m.ei + m.sp + m.other}
                    <td
                      class="border-r border-grid py-1 max-md:py-2.5 print:border-black {aus > 0 ? 'text-red font-bold' : 'text-dim'}"
                      title="{months[mi].name}: P {m.p} · EI {m.ei} · SP {m.sp} · Otras {m.other}"
                    >
                      {m.loaded ? (aus > 0 ? aus : '·') : ''}
                    </td>
                  {/each}
                  <td class="border-r border-grid py-1 font-bold text-green print:text-black print:border-black">{tp}</td>
                  <td class="border-r border-grid py-1 font-bold text-cyan print:text-black print:border-black">{tei}</td>
                  <td class="border-r border-grid py-1 font-bold text-red print:text-black print:border-black">{tsp}</td>
                  <td class="py-1 font-bold text-ink print:text-neutral-600 print:border-black">{tot}</td>
                </tr>
              {/each}
            </tbody>
            <tfoot>
              <tr class="bg-alt border-t-2 border-grid font-mono text-[9px] font-bold print:border-black print:bg-neutral-100">
                <td class="border-r border-grid py-1.5 text-center text-dim print:border-black max-md:hidden">Σ</td>
                <td class="border-r border-grid py-1.5 px-3 text-left uppercase text-ink print:text-black print:border-black max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-alt max-md:text-[11px]">Ausencias del mes</td>
                {#each monthsShort as _, mi}
                  {@const tm = yearRows.reduce((a, yr) => a + yr.months[mi].ei + yr.months[mi].sp + yr.months[mi].other, 0)}
                  <td class="border-r border-grid py-1.5 print:border-black {tm > 0 ? 'text-red' : 'text-dim'}">{tm > 0 ? tm : '·'}</td>
                {/each}
                <td class="border-r border-grid py-1.5 text-green print:text-black print:border-black">{yearRows.reduce((a, yr) => a + yr.months.reduce((b, m) => b + m.p, 0), 0)}</td>
                <td class="border-r border-grid py-1.5 text-cyan print:text-black print:border-black">{yearRows.reduce((a, yr) => a + yr.months.reduce((b, m) => b + m.ei, 0), 0)}</td>
                <td class="border-r border-grid py-1.5 text-red print:text-black print:border-black">{yearRows.reduce((a, yr) => a + yr.months.reduce((b, m) => b + m.sp, 0), 0)}</td>
                <td class="py-1.5 text-ink print:text-neutral-600 print:border-black">{yearRows.reduce((a, yr) => a + yr.months.reduce((b, m) => b + m.other, 0), 0)}</td>
              </tr>
            </tfoot>
          </table>
        {:else if matrix}
          
          <!-- Título Impreso Oficial -->
          <div class="p-2 text-center border-b border-grid bg-panel print:bg-white print:border-black print:pb-3">
            <h1 class="text-xs md:text-sm font-black font-sans uppercase tracking-widest text-ink print:text-black">
              AUSENTISMO BOP {periodLabel}
            </h1>
          </div>

          <!-- Anotaciones del mes y leyenda: legibles sin tener que adivinar la columna -->
          {#if shownDays.some(d => d.day_annotation)}
            <div class="px-3 py-2 border-b border-grid bg-raised print:hidden flex flex-wrap gap-x-4 gap-y-1">
              {#each shownDays.filter(d => d.day_annotation) as d}
                <span class="text-sm text-amber"><strong class="font-mono">Día {d.day_number}</strong> · {d.day_annotation}</span>
              {/each}
            </div>
          {/if}
          <div class="lg:hidden px-3 py-2 border-b border-grid text-xs font-mono text-dim flex flex-wrap gap-x-3 gap-y-1 print:hidden">
            <span><strong class="text-green">P</strong> Presente</span>
            <span><strong class="text-cyan">EI</strong> Médica</span>
            <span><strong class="text-red">SP</strong> Sin presencia</span>
            <span><strong class="text-violet">S/P</strong> Sin parte</span>
            <span><strong class="text-amber">LC</strong> Compens.</span>
            <span><strong class="text-blue">IJ</strong> Justificada (2/mes, 10/año)</span>
            <span><strong class="text-blue">AT</strong> Adec. de tareas</span>
            <span>Tocá una celda para elegir</span>
          </div>

          <table class="w-full max-md:w-max md:table-fixed border-collapse text-[10px] max-md:text-xs font-mono text-center select-none">
            <thead>
              <!-- Fila 1: Números de Día -->
              <tr class="bg-panel text-ink border-b border-grid print:bg-neutral-200 print:text-black print:border-black">
                <th class="border-r border-grid px-1 py-1.5 w-8 font-bold print:border-black max-md:hidden">Nro</th>
                <th class="border-r border-grid px-2.5 py-1.5 text-left font-bold w-48 sm:w-56 max-md:w-[120px] max-md:min-w-[120px] max-md:max-w-[120px] max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-panel shrink-0 truncate print:border-black">
                  Apellido y Nombre
                </th>

                {#each shownDays as day}
                  <th
                    class="border-r border-grid px-0 py-1 max-md:py-2 max-md:min-w-[40px] font-bold whitespace-nowrap transition-colors cursor-pointer print:border-black {day.is_weekend ? 'bg-alt text-dim print:bg-neutral-300' : 'hover:bg-raised'}"
                    title="Día {day.day_number} ({day.date}). Clic para fijar evento, feriado o marcar presentes."
                    onclick={() => openDayAnnotation(day.day_number, day.date, day.day_annotation)}
                  >
                    {day.day_number}
                    {#if day.src !== 'cur'}<span class="block text-[8px] font-normal opacity-70">{monthsShort[Number(day.date.slice(5, 7)) - 1]}</span>{/if}
                  </th>
                {/each}

                <!-- Totales -->
                <th class="border-r border-grid px-1 py-1 text-green font-bold w-8 print:border-black" title="Total Presentes (P)">P</th>
                <th class="border-r border-grid px-1 py-1 text-cyan font-bold w-8 print:border-black" title="Total Enfermedad Inculpable / Carpeta Médica (EI)">EI</th>
                <th class="border-r border-grid px-1 py-1 text-red font-bold w-8 print:border-black" title="Total Sin Presencia / Inasistencia Injustificada (SP)">SP</th>
                <th class="px-1 py-1 text-ink font-bold w-11 print:border-black" title="Otras Licencias Justificadas (LC: Franco Compensatorio, ART: Accidente Laboral, LA: Vacaciones, ES: Estudio)">Otras</th>
              </tr>

              <!-- Fila 2: Letra del día de la semana (L, M, M, J, V, S, D) -->
              <tr class="bg-void text-dim border-b border-grid text-[9px] print:bg-neutral-100 print:text-neutral-700 print:border-black">
                <th class="border-r border-grid print:border-black max-md:hidden"></th>
                <th class="border-r border-grid text-left px-2.5 text-[9px] print:border-black truncate max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-void">
                  {matrix.rows.length} integrantes
                </th>

                {#each shownDays as day}
                  <th class="border-r border-grid py-0.5 print:border-black {day.is_weekend ? 'bg-void text-red/70 print:bg-neutral-300' : ''}">
                    {day.day_name}
                  </th>
                {/each}

                <th colspan="4" class="text-[8px] uppercase tracking-wider font-bold text-ink" title="Resumen acumulado del mes para cada bailarín">Totales</th>
              </tr>

              <!-- Fila 3 Permanente: Anotaciones de Inspección (Feriados, Suspensiones, Giras) -->
              <tr class="bg-raised/60 text-[8px] border-b border-grid font-sans print:border-black">
                <td class="border-r border-grid print:border-black max-md:hidden"></td>
                <td class="border-r border-grid text-left px-2.5 text-[9px] max-md:text-[11px] font-bold uppercase text-amber print:border-black truncate max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-raised" title="Clic en cualquier día para fijar feriados, suspensiones o giras">
                  Anotaciones Inspección
                </td>
                {#each shownDays as day}
                  <td
                    class="border-r border-grid p-0 overflow-hidden cursor-pointer transition-colors print:border-black hover:bg-amber/30 {day.day_annotation ? 'bg-amber/20 text-amber font-bold print:bg-neutral-200 print:text-black' : 'text-dim/40 hover:text-amber'}"
                    onclick={() => openDayAnnotation(day.day_number, day.date, day.day_annotation)}
                    title="Día {day.day_number}: {day.day_annotation || 'Sin anotación'}. Clic para agregar, editar o borrar anotación."
                  >
                    {#if day.day_annotation}
                      <span class="block mx-auto py-1 text-[11px] leading-tight font-bold uppercase tracking-tight [writing-mode:vertical-rl] rotate-180 max-h-44 overflow-hidden" title="{day.day_annotation}">
                        {day.day_annotation}
                      </span>
                    {:else}
                      <span class="block text-[11px] opacity-0 hover:opacity-100 font-mono text-center print:hidden">+</span>
                    {/if}
                  </td>
                {/each}
                <td colspan="4" class="print:border-black"></td>
              </tr>
            </thead>

            <tbody>
              {#each shownRows as sr}
                {@const row = sr.row}
                <tr class="border-b border-raised hover:bg-panel transition-colors print:border-black print:hover:bg-transparent">
                  <!-- Nro -->
                  <td class="border-r border-grid py-1 text-dim font-mono text-[9px] print:border-black max-md:hidden">
                    {row.nro}
                  </td>

                  <!-- Apellido y Nombre -->
                  <td class="border-r border-grid py-1 px-2.5 max-md:py-2.5 max-md:w-[120px] max-md:min-w-[120px] max-md:max-w-[120px] max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-panel text-left font-sans text-xs max-md:text-[13px] font-semibold text-ink truncate print:text-black print:border-black" title="{row.dancer_name}">
                    {row.dancer_name}
                  </td>

                  <!-- Celdas de Días -->
                  {#each sr.cells as cell}
                    {@const isSuspended = shownDays.find(d => d.date === cell.date)?.day_annotation === 'ENSAYO SUSPENDIDO'}
                    <td
                      class="border-r border-grid py-1 max-md:py-2.5 max-md:min-w-[40px] text-xs max-md:text-sm cursor-pointer font-bold select-none print:border-black transition-colors {cell.is_weekend ? 'bg-alt print:bg-neutral-200' : 'hover:bg-raised'} {cell.code === 'P' ? 'text-green print:text-black' : cell.code === 'EI' ? 'text-cyan print:text-black print:font-black' : cell.code === 'SP' ? 'text-red print:text-black print:font-black' : cell.code === 'S/P' ? 'text-violet print:text-black font-black' : cell.code === 'LC' ? 'text-amber' : cell.code === 'IJ' || cell.code === 'AT' ? 'text-blue' : 'text-neutral-500'}"
                      onclick={(e) => handleCellClick(row, cell, e)}
                      oncontextmenu={(e) => handleCellContextMenu(row, cell, e)}
                      title="{row.dancer_name} - Día {cell.day_number}: {cell.code || 'Vacío'}. Clic para ciclar, Clic der. para elegir."
                    >
                      {cell.code}
                    </td>
                  {/each}

                  <!-- Totales -->
                  <td class="border-r border-grid py-1 font-bold text-green print:text-black print:border-black" title="Presentes: {sr.total_p}">{sr.total_p}</td>
                  <td class="border-r border-grid py-1 font-bold text-cyan print:text-black print:border-black" title="Carpetas Médicas (EI): {sr.total_ei}">{sr.total_ei}</td>
                  <td class="border-r border-grid py-1 font-bold text-red print:text-black print:border-black" title="Sin Presencia / Injustificadas (SP): {sr.total_sp}">{sr.total_sp}</td>
                  <td class="py-1 text-ink font-bold print:text-neutral-600 print:border-black" title="Otras Licencias (LC, ART, LA, IJ, AT, ES): {sr.total_other}">{sr.total_other}</td>
                </tr>
              {/each}
            </tbody>
            <tfoot>
              <!-- Fila: Presentes diarios -->
              <tr class="bg-alt border-t-2 border-grid font-mono text-[9px] font-bold print:border-black print:bg-neutral-100">
                <td class="border-r border-grid py-1.5 text-center text-dim print:border-black max-md:hidden">Σ</td>
                <td class="border-r border-grid py-1.5 px-3 text-left uppercase text-green print:text-black print:border-black max-md:sticky max-md:left-0 max-md:z-10 max-md:bg-alt max-md:text-[11px]">Total P</td>
                {#each shownDays as day}
                  {@const pCount = getDailyPresentCount(day)}
                  <td class="border-r border-grid py-1.5 text-center print:border-black {pCount > 0 ? 'text-green font-bold' : 'text-dim'} {day.is_weekend ? 'bg-void print:bg-neutral-200' : ''}">
                    {pCount > 0 ? pCount : '—'}
                  </td>
                {/each}
                <td class="border-r border-grid py-1.5 font-bold text-green print:text-black print:border-black" title="Total Presentes del Elenco">
                  {shownRows.reduce((acc, r) => acc + r.total_p, 0)}
                </td>
                <td class="border-r border-grid py-1.5 font-bold text-cyan print:text-black print:border-black" title="Total Carpetas Médicas (EI)">
                  {shownRows.reduce((acc, r) => acc + r.total_ei, 0)}
                </td>
                <td class="border-r border-grid py-1.5 font-bold text-red print:text-black print:border-black" title="Total Inasistencias Sin Presencia (SP)">
                  {shownRows.reduce((acc, r) => acc + r.total_sp, 0)}
                </td>
                <td class="py-1.5 font-bold text-ink print:text-neutral-600 print:border-black" title="Total Otras Licencias Justificadas (LC, ART, LA, IJ, AT, ES)">
                  {shownRows.reduce((acc, r) => acc + r.total_other, 0)}
                </td>
              </tr>
            </tfoot>
          </table>

        {/if}

      </div>

    </div>
  </div>
{/if}

<PrintPreviewModal
  open={printPreviewOpen}
  title="Vista previa de impresión"
  subtitle="Matriz BOP — ediciones solo para la copia impresa"
  source={printableEl}
  onclose={() => (printPreviewOpen = false)}
/>
<!-- MODAL FLOTANTE DE SELECCIÓN RÁPIDA DE CÓDIGO (CLIC DERECHO EN CELDA) -->
{#if activeCellPopup}
  <div class="fixed inset-0 z-50 bg-black/60 flex items-center justify-center max-md:items-end p-3 max-md:p-0" use:modalBehavior={{ onclose: () => activeCellPopup = null }}>
    <div class="tg-modal w-80 max-md:w-full max-md:max-h-[85dvh] max-md:rounded-b-none max-md:overflow-y-auto">
      <ModalHeader
        title="Asignar Asistencia"
        subtitle={activeCellPopup.dancerName + ' — ' + activeCellPopup.date + ' (Día ' + activeCellPopup.dayNumber + ')'}
        onclose={() => activeCellPopup = null}
      />

      <div class="tg-modal-body space-y-1">
        {#each CODES as item}
          <button
            onclick={() => setCellCode(activeCellPopup!.personnelId, activeCellPopup!.date, item.code)}
            class="w-full text-left px-2.5 py-1.5 max-md:py-3 max-md:text-sm rounded text-xs hover:bg-raised flex items-center justify-between transition-colors {item.color}"
          >
            <span>{item.label}</span>
            {#if activeCellPopup.currentCode === item.code}
              <span class="text-amber text-xs font-mono">✓</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="tg-modal-foot">
        <button
          onclick={() => activeCellPopup = null}
          class="tg-btn"
        >
          Cerrar
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL PARA FIJAR ANOTACIÓN DE DÍA (EJ. ENSAYO SUSPENDIDO O FERIADO) -->
{#if dayAnnotationPrompt}
  <div class="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-3" use:modalBehavior={{ onclose: () => dayAnnotationPrompt = null, backdrop: false }}>
    <div class="tg-modal w-96 max-w-full">
      <ModalHeader
        title="Anotación de Día {dayAnnotationPrompt.dayNumber} ({dayAnnotationPrompt.date})"
        subtitle="Leyenda colectiva transversal para la columna completa (ej. Feriado, Ensayo Suspendido)"
        onclose={() => dayAnnotationPrompt = null}
      >
        {#if dayAnnotationPrompt.currentAnnotation}
          <span class="bg-amber/15 text-amber border border-amber/30 text-[9px] px-1.5 py-0.5 rounded font-mono uppercase font-bold shrink-0">
            Editando
          </span>
        {/if}
      </ModalHeader>

      <div class="tg-modal-body space-y-2">
        <label for="bop-day-annot" class="block text-[10px] font-mono uppercase text-dim">Texto de la anotación:</label>
        <input
          id="bop-day-annot"
          type="text"
          bind:value={annotationInput}
          placeholder="Ej: ENSAYO SUSPENDIDO, FERIADO, GIRA"
          class="w-full h-8 px-2.5 bg-void border border-grid rounded text-xs text-ink uppercase font-bold outline-none focus:border-amber"
        />

        <div class="text-[10px] text-dim font-mono">Accesos rápidos:</div>
        <div class="flex flex-wrap gap-1.5">
          <button
            type="button"
            onclick={() => annotationInput = 'ENSAYO SUSPENDIDO'}
            class="px-2 py-0.5 bg-raised text-[10px] font-mono text-amber rounded hover:bg-grid border border-grid"
          >
            ENSAYO SUSPENDIDO
          </button>
          <button
            type="button"
            onclick={() => annotationInput = 'FERIADO NACIONAL'}
            class="px-2 py-0.5 bg-raised text-[10px] font-mono text-ink rounded hover:bg-grid border border-grid"
          >
            FERIADO
          </button>
          <button
            type="button"
            onclick={() => annotationInput = 'DESCANSO COMPENSATORIO'}
            class="px-2 py-0.5 bg-raised text-[10px] font-mono text-ink rounded hover:bg-grid border border-grid"
          >
            COMPENSATORIO
          </button>
          <button
            type="button"
            onclick={() => annotationInput = 'GIRA / TRASLADO'}
            class="px-2 py-0.5 bg-raised text-[10px] font-mono text-ink rounded hover:bg-grid border border-grid"
          >
            GIRA
          </button>
        </div>
      </div>

      <div class="tg-modal-foot justify-between">
        {#if dayAnnotationPrompt.currentAnnotation}
          <button
            type="button"
            onclick={() => { annotationInput = ''; saveDayAnnotation(); }}
            class="tg-btn tg-btn-danger font-mono"
            title="Borrar completamente la anotación de este día"
          >
            Borrar Anotación
          </button>
        {:else}
          <button
            type="button"
            onclick={() => setAllPresentForDay(dayAnnotationPrompt!.date)}
            class="tg-btn text-green"
            title="Marcar 'P' en toda la columna a la vez"
          >
            Marcar Todos "P"
          </button>
        {/if}

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => dayAnnotationPrompt = null}
            class="tg-btn"
          >
            Cancelar
          </button>
          <button
            type="button"
            onclick={saveDayAnnotation}
            class="tg-btn tg-btn-primary font-mono"
          >
            Guardar
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
