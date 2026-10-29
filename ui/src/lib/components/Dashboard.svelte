<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type DailyScheduleAggregate,
    type ScheduleConflict,
    type AttendanceLogDto,
    type LicenseInfo,
  } from '../ipc/client';

  let {
    schedule,
    conflicts,
    currentDate,
    licenseInfo,
    onGoAttendance,
    onGoSchedule,
    onOpenBop,
    onOpenParts,
    onOpenInventory,
    onOpenLicense,
  } = $props<{
    schedule: DailyScheduleAggregate;
    conflicts: ScheduleConflict[];
    currentDate: string;
    licenseInfo: LicenseInfo | null;
    onGoAttendance: () => void;
    onGoSchedule: () => void;
    onOpenBop: () => void;
    onOpenParts: () => void;
    onOpenInventory: () => void;
    onOpenLicense: () => void;
  }>();

  const IJ_MONTH_MAX = 2;

  let attendance = $state<AttendanceLogDto[]>([]);
  let ijAtLimit = $state<{ name: string; count: number }[]>([]);
  let lowStock = $state(0);
  let physioToday = $state(0);
  let loaded = $state(false);

  async function load(date: string) {
    const [y, m] = date.split('-').map(Number);
    const [att, matrix, stock, physio] = await Promise.allSettled([
      api.listAttendanceForDate(date),
      api.getMonthlyBopMatrix(y, m),
      api.listStockCatalog(),
      api.listPhysiotherapySessions(date, null),
    ]);
    attendance = att.status === 'fulfilled' ? att.value : [];
    ijAtLimit =
      matrix.status === 'fulfilled'
        ? matrix.value.rows
            .map((r) => ({ name: r.dancer_name, count: r.cells.filter((c) => c.code === 'IJ').length }))
            .filter((r) => r.count >= IJ_MONTH_MAX)
        : [];
    lowStock = stock.status === 'fulfilled' ? stock.value.filter((s) => s.is_low_stock).length : 0;
    physioToday = physio.status === 'fulfilled' ? physio.value.length : 0;
    loaded = true;
  }

  onMount(() => { load(currentDate); });
  $effect(() => { load(currentDate); });

  const count = (...st: string[]) => attendance.filter((a) => st.includes(a.attendance_status)).length;
  let total = $derived(attendance.length);
  let present = $derived(count('PRESENT', 'NOT_IN_CLASS', 'IN_PHYSIOTHERAPY', 'TASK_ADJUSTMENT'));
  let late = $derived(attendance.filter((a) => a.attendance_status === 'LATE'));
  let medical = $derived(attendance.filter((a) => a.attendance_status === 'MEDICAL_LEAVE'));
  let absent = $derived(attendance.filter((a) => a.attendance_status === 'ABSENT'));
  let pending = $derived(count('PENDING'));
  let noPart = $derived(count('NO_PART', 'RETIRED_EXEMPT'));
  let expected = $derived(total - noPart);
  let arrived = $derived(present + late.length);
  let pct = $derived(expected > 0 ? Math.round((arrived / expected) * 100) : 0);

  const todayIso = (() => {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  })();
  let isToday = $derived(currentDate === todayIso);
  const nowHM = () => { const d = new Date(); return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`; };

  const ROOMS: Record<string, string> = {
    SALA_1: 'Sala 1', SALA_2: 'Sala 2', SALA_3: 'Sala 3', ESCENARIO: 'Escenario',
    GIRA_TRASLADO: 'Traslado', GIRA_ESCENARIO: 'Gira · Escenario', GIRA_SALA: 'Gira · Sala', SALA_AUX: 'Sala aux.',
  };
  const REGIMES: Record<string, string> = { ORDINARY: 'Ordinario', PRE_FUNCTION: 'Pre-función', EXTRAORDINARY: 'Extraordinario' };

  type Item = { start: string; end: string; title: string; detail: string; kind: 'clase' | 'ensayo' };
  let agenda = $derived<Item[]>(
    [
      ...schedule.morning_classes.map((c: any) => ({
        start: c.start_time, end: c.end_time, kind: 'clase' as const,
        title: `Clase · ${ROOMS[c.room] ?? c.room}`,
        detail: [c.master_name, c.pianist_name, c.sound_tech_name ? 'Sonido: ' + c.sound_tech_name : ''].filter(Boolean).join(' · '),
      })),
      ...schedule.rehearsal_blocks.map((b: any) => ({
        start: b.start_time, end: b.end_time, kind: 'ensayo' as const,
        title: `${b.work_title ?? b.passage_name} · ${ROOMS[b.room] ?? b.room}`,
        detail: [b.work_title ? b.passage_name : b.master_name, b.sound_tech_name ? 'Sonido: ' + b.sound_tech_name : ''].filter(Boolean).join(' · '),
      })),
    ].sort((a, b) => a.start.localeCompare(b.start)),
  );
  let nowItem = $derived(isToday ? agenda.find((i) => i.start <= nowHM() && nowHM() < i.end) : undefined);
  let nextItem = $derived(isToday ? agenda.find((i) => i.start > nowHM()) : undefined);

  let alerts = $derived.by(() => {
    const out: { tone: 'red' | 'amber' | 'cyan'; text: string; action?: () => void; label?: string }[] = [];
    if (licenseInfo?.warning) out.push({ tone: 'amber', text: licenseInfo.warning, action: onOpenLicense, label: 'Ver licencia' });
    if (conflicts.length) out.push({ tone: 'red', text: `${conflicts.length} conflicto${conflicts.length > 1 ? 's' : ''} en el cronograma de hoy`, action: onGoSchedule, label: 'Revisar' });
    if (pending > 0 && isToday) out.push({ tone: 'amber', text: `${pending} sin marcar en la asistencia de hoy`, action: onGoAttendance, label: 'Marcar' });
    if (ijAtLimit.length) {
      const names = ijAtLimit.slice(0, 3).map((r) => short(r.name)).join(', ');
      const more = ijAtLimit.length > 3 ? ` y ${ijAtLimit.length - 3} más` : '';
      out.push({ tone: 'cyan', text: `Tope de IJ del mes alcanzado (${IJ_MONTH_MAX}/${IJ_MONTH_MAX}): ${names}${more}`, action: onOpenBop, label: 'Ver planilla' });
    }
    if (lowStock > 0) out.push({ tone: 'amber', text: `${lowStock} artículo${lowStock > 1 ? 's' : ''} con stock bajo`, action: onOpenInventory, label: 'Ver stock' });
    return out;
  });

  const tone = { red: 'border-red]/50 bg-void] text-red]', amber: 'border-amber]/40 bg-raised] text-amber]', cyan: 'border-cyan]/40 bg-void] text-cyan]' };
  const longDate = $derived(new Date(currentDate + 'T12:00:00').toLocaleDateString('es-AR', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }));
  const short = (n: string) => n.split(',')[0];
</script>

<div class="space-y-3 md:space-y-4">
  <!-- Encabezado del día -->
  <section class="bg-panel] border border-grid] rounded p-4">
    <p class="text-xs font-mono uppercase text-dim]">{isToday ? 'Hoy' : 'Día seleccionado'}</p>
    <h2 class="text-lg md:text-xl font-bold text-ink] leading-tight first-letter:uppercase">{longDate}</h2>
    <div class="mt-2 flex flex-wrap gap-2 text-xs font-mono">
      <span class="px-2 py-1 rounded bg-raised] border border-grid] text-amber]">{REGIMES[schedule.regime] ?? schedule.regime}</span>
      <span class="px-2 py-1 rounded bg-raised] border border-grid] text-green]">Fichaje {schedule.mandatory_checkin_time}</span>
      <span class="px-2 py-1 rounded bg-raised] border border-grid] text-ink]">Fin {schedule.nominal_end_time}</span>
      {#if schedule.is_compensatory_rest_day}
        <span class="px-2 py-1 rounded bg-raised] border border-amber]/40 text-amber]">Día de descanso compensatorio</span>
      {/if}
    </div>
  </section>

  <!-- Atención: solo si hay algo -->
  {#if alerts.length}
    <section class="space-y-2" aria-label="Atención">
      {#each alerts as a}
        <div class="flex items-center justify-between gap-3 border rounded px-3 py-2.5 {tone[a.tone as 'red' | 'amber' | 'cyan']}">
          <span class="text-sm leading-snug">{a.text}</span>
          {#if a.action}
            <button onclick={a.action} class="shrink-0 px-3 py-2 rounded bg-black/30 border border-current text-xs font-bold">{a.label}</button>
          {/if}
        </div>
      {/each}
    </section>
  {/if}

  <div class="grid gap-3 md:gap-4 lg:grid-cols-2">
    <!-- Asistencia -->
    <section class="bg-panel] border border-grid] rounded p-4 space-y-3">
      <div class="flex items-baseline justify-between">
        <h3 class="text-sm font-bold uppercase tracking-wide text-ink]">Asistencia</h3>
        <button onclick={onGoAttendance} class="text-xs font-bold text-amber] py-1">Abrir planilla →</button>
      </div>
      {#if !loaded}
        <p class="text-sm text-dim]">Cargando…</p>
      {:else if total === 0}
        <p class="text-sm text-dim]">Todavía no hay asistencia cargada para este día.</p>
      {:else}
        <div>
          <div class="flex items-end justify-between">
            <div class="text-3xl font-bold font-mono text-green]">{arrived}<span class="text-lg text-dim]"> / {expected}</span></div>
            <div class="text-sm text-dim]">{pct}% presentes</div>
          </div>
          <div class="mt-2 h-2.5 rounded bg-void] overflow-hidden flex">
            <div class="bg-green]" style="width:{expected ? (present / expected) * 100 : 0}%"></div>
            <div class="bg-amber]" style="width:{expected ? (late.length / expected) * 100 : 0}%"></div>
          </div>
        </div>
        <div class="grid grid-cols-4 gap-2 text-center">
          <div class="rounded bg-raised] py-2"><div class="text-lg font-bold font-mono text-amber]">{late.length}</div><div class="text-[11px] text-dim]">Tarde</div></div>
          <div class="rounded bg-raised] py-2"><div class="text-lg font-bold font-mono text-violet]">{medical.length}</div><div class="text-[11px] text-dim]">Licencia</div></div>
          <div class="rounded bg-raised] py-2"><div class="text-lg font-bold font-mono text-red]">{absent.length}</div><div class="text-[11px] text-dim]">No está</div></div>
          <div class="rounded bg-raised] py-2"><div class="text-lg font-bold font-mono text-cyan]">{physioToday}</div><div class="text-[11px] text-dim]">Fisio</div></div>
        </div>
        {#if late.length || medical.length || absent.length}
          <ul class="text-sm space-y-1">
            {#each absent as a}<li><span class="text-red] font-bold">No está</span> · {a.dancer_name}</li>{/each}
            {#each medical as a}<li><span class="text-violet] font-bold">Licencia</span> · {a.dancer_name}{a.leave_end_date ? ` (hasta ${a.leave_end_date})` : ''}</li>{/each}
            {#each late as a}<li><span class="text-amber] font-bold">Tarde</span> · {a.dancer_name}{a.minutes_late ? ` (${a.minutes_late} min)` : ''}</li>{/each}
          </ul>
        {/if}
      {/if}
    </section>

    <!-- Agenda -->
    <section class="bg-panel] border border-grid] rounded p-4 space-y-3">
      <div class="flex items-baseline justify-between">
        <h3 class="text-sm font-bold uppercase tracking-wide text-ink]">Agenda del día</h3>
        <button onclick={onGoSchedule} class="text-xs font-bold text-amber] py-1">Ver parte →</button>
      </div>
      {#if agenda.length === 0}
        <p class="text-sm text-dim]">No hay clases ni ensayos cargados para este día.</p>
      {:else}
        <ol class="space-y-2">
          {#each agenda as it}
            {@const isNow = it === nowItem}
            {@const isNext = it === nextItem}
            <li class="flex gap-3 rounded border px-3 py-2 {isNow ? 'border-green] bg-void]' : isNext ? 'border-amber]/50 bg-raised]' : 'border-grid] bg-void]'}">
              <div class="font-mono text-sm shrink-0 w-[3.2rem] {it.kind === 'clase' ? 'text-cyan]' : 'text-amber]'}">
                <div class="font-bold">{it.start}</div><div class="text-dim] text-xs">{it.end}</div>
              </div>
              <div class="min-w-0">
                <div class="text-sm font-semibold text-ink] leading-snug">{it.title}
                  {#if isNow}<span class="ml-1 text-[10px] font-mono font-bold text-green]">AHORA</span>{/if}
                  {#if isNext}<span class="ml-1 text-[10px] font-mono font-bold text-amber]">PRÓXIMO</span>{/if}
                </div>
                {#if it.detail}<div class="text-xs text-dim] leading-snug">{it.detail}</div>{/if}
              </div>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  </div>

  <!-- Accesos -->
  <section class="grid grid-cols-2 md:grid-cols-4 gap-2">
    <button onclick={onGoAttendance} class="min-h-14 px-3 py-3 rounded bg-raised] border border-grid] text-sm font-bold text-green] text-left">Tomar asistencia</button>
    <button onclick={onOpenBop} class="min-h-14 px-3 py-3 rounded bg-raised] border border-cyan]/40 text-sm font-bold text-cyan] text-left">Planilla BOP</button>
    <button onclick={onOpenParts} class="min-h-14 px-3 py-3 rounded bg-raised] border border-amber]/40 text-sm font-bold text-amber] text-left">Parte oficial</button>
    <button onclick={onOpenInventory} class="min-h-14 px-3 py-3 rounded bg-raised] border border-grid] text-sm font-bold text-ink] text-left">Inventario</button>
  </section>
</div>
