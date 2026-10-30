<script lang="ts">
  import { regimeLabel } from '../../labels';
  import {
    api,
    type DailyScheduleAggregate,
    type DailyScheduleSummaryDto,
    type UpdateScheduleRulesPayload,
    type WorkRegime
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let {
    schedule,
    onRegimeChange,
    onDateChange
  } = $props<{
    schedule: DailyScheduleAggregate;
    onRegimeChange: (regime: WorkRegime) => void;
    onDateChange: (date: string) => void;
  }>();

  const regimes: { key: WorkRegime; label: string; limit: string; desc: string }[] = [
    { key: 'ORDINARY', label: 'Ordinario', limit: '13:45 h', desc: 'Jornada reglamentaria de 5 horas' },
    { key: 'PRE_FUNCTION', label: 'Pre-Función', limit: '14:45 h', desc: '+1 hora máxima autorizada' },
    { key: 'EXTRAORDINARY', label: 'Extraordinario', limit: '23:59 h', desc: 'Ensayos en escenario y funciones' },
  ];

  // Estado del Modal de Ajuste de Reglas
  let showRulesModal = $state(false);
  let mandatoryCheckin = $state('08:45');
  let nominalStart = $state('09:00');
  let nominalEnd = $state('13:45');
  let minimumRest = $state(12);
  let classToRehearsalRest = $state(15);
  let isCompensatoryRestDay = $state(false);
  let compensatoryRestDays = $state(1);
  let compensatoryRestPolicy = $state('NEXT_DAY');
  let legalClause = $state('★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★');
  let generalNotes = $state('');
  let isSavingRules = $state(false);

  // Sincronizar estado cuando cambia schedule
  $effect(() => {
    if (schedule) {
      mandatoryCheckin = schedule.mandatory_checkin_time || '08:45';
      nominalEnd = schedule.nominal_end_time || '13:45';
      minimumRest = schedule.minimum_intershift_rest_hours || 12;
      classToRehearsalRest = schedule.class_to_rehearsal_rest_minutes ?? 15;
      isCompensatoryRestDay = schedule.is_compensatory_rest_day ?? false;
      compensatoryRestDays = schedule.compensatory_rest_days || 1;
      compensatoryRestPolicy = schedule.compensatory_rest_policy || 'NEXT_DAY';
      legalClause = schedule.footer_legal_clause || '★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★';
    }
  });

  async function saveRules() {
    isSavingRules = true;
    try {
      const payload: UpdateScheduleRulesPayload = {
        schedule_id: schedule.id,
        mandatory_checkin_time: mandatoryCheckin,
        nominal_start_time: nominalStart,
        nominal_end_time: nominalEnd,
        minimum_intershift_rest_hours: Number(minimumRest),
        class_to_rehearsal_rest_minutes: Number(classToRehearsalRest),
        is_compensatory_rest_day: isCompensatoryRestDay,
        compensatory_rest_days: Number(compensatoryRestDays),
        compensatory_rest_policy: compensatoryRestPolicy,
        footer_legal_clause: legalClause,
        general_notes: generalNotes || null,
      };
      await api.updateScheduleRules(payload);
      showRulesModal = false;
      onDateChange(schedule.schedule_date);
    } catch (err: any) {
      console.error('Error al actualizar reglas:', err);
      alert(`No se pudieron guardar las reglas: ${err?.message || err}`);
    } finally {
      isSavingRules = false;
    }
  }

  // Estado del Modal de Historial de Partes
  let showHistoryModal = $state(false);
  let historySchedules = $state<DailyScheduleSummaryDto[]>([]);
  let isLoadingHistory = $state(false);
  let newDateInput = $state('');

  async function openHistory() {
    isLoadingHistory = true;
    showHistoryModal = true;
    try {
      historySchedules = await api.listDailySchedules();
    } catch (e) {
      console.error('Error al listar historial:', e);
    } finally {
      isLoadingHistory = false;
    }
  }

  function handleSelectHistoricalDate(dateStr: string) {
    showHistoryModal = false;
    onDateChange(dateStr);
  }

  function handleCreateNewDate() {
    if (!newDateInput) return;
    showHistoryModal = false;
    onDateChange(newDateInput);
  }
</script>

<div class="bg-panel border border-grid rounded p-2.5 shadow-md mb-3">
  <div class="flex flex-wrap items-end justify-between gap-2.5">
    
    <!-- Selector de Fecha & Parámetros -->
    <div class="flex items-end gap-2 flex-wrap">
      <div>
        <label for="parte-date-picker" class="block text-[10px] uppercase font-bold tracking-wider text-dim mb-1 font-mono">Fecha del Parte</label>
        <input
          id="parte-date-picker"
          type="date"
          value={schedule.schedule_date}
          onchange={(e) => onDateChange((e.target as HTMLInputElement).value)}
          class="bg-void border border-grid rounded px-2 text-xs font-bold text-ink outline-none focus:border-amber font-mono h-8"
        />
      </div>

      <!-- Fichaje / Marcada Obligatoria -->
      <div>
        <span class="block text-[10px] uppercase font-bold tracking-wider text-dim mb-1 font-mono">Marcada (L-V)</span>
        <div class="bg-void px-2.5 rounded border border-grid h-8 flex items-center">
          <span class="text-xs font-mono font-bold text-green">
            {schedule.mandatory_checkin_time} h
          </span>
        </div>
      </div>

      <!-- Descanso Inter-Jornada -->
      <div>
        <span class="block text-[10px] uppercase font-bold tracking-wider text-dim mb-1 font-mono">Descanso Regl.</span>
        <div class="bg-void px-2.5 rounded border border-grid h-8 flex items-center">
          <span class="text-xs font-mono font-bold text-cyan">
            {schedule.minimum_intershift_rest_hours}h Mínimo
          </span>
        </div>
      </div>

      <!-- Pausa Clase a Ensayos -->
      <div>
        <span class="block text-[10px] uppercase font-bold tracking-wider text-dim mb-1 font-mono">Pausa Clase</span>
        <div class="bg-void px-2.5 rounded border border-grid h-8 flex items-center" title="Pausa reglamentaria entre clase técnica y primer ensayo">
          <span class="text-xs font-mono font-bold text-amber">
            {schedule.class_to_rehearsal_rest_minutes ?? 15}m
          </span>
        </div>
      </div>

      <!-- Badge si es Franco Compensatorio -->
      {#if schedule.is_compensatory_rest_day}
        <div>
          <span class="block text-[10px] uppercase font-bold tracking-wider text-red mb-1 font-mono">Franco Especial</span>
          <div class="bg-void px-2.5 rounded border border-red/60 h-8 flex items-center gap-1.5" title="Franco Compensatorio acordado con Dirección">
            <span class="text-xs font-mono font-bold text-red">
              COMPENSATORIO
            </span>
            <span class="px-1.5 py-0.5 rounded bg-red/20 text-[10px] font-mono font-bold text-red">
              {schedule.compensatory_rest_days || 1} {(schedule.compensatory_rest_days || 1) === 1 ? 'DÍA' : 'DÍAS'}
            </span>
          </div>
        </div>
      {/if}

      <span class="w-px h-6 bg-grid self-end mb-1"></span>

      <!-- Botones de Acción -->
      <div>
        <span class="block text-[10px] uppercase font-bold tracking-wider text-dim mb-1 font-mono opacity-0 select-none">Acción</span>
        <div class="flex items-center gap-1.5 h-8">
          <button
            onclick={() => showRulesModal = true}
            class="h-8 px-2.5 bg-raised hover:bg-raised-hover text-ink hover:text-amber border border-grid rounded text-xs font-bold flex items-center gap-1 transition-colors"
            title="Editar marcadas, descansos reglamentarios y notas de inspección"
          >
            <span>Reglas</span>
          </button>

          <button
            onclick={openHistory}
            class="h-8 px-2.5 bg-raised hover:bg-raised-hover text-cyan border border-grid rounded text-xs font-bold flex items-center gap-1 transition-colors"
            title="Ver historial de partes diarios y cambiar de fecha"
          >
            <span>Historial</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Conmutador de Régimen Laboral -->
    <div>
      <div class="text-[10px] uppercase font-bold tracking-wider text-dim mb-1 font-mono text-left sm:text-right">Régimen Horario</div>
      <div class="inline-flex bg-void p-0.5 rounded border border-grid gap-0.5 h-8 items-center">
        {#each regimes as r}
          {@const isActive = schedule.regime === r.key}
          <button
            onclick={() => onRegimeChange(r.key)}
            class="h-7 px-2.5 rounded text-xs font-bold transition-colors flex items-center gap-1.5 {isActive ? 'bg-amber text-void' : 'text-dim hover:text-ink'}"
          >
            <span>{r.label}</span>
            <span class="text-[10px] font-mono opacity-80 px-1 rounded {isActive ? 'bg-black/20 text-void' : 'bg-raised text-dim'}">
              {r.limit}
            </span>
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<!-- ========================================================================= -->
<!-- MODAL DE AJUSTE DE REGLAS DE INSPECCIÓN                                    -->
<!-- ========================================================================= -->
{#if showRulesModal}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3" use:modalBehavior={{ onclose: () => showRulesModal = false, backdrop: false }}>
    <div class="tg-modal max-w-lg">
      <ModalHeader title="Reglas del Cronograma & Parámetros" subtitle="Marcadas horarias y descanso reglamentario" onclose={() => showRulesModal = false} />

      <div class="tg-modal-body space-y-3">
        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="mandatory-checkin-input" class="block text-[10px] font-bold uppercase text-dim mb-1 font-mono">Marcada Obligatoria</label>
            <input
              id="mandatory-checkin-input"
              type="text"
              bind:value={mandatoryCheckin}
              placeholder="08:45"
              class="w-full h-8 px-2.5 bg-void border border-grid rounded text-xs font-mono font-bold text-green text-center outline-none focus:border-amber"
            />
          </div>
          <div>
            <label for="minimum-rest-input" class="block text-[10px] font-bold uppercase text-dim mb-1 font-mono">Descanso Mínimo (Horas)</label>
            <input
              id="minimum-rest-input"
              type="number"
              bind:value={minimumRest}
              min="8"
              max="24"
              class="w-full h-8 px-2.5 bg-void border border-grid rounded text-xs font-mono font-bold text-cyan text-center outline-none focus:border-amber"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="nominal-start-input" class="block text-[10px] font-bold uppercase text-dim mb-1 font-mono">Inicio Nominal de Clase</label>
            <input
              id="nominal-start-input"
              type="text"
              bind:value={nominalStart}
              placeholder="09:00"
              class="w-full h-8 px-2.5 bg-void border border-grid rounded text-xs font-mono font-bold text-ink text-center outline-none focus:border-amber"
            />
          </div>
          <div>
            <label for="nominal-end-input" class="block text-[10px] font-bold uppercase text-dim mb-1 font-mono">Fin Nominal de Jornada</label>
            <input
              id="nominal-end-input"
              type="text"
              bind:value={nominalEnd}
              placeholder="13:45"
              class="w-full h-8 px-2.5 bg-void border border-grid rounded text-xs font-mono font-bold text-ink text-center outline-none focus:border-amber"
            />
          </div>
        </div>

        <!-- Reglas de Pausa e Inspección (Carina Lion) -->
        <div class="p-3 bg-alt border border-grid rounded space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-bold uppercase tracking-wider text-ink font-mono">
              Reglas de Inspección y Descansos
            </span>
            <span class="text-[10px] text-amber font-mono font-bold">Estatuto de Ballet</span>
          </div>

          <!-- Pausa Clase a Ensayos -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label for="class-rest-input" class="text-[10px] font-bold uppercase text-dim font-mono">Pausa Clase Técnica → Ensayos</label>
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  onclick={() => classToRehearsalRest = 15}
                  class="px-2 py-0.5 rounded text-[10px] font-mono font-bold border transition-colors {classToRehearsalRest === 15 ? 'bg-amber text-void border-amber' : 'bg-raised text-dim border-grid hover:text-ink'}"
                >
                  15m
                </button>
                <button
                  type="button"
                  onclick={() => classToRehearsalRest = 20}
                  class="px-2 py-0.5 rounded text-[10px] font-mono font-bold border transition-colors {classToRehearsalRest === 20 ? 'bg-amber text-void border-amber' : 'bg-raised text-dim border-grid hover:text-ink'}"
                >
                  20m
                </button>
                <button
                  type="button"
                  onclick={() => classToRehearsalRest = 30}
                  class="px-2 py-0.5 rounded text-[10px] font-mono font-bold border transition-colors {classToRehearsalRest === 30 ? 'bg-amber text-void border-amber' : 'bg-raised text-dim border-grid hover:text-ink'}"
                >
                  30m
                </button>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <input
                id="class-rest-input"
                type="number"
                bind:value={classToRehearsalRest}
                min="0"
                max="60"
                class="w-24 h-8 px-2.5 bg-void border border-grid rounded text-xs font-mono font-bold text-amber text-center outline-none focus:border-amber"
              />
              <span class="text-xs text-dim font-mono">minutos reglamentarios</span>
            </div>
            <p class="text-[10px] text-dim mt-1">
              Tiempo obligatorio de hidratación, reposo físico y cambio de calzado (puntas/medias) antes de entrar a ensayos.
            </p>
          </div>

          <!-- Franco Compensatorio -->
          <div class="pt-2 border-t border-grid/60">
            <label class="flex items-start gap-2.5 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={isCompensatoryRestDay}
                class="mt-0.5 accent-red rounded"
              />
              <div>
                <span class="block text-xs font-bold text-red">
                  Declarar jornada como Franco Compensatorio tras Función
                </span>
                <span class="text-[10px] text-dim block mt-0.5">
                  El motor de auditoría alertará si se citan ensayos obligatorios a los bailarines en este día.
                </span>
              </div>
            </label>

            {#if isCompensatoryRestDay}
              <div class="mt-2.5 pl-6 space-y-3">
                <!-- Selector de cantidad de días acordados con Dirección -->
                <div>
                  <div class="flex items-center justify-between mb-1">
                    <label for="comp-days-input" class="text-[10px] font-bold uppercase text-red font-mono">
                      Días de Franco Acordados con Dirección
                    </label>
                    <div class="flex items-center gap-1">
                      {#each [1, 2, 3, 4] as d}
                        <button
                          type="button"
                          onclick={() => compensatoryRestDays = d}
                          class="px-2 py-0.5 rounded text-[10px] font-mono font-bold border transition-colors {compensatoryRestDays === d ? 'bg-red text-white border-red' : 'bg-raised text-dim border-grid hover:text-ink'}"
                        >
                          {d} {d === 1 ? 'día' : 'días'}
                        </button>
                      {/each}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <input
                      id="comp-days-input"
                      type="number"
                      bind:value={compensatoryRestDays}
                      min="1"
                      max="14"
                      class="w-20 h-8 px-2.5 bg-void border border-red/50 rounded text-xs font-mono font-bold text-red text-center outline-none focus:border-amber"
                    />
                    <span class="text-xs text-dim font-mono">días continuos de compensación acordados</span>
                  </div>
                </div>

                <div class="space-y-1.5">
                  <label for="comp-policy-select" class="block text-[10px] font-bold uppercase text-dim font-mono">
                    Modalidad de Compensación
                  </label>
                  <select
                    id="comp-policy-select"
                    bind:value={compensatoryRestPolicy}
                    class="w-full h-8 px-2.5 bg-void border border-red/40 rounded text-xs font-mono font-bold text-ink outline-none focus:border-amber"
                  >
                    <option value="NEXT_DAY">Días Hábiles Posteriores Inmediatos (Descanso continuo)</option>
                    <option value="EXTENDED_48H">Franco Extendido Fin de Semana (48 horas continuas o más)</option>
                    <option value="ACCRUED">Acumulación de Crédito Compensatorio de Licencia</option>
                  </select>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <div>
          <label for="legal-clause-input" class="block text-[10px] font-bold uppercase text-dim mb-1 font-mono">Cláusula Legal en Cartelera / Parte</label>
          <input
            id="legal-clause-input"
            type="text"
            bind:value={legalClause}
            class="w-full h-8 px-2.5 bg-void border border-grid rounded text-xs font-mono text-amber outline-none focus:border-amber"
          />
        </div>

        <div>
          <label for="general-notes-input" class="block text-[10px] font-bold uppercase text-dim mb-1 font-mono">Notas Generales / Inspección del Día</label>
          <textarea
            id="general-notes-input"
            bind:value={generalNotes}
            rows="2"
            placeholder="Observaciones de piso, camarines, acondicionamiento de sala..."
            class="w-full p-2 bg-void border border-grid rounded text-xs text-ink resize-none outline-none focus:border-amber"
          ></textarea>
        </div>
      </div>

      <div class="tg-modal-foot">
        <button
          onclick={() => showRulesModal = false}
          class="tg-btn"
        >
          Cancelar
        </button>
        <button
          disabled={isSavingRules}
          onclick={saveRules}
          class="tg-btn tg-btn-primary disabled:opacity-50"
        >
          {isSavingRules ? 'Guardando...' : 'Aplicar Reglas'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ========================================================================= -->
<!-- MODAL DE HISTORIAL DE PARTES DIARIOS CREADOS                              -->
<!-- ========================================================================= -->
{#if showHistoryModal}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3" use:modalBehavior={{ onclose: () => showHistoryModal = false }}>
    <div class="tg-modal max-w-2xl">
      <ModalHeader title="Historial de Partes Diarios" subtitle="Partes registrados en la base de datos local" onclose={() => showHistoryModal = false} />

      <!-- Barra para iniciar nuevo parte en fecha específica -->
      <div class="p-3 bg-alt border-b border-grid flex items-center justify-between gap-3">
        <div class="flex items-center gap-2 flex-1">
          <label for="history-new-date" class="text-xs text-dim font-bold font-mono">Abrir fecha:</label>
          <input
            id="history-new-date"
            type="date"
            bind:value={newDateInput}
            class="bg-void border border-grid rounded px-2.5 py-1 text-xs font-mono font-bold text-ink outline-none focus:border-amber h-8"
          />
        </div>
        <button
          onclick={handleCreateNewDate}
          disabled={!newDateInput}
          class="h-8 px-3 bg-amber hover:bg-amber disabled:opacity-50 text-void font-bold text-xs rounded transition-colors"
        >
          Cargar Fecha
        </button>
      </div>

      <!-- Lista del Historial -->
      <div class="flex-1 overflow-y-auto p-3 space-y-1.5">
        {#if isLoadingHistory}
          <div class="py-12 text-center text-xs text-dim font-mono">
            Cargando historial de partes...
          </div>
        {:else if historySchedules.length === 0}
          <div class="py-12 text-center text-xs text-dim font-mono">
            No hay partes registrados en el sistema.
          </div>
        {:else}
          {#each historySchedules as s}
            {@const isCurrent = s.schedule_date === schedule.schedule_date}
            <div class="p-2.5 rounded border transition-colors flex items-center justify-between gap-3 {isCurrent ? 'bg-raised border-amber' : 'bg-alt border-grid hover:border-dim'}">
              <div>
                <div class="flex items-center gap-2">
                  <span class="font-mono font-bold text-xs text-ink">{s.schedule_date}</span>
                  <span class="text-[9px] font-bold px-1.5 py-0.5 rounded font-mono uppercase {s.regime === 'ORDINARY' ? 'bg-green/20 text-green border border-green/40' : s.regime === 'PRE_FUNCTION' ? 'bg-amber/20 text-amber border border-amber/40' : 'bg-cyan/20 text-cyan border border-cyan/40'}">
                    {regimeLabel(s.regime)}
                  </span>
                  {#if isCurrent}
                    <span class="text-[9px] bg-amber text-void font-bold px-1.5 py-0.2 rounded uppercase">
                      Activo
                    </span>
                  {/if}
                </div>
                <div class="text-[11px] text-dim mt-1 flex items-center gap-3 font-mono">
                  <span>Marcada: <strong class="text-ink">{s.mandatory_checkin_time} h</strong></span>
                  <span>•</span>
                  <span>Ensayos: <strong class="text-amber">{s.rehearsal_blocks_count}</strong></span>
                  <span>•</span>
                  <span>Clases: <strong class="text-cyan">{s.morning_classes_count}</strong></span>
                </div>
              </div>

              <div>
                <button
                  onclick={() => handleSelectHistoricalDate(s.schedule_date)}
                  class="h-7 px-2.5 rounded text-xs font-bold transition-colors {isCurrent ? 'bg-raised text-dim border border-grid cursor-default' : 'bg-amber hover:bg-amber text-void'}"
                >
                  {isCurrent ? 'Seleccionado' : 'Cargar Día'}
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
