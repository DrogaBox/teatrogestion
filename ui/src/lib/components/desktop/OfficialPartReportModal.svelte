<script lang="ts">
  import { regimeLabel, roomLabel } from '../../labels';
  import type { DailyScheduleAggregate } from '../../ipc/client';
  import { modalBehavior } from '../../actions/modalBehavior';

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

  // Datos de la semana oficial (19 al 23 de octubre 2026) según la planilla oficial de la inspección
  const weekClasses = [
    {
      day: 'Lunes 19/10',
      slots: [
        { time: '9 a 10.15 h', room: '1', dancers: 'Mujeres', master: 'Ananiev', pianist: 'Romero' },
        { time: '9 a 10.15 h', room: '3', dancers: 'Varones', master: 'Benites', pianist: 'Villegas' },
      ],
    },
    {
      day: 'Martes 20/10',
      slots: [
        { time: '9 a 10.15 h', room: '1', dancers: 'Varones', master: 'Ananiev', pianist: 'Brusutti' },
        { time: '9 a 10.15 h', room: '3', dancers: 'Mujeres', master: 'Pino', pianist: 'Romero' },
      ],
    },
    {
      day: 'Miércoles 21/10',
      slots: [
        { time: '9 a 10.15 h', room: '1', dancers: 'Mujeres', master: 'Ananiev', pianist: 'Villegas' },
        { time: '9 a 10.15 h', room: '3', dancers: 'Varones', master: 'Benites', pianist: 'Brusutti' },
      ],
    },
    {
      day: 'Jueves 22/10',
      slots: [
        { time: '9 a 10.15 h', room: '1', dancers: 'Varones', master: 'Pino', pianist: 'Romero' },
        { time: '9 a 10.15 h', room: '3', dancers: 'Mujeres', master: 'Benites', pianist: 'Villegas' },
      ],
    },
    {
      day: 'Viernes 23/10',
      slots: [
        { time: '9 a 10.15 h', room: '1', dancers: 'Mujeres', master: 'Pino', pianist: 'Brusutti' },
        { time: '9 a 10.15 h', room: '3', dancers: 'Varones', master: 'Ananiev', pianist: 'Romero' },
      ],
    },
  ];

  const weekRehearsals = [
    {
      day: 'Lunes 19/10',
      masterGroup: 'Baca – Benites – Ananiev – Pino',
      soundReq: 'Sonido',
      blocks: [
        { time: '10.30h a 11.30h', room: 'Sala 1', work: 'Princesas y Bufón\n(Refuerzos – Gacitúa – Beacon – Hazel – Martini)', master: '', sound: '' },
        { time: '11.30 a 12.30h', room: 'Sala 1', work: 'Danza Española\n(Álvarez/Marcato – Morchio C/Gil)', master: 'Baca – Benites – Ananiev – Pino', sound: 'Sonido' },
        { time: '12.30 a 13.45h', room: 'Sala 1', work: 'Danza Napolitana\n(Pérez L/Natali – Pérez F/Paredes)', master: '', sound: '' },
      ],
    },
    {
      day: 'Martes 20/10',
      masterGroup: 'Baca – Benites – Ananiev – Pino',
      soundReq: 'Sonido',
      blocks: [
        { time: '10.30 a 12h', room: 'Sala 1', work: 'Pas de deux – Odette y Sigfrido\n(Morchio J/Bravo -Vera/Altamirano)', master: 'Baca – Benites – Ananiev – Pino', sound: 'Sonido' },
        { time: '12 a 13h', room: 'Sala 1', work: 'Diana y Action\n(González/Altamirano – Morchio C/Cornejo – Ragessi/Mauro)', master: '', sound: '' },
      ],
    },
    {
      day: 'Miércoles 21/10',
      masterGroup: 'Baca – Benites – Ananiev – Pino',
      soundReq: 'Sonido',
      blocks: [
        { time: '10.30h a 11.30h', room: 'Sala 1', work: 'Danza Húngara\n(Reparto a confirmar)', master: 'Baca – Benites – Ananiev – Pino', sound: 'Sonido' },
        { time: '11.30 a 12h', room: 'Sala 1', work: 'Princesas y Bufón', master: '', sound: '' },
        { time: '12 a 13.30h', room: 'Sala 1', work: 'Pas de deux – Cisne Negro\n(Reparto a confirmar)', master: '', sound: '' },
      ],
    },
    {
      day: 'Jueves 22/10',
      masterGroup: 'Baca – Benites – Ananiev – Pino',
      soundReq: 'Sonido',
      blocks: [
        { time: '10.30 a 11.30h', room: 'Sala 1', work: 'Danza Mazurca\n(Reparto a confirmar)', master: 'Baca – Benites – Ananiev – Pino', sound: 'Sonido' },
        { time: '11.30 a 12h', room: 'Sala 1', work: 'Danza Napolitana\n(Pérez L/Natali – Pérez F/Paredes)', master: '', sound: '' },
        { time: '12 a 13.45h', room: 'Sala 1', work: 'Danza Húngara y Danza Española', master: '', sound: '' },
      ],
    },
    {
      day: 'Viernes 23/10',
      masterGroup: 'Baca – Benites – Ananiev – Pino',
      soundReq: 'Sonido',
      blocks: [
        { time: '10.30 a fin', room: 'Sala 1', work: 'ENSAYO A CONFIRMAR', master: 'Baca – Benites – Ananiev – Pino', sound: 'Sonido' },
      ],
    },
  ];

  function exportClassesExcel() {
    let csv = '\uFEFF'; // BOM UTF-8
    csv += 'PARTE DE CLASES - SEMANA 19 al 23 octubre 2026\n\n';
    csv += 'DIA;HORARIO;SALA;BAILARINES;MAESTROS;PIANISTAS\n';

    weekClasses.forEach((d) => {
      d.slots.forEach((s) => {
        csv += `"${d.day}";"${s.time}";"${s.room}";"${s.dancers}";"${s.master}";"${s.pianist}"\n`;
      });
    });

    downloadCsv(csv, 'Parte_de_Clases_Semana_19_23_Octubre.csv');
  }

  function exportRehearsalsExcel() {
    let csv = '\uFEFF'; // BOM UTF-8
    csv += 'PARTE DE ENSAYO - Semana 19 al 23 octubre\n\n';
    csv += 'DIA;HORARIO;SALA;OBRA;MAESTROS;SONIDO\n';

    weekRehearsals.forEach((d) => {
      d.blocks.forEach((b) => {
        const cleanWork = b.work.replace(/\n/g, ' ');
        csv += `"${d.day}";"${b.time}";"${b.room}";"${cleanWork}";"${b.master || d.masterGroup}";"${b.sound || d.soundReq}"\n`;
      });
    });

    csv += '\n"REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS."\n';
    downloadCsv(csv, 'Parte_de_Ensayo_Semana_19_23_Octubre.csv');
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
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-5xl overflow-y-auto p-4 md:p-6 relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">
      
      <!-- Barra superior no imprimible -->
      <div class="flex flex-wrap items-center justify-between pb-3 mb-4 border-b border-grid] print:hidden gap-3">
        <div>
          <h2 class="text-sm font-semibold text-ink] uppercase tracking-wide">
            Generador Oficial de Partes Diarios y Semanales
          </h2>
          <p class="text-xs text-dim] mt-0.5">
            Formato exacto de inspección del Ballet Oficial de Córdoba
          </p>
        </div>

        <!-- Conmutador Parte del Día vs Semana -->
        <div class="inline-flex bg-void] p-1 rounded border border-grid]">
          <button
            onclick={() => activeReportTab = 'today'}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeReportTab === 'today' ? 'bg-amber] text-black font-semibold' : 'text-dim] hover:text-ink]'}"
          >
            Parte de Hoy ({schedule.schedule_date})
          </button>
          <button
            onclick={() => activeReportTab = 'classes'}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeReportTab === 'classes' ? 'bg-amber] text-black font-semibold' : 'text-dim] hover:text-ink]'}"
          >
            Semana Clases
          </button>
          <button
            onclick={() => activeReportTab = 'rehearsals'}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeReportTab === 'rehearsals' ? 'bg-amber] text-black font-semibold' : 'text-dim] hover:text-ink]'}"
          >
            Semana Ensayos
          </button>
        </div>

        <!-- Botones de Exportación -->
        <div class="flex items-center gap-2">
          {#if activeReportTab === 'classes'}
            <button
              onclick={exportClassesExcel}
              class="px-3 py-1.5 bg-raised] hover:bg-grid] text-green] border border-grid] rounded text-xs font-semibold transition-colors"
            >
              Exportar Excel (.csv)
            </button>
          {:else}
            <button
              onclick={exportRehearsalsExcel}
              class="px-3 py-1.5 bg-raised] hover:bg-grid] text-green] border border-grid] rounded text-xs font-semibold transition-colors"
            >
              Exportar Excel (.csv)
            </button>
          {/if}

          <button
            onclick={() => window.print()}
            class="px-3 py-1.5 bg-raised] hover:bg-grid] text-ink] border border-grid] rounded text-xs font-semibold transition-colors"
          >
            Imprimir / Guardar PDF
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

      <!-- ================================================================= -->
      <!-- DOCUMENTO IMPRIMIBLE A4 (DISEÑO IDÉNTICO A LAS HOJAS DE INSPECCIÓN) -->
      <!-- ================================================================= -->
      <div class="bg-white text-black p-6 md:p-10 rounded font-sans print:shadow-none print:p-0 print:m-0">
        
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

        <!-- PARTE DE CLASES SEMANALES -->
        {:else if activeReportTab === 'classes'}
          <div class="space-y-4">
            <div class="text-center mb-6">
              <h2 class="text-lg md:text-xl font-black uppercase tracking-wide underline underline-offset-4 decoration-2">
                PARTE DE CLASES - SEMANA 19 al 23 octubre 2026
              </h2>
            </div>

            {#each weekClasses as d}
              <table class="w-full border-collapse border border-black text-center text-xs md:text-sm mb-4">
                <thead>
                  <tr class="bg-slate-300 font-bold border-b border-black text-[11px] md:text-xs">
                    <th class="border border-black py-1.5 px-2 w-[16%]">DIA</th>
                    <th class="border border-black py-1.5 px-2 w-[18%]">HORARIO</th>
                    <th class="border border-black py-1.5 px-2 w-[12%]">SALA</th>
                    <th class="border border-black py-1.5 px-2 w-[18%]">BAILARINES</th>
                    <th class="border border-black py-1.5 px-2 w-[18%]">MAESTROS</th>
                    <th class="border border-black py-1.5 px-2 w-[18%]">PIANISTAS</th>
                  </tr>
                </thead>
                <tbody>
                  {#each d.slots as s, idx}
                    <tr class="border-b border-black">
                      {#if idx === 0}
                        <td rowspan="2" class="border border-black font-bold bg-amber] text-xs md:text-sm py-2 px-2 align-middle">
                          {d.day}
                        </td>
                      {/if}
                      <td class="border border-black py-2 px-2 font-medium">{s.time}</td>
                      <td class="border border-black py-2 px-2 font-bold">{roomLabel(s.room)}</td>
                      <td class="border border-black py-2 px-2">{s.dancers}</td>
                      <td class="border border-black py-2 px-2">{s.master}</td>
                      <td class="border border-black py-2 px-2">{s.pianist}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/each}
          </div>

        <!-- PARTE DE ENSAYOS (Semana 19 al 23 de octubre) -->
        {:else}
          <div class="space-y-4">
            <div class="text-center mb-6">
              <h2 class="text-lg md:text-xl font-black uppercase tracking-wide underline underline-offset-4 decoration-2">
                PARTE DE ENSAYO – Semana 19 al 23 octubre
              </h2>
            </div>

            {#each weekRehearsals as d}
              <table class="w-full border-collapse border border-black text-center text-xs md:text-sm mb-4">
                <thead>
                  <tr class="bg-slate-300 font-bold border-b border-black text-[11px] md:text-xs">
                    <th class="border border-black py-1.5 px-2 w-[14%]">DIA</th>
                    <th class="border border-black py-1.5 px-2 w-[16%]">HORARIO</th>
                    <th class="border border-black py-1.5 px-2 w-[12%]">SALA</th>
                    <th class="border border-black py-1.5 px-2 w-[34%]">OBRA</th>
                    <th class="border border-black py-1.5 px-2 w-[14%]">MAESTROS</th>
                    <th class="border border-black py-1.5 px-2 w-[10%]">SONIDO</th>
                  </tr>
                </thead>
                <tbody>
                  {#each d.blocks as b, idx}
                    <tr class="border-b border-black">
                      {#if idx === 0}
                        <td rowspan={d.blocks.length} class="border border-black font-bold bg-amber] text-xs md:text-sm py-2 px-2 align-middle">
                          {d.day}
                        </td>
                      {/if}
                      <td class="border border-black py-2 px-2 font-medium">{b.time}</td>
                      <td class="border border-black py-2 px-2 font-bold">{roomLabel(b.room)}</td>
                      <td class="border border-black py-2 px-3 text-left whitespace-pre-line leading-tight font-medium">
                        {b.work}
                      </td>
                      {#if idx === 0}
                        <td rowspan={d.blocks.length} class="border border-black py-2 px-2 align-middle text-xs font-semibold">
                          {d.masterGroup}
                        </td>
                        <td rowspan={d.blocks.length} class="border border-black py-2 px-2 align-middle font-bold text-xs">
                          {d.soundReq}
                        </td>
                      {/if}
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/each}

            <!-- Cláusula Legal Invariable al pie -->
            <div class="mt-8 pt-4 text-center">
              <p class="text-xs md:text-sm font-black tracking-wider uppercase font-mono border-t-2 border-black pt-3">
                REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.
              </p>
            </div>
          </div>
        {/if}
      </div>

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
