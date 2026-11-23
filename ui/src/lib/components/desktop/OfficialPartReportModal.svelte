<script lang="ts">
  import { saveBlob } from '../../nativeOutput';
  import { regimeLabel, roomLabel } from '../../labels';
  import { toLocalIso } from '../../date';
  import { api, type DailyScheduleAggregate } from '../../ipc/client';
  import { modalBehavior } from '../../actions/modalBehavior';
  import PrintPreviewModal from '../shared/PrintPreviewModal.svelte';
  import ZoomControls from '../shared/ZoomControls.svelte';
  import { zoomable, defaultDocumentZoom } from '../../actions/zoomable';

  let printPreviewOpen = $state(false);
  let zoom = $state(1);
  $effect(() => {
    if (isOpen) zoom = defaultDocumentZoom();
  });
  let printableEl: HTMLElement | null = $state(null);

  let {
    schedule,
    isOpen,
    onClose
  } = $props<{
    schedule: DailyScheduleAggregate;
    isOpen: boolean;
    onClose: () => void;
  }>();

  let activeReportTab = $state<'classes' | 'rehearsals' | 'today'>('today');

  interface WeeklyDayReport {
    iso: string;
    label: string;
    schedule: DailyScheduleAggregate | null;
  }

  let weekDays = $state<WeeklyDayReport[]>([]);
  let weekRangeLabel = $state('');
  let isLoadingWeek = $state(false);
  let loadedWeekKey = $state('');

  const DAY_NAMES = ['Lunes', 'Martes', 'Miércoles', 'Jueves', 'Viernes', 'Sábado', 'Domingo'];

  function getMondayFromIso(iso: string): Date {
    const [y, m, d] = iso.split('-').map(Number);
    const base = new Date(y, (m || 1) - 1, d || 1);
    const dow = base.getDay();
    const diff = base.getDate() - dow + (dow === 0 ? -6 : 1);
    return new Date(base.getFullYear(), base.getMonth(), diff);
  }

  function formatDayMonth(d: Date): string {
    return `${String(d.getDate()).padStart(2, '0')}/${String(d.getMonth() + 1).padStart(2, '0')}`;
  }

  async function loadWeekSchedules(baseIso: string) {
    const monday = getMondayFromIso(baseIso);
    const weekKey = toLocalIso(monday);
    isLoadingWeek = true;
    try {
      const daysMeta = Array.from({ length: 7 }, (_, i) => {
        const dt = new Date(monday.getFullYear(), monday.getMonth(), monday.getDate() + i);
        return {
          iso: toLocalIso(dt),
          label: `${DAY_NAMES[i]} ${formatDayMonth(dt)}`,
          dt,
        };
      });

      const sunday = daysMeta[6].dt;
      weekRangeLabel = `Semana del ${formatDayMonth(monday)} al ${formatDayMonth(sunday)}/${sunday.getFullYear()}`;

      const results = await Promise.all(
        daysMeta.map(async (dm) => {
          if (dm.iso === schedule.schedule_date) {
            return { iso: dm.iso, label: dm.label, schedule };
          }
          try {
            const res = await api.getOrCreateDailySchedule(dm.iso);
            return { iso: dm.iso, label: dm.label, schedule: res.schedule };
          } catch (e) {
            console.error(`Error cargando día ${dm.iso}:`, e);
            return { iso: dm.iso, label: dm.label, schedule: null };
          }
        })
      );

      weekDays = results;
      loadedWeekKey = weekKey;
    } finally {
      isLoadingWeek = false;
    }
  }

  $effect(() => {
    if (isOpen && schedule?.schedule_date) {
      loadWeekSchedules(schedule.schedule_date);
    }
  });

  function shortName(fullName?: string | null): string {
    if (!fullName) return '—';
    return fullName.split(',')[0].trim();
  }

  function csvEscape(v: string): string {
    return `"${(v || '').replace(/"/g, '""').replace(/\r?\n/g, ' ')}"`;
  }

  function exportTodayExcel() {
    let csv = '\uFEFF'; // BOM UTF-8
    csv += `PARTE OFICIAL DE CLASES Y ENSAYOS - ${schedule.schedule_date}\n`;
    csv += `REGIMEN;${csvEscape(regimeLabel(schedule.regime))};MARCADA;${csvEscape(schedule.mandatory_checkin_time)};CIERRE;${csvEscape(schedule.nominal_end_time)}\n\n`;

    csv += 'CLASES TECNICAS MATUTINAS\n';
    csv += 'HORARIO;SALA;DIVISION;MAESTRO/A;ACOMPANAMIENTO;SONIDISTA\n';
    if (schedule.morning_classes.length === 0) {
      csv += '"Sin clases matutinas programadas";"";"";"";"";""\n';
    } else {
      schedule.morning_classes.forEach((c) => {
        const div = c.gender_division === 'FEMALE' ? 'Mujeres' : c.gender_division === 'MALE' ? 'Varones' : 'General Compañía';
        csv += [
          `${c.start_time} - ${c.end_time}`,
          roomLabel(c.room),
          div,
          c.master_name || '',
          c.pianist_name || '',
          shortName(c.sound_tech_name),
        ].map(csvEscape).join(';') + '\n';
      });
    }

    csv += '\nENSAYOS Y PUESTA EN ESCENA\n';
    csv += 'HORARIO;SALA;PASAJE / OBRA;REPARTO DE ESTUDIO;MAESTRO;SONIDO;SONIDISTA\n';
    if (schedule.rehearsal_blocks.length === 0) {
      csv += '"Sin bloques de ensayo asignados";"";"";"";"";"";""\n';
    } else {
      schedule.rehearsal_blocks.forEach((b) => {
        const cast = b.cast_groups.length > 0 && b.cast_groups[0].formatted_line
          ? b.cast_groups[0].formatted_line
          : 'Reparto a confirmar';
        csv += [
          `${b.start_time} - ${b.end_time}`,
          roomLabel(b.room),
          b.passage_name,
          cast,
          shortName(b.master_name),
          b.sound_requirement || '',
          shortName(b.sound_tech_name),
        ].map(csvEscape).join(';') + '\n';
      });
    }

    csv += `\n${csvEscape(schedule.footer_legal_clause || 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.')}\n`;
    downloadCsv(csv, `Parte_Diario_${schedule.schedule_date}.csv`);
  }

  function exportClassesExcel() {
    let csv = '\uFEFF'; // BOM UTF-8
    csv += `PARTE DE CLASES - ${weekRangeLabel.toUpperCase()}\n\n`;
    csv += 'DIA;FECHA;HORARIO;SALA;BAILARINES;MAESTROS;ACOMPANAMIENTO;SONIDISTA\n';

    weekDays.forEach((d) => {
      const classes = d.schedule?.morning_classes ?? [];
      if (classes.length === 0) {
        csv += [d.label, d.iso, '—', '—', 'Sin clases programadas', '—', '—', '—'].map(csvEscape).join(';') + '\n';
      } else {
        classes.forEach((s) => {
          const div = s.gender_division === 'FEMALE' ? 'Mujeres' : s.gender_division === 'MALE' ? 'Varones' : 'General Compañía';
          csv += [
            d.label,
            d.iso,
            `${s.start_time} a ${s.end_time} h`,
            roomLabel(s.room),
            div,
            shortName(s.master_name),
            shortName(s.pianist_name),
            shortName(s.sound_tech_name),
          ].map(csvEscape).join(';') + '\n';
        });
      }
    });

    const slug = loadedWeekKey || schedule.schedule_date;
    downloadCsv(csv, `Parte_de_Clases_Semana_${slug}.csv`);
  }

  function exportRehearsalsExcel() {
    let csv = '\uFEFF'; // BOM UTF-8
    csv += `PARTE DE ENSAYO - ${weekRangeLabel.toUpperCase()}\n\n`;
    csv += 'DIA;FECHA;HORARIO;SALA;OBRA / PASAJE;REPARTO;MAESTRO;SONIDO;SONIDISTA\n';

    weekDays.forEach((d) => {
      const blocks = d.schedule?.rehearsal_blocks ?? [];
      if (blocks.length === 0) {
        csv += [d.label, d.iso, '—', '—', 'Sin ensayos citados', '—', '—', '—', '—'].map(csvEscape).join(';') + '\n';
      } else {
        blocks.forEach((b) => {
          const cast = b.cast_groups.length > 0 && b.cast_groups[0].formatted_line
            ? b.cast_groups[0].formatted_line
            : 'Reparto a confirmar';
          csv += [
            d.label,
            d.iso,
            `${b.start_time} a ${b.end_time} h`,
            roomLabel(b.room),
            b.passage_name,
            cast,
            shortName(b.master_name),
            b.sound_requirement || '',
            shortName(b.sound_tech_name),
          ].map(csvEscape).join(';') + '\n';
        });
      }
    });

    csv += `\n${csvEscape(schedule.footer_legal_clause || 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.')}\n`;
    const slug = loadedWeekKey || schedule.schedule_date;
    downloadCsv(csv, `Parte_de_Ensayo_Semana_${slug}.csv`);
  }

  function exportActiveTabExcel() {
    if (activeReportTab === 'today') {
      exportTodayExcel();
    } else if (activeReportTab === 'classes') {
      exportClassesExcel();
    } else {
      exportRehearsalsExcel();
    }
  }

  function downloadCsv(content: string, filename: string) {
    const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
    saveBlob(blob, filename).catch((e) => alert(e?.message || e));
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-5xl overflow-y-auto p-4 md:p-6 relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">
      
      <!-- Barra superior no imprimible -->
      <div class="flex pb-3 mb-4 border-b border-grid print:hidden gap-3 items-start">
        <!-- cierre fijo en la esquina: el resto de la barra puede partirse en filas -->
        <div class="flex flex-1 min-w-0 flex-wrap items-center justify-between gap-3">
          <div>
            <h2 class="text-sm font-semibold text-ink uppercase tracking-wide">
              Generador Oficial de Partes Diarios y Semanales
            </h2>
            <p class="text-xs text-dim mt-0.5">
              Formato exacto de inspección del Ballet Oficial de Córdoba
            </p>
          </div>

          <!-- Conmutador Parte del Día vs Semana -->
          <div class="inline-flex bg-void p-1 rounded border border-grid">
            <button
              onclick={() => activeReportTab = 'today'}
              class="px-3 py-1 rounded text-xs font-medium transition-colors {activeReportTab === 'today' ? 'bg-amber text-black font-semibold' : 'text-dim hover:text-ink'}"
            >
              Parte de Hoy ({schedule.schedule_date})
            </button>
            <button
              onclick={() => activeReportTab = 'classes'}
              class="px-3 py-1 rounded text-xs font-medium transition-colors {activeReportTab === 'classes' ? 'bg-amber text-black font-semibold' : 'text-dim hover:text-ink'}"
            >
              Semana Clases
            </button>
            <button
              onclick={() => activeReportTab = 'rehearsals'}
              class="px-3 py-1 rounded text-xs font-medium transition-colors {activeReportTab === 'rehearsals' ? 'bg-amber text-black font-semibold' : 'text-dim hover:text-ink'}"
            >
              Semana Ensayos
            </button>
          </div>

          <!-- Zoom y Botones de Exportación -->
          <div class="flex flex-wrap items-center gap-2">
            <ZoomControls bind:zoom />
            <button
              onclick={exportActiveTabExcel}
              class="px-3 py-1.5 bg-raised hover:bg-grid text-green border border-grid rounded text-xs font-semibold transition-colors"
            >
              Exportar Excel (.csv)
            </button>

            <button
              onclick={() => (printPreviewOpen = true)}
              class="px-3 py-1.5 bg-raised hover:bg-grid text-ink border border-grid rounded text-xs font-semibold transition-colors"
            >
              Imprimir / Guardar PDF
            </button>

          </div>
        </div>
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

      <!-- ================================================================= -->
      <!-- DOCUMENTO IMPRIMIBLE A4 (DISEÑO IDÉNTICO A LAS HOJAS DE INSPECCIÓN) -->
      <!-- ================================================================= -->
      <div class="overflow-x-auto bg-white bg-local rounded print:overflow-visible" use:zoomable={{ zoom, onzoom: (z) => (zoom = z) }}>
      <div bind:this={printableEl} style:zoom={zoom} class="tg-zoom-doc bg-white text-black p-6 md:p-10 rounded font-sans print:shadow-none print:p-0 print:m-0">
        
        <!-- PARTE DIARIO ACTIVO -->
        {#if activeReportTab === 'today'}
          <div class="space-y-6">
            <div class="text-center pb-3 border-b-2 border-black">
              <h1 class="text-sm md:text-base font-black uppercase tracking-wider">
                Teatro del Libertador San Martín • Ballet Oficial de Córdoba
              </h1>
              <h2 class="text-base md:text-xl font-black uppercase tracking-wide underline underline-offset-4 decoration-2 mt-1">
                PARTE OFICIAL DE CLASES Y ENSAYOS • {schedule.schedule_date}
              </h2>
              <div class="flex flex-wrap justify-between items-center text-xs font-mono mt-2 pt-1 border-t border-slate-300 gap-2">
                <span>Régimen: <strong>{regimeLabel(schedule.regime)}</strong></span>
                <span>Marcada Obligatoria: <strong>{schedule.mandatory_checkin_time} h</strong></span>
                <span>Cierre Jornada: <strong>{schedule.nominal_end_time} h</strong></span>
                <span>Descanso Mínimo: <strong>{schedule.minimum_intershift_rest_hours}h</strong></span>
              </div>
            </div>

            <!-- Clases Técnicas Matutinas -->
            <div>
              <h3 class="text-xs font-black uppercase tracking-wider bg-slate-200 border border-black p-1.5 mb-2">
                I. Clases Técnicas Matutinas ({schedule.morning_classes.length > 0 ? `${schedule.morning_classes[0].start_time} a ${schedule.morning_classes[0].end_time} h` : '09:00 a 10:15 h'})
              </h3>
              {#if schedule.morning_classes.length === 0}
                <p class="text-xs italic text-slate-600">Sin clases matutinas programadas para este día.</p>
              {:else}
                <table class="w-full border-collapse border border-black text-center text-xs mb-4">
                  <thead>
                    <tr class="bg-slate-100 font-bold border-b border-black text-[11px]">
                      <th class="border border-black py-1.5 px-2">HORARIO</th>
                      <th class="border border-black py-1.5 px-2">SALA</th>
                      <th class="border border-black py-1.5 px-2">DIVISIÓN</th>
                      <th class="border border-black py-1.5 px-2">MAESTRO/A</th>
                      <th class="border border-black py-1.5 px-2">ACOMPAÑAMIENTO</th>
                      <th class="border border-black py-1.5 px-2">SONIDISTA</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each schedule.morning_classes as c}
                      <tr class="border-b border-black">
                        <td class="border border-black py-1.5 px-2 font-mono font-bold">{c.start_time} - {c.end_time}</td>
                        <td class="border border-black py-1.5 px-2 font-black">{roomLabel(c.room)}</td>
                        <td class="border border-black py-1.5 px-2">{c.gender_division === 'FEMALE' ? 'Mujeres' : c.gender_division === 'MALE' ? 'Varones' : 'General'}</td>
                        <td class="border border-black py-1.5 px-2 font-bold">{c.master_name}</td>
                        <td class="border border-black py-1.5 px-2">{c.pianist_name}</td>
                        <td class="border border-black py-1.5 px-2">{c.sound_tech_name ? c.sound_tech_name.split(',')[0] : '—'}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              {/if}
            </div>

            <!-- Bloques de Ensayo y Puesta -->
            <div>
              <h3 class="text-xs font-black uppercase tracking-wider bg-slate-200 border border-black p-1.5 mb-2">
                II. Citaciones y Ensayos de Repertorio & Puesta en Escena
              </h3>
              {#if schedule.rehearsal_blocks.length === 0}
                <p class="text-xs italic text-slate-600">Sin bloques de ensayo asignados para este día.</p>
              {:else}
                <table class="w-full border-collapse border border-black text-center text-xs mb-4">
                  <thead>
                    <tr class="bg-slate-100 font-bold border-b border-black text-[11px]">
                      <th class="border border-black py-1.5 px-2 w-[16%]">HORARIO</th>
                      <th class="border border-black py-1.5 px-2 w-[12%]">SALA</th>
                      <th class="border border-black py-1.5 px-2 w-[22%]">PASAJE / OBRA</th>
                      <th class="border border-black py-1.5 px-2 w-[28%]">REPARTO DE ESTUDIO</th>
                      <th class="border border-black py-1.5 px-2 w-[12%]">MAESTRO</th>
                      <th class="border border-black py-1.5 px-2 w-[10%]">SONIDO</th>
                      <th class="border border-black py-1.5 px-2 w-[10%]">SONIDISTA</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each schedule.rehearsal_blocks as b}
                      <tr class="border-b border-black">
                        <td class="border border-black py-1.5 px-2 font-mono font-bold">{b.start_time} - {b.end_time}</td>
                        <td class="border border-black py-1.5 px-2 font-black">{roomLabel(b.room)}</td>
                        <td class="border border-black py-1.5 px-2 text-left font-bold">{b.passage_name}</td>
                        <td class="border border-black py-1.5 px-2 text-left font-mono text-[11px]">
                          {#if b.cast_groups.length > 0 && b.cast_groups[0].formatted_line}
                            {b.cast_groups[0].formatted_line}
                          {:else}
                            Reparto a confirmar
                          {/if}
                        </td>
                        <td class="border border-black py-1.5 px-2 text-xs font-semibold">
                          {b.master_name ? b.master_name.split(',')[0] : '—'}
                        </td>
                        <td class="border border-black py-1.5 px-2 font-mono text-[11px] font-bold">
                          {b.sound_requirement}
                        </td>
                        <td class="border border-black py-1.5 px-2 text-xs font-semibold">
                          {b.sound_tech_name ? b.sound_tech_name.split(',')[0] : '—'}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              {/if}
            </div>

            <!-- Cláusula Legal Invariable al pie -->
            <div class="mt-8 pt-4 text-center">
              <p class="text-xs md:text-sm font-black tracking-wider uppercase font-mono border-t-2 border-black pt-3">
                {schedule.footer_legal_clause || '★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★'}
              </p>
            </div>
          </div>

        <!-- PARTE DE CLASES SEMANALES (7 DÍAS REALES DESDE SQLITE) -->
        {:else if activeReportTab === 'classes'}
          <div class="space-y-4">
            <div class="text-center mb-6">
              <h2 class="text-lg md:text-xl font-black uppercase tracking-wide underline underline-offset-4 decoration-2">
                PARTE DE CLASES — {weekRangeLabel}
              </h2>
            </div>

            {#if isLoadingWeek && weekDays.length === 0}
              <p class="text-xs italic text-slate-600 text-center py-8">Cargando cronograma semanal desde la base de datos...</p>
            {:else}
              {#each weekDays as d}
                {@const slots = d.schedule?.morning_classes ?? []}
                <table class="w-full border-collapse border border-black text-center text-xs md:text-sm mb-4">
                  <thead>
                    <tr class="bg-slate-300 font-bold border-b border-black text-[11px] md:text-xs">
                      <th class="border border-black py-1.5 px-2 w-[16%]">DIA</th>
                      <th class="border border-black py-1.5 px-2 w-[16%]">HORARIO</th>
                      <th class="border border-black py-1.5 px-2 w-[12%]">SALA</th>
                      <th class="border border-black py-1.5 px-2 w-[16%]">BAILARINES</th>
                      <th class="border border-black py-1.5 px-2 w-[16%]">MAESTROS</th>
                      <th class="border border-black py-1.5 px-2 w-[14%]">PIANISTAS</th>
                      <th class="border border-black py-1.5 px-2 w-[10%]">SONIDISTA</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#if slots.length === 0}
                      <tr class="border-b border-black">
                        <td class="border border-black font-bold bg-amber text-xs md:text-sm py-2 px-2 align-middle">
                          {d.label}
                        </td>
                        <td colspan="6" class="border border-black py-2 px-2 italic text-slate-500">
                          Sin clases programadas
                        </td>
                      </tr>
                    {:else}
                      {#each slots as s, idx}
                        <tr class="border-b border-black">
                          {#if idx === 0}
                            <td rowspan={slots.length} class="border border-black font-bold bg-amber text-xs md:text-sm py-2 px-2 align-middle">
                              {d.label}
                            </td>
                          {/if}
                          <td class="border border-black py-2 px-2 font-medium font-mono">{s.start_time} a {s.end_time} h</td>
                          <td class="border border-black py-2 px-2 font-bold">{roomLabel(s.room)}</td>
                          <td class="border border-black py-2 px-2">{s.gender_division === 'FEMALE' ? 'Mujeres' : s.gender_division === 'MALE' ? 'Varones' : 'General Compañía'}</td>
                          <td class="border border-black py-2 px-2">{shortName(s.master_name)}</td>
                          <td class="border border-black py-2 px-2">{shortName(s.pianist_name)}</td>
                          <td class="border border-black py-2 px-2">{shortName(s.sound_tech_name)}</td>
                        </tr>
                      {/each}
                    {/if}
                  </tbody>
                </table>
              {/each}
            {/if}
          </div>

        <!-- PARTE DE ENSAYOS SEMANALES (7 DÍAS REALES DESDE SQLITE) -->
        {:else}
          <div class="space-y-4">
            <div class="text-center mb-6">
              <h2 class="text-lg md:text-xl font-black uppercase tracking-wide underline underline-offset-4 decoration-2">
                PARTE DE ENSAYO — {weekRangeLabel}
              </h2>
            </div>

            {#if isLoadingWeek && weekDays.length === 0}
              <p class="text-xs italic text-slate-600 text-center py-8">Cargando ensayos semanales desde la base de datos...</p>
            {:else}
              {#each weekDays as d}
                {@const dayBlocks = d.schedule?.rehearsal_blocks ?? []}
                <table class="w-full border-collapse border border-black text-center text-xs md:text-sm mb-4">
                  <thead>
                    <tr class="bg-slate-300 font-bold border-b border-black text-[11px] md:text-xs">
                      <th class="border border-black py-1.5 px-2 w-[14%]">DIA</th>
                      <th class="border border-black py-1.5 px-2 w-[15%]">HORARIO</th>
                      <th class="border border-black py-1.5 px-2 w-[11%]">SALA</th>
                      <th class="border border-black py-1.5 px-2 w-[32%]">OBRA / REPARTO</th>
                      <th class="border border-black py-1.5 px-2 w-[12%]">MAESTRO</th>
                      <th class="border border-black py-1.5 px-2 w-[8%]">SONIDO</th>
                      <th class="border border-black py-1.5 px-2 w-[8%]">SONIDISTA</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#if dayBlocks.length === 0}
                      <tr class="border-b border-black">
                        <td class="border border-black font-bold bg-amber text-xs md:text-sm py-2 px-2 align-middle">
                          {d.label}
                        </td>
                        <td colspan="6" class="border border-black py-2 px-2 italic text-slate-500">
                          Sin ensayos citados
                        </td>
                      </tr>
                    {:else}
                      {#each dayBlocks as b, idx}
                        <tr class="border-b border-black">
                          {#if idx === 0}
                            <td rowspan={dayBlocks.length} class="border border-black font-bold bg-amber text-xs md:text-sm py-2 px-2 align-middle">
                              {d.label}
                            </td>
                          {/if}
                          <td class="border border-black py-2 px-2 font-medium font-mono">{b.start_time} a {b.end_time} h</td>
                          <td class="border border-black py-2 px-2 font-bold">{roomLabel(b.room)}</td>
                          <td class="border border-black py-2 px-3 text-left leading-tight font-medium">
                            <div class="font-bold">{b.passage_name}</div>
                            {#if b.cast_groups.length > 0 && b.cast_groups[0].formatted_line}
                              <div class="text-[11px] font-mono text-slate-700 mt-0.5">({b.cast_groups[0].formatted_line})</div>
                            {:else if b.confirmation_state !== 'CONFIRMED'}
                              <div class="text-[11px] italic text-slate-600 mt-0.5">(A confirmar)</div>
                            {/if}
                          </td>
                          <td class="border border-black py-2 px-2 align-middle text-xs font-semibold">
                            {shortName(b.master_name)}
                          </td>
                          <td class="border border-black py-2 px-2 align-middle font-bold text-xs">
                            {b.sound_requirement || '—'}
                          </td>
                          <td class="border border-black py-2 px-2 align-middle text-xs font-semibold">
                            {shortName(b.sound_tech_name)}
                          </td>
                        </tr>
                      {/each}
                    {/if}
                  </tbody>
                </table>
              {/each}
            {/if}

            <!-- Cláusula Legal Invariable al pie -->
            <div class="mt-8 pt-4 text-center">
              <p class="text-xs md:text-sm font-black tracking-wider uppercase font-mono border-t-2 border-black pt-3">
                {schedule.footer_legal_clause || 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.'}
              </p>
            </div>
          </div>
        {/if}
      </div>
      </div>

      <PrintPreviewModal
        open={printPreviewOpen}
        title="Vista previa de impresión"
        subtitle="Parte oficial — ediciones solo para la copia impresa"
        source={printableEl}
        onclose={() => (printPreviewOpen = false)}
      />

    </div>
  </div>
{/if}

<style>
  @media print {
    :global(body) {
      background: white !important;
      color: black !important;
    }
  }
</style>
