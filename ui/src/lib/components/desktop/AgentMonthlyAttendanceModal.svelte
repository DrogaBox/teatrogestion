<script lang="ts">
  import { roleLabel } from '../../labels';
  import { onMount } from 'svelte';
  import {
    api,
    type MonthlyAgentSummaryDto,
    type MonthlyAttendanceSheetDto,
    type PersonnelWithQuotaDto
  } from '../../ipc/client';
  import { modalBehavior } from '../../actions/modalBehavior';

  let {
    isOpen,
    initialPersonnelId = '',
    onClose
  } = $props<{
    isOpen: boolean;
    initialPersonnelId?: string;
    onClose: () => void;
  }>();

  let activeTab = $state<'individual' | 'summary'>('individual');
  let selectedPersonnelId = $state<string>('');
  let selectedYear = $state<number>(new Date().getFullYear());
  let selectedMonth = $state<number>(new Date().getMonth() + 1);

  $effect(() => {
    if (initialPersonnelId) {
      selectedPersonnelId = initialPersonnelId;
    }
  });

  let personnelList = $state<PersonnelWithQuotaDto[]>([]);
  let currentSheet = $state<MonthlyAttendanceSheetDto | null>(null);
  let companySummary = $state<MonthlyAgentSummaryDto[]>([]);
  let isLoadingSheet = $state(false);
  let isLoadingSummary = $state(false);

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

  async function loadPersonnel() {
    try {
      personnelList = await api.listPersonnel(null, true);
      if (!selectedPersonnelId && personnelList.length > 0) {
        selectedPersonnelId = personnelList[0].id;
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function loadSheet() {
    if (!selectedPersonnelId) return;
    isLoadingSheet = true;
    try {
      currentSheet = await api.getAgentMonthlyAttendanceSheet(selectedPersonnelId, selectedYear, selectedMonth);
    } catch (e: any) {
      console.error('Error cargando planilla mensual:', e);
      alert(`Error al generar planilla mensual: ${e?.message || e}`);
    } finally {
      isLoadingSheet = false;
    }
  }

  async function loadCompanySummary() {
    isLoadingSummary = true;
    try {
      companySummary = await api.listMonthlyAgentsSummary(selectedYear, selectedMonth);
    } catch (e: any) {
      console.error('Error cargando resumen mensual RRHH:', e);
    } finally {
      isLoadingSummary = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      if (personnelList.length === 0) loadPersonnel();
      if (selectedPersonnelId) loadSheet();
      if (activeTab === 'summary') loadCompanySummary();
    }
  });

  function handleTabChange(tab: 'individual' | 'summary') {
    activeTab = tab;
    if (tab === 'summary' && companySummary.length === 0) {
      loadCompanySummary();
    }
  }

  function exportIndividualCsv() {
    if (!currentSheet) return;
    let csv = '\uFEFF'; // BOM UTF-8
    csv += `TEATRO DEL LIBERTADOR SAN MARTIN - BALLET OFICIAL DE CORDOBA\n`;
    csv += `PLANILLA MENSUAL INDIVIDUAL DE ASISTENCIA Y NOVEDADES - ${currentSheet.month_name} ${currentSheet.year}\n\n`;
    csv += `AGENTE;"${currentSheet.last_name}, ${currentSheet.first_name}";DNI;"${currentSheet.legajo || 'S/D'}";CARGO;"${currentSheet.category_name}"\n\n`;
    csv += `DIA;FECHA;CONTROL_EN_CLASE;ESTADO_ASISTENCIA;DEMORA_MINUTOS;DIAGNOSTICO_MEDICO;OBSERVACIONES\n`;

    currentSheet.entries.forEach(e => {
      let st = e.status as string;
      if (e.status === 'PRESENT') st = 'PRESENTE';
      else if (e.status === 'LATE') st = `TARDE (+${e.minutes_late}m)`;
      else if (e.status === 'MEDICAL_LEAVE') st = 'LICENCIA MEDICA';
      else if (e.status === 'IN_PHYSIOTHERAPY') st = 'FISIOTERAPIA';
      else if (e.status === 'NOT_IN_CLASS') st = 'NO EN CLASE';
      else if (e.status === 'ABSENT') st = 'AUSENTE';
      else if (e.status === 'JUSTIFIED_ABSENCE') st = 'INASISTENCIA JUSTIFICADA';
      else if (e.status === 'TASK_ADJUSTMENT') st = 'ADECUACION DE TAREAS';
      else if (e.status === 'NO_PART') st = 'SIN PARTE';
      else if (e.is_weekend) st = 'FRANCO/DESCANSO';
      else st = 'PENDIENTE';

      const clase = e.is_weekend ? 'FRANCO' : e.is_in_class ? 'EN CLASE' : 'NO EN CLASE';
      csv += `"${e.day_of_week}";"${e.date}";"${clase}";"${st}";${e.minutes_late > 0 ? e.minutes_late : 0};"${(e.medical_diagnosis || '').replace(/;/g, ',')}";"${(e.remarks || '').replace(/;/g, ',')}"\n`;
    });

    csv += `\nRESUMEN MENSUAL RRHH:\n`;
    csv += `DIAS_HABILES;${currentSheet.total_working_days}\n`;
    csv += `DIAS_PRESENTE;${currentSheet.days_present}\n`;
    csv += `DIAS_TARDE;${currentSheet.days_late}\n`;
    csv += `TOTAL_MINUTOS_TC;${currentSheet.total_late_minutes}\n`;
    csv += `DIAS_LICENCIA_MEDICA;${currentSheet.days_medical_leave}\n`;
    csv += `DIAS_FISIOTERAPIA;${currentSheet.days_physiotherapy}\n`;
    csv += `INASISTENCIAS;${currentSheet.days_absent}\n`;
    csv += `PORCENTAJE_PRESENTISMO;${currentSheet.presence_percentage.toFixed(1)}%\n`;

    downloadCsv(csv, `Planilla_Mensual_${currentSheet.last_name}_${currentSheet.month_name}_${currentSheet.year}.csv`);
  }

  function exportSummaryCsv() {
    let csv = '\uFEFF';
    csv += `RESUMEN GENERAL MENSUAL DE ASISTENCIA Y NOVEDADES PARA RRHH - ${months.find(m => m.value === selectedMonth)?.name.toUpperCase()} ${selectedYear}\n\n`;
    csv += `DNI;APELLIDO Y NOMBRE;ROL;DIAS_PRESENTE;DIAS_TARDE;TC_TOTAL_MINUTOS;LICENCIAS_MEDICAS;FISIOTERAPIA;INASISTENCIAS;PORCENTAJE_PRESENTISMO\n`;

    companySummary.forEach(s => {
      csv += `"${s.legajo || ''}";"${s.dancer_name}";"${s.role}";${s.days_present};${s.days_late};${s.total_late_minutes};${s.days_medical_leave};${s.days_physio};${s.days_absent};${s.presence_percentage.toFixed(1)}%\n`;
    });

    downloadCsv(csv, `Resumen_RRHH_Compania_${selectedMonth}_${selectedYear}.csv`);
  }

  function downloadCsv(content: string, filename: string) {
    const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', filename);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-2 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-6xl relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible p-4 md:p-6">
      
      <!-- Barra Superior (No imprimible) -->
      <div class="flex flex-wrap items-center justify-between pb-3 mb-3 border-b border-grid] print:hidden gap-3">
        <div>
          <h2 class="text-sm font-semibold text-ink] uppercase tracking-wide">
            Planilla Mensual de Asistencia & Novedades para RRHH
          </h2>
          <p class="text-xs text-dim] mt-0.5">
            Cómputo individual de asistencia, tardanzas (TC), carpetas médicas y presentismo mensual
          </p>
        </div>

        <!-- Conmutador Individual vs Resumen General -->
        <div class="inline-flex bg-void] p-1 rounded border border-grid]">
          <button
            onclick={() => handleTabChange('individual')}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeTab === 'individual' ? 'bg-amber] text-black font-semibold' : 'text-dim] hover:text-ink]'}"
          >
            Planilla por Agente
          </button>
          <button
            onclick={() => handleTabChange('summary')}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeTab === 'summary' ? 'bg-amber] text-black font-semibold' : 'text-dim] hover:text-ink]'}"
          >
            Resumen General Compañía (RRHH)
          </button>
        </div>

        <!-- Botones de Acción -->
        <div class="flex items-center gap-2">
          {#if activeTab === 'individual'}
            <button
              onclick={exportIndividualCsv}
              class="px-3 py-1.5 bg-raised] hover:bg-grid] text-green] border border-grid] rounded text-xs font-semibold transition-colors"
            >
              Exportar Excel (.csv)
            </button>
          {:else}
            <button
              onclick={exportSummaryCsv}
              class="px-3 py-1.5 bg-raised] hover:bg-grid] text-green] border border-grid] rounded text-xs font-semibold transition-colors"
            >
              Exportar Resumen RRHH
            </button>
          {/if}

          <button
            onclick={() => window.print()}
            class="px-3 py-1.5 bg-raised] hover:bg-grid] text-ink] border border-grid] rounded text-xs font-semibold transition-colors"
          >
            Imprimir / PDF Oficial
          </button>

          <button
            onclick={onClose}
            class="tg-modal-close"
            aria-label="Cerrar"
            title="Cerrar"
          >
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
              <path d="M2 2l10 10M12 2L2 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Barra de Filtros: Selección de Agente, Mes y Año (No imprimible) -->
      <div class="bg-void] p-3 rounded border border-grid] mb-4 flex flex-wrap items-center justify-between gap-3 print:hidden">
        {#if activeTab === 'individual'}
          <div class="flex items-center gap-2 flex-1 min-w-[280px]">
            <label for="filter-agent-select" class="text-xs font-semibold text-dim] uppercase tracking-wider shrink-0">Agente:</label>
            <select
              id="filter-agent-select"
              bind:value={selectedPersonnelId}
              onchange={loadSheet}
              class="w-full h-9 px-3 bg-panel] border border-grid] rounded text-xs font-medium text-ink] outline-none focus:border-amber]"
            >
              {#each personnelList as p}
                <option value={p.id}>
                  {p.last_name}, {p.first_name} {p.legajo ? `[DNI: ${p.legajo}]` : ''} - {p.category_name}
                </option>
              {/each}
            </select>
          </div>
        {/if}

        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5">
            <label for="filter-month-select" class="text-xs font-semibold text-dim] uppercase tracking-wider">Mes:</label>
            <select
              id="filter-month-select"
              bind:value={selectedMonth}
              onchange={() => { if (activeTab === 'individual') loadSheet(); else loadCompanySummary(); }}
              class="h-9 px-3 bg-panel] border border-grid] rounded text-xs font-medium text-ink] outline-none focus:border-amber]"
            >
              {#each months as m}
                <option value={m.value}>{m.name}</option>
              {/each}
            </select>
          </div>

          <div class="flex items-center gap-1.5">
            <label for="filter-year-input" class="text-xs font-semibold text-dim] uppercase tracking-wider">Año:</label>
            <input
              id="filter-year-input"
              type="number"
              bind:value={selectedYear}
              onchange={() => { if (activeTab === 'individual') loadSheet(); else loadCompanySummary(); }}
              class="h-9 w-24 px-2.5 bg-panel] border border-grid] rounded text-xs font-mono font-semibold text-amber] text-center outline-none focus:border-amber]"
            />
          </div>
        </div>
      </div>

      <!-- ================================================================= -->
      <!-- CUERPO PRINCIPAL DEL DOCUMENTO (IMPRIMIBLE A4)                   -->
      <!-- ================================================================= -->
      <div class="flex-1 overflow-y-auto pr-1 text-black bg-white p-6 md:p-8 rounded shadow-inner font-sans print:p-0 print:m-0 print:shadow-none print:overflow-visible">
        
        <!-- PESTAÑA 1: PLANILLA INDIVIDUAL DEL AGENTE -->
        {#if activeTab === 'individual'}
          {#if isLoadingSheet}
            <div class="py-16 text-center text-xs font-mono text-slate-500">
              Generando planilla mensual del agente...
            </div>
          {:else if currentSheet}
            <!-- Encabezado Institucional de la Planilla -->
            <div class="text-center pb-3 mb-4 border-b-2 border-black">
              <h1 class="text-xs md:text-sm font-black uppercase tracking-wider">
                Teatro del Libertador San Martín • Ballet Oficial de Córdoba
              </h1>
              <h2 class="text-sm md:text-base font-black uppercase tracking-wide underline underline-offset-4 decoration-2 mt-0.5">
                PLANILLA MENSUAL INDIVIDUAL DE ASISTENCIA Y NOVEDADES • {currentSheet.month_name} {currentSheet.year}
              </h2>
            </div>

            <!-- Ficha del Agente y Resumen Métrico -->
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 p-3 bg-slate-100 border border-black rounded-lg mb-4 text-xs font-mono">
              <div>
                <span class="text-slate-600 block text-[10px] uppercase font-bold">Apellido y Nombre:</span>
                <strong class="text-sm font-sans">{currentSheet.last_name}, {currentSheet.first_name}</strong>
              </div>
              <div>
                <span class="text-slate-600 block text-[10px] uppercase font-bold">DNI:</span>
                <strong class="text-sm">{currentSheet.legajo || 'S/D'}</strong>
              </div>
              <div>
                <span class="text-slate-600 block text-[10px] uppercase font-bold">Categoría / Cargo:</span>
                <strong class="text-xs">{currentSheet.category_name}</strong>
              </div>
              <div>
                <span class="text-slate-600 block text-[10px] uppercase font-bold">Presentismo Mensual:</span>
                <strong class="text-sm {currentSheet.presence_percentage >= 90 ? 'text-emerald-700' : 'text-rose-700'}">
                  {currentSheet.presence_percentage.toFixed(1)}%
                </strong>
              </div>
            </div>

            <!-- Grilla Diaria de los 30 / 31 Días del Mes (Control en Clase) -->
            <table class="w-full border-collapse border border-black text-center text-[10px] mb-4">
              <thead>
                <tr class="bg-slate-200 font-bold border-b border-black uppercase text-[9px]">
                  <th class="border border-black py-1 px-1.5 w-[8%]">Día</th>
                  <th class="border border-black py-1 px-1.5 w-[14%]">Fecha</th>
                  <th class="border border-black py-1 px-1.5 w-[16%]">Control en Clase (09:00)</th>
                  <th class="border border-black py-1 px-1.5 w-[20%]">Estado de Asistencia</th>
                  <th class="border border-black py-1 px-1.5 w-[12%]">Demora Declarada</th>
                  <th class="border border-black py-1 px-2 w-[30%] text-left">Diagnóstico / Novedades / Firma</th>
                </tr>
              </thead>
              <tbody>
                {#each currentSheet.entries as e}
                  <tr class="border-b border-black {e.is_weekend ? 'bg-slate-50 text-slate-500 italic' : e.status === 'LATE' ? 'bg-amber-50 font-medium' : e.status === 'MEDICAL_LEAVE' ? 'bg-purple-50 font-medium' : ''}">
                    <td class="border border-black py-1 px-1 font-bold">{e.day_of_week.slice(0, 3)}</td>
                    <td class="border border-black py-1 px-1.5 font-mono">{e.date}</td>
                    <td class="border border-black py-1 px-1 font-semibold">
                      {#if e.is_weekend}
                        <span class="text-slate-400">Franco / Descanso</span>
                      {:else if e.status === 'IN_PHYSIOTHERAPY'}
                        <span class="text-cyan-800">En Fisioterapia</span>
                      {:else if e.is_in_class}
                        <span class="text-emerald-800">En Barra / Salón</span>
                      {:else}
                        <span class="text-amber-800">No en clase</span>
                      {/if}
                    </td>
                    <td class="border border-black py-1 px-1.5 font-bold">
                      {#if e.is_weekend}
                        <span class="text-slate-500">Franco / Descanso</span>
                      {:else if e.status === 'PRESENT'}
                        <span class="text-emerald-800">Presente</span>
                      {:else if e.status === 'LATE'}
                        <span class="text-amber-800">Tardanza ({e.minutes_late}m)</span>
                      {:else if e.status === 'MEDICAL_LEAVE'}
                        <span class="text-purple-800">Licencia Médica</span>
                      {:else if e.status === 'IN_PHYSIOTHERAPY'}
                        <span class="text-cyan-800">Fisioterapia</span>
                      {:else if e.status === 'NOT_IN_CLASS'}
                        <span class="text-blue-800">Eximido / En teatro</span>
                      {:else if e.status === 'ABSENT'}
                        <span class="text-rose-800">Ausente</span>
                      {:else if e.status === 'JUSTIFIED_ABSENCE'}
                        <span class="text-blue-800">Inasistencia justificada</span>
                      {:else if e.status === 'TASK_ADJUSTMENT'}
                        <span class="text-blue-800">Adecuación de tareas</span>
                      {:else if e.status === 'NO_PART'}
                        <span class="text-purple-800">Sin parte</span>
                      {:else}
                        <span class="text-slate-400">Sin registro</span>
                      {/if}
                    </td>
                    <td class="border border-black py-1 px-1 font-mono font-bold {e.minutes_late > 0 ? 'text-amber-800' : 'text-slate-400'}">
                      {e.minutes_late > 0 ? `+${e.minutes_late} min` : '—'}
                    </td>
                    <td class="border border-black py-1 px-2 text-left truncate">
                      {#if e.medical_diagnosis}
                        <span class="font-bold text-purple-900">Diag: {e.medical_diagnosis}</span>
                      {/if}
                      {#if e.remarks}
                        <span class="text-slate-700 ml-1">{e.remarks}</span>
                      {/if}
                      {#if e.has_certificate}
                        <span class="text-[9px] font-mono border border-black px-1 rounded ml-1">CERT</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>

            <!-- Resumen de Totales Mensuales para RRHH -->
            <div class="grid grid-cols-2 sm:grid-cols-6 gap-2 p-3 bg-slate-200 border border-black rounded-lg text-xs font-mono mb-6">
              <div>
                <span class="text-slate-700 block text-[9px] uppercase font-bold">Días Hábiles:</span>
                <strong class="text-sm">{currentSheet.total_working_days}</strong>
              </div>
              <div>
                <span class="text-slate-700 block text-[9px] uppercase font-bold">Días Presente:</span>
                <strong class="text-sm text-emerald-800">{currentSheet.days_present}</strong>
              </div>
              <div>
                <span class="text-slate-700 block text-[9px] uppercase font-bold">Tardanzas (TC):</span>
                <strong class="text-sm text-amber-800">{currentSheet.days_late} ({currentSheet.total_late_minutes} min)</strong>
              </div>
              <div>
                <span class="text-slate-700 block text-[9px] uppercase font-bold">Licencias Médicas:</span>
                <strong class="text-sm text-purple-800">{currentSheet.days_medical_leave}</strong>
              </div>
              <div>
                <span class="text-slate-700 block text-[9px] uppercase font-bold">En Fisioterapia:</span>
                <strong class="text-sm text-cyan-800">{currentSheet.days_physiotherapy}</strong>
              </div>
              <div>
                <span class="text-slate-700 block text-[9px] uppercase font-bold">Inasistencias:</span>
                <strong class="text-sm text-rose-800">{currentSheet.days_absent}</strong>
              </div>
            </div>

            <!-- Cláusula Reglamentaria y Espacios de Firma Oficial -->
            <div class="pt-2 text-center border-t border-black mb-8">
              <p class="text-[10px] font-mono uppercase tracking-wider text-slate-700">
                Planilla individual mensual de constancia para cómputo de haberes, presentismo y deducciones estatutarias.
              </p>
            </div>

            <div class="grid grid-cols-3 gap-6 pt-6 text-center text-xs font-mono">
              <div class="border-t border-black pt-2">
                <span class="block font-bold">Firma del Agente</span>
                <span class="text-[10px] text-slate-500">Conformidad de Asistencia</span>
              </div>
              <div class="border-t border-black pt-2">
                <span class="block font-bold">Inspección del Ballet</span>
                <span class="text-[10px] text-slate-500">Carina Lion / Mtro. Ensayador</span>
              </div>
              <div class="border-t border-black pt-2">
                <span class="block font-bold">Recursos Humanos / Personal</span>
                <span class="text-[10px] text-slate-500">Recepción & Cómputo</span>
              </div>
            </div>
          {/if}

        <!-- PESTAÑA 2: RESUMEN GENERAL DE TODA LA COMPAÑÍA PARA RRHH -->
        {:else}
          {#if isLoadingSummary}
            <div class="py-16 text-center text-xs font-mono text-slate-500">
              Consolidando datos de todos los agentes para RRHH...
            </div>
          {:else}
            <div class="text-center pb-3 mb-4 border-b-2 border-black">
              <h1 class="text-xs md:text-sm font-black uppercase tracking-wider">
                Teatro del Libertador San Martín • Ballet Oficial de Córdoba
              </h1>
              <h2 class="text-sm md:text-base font-black uppercase tracking-wide underline underline-offset-4 decoration-2 mt-0.5">
                CONSOLIDADO GENERAL DE ASISTENCIA Y NOVEDADES PARA RRHH • {months.find(m => m.value === selectedMonth)?.name.toUpperCase()} {selectedYear}
              </h2>
            </div>

            <table class="w-full border-collapse border border-black text-center text-[10px] mb-6">
              <thead>
                <tr class="bg-slate-200 font-bold border-b border-black uppercase">
                  <th class="border border-black py-1.5 px-2 w-[10%]">DNI</th>
                  <th class="border border-black py-1.5 px-3 w-[26%] text-left">Apellido y Nombre</th>
                  <th class="border border-black py-1.5 px-2 w-[12%]">Cargo</th>
                  <th class="border border-black py-1.5 px-2 w-[10%]">Presentes</th>
                  <th class="border border-black py-1.5 px-2 w-[12%]">Tardanzas (TC)</th>
                  <th class="border border-black py-1.5 px-2 w-[10%]">Lic. Médica</th>
                  <th class="border border-black py-1.5 px-2 w-[10%]">Fisio</th>
                  <th class="border border-black py-1.5 px-2 w-[10%]">Faltas</th>
                  <th class="border border-black py-1.5 px-2 w-[10%]">% Present.</th>
                </tr>
              </thead>
              <tbody>
                {#each companySummary as s}
                  <tr class="border-b border-black hover:bg-slate-100 transition-colors">
                    <td class="border border-black py-1 px-2 font-mono font-bold">{s.legajo || '—'}</td>
                    <td class="border border-black py-1 px-3 text-left font-bold">{s.dancer_name}</td>
                    <td class="border border-black py-1 px-2 font-mono text-[9px]">{roleLabel(s.role)}</td>
                    <td class="border border-black py-1 px-2 font-mono font-bold text-emerald-800">{s.days_present}</td>
                    <td class="border border-black py-1 px-2 font-mono {s.days_late > 0 ? 'text-amber-800 font-bold' : ''}">
                      {s.days_late > 0 ? `${s.days_late} (${s.total_late_minutes}m)` : '0'}
                    </td>
                    <td class="border border-black py-1 px-2 font-mono {s.days_medical_leave > 0 ? 'text-purple-800 font-bold' : ''}">
                      {s.days_medical_leave}
                    </td>
                    <td class="border border-black py-1 px-2 font-mono text-cyan-800">{s.days_physio}</td>
                    <td class="border border-black py-1 px-2 font-mono {s.days_absent > 0 ? 'text-rose-800 font-bold' : ''}">
                      {s.days_absent}
                    </td>
                    <td class="border border-black py-1 px-2 font-mono font-black {s.presence_percentage >= 90 ? 'text-emerald-800' : 'text-rose-800'}">
                      {s.presence_percentage.toFixed(1)}%
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>

            <div class="grid grid-cols-2 gap-6 pt-6 text-center text-xs font-mono">
              <div class="border-t border-black pt-2">
                <span class="block font-bold">Inspección General del Ballet</span>
                <span class="text-[10px] text-slate-500">Certificación de Planillas</span>
              </div>
              <div class="border-t border-black pt-2">
                <span class="block font-bold">Dirección de Recursos Humanos</span>
                <span class="text-[10px] text-slate-500">Recepción Oficial Teatro del Libertador</span>
              </div>
            </div>
          {/if}
        {/if}

      </div>
    </div>
  </div>
{/if}
