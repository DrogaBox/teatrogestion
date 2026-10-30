<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type AttendanceLogDto,
    type AttendanceStatus,
    type ActivityRestriction,
    type CreatePhysioSessionPayload,
    type LeaveType,
    type RecordAttendancePayload
  } from '../../ipc/client';
  import AgentMonthlyAttendanceModal from './AgentMonthlyAttendanceModal.svelte';
  import BopAttendanceMatrixModal from './BopAttendanceMatrixModal.svelte';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import PrintPreviewModal from '../shared/PrintPreviewModal.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let { scheduleDate, onDateChange } = $props<{ scheduleDate: string; onDateChange?: (date: string) => void }>();

  let printPreviewOpen = $state(false);
  let printableEl: HTMLElement | null = $state(null);

  let activeDate = $state('');
  let logs = $state<AttendanceLogDto[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state('');

  // Modales
  let medicalModalDancer = $state<AttendanceLogDto | null>(null);
  let physioModalDancer = $state<AttendanceLogDto | null>(null);
  let lateModalDancer = $state<AttendanceLogDto | null>(null);
  let rectifyModalDancer = $state<AttendanceLogDto | null>(null);
  let showMonthlyModal = $state(false);
  let showBopMatrixModal = $state(false);
  let monthlyInitialPersonnelId = $state<string>('');

  // Formulario Rectificación Integral
  let rectifyStatus = $state<AttendanceStatus>('PRESENT');
  let rectifyCheckinTime = $state('08:45');
  let rectifyMinutesLate = $state(0);
  let rectifyRemarks = $state('');
  let rectifyDiagnosis = $state('');

  // Formulario Carpeta Médica
  let medDiagnosis = $state('');
  let medDoctor = $state('');
  let medLeaveType = $state<LeaveType>('ENFERMEDAD');
  let medLeaveStartDate = $state('');
  let medLeaveEndDate = $state('');
  let medRemarks = $state('');
  let medFileName = $state('');
  let medFileData = $state('');

  // Visor de Zoom de Certificado
  let zoomCertificateModal = $state(false);
  let zoomCertificateUrl = $state<string | null>(null);
  let zoomCertificateTitle = $state('');

  // Formulario Fisioterapia
  let physDiagnosis = $state('');
  let physTreatment = $state('Masoterapia y kinesiología');
  let physEvolution = $state('');
  let physRestriction = $state<ActivityRestriction>('NORMAL');
  let physDoctorName = $state('Dra. de Fisioterapia');
  let physAttachmentName = $state('');
  let physAttachmentData = $state('');

  // Formulario Llegada Tarde
  let lateMinutes = $state(15);
  let lateCheckinTime = $state('09:00');
  let lateRemarks = $state('');

  async function loadAttendance() {
    isLoading = true;
    try {
      logs = await api.listAttendanceForDate(activeDate);
    } catch (e) {
      console.error('Error cargando asistencia:', e);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (scheduleDate && scheduleDate !== activeDate) {
      activeDate = scheduleDate;
    }
  });

  $effect(() => {
    if (activeDate) {
      loadAttendance();
    }
  });

  function selectDate(newDate: string) {
    if (!newDate) return;
    activeDate = newDate;
    onDateChange?.(newDate);
  }

  function shiftDay(offset: number) {
    try {
      const [y, m, d] = activeDate.split('-').map(Number);
      const date = new Date(y, m - 1, d + offset);
      const yStr = date.getFullYear();
      const mStr = String(date.getMonth() + 1).padStart(2, '0');
      const dStr = String(date.getDate()).padStart(2, '0');
      selectDate(`${yStr}-${mStr}-${dStr}`);
    } catch (e) {
      console.error(e);
    }
  }

  function goToToday() {
    const d = new Date();
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    selectDate(`${y}-${m}-${day}`);
  }

  async function updateStatus(
    dancer: AttendanceLogDto,
    status: AttendanceStatus,
    extra?: Partial<RecordAttendancePayload>
  ) {
    try {
      const payload: RecordAttendancePayload = {
        schedule_date: activeDate,
        personnel_id: dancer.personnel_id,
        attendance_status: status,
        actual_checkin_time: extra?.actual_checkin_time,
        minutes_late: extra?.minutes_late,
        remarks: extra?.remarks,
        certificate_file_name: extra?.certificate_file_name,
        certificate_data: extra?.certificate_data,
        medical_diagnosis: extra?.medical_diagnosis,
        doctor_details: extra?.doctor_details,
        leave_start_date: extra?.leave_start_date,
        leave_end_date: extra?.leave_end_date,
        leave_type: extra?.leave_type,
        physio_notes: extra?.physio_notes,
        update_permanent_physio_tracking: extra?.update_permanent_physio_tracking,
      };
      const updated = await api.recordAttendance(payload);
      logs = logs.map((l) => (l.personnel_id === updated.personnel_id ? updated : l));
    } catch (err: any) {
      alert(`Error al actualizar estado: ${err?.message || err}`);
    }
  }

  function openRectify(dancer: AttendanceLogDto) {
    rectifyModalDancer = dancer;
    rectifyStatus = dancer.attendance_status;
    rectifyCheckinTime = dancer.actual_checkin_time || '08:45';
    rectifyMinutesLate = dancer.minutes_late || 0;
    rectifyRemarks = dancer.remarks || '';
    rectifyDiagnosis = dancer.medical_diagnosis || dancer.physio_notes || '';
  }

  async function submitRectification() {
    if (!rectifyModalDancer) return;
    try {
      const payload: RecordAttendancePayload = {
        schedule_date: activeDate,
        personnel_id: rectifyModalDancer.personnel_id,
        attendance_status: rectifyStatus,
        actual_checkin_time: rectifyCheckinTime,
        minutes_late: rectifyMinutesLate,
        remarks: rectifyRemarks,
        medical_diagnosis: rectifyDiagnosis || undefined,
      };
      const updated = await api.recordAttendance(payload);
      logs = logs.map((l) => (l.personnel_id === updated.personnel_id ? updated : l));
      rectifyModalDancer = null;
    } catch (err: any) {
      alert(`Error al rectificar fichada: ${err?.message || err}`);
    }
  }

  function openMonthlyForDancer(dancerId: string) {
    monthlyInitialPersonnelId = dancerId;
    showMonthlyModal = true;
  }

  function handleFileUpload(e: Event, type: 'medical' | 'physio') {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      const file = input.files[0];
      const reader = new FileReader();
      reader.onload = (uploadEvent) => {
        const base64 = uploadEvent.target?.result as string;
        if (type === 'medical') {
          medFileName = file.name;
          medFileData = base64;
        } else {
          physAttachmentName = file.name;
          physAttachmentData = base64;
        }
      };
      reader.readAsDataURL(file);
    }
  }

  async function submitMedicalLeave() {
    if (!medicalModalDancer) return;
    await updateStatus(medicalModalDancer, 'MEDICAL_LEAVE', {
      medical_diagnosis: medDiagnosis,
      doctor_details: medDoctor,
      leave_start_date: medLeaveStartDate || scheduleDate,
      leave_end_date: medLeaveEndDate || scheduleDate,
      leave_type: medLeaveType,
      remarks: medRemarks,
      certificate_file_name: medFileName || undefined,
      certificate_data: medFileData || undefined,
    });
    medicalModalDancer = null;
  }

  const LEAVE_TYPE_LABELS: Record<string, string> = {
    ENFERMEDAD: 'Licencia por enfermedad',
    ESTUDIO: 'Licencia por estudio',
    SIN_GOCE_DE_HABERES: 'Licencia sin goce de haberes',
    OTRAS: 'Otras licencias'
  };

  function leaveTypeLabel(dancer: AttendanceLogDto): string {
    if (!dancer.leave_type) return 'Licencia médica';
    return LEAVE_TYPE_LABELS[dancer.leave_type] ?? 'Licencia médica';
  }

  function viewCertificate(dancer: AttendanceLogDto) {
    zoomCertificateUrl = dancer.certificate_data || null;
    zoomCertificateTitle = `Certificado Médico - ${dancer.dancer_name} (${dancer.certificate_file_name || 'Archivo'})`;
    zoomCertificateModal = true;
  }

  async function submitPhysioSession() {
    if (!physioModalDancer) return;
    try {
      const payload: CreatePhysioSessionPayload = {
        personnel_id: physioModalDancer.personnel_id,
        session_date: scheduleDate,
        session_time: '09:15',
        doctor_name: physDoctorName,
        diagnosis: physDiagnosis || undefined,
        treatment_applied: physTreatment,
        evolution_notes: physEvolution || undefined,
        activity_restriction: physRestriction,
        attachment_name: physAttachmentName || undefined,
        attachment_data: physAttachmentData || undefined,
        sync_attendance_status: true,
      };
      await api.createPhysiotherapySession(payload);
      await loadAttendance();
      physioModalDancer = null;
    } catch (err: any) {
      alert(`Error guardando reporte de fisioterapia: ${err?.message || err}`);
    }
  }

  async function submitLate() {
    if (!lateModalDancer) return;
    await updateStatus(lateModalDancer, 'LATE', {
      actual_checkin_time: lateCheckinTime,
      minutes_late: lateMinutes,
      remarks: lateRemarks,
    });
    lateModalDancer = null;
  }

  // Filtrado de bailarines
  const filteredLogs = $derived(
    logs.filter((l) =>
      l.dancer_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (l.legajo && l.legajo.toLowerCase().includes(searchQuery.toLowerCase()))
    )
  );

  // Estadísticas rápidas
  const countPresent = $derived(logs.filter((l) => l.attendance_status === 'PRESENT').length);
  const countNotInClass = $derived(logs.filter((l) => l.attendance_status === 'NOT_IN_CLASS').length);
  const countPhysio = $derived(logs.filter((l) => l.attendance_status === 'IN_PHYSIOTHERAPY').length);
  const countLate = $derived(logs.filter((l) => l.attendance_status === 'LATE').length);
  const countMedical = $derived(logs.filter((l) => l.attendance_status === 'MEDICAL_LEAVE').length);
  const countAbsent = $derived(logs.filter((l) => l.attendance_status === 'ABSENT').length);
  const countNoPart = $derived(logs.filter((l) => l.attendance_status === 'NO_PART').length);
  const countTask = $derived(logs.filter((l) => l.attendance_status === 'TASK_ADJUSTMENT').length);
  const countJustified = $derived(logs.filter((l) => l.attendance_status === 'JUSTIFIED_ABSENCE').length);
  const countPending = $derived(logs.filter((l) => l.attendance_status === 'PENDING').length);
  const countRetired = $derived(logs.filter((l) => l.is_retired).length);

  // Exportar Excel (CSV con BOM)
  function exportExcel() {
    let csv = '\uFEFF'; // BOM UTF-8 para Excel
    csv += 'FECHA;DNI;NOMBRE Y APELLIDO;ESTADO ASISTENCIA;CLASE MATUTINA;FICHADA;DEMORA (MIN);DIAGNOSTICO / NOTAS FISIO;CERTIFICADO;MEDICO\n';

    logs.forEach((l) => {
      let stLabel = l.attendance_status;
      if (l.attendance_status === 'PRESENT') stLabel = 'Está (Presente)';
      else if (l.attendance_status === 'NOT_IN_CLASS') stLabel = 'No está en clase';
      else if (l.attendance_status === 'IN_PHYSIOTHERAPY') stLabel = 'En fisioterapia';
      else if (l.attendance_status === 'LATE') stLabel = 'Llegó tarde';
      else if (l.attendance_status === 'MEDICAL_LEAVE') stLabel = leaveTypeLabel(l);
      else if (l.attendance_status === 'ABSENT') stLabel = 'No está';
      else if (l.attendance_status === 'NO_PART') stLabel = 'Sin parte';
      else if (l.attendance_status === 'JUSTIFIED_ABSENCE') stLabel = 'Inasistencia justificada (IJ)';
      else if (l.attendance_status === 'TASK_ADJUSTMENT') stLabel = 'Adecuación de tareas';
      else if (l.attendance_status === 'PENDING') stLabel = '?';

      const inClass = l.is_in_class ? 'Sí' : 'No';
      const checkin = l.actual_checkin_time || '08:45';
      const diag = (l.medical_diagnosis || l.physio_notes || l.current_physio_diagnosis || '').replace(/;/g, ',');
      const cert = l.certificate_file_name ? 'Adjunto' : 'Sin adjunto';
      const doc = (l.doctor_details || '').replace(/;/g, ',');

      csv += `${l.schedule_date};${l.legajo || ''};"${l.dancer_name}";${stLabel};${inClass};${checkin};${l.minutes_late};"${diag}";${cert};"${doc}"\n`;
    });

    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', `Asistencia_Ballet_${scheduleDate}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  function formatFullDate(isoDate: string): string {
    if (!isoDate) return '';
    try {
      const [y, m, d] = isoDate.split('-').map(Number);
      const date = new Date(y, m - 1, d);
      const days = ['DOMINGO', 'LUNES', 'MARTES', 'MIÉRCOLES', 'JUEVES', 'VIERNES', 'SÁBADO'];
      const months = [
        'ENERO', 'FEBRERO', 'MARZO', 'ABRIL', 'MAYO', 'JUNIO',
        'JULIO', 'AGOSTO', 'SEPTIEMBRE', 'OCTUBRE', 'NOVIEMBRE', 'DICIEMBRE'
      ];
      const dayName = days[date.getDay()];
      const monthName = months[m - 1];
      return `${dayName} ${d} DE ${monthName} DE ${y}`;
    } catch {
      return isoDate;
    }
  }

  const effectivePresenceCount = $derived(countPresent + countNotInClass);
  const operationalPercent = $derived(
    logs.length > 0 ? ((effectivePresenceCount / logs.length) * 100).toFixed(1) : '100.0'
  );
</script>

<div bind:this={printableEl} class="bg-panel border border-grid rounded-md p-4 mb-6">
  <!-- Banner Formal del Día y Detalles Técnicos de Inspección -->
  <div class="bg-void border border-grid rounded p-4 mb-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <div class="flex items-center gap-2">
          <h1 class="text-base md:text-lg font-bold text-amber tracking-wide uppercase font-mono">
            {formatFullDate(activeDate)}
          </h1>
        </div>
        <p class="text-xs text-dim mt-1 flex items-center gap-2 font-mono">
          <span>Ballet Oficial de Córdoba</span>
          <span>•</span>
          <span>Teatro del Libertador San Martín</span>
        </p>

        <!-- Selector Interactivo de Fecha (Consultar Ausentismo y TC de cualquier día) -->
        <div class="flex items-center gap-1.5 mt-2 flex-wrap">
          <button
            onclick={() => shiftDay(-1)}
            class="px-2.5 py-1 bg-raised hover:bg-grid text-dim hover:text-ink rounded text-xs font-medium transition-colors border border-grid"
            title="Día Anterior"
          >
            ◀ Anterior
          </button>
          <input
            type="date"
            value={activeDate}
            onchange={(e) => selectDate(e.currentTarget.value)}
            class="h-8 px-2 bg-void border border-grid rounded text-xs font-semibold text-amber font-mono outline-none focus:border-amber cursor-pointer"
          />
          <button
            onclick={() => shiftDay(1)}
            class="px-2.5 py-1 bg-raised hover:bg-grid text-dim hover:text-ink rounded text-xs font-medium transition-colors border border-grid"
            title="Día Siguiente"
          >
            Siguiente ▶
          </button>
          <button
            onclick={goToToday}
            class="px-2.5 py-1 bg-amber hover:bg-amber text-black rounded text-xs font-semibold transition-colors"
          >
            Hoy
          </button>
        </div>
      </div>

      <!-- Fichas Técnicas -->
      <div class="flex flex-wrap items-center gap-2 font-mono text-xs">
        <div class="bg-panel border border-grid px-3 py-1.5 rounded">
          <span class="text-dim uppercase text-[10px] block font-semibold">Fichaje Reglamentario</span>
          <span class="text-green font-semibold">08:45 h L-V</span>
        </div>

        <div class="bg-panel border border-grid px-3 py-1.5 rounded">
          <span class="text-dim uppercase text-[10px] block font-semibold">Clases Técnicas</span>
          <span class="text-cyan font-semibold">09:00 - 10:15 h (S1/S3)</span>
        </div>

        <div class="bg-panel border border-grid px-3 py-1.5 rounded">
          <span class="text-dim uppercase text-[10px] block font-semibold">Dotación Operativa</span>
          <span class="text-amber font-semibold">{effectivePresenceCount}/{logs.length} ({operationalPercent}%)</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Encabezado y Métricas Rápidas -->
  <div class="flex flex-wrap items-center justify-between gap-4 pb-3 mb-4 border-b border-grid">
    <div>
      <h2 class="text-sm font-semibold text-ink uppercase tracking-wide">
        Planilla Oficial de Asistencia Técnica, Tardanzas y Fisioterapia
      </h2>
      <p class="text-xs text-dim mt-0.5">
        Registro nominal oficial para inspección, carpetas médicas con adjuntos y seguimiento clínico kinesiológico
      </p>
    </div>

    <!-- Botones de Acción Global -->
    <div class="flex items-center gap-2 flex-wrap">
      <button
        onclick={() => showBopMatrixModal = true}
        class="px-3 py-1.5 bg-raised hover:bg-grid text-amber border border-grid rounded text-xs font-semibold font-mono transition-colors"
        title="Planilla Matriz Mensual de Ausentismo Histórica (Carina Lion)"
      >
        Matriz Ausentismo BOP
      </button>
      <button
        onclick={() => {
          monthlyInitialPersonnelId = '';
          showMonthlyModal = true;
        }}
        class="px-3 py-1.5 bg-raised hover:bg-grid text-cyan border border-grid rounded text-xs font-medium transition-colors"
        title="Planillas Mensuales de 31 días y Resumen para Recursos Humanos"
      >
        Planillas Mensuales (RRHH)
      </button>
      <button
        onclick={exportExcel}
        class="px-3 py-1.5 bg-raised hover:bg-grid text-green border border-grid rounded text-xs font-medium transition-colors"
      >
        Exportar a Excel (.csv)
      </button>
      <button
        onclick={() => (printPreviewOpen = true)}
        class="px-3 py-1.5 bg-raised hover:bg-grid text-ink border border-grid rounded text-xs font-medium transition-colors"
      >
        Imprimir / PDF
      </button>
    </div>
  </div>

  <!-- Contadores / Resumen de Estado -->
  <div class="grid grid-cols-3 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-11 gap-2 mb-4">
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-green">Está</div>
      <div class="text-base font-bold text-ink mt-0.5 font-mono">{countPresent}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-amber">No en clase</div>
      <div class="text-base font-bold text-ink mt-0.5 font-mono">{countNotInClass}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-cyan">Fisio</div>
      <div class="text-base font-bold text-ink mt-0.5 font-mono">{countPhysio}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-amber-400">Tarde</div>
      <div class="text-base font-bold text-ink mt-0.5 font-mono">{countLate}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-purple-400">Licencia</div>
      <div class="text-base font-bold text-ink mt-0.5 font-mono">{countMedical}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-red">No está</div>
      <div class="text-base font-bold text-ink mt-0.5 font-mono">{countAbsent}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-purple-300">Sin parte</div>
      <div class="text-base font-bold text-purple-300 mt-0.5 font-mono">{countNoPart}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center" title="Inasistencias justificadas del día (tope: 2 por mes y 10 por año)">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-blue">Justificada</div>
      <div class="text-base font-bold text-blue mt-0.5 font-mono">{countJustified}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-blue">Adec. tareas</div>
      <div class="text-base font-bold text-blue mt-0.5 font-mono">{countTask}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-amber">Jubilados</div>
      <div class="text-base font-bold text-amber mt-0.5 font-mono">{countRetired}</div>
    </div>
    <div class="bg-void p-2 rounded border border-grid text-center">
      <div class="text-[10px] font-semibold uppercase tracking-wider text-dim">Pendiente</div>
      <div class="text-base font-bold text-dim mt-0.5 font-mono">{countPending}</div>
    </div>
  </div>

  <!-- Buscador de Bailarines -->
  <div class="mb-4">
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Filtrar por nombre, apellido o DNI..."
      class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-xs font-medium text-ink outline-none"
    />
  </div>

  <!-- 1. VISTA DE ESCRITORIO (TABLA MULTI-COLUMNA) -->
  <div class="hidden md:block overflow-x-auto">
    <table class="w-full text-left text-xs border-collapse">
      <thead>
        <tr class="bg-raised text-[10px] uppercase font-semibold tracking-wider text-dim border-b border-grid">
          <th class="p-3">Bailarín / Agente</th>
          <th class="p-3">Rol / Cat.</th>
          <th class="p-3">Estado Operativo</th>
          <th class="p-3">Acciones Rápidas de Inspección</th>
          <th class="p-3">Seguimiento Kinésico / Médico</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-grid/60 font-medium">
        {#each filteredLogs as dancer}
          <tr class="hover:bg-raised/40 transition-colors">
            <!-- Nombre y DNI -->
            <td class="p-3">
              <div class="font-semibold text-ink text-sm">{dancer.dancer_name}</div>
              <div class="text-[10px] font-mono text-dim">DNI: {dancer.legajo || 'S/D'}</div>
            </td>

            <!-- Rol -->
            <td class="p-3">
              <span class="px-2 py-0.5 rounded text-[10px] font-medium bg-raised text-dim border border-grid">
                {dancer.category_name}
              </span>
            </td>

            <!-- Estado Operativo Actual con Badge -->
            <td class="p-3">
              {#if dancer.is_retired}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-amber/15 text-amber border border-amber/30" title={dancer.retirement_notes || 'Estado jubilatorio'}>
                  Jubilatorio (No concurre)
                </span>
              {:else if dancer.attendance_status === 'PRESENT'}
                <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-semibold bg-green/15 text-green border border-green/30">
                  <span class="w-1.5 h-1.5 rounded-full bg-green"></span>
                  Está (Presente)
                </span>
              {:else if dancer.attendance_status === 'NOT_IN_CLASS'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-amber/15 text-amber border border-amber/30">
                  No está en clase
                </span>
              {:else if dancer.attendance_status === 'IN_PHYSIOTHERAPY'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-cyan/15 text-cyan border border-cyan/30">
                  En Fisioterapia
                </span>
              {:else if dancer.attendance_status === 'LATE'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-amber-500/15 text-amber-400 border border-amber-500/30">
                  Llegó tarde (+{dancer.minutes_late}m)
                </span>
              {:else if dancer.attendance_status === 'MEDICAL_LEAVE'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-purple-500/15 text-purple-400 border border-purple-500/30">
                  {leaveTypeLabel(dancer)}
                </span>
              {:else if dancer.attendance_status === 'ABSENT'}
                <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-semibold bg-red/15 text-red border border-red/30">
                  <span class="w-1.5 h-1.5 rounded-full bg-red"></span>
                  No está
                </span>
              {:else if dancer.attendance_status === 'NO_PART'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-purple-500/15 text-purple-300 border border-purple-500/30">
                  Sin parte
                </span>
              {:else if dancer.attendance_status === 'JUSTIFIED_ABSENCE'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-blue/15 text-blue border border-blue/30">
                  Justificada (IJ)
                </span>
              {:else if dancer.attendance_status === 'TASK_ADJUSTMENT'}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-blue/15 text-blue border border-blue/30">
                  Adecuación de tareas
                </span>
              {:else}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-raised text-dim border border-grid">
                  ? (Pendiente)
                </span>
              {/if}
            </td>

            <!-- Botonera Táctica Rápida (1 solo toque) -->
            <td class="p-3">
              <div class="flex flex-wrap items-center gap-1">
                <button
                  onclick={() => updateStatus(dancer, 'PRESENT')}
                  title="Marcar Presente en horario (08:45)"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-green/15 hover:bg-green/25 text-green border border-green/30 transition-colors"
                >
                  Está
                </button>
                <button
                  onclick={() => updateStatus(dancer, 'NOT_IN_CLASS', { remarks: 'Eximido/a de clase matutina' })}
                  title="Presente en el teatro pero no hace la barra/clase"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-amber/15 hover:bg-amber/25 text-amber border border-amber/30 transition-colors"
                >
                  No en clase
                </button>
                <button
                  onclick={() => {
                    physioModalDancer = dancer;
                    physDiagnosis = dancer.current_physio_diagnosis || '';
                    physEvolution = dancer.physio_notes || '';
                  }}
                  title="Registrar atención con la Dra. de Fisioterapia"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-cyan/15 hover:bg-cyan/25 text-cyan border border-cyan/30 transition-colors"
                >
                  Fisio
                </button>
                <button
                  onclick={() => {
                    lateModalDancer = dancer;
                    lateMinutes = 15;
                    lateCheckinTime = '09:00';
                  }}
                  title="Registrar llegada tarde con minutos de demora"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-amber-500/15 hover:bg-amber-500/25 text-amber-400 border border-amber-500/30 transition-colors"
                >
                  Tarde
                </button>
                <button
                  onclick={() => {
                    medicalModalDancer = dancer;
                    medDiagnosis = dancer.medical_diagnosis || '';
                    medDoctor = dancer.doctor_details || '';
                    medLeaveType = dancer.leave_type ?? 'ENFERMEDAD';
                    medLeaveStartDate = dancer.leave_start_date || scheduleDate;
                    medLeaveEndDate = dancer.leave_end_date || scheduleDate;
                  }}
                  title="Cargar carpeta médica y adjuntar certificado"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-purple-500/15 hover:bg-purple-500/25 text-purple-400 border border-purple-500/30 transition-colors"
                >
                  Licencia
                </button>
                <button
                  onclick={() => updateStatus(dancer, 'ABSENT')}
                  title="Marcar Ausente"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-red/15 hover:bg-red/25 text-red border border-red/30 transition-colors"
                >
                  No está
                </button>
                <button
                  onclick={() => updateStatus(dancer, 'NO_PART', { remarks: 'Sin convocar al parte diario' })}
                  title="Sin parte / No convocado al parte del día"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-purple-500/15 hover:bg-purple-500/25 text-purple-300 border border-purple-500/30 transition-colors"
                >
                  Sin parte
                </button>
                <button
                  onclick={() => updateStatus(dancer, 'JUSTIFIED_ABSENCE', { remarks: 'Inasistencia justificada' })}
                  title="Inasistencia justificada (IJ): máximo 2 por mes y 10 por año"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-blue/15 hover:bg-blue/25 text-blue border border-blue/30 transition-colors"
                >
                  Justificada
                </button>
                <button
                  onclick={() => updateStatus(dancer, 'TASK_ADJUSTMENT', { remarks: 'Adecuación de tareas (trabaja en otra área)' })}
                  title="Adecuación de tareas: trabaja en otra área, no es falta"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-blue/15 hover:bg-blue/25 text-blue border border-blue/30 transition-colors"
                >
                  Adecuación
                </button>
                <button
                  onclick={() => updateStatus(dancer, 'PENDING')}
                  title="Restablecer a pendiente / ?"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-raised hover:bg-grid text-dim border border-grid transition-colors"
                >
                  ?
                </button>
                <button
                  onclick={() => openRectify(dancer)}
                  title="Rectificar fichada, hora real o minutos de tardanza computada"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-raised hover:bg-amber hover:text-black text-amber border border-amber/30 transition-colors"
                >
                  Rectificar
                </button>
                <button
                  onclick={() => openMonthlyForDancer(dancer.personnel_id)}
                  title="Ver planilla mensual individual de 31 días para RRHH"
                  class="px-2 py-1 rounded text-[10px] font-semibold bg-raised hover:bg-grid text-cyan border border-cyan/30 transition-colors"
                >
                  Mes
                </button>
              </div>
            </td>

            <!-- Notas de Fisioterapia y Médicas -->
            <td class="p-3">
              {#if dancer.current_physio_diagnosis || dancer.physio_notes}
                <div class="text-[11px] text-cyan flex items-start gap-1">
                  <div>
                    <strong>Fisio:</strong> {dancer.current_physio_diagnosis || dancer.physio_notes}
                  </div>
                </div>
              {/if}
              {#if dancer.certificate_file_name}
                <button
                  type="button"
                  onclick={() => viewCertificate(dancer)}
                  class="text-[10px] text-purple-300 hover:text-purple-200 underline font-mono mt-0.5 flex items-center gap-1 transition-colors text-left"
                  title="Ver y ampliar certificado médico"
                >
                  Adjunto: {dancer.certificate_file_name} <span class="text-[9px] bg-raised border border-grid px-1 py-0.5 rounded font-sans">Ver</span>
                </button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <!-- 2. VISTA MÓVIL TÁCTIL (TARJETAS OPTIMIZADAS PARA GALAXY A53 / 1-TOUCH) -->
  <div class="block md:hidden space-y-3">
    {#each filteredLogs as dancer}
      <div class="bg-void border border-grid rounded-md p-3.5 space-y-2">
        <!-- Encabezado de la Tarjeta Móvil -->
        <div class="flex items-start justify-between gap-2">
          <div>
            <h4 class="font-bold text-ink text-sm">{dancer.dancer_name}</h4>
            <div class="flex items-center gap-2 mt-0.5">
              <span class="px-1.5 py-0.5 rounded text-[9px] font-medium bg-raised text-dim border border-grid uppercase">
                {dancer.category_name}
              </span>
              <span class="text-[10px] font-mono text-dim">
                DNI: {dancer.legajo || 'S/D'}
              </span>
            </div>
          </div>

          <!-- Badge de Estado Actual -->
          <div>
            {#if dancer.is_retired}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-amber/15 text-amber border border-amber/30" title={dancer.retirement_notes || 'Estado jubilatorio'}>
                Jubilatorio
              </span>
            {:else if dancer.attendance_status === 'PRESENT'}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-green/15 text-green border border-green/30">
                <span class="w-1.5 h-1.5 rounded-full bg-green"></span>
                Está
              </span>
            {:else if dancer.attendance_status === 'NOT_IN_CLASS'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-amber/15 text-amber border border-amber/30">
                No en clase
              </span>
            {:else if dancer.attendance_status === 'IN_PHYSIOTHERAPY'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-cyan/15 text-cyan border border-cyan/30">
                Fisio
              </span>
            {:else if dancer.attendance_status === 'LATE'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-amber-500/15 text-amber-400 border border-amber-500/30">
                Tarde (+{dancer.minutes_late}m)
              </span>
            {:else if dancer.attendance_status === 'MEDICAL_LEAVE'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-purple-500/15 text-purple-400 border border-purple-500/30">
                {leaveTypeLabel(dancer)}
              </span>
            {:else if dancer.attendance_status === 'ABSENT'}
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-red/15 text-red border border-red/30">
                <span class="w-1.5 h-1.5 rounded-full bg-red"></span>
                No está
              </span>
            {:else if dancer.attendance_status === 'NO_PART'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-purple-500/15 text-purple-300 border border-purple-500/30">
                Sin parte
              </span>
            {:else if dancer.attendance_status === 'JUSTIFIED_ABSENCE'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-blue/15 text-blue border border-blue/30">
                Justificada (IJ)
              </span>
            {:else if dancer.attendance_status === 'TASK_ADJUSTMENT'}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-blue/15 text-blue border border-blue/30">
                Adec. de tareas
              </span>
            {:else}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-semibold bg-raised text-dim border border-grid">
                ?
              </span>
            {/if}
          </div>
        </div>

        <!-- Banner de Fisioterapia o Certificado si existe -->
        {#if dancer.current_physio_diagnosis || dancer.physio_notes || dancer.certificate_file_name}
          <div class="p-2 rounded bg-panel border border-grid text-[11px] space-y-1">
            {#if dancer.current_physio_diagnosis || dancer.physio_notes}
              <div class="text-cyan flex items-start gap-1">
                <div>
                  <strong>Fisio:</strong> {dancer.current_physio_diagnosis || dancer.physio_notes}
                </div>
              </div>
            {/if}
            {#if dancer.certificate_file_name}
              <button
                type="button"
                onclick={() => viewCertificate(dancer)}
                class="text-purple-300 hover:text-purple-200 underline font-mono text-[10px] flex items-center gap-1 transition-colors text-left"
              >
                Certificado: {dancer.certificate_file_name} <span class="text-[9px] bg-raised border border-grid px-1 py-0.5 rounded font-sans">Ver</span>
              </button>
            {/if}
          </div>
        {/if}

        <!-- Botonera Táctica Móvil (>= 40px para pulsación táctil precisa) -->
        <div class="grid grid-cols-4 gap-1.5 mb-1.5">
          <button
            onclick={() => updateStatus(dancer, 'PRESENT')}
            class="h-10 rounded text-[11px] font-semibold bg-green/15 hover:bg-green/25 text-green border border-green/30 transition-colors flex items-center justify-center"
          >
            Está
          </button>
          <button
            onclick={() => updateStatus(dancer, 'NOT_IN_CLASS', { remarks: 'Eximido/a de clase matutina' })}
            class="h-10 rounded text-[10px] font-semibold bg-amber/15 hover:bg-amber/25 text-amber border border-amber/30 transition-colors flex items-center justify-center"
          >
            No clase
          </button>
          <button
            onclick={() => {
              physioModalDancer = dancer;
              physDiagnosis = dancer.current_physio_diagnosis || '';
              physEvolution = dancer.physio_notes || '';
            }}
            class="h-10 rounded text-[11px] font-semibold bg-cyan/15 hover:bg-cyan/25 text-cyan border border-cyan/30 transition-colors flex items-center justify-center"
          >
            Fisio
          </button>
          <button
            onclick={() => {
              lateModalDancer = dancer;
              lateMinutes = 15;
              lateCheckinTime = '09:00';
            }}
            class="h-10 rounded text-[11px] font-semibold bg-amber-500/15 hover:bg-amber-500/25 text-amber-400 border border-amber-500/30 transition-colors flex items-center justify-center"
          >
            Tarde
          </button>
        </div>

        <div class="grid grid-cols-4 gap-1.5 mb-1.5">
          <button
            onclick={() => {
              medicalModalDancer = dancer;
              medDiagnosis = dancer.medical_diagnosis || '';
              medDoctor = dancer.doctor_details || '';
              medLeaveType = dancer.leave_type ?? 'ENFERMEDAD';
              medLeaveStartDate = dancer.leave_start_date || scheduleDate;
              medLeaveEndDate = dancer.leave_end_date || scheduleDate;
            }}
            class="h-9 rounded text-[10px] font-semibold bg-purple-500/15 hover:bg-purple-500/25 text-purple-400 border border-purple-500/30 transition-colors flex items-center justify-center"
          >
            Licencia
          </button>
          <button
            onclick={() => updateStatus(dancer, 'ABSENT')}
            class="h-9 rounded text-[10px] font-semibold bg-red/15 hover:bg-red/25 text-red border border-red/30 transition-colors flex items-center justify-center"
          >
            No está
          </button>
          <button
            onclick={() => updateStatus(dancer, 'NO_PART', { remarks: 'Sin convocar al parte diario' })}
            class="h-9 rounded text-[10px] font-semibold bg-purple-500/15 hover:bg-purple-500/25 text-purple-300 border border-purple-500/30 transition-colors flex items-center justify-center"
          >
            Sin parte
          </button>
          <button
            onclick={() => updateStatus(dancer, 'JUSTIFIED_ABSENCE', { remarks: 'Inasistencia justificada' })}
            class="h-9 rounded text-[10px] font-semibold bg-blue/15 hover:bg-blue/25 text-blue border border-blue/30 transition-colors flex items-center justify-center"
          >
            Justificada
          </button>
          <button
            onclick={() => updateStatus(dancer, 'TASK_ADJUSTMENT', { remarks: 'Adecuación de tareas (trabaja en otra área)' })}
            class="h-9 rounded text-[10px] font-semibold bg-blue/15 hover:bg-blue/25 text-blue border border-blue/30 transition-colors flex items-center justify-center"
          >
            Adecuación
          </button>
          <button
            onclick={() => updateStatus(dancer, 'PENDING')}
            class="h-9 rounded text-[10px] font-semibold bg-raised hover:bg-grid text-dim border border-grid transition-colors flex items-center justify-center"
          >
            ?
          </button>
        </div>

        <!-- Acciones Administrativas Adicionales Móviles -->
        <div class="grid grid-cols-2 gap-1.5 pt-1.5 border-t border-grid">
          <button
            onclick={() => openRectify(dancer)}
            class="h-9 rounded text-[10px] font-semibold bg-raised hover:bg-amber hover:text-black text-amber border border-amber/30 transition-colors flex items-center justify-center"
          >
            Rectificar
          </button>
          <button
            onclick={() => openMonthlyForDancer(dancer.personnel_id)}
            class="h-9 rounded text-[10px] font-semibold bg-raised hover:bg-grid text-cyan border border-cyan/30 transition-colors flex items-center justify-center"
          >
            Planilla Mes
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- ======================================================================= -->
<!-- MODAL DE CARPETA MÉDICA (CON ADJUNTO DE CERTIFICADO MÉDICO)             -->
<!-- ======================================================================= -->
{#if medicalModalDancer}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4" use:modalBehavior={{ onclose: () => medicalModalDancer = null, backdrop: false }}>
    <div class="tg-modal max-w-lg">
      <ModalHeader title="Cargar Carpeta Médica" subtitle={medicalModalDancer.dancer_name} onclose={() => medicalModalDancer = null} />

      <div class="tg-modal-body space-y-3">
        <div>
          <label for="med-diagnosis-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Diagnóstico Médico</label>
          <input
            id="med-diagnosis-input"
            type="text"
            bind:value={medDiagnosis}
            placeholder="Ej: Esguince de tobillo, desgarro miofascial, tendinopatía..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-amber"
          />
        </div>

        <!-- Tipo de Licencia -->
        <div>
          <label for="med-leave-type" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Tipo de Licencia</label>
          <select
            id="med-leave-type"
            bind:value={medLeaveType}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-amber"
          >
            <option value="ENFERMEDAD">Licencia por enfermedad</option>
            <option value="ESTUDIO">Licencia por estudio</option>
            <option value="SIN_GOCE_DE_HABERES">Licencia sin goce de haberes</option>
            <option value="OTRAS">Otras</option>
          </select>
        </div>

        <!-- Rango de Fechas [Desde, Hasta] de la Licencia -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1" for="med-leave-start">Desde (Fecha Inicio)</label>
            <input
              id="med-leave-start"
              type="date"
              bind:value={medLeaveStartDate}
              class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-semibold text-ink font-mono outline-none focus:border-amber"
            />
          </div>
          <div>
            <label class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1" for="med-leave-end">Hasta (Fecha Fin / Alta)</label>
            <input
              id="med-leave-end"
              type="date"
              bind:value={medLeaveEndDate}
              class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-semibold text-ink font-mono outline-none focus:border-amber"
            />
          </div>
        </div>

        <div>
          <label class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1" for="med-doctor">Médico / Matrícula</label>
          <input
            id="med-doctor"
            type="text"
            bind:value={medDoctor}
            placeholder="Dr/a. Nombre - MP..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-amber"
          />
        </div>

        <div>
          <label for="med-file-upload" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Adjuntar Certificado Médico (Foto o PDF)</label>
          <input
            id="med-file-upload"
            type="file"
            accept="image/*,application/pdf"
            onchange={(e) => handleFileUpload(e, 'medical')}
            class="w-full text-xs text-dim file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-xs file:font-semibold file:bg-raised file:text-ink hover:file:bg-grid cursor-pointer"
          />
          {#if medFileName}
            <div class="text-[11px] font-mono text-green mt-1">Archivo cargado: {medFileName}</div>
          {/if}
        </div>

        <div>
          <label for="med-remarks-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Observaciones</label>
          <textarea
            id="med-remarks-input"
            bind:value={medRemarks}
            rows="2"
            placeholder="Indicaciones de reposo o tratamiento..."
            class="w-full p-2.5 bg-void border border-grid rounded text-xs text-ink outline-none focus:border-amber"
          ></textarea>
        </div>
      </div>

      <div class="tg-modal-foot">
        <button onclick={() => medicalModalDancer = null} class="tg-btn">Cancelar</button>
        <button onclick={submitMedicalLeave} class="tg-btn tg-btn-primary">
          Guardar Carpeta Médica
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ======================================================================= -->
<!-- MODAL DE ATENCIÓN EN FISIOTERAPIA (OFICINA DE KINESIOLOGÍA)            -->
<!-- ======================================================================= -->
{#if physioModalDancer}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4" use:modalBehavior={{ onclose: () => physioModalDancer = null, backdrop: false }}>
    <div class="tg-modal max-w-lg">
      <ModalHeader title="Atención en Fisioterapia" subtitle={physioModalDancer.dancer_name} onclose={() => physioModalDancer = null} />

      <div class="tg-modal-body space-y-3">
        <div>
          <label for="phys-diag-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Diagnóstico Kinésico</label>
          <input
            id="phys-diag-input"
            type="text"
            bind:value={physDiagnosis}
            placeholder="Ej: Sobrecarga en sóleo, tendinopatía aquiliana, contractura lumbar..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-cyan"
          />
        </div>

        <div>
          <label for="phys-treat-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Tratamiento Aplicado en la Sesión</label>
          <input
            id="phys-treat-input"
            type="text"
            bind:value={physTreatment}
            placeholder="Masoterapia, ultrasonido, magnetoterapia, vendaje..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-cyan"
          />
        </div>

        <div>
          <label for="phys-restr-select" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Restricción para Ensayos y Clases</label>
          <select
            id="phys-restr-select"
            bind:value={physRestriction}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-cyan"
          >
            <option value="NORMAL">Apto sin restricciones</option>
            <option value="LIGHT_BARRE_ONLY">Solo barra suave</option>
            <option value="NO_JUMPS">Eximido/a de saltos</option>
            <option value="NO_POINTE">Eximida de zapatillas de punta</option>
            <option value="TOTAL_REST">Reposo deportivo absoluto</option>
          </select>
        </div>

        <div>
          <label for="phys-evol-textarea" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Notas de Evolución de la Dra.</label>
          <textarea
            id="phys-evol-textarea"
            bind:value={physEvolution}
            rows="2"
            placeholder="Observaciones de la evolución clínica del bailarín..."
            class="w-full p-2.5 bg-void border border-grid rounded text-xs text-ink outline-none focus:border-cyan"
          ></textarea>
        </div>

        <div>
          <label for="phys-file-upload" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Adjuntar Estudio o Informe (Ecografía, Rx, Certificado)</label>
          <input
            id="phys-file-upload"
            type="file"
            accept="image/*,application/pdf"
            onchange={(e) => handleFileUpload(e, 'physio')}
            class="w-full text-xs text-dim file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-xs file:font-semibold file:bg-raised file:text-ink hover:file:bg-grid cursor-pointer"
          />
          {#if physAttachmentName}
            <div class="text-[11px] font-mono text-green mt-1">Archivo cargado: {physAttachmentName}</div>
          {/if}
        </div>
      </div>

      <div class="tg-modal-foot">
        <button onclick={() => physioModalDancer = null} class="tg-btn">Cancelar</button>
        <button onclick={submitPhysioSession} class="tg-btn tg-btn-primary">
          Guardar Sesión en Fisioterapia
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ======================================================================= -->
<!-- MODAL DE LLEGADA TARDE                                                  -->
<!-- ======================================================================= -->
{#if lateModalDancer}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4" use:modalBehavior={{ onclose: () => lateModalDancer = null, backdrop: false }}>
    <div class="tg-modal max-w-sm">
      <ModalHeader title="Llegada Tarde" subtitle={lateModalDancer.dancer_name} onclose={() => lateModalDancer = null} />

      <div class="tg-modal-body space-y-3">
        <div>
          <label for="late-time-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Hora de Llegada / Aviso a Clase</label>
          <input
            id="late-time-input"
            type="time"
            bind:value={lateCheckinTime}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-semibold text-ink font-mono outline-none focus:border-amber"
          />
        </div>

        <div>
          <label for="late-min-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Minutos de Demora ({lateMinutes} min)</label>
          <input
            id="late-min-input"
            type="number"
            min="1"
            max="180"
            bind:value={lateMinutes}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-semibold text-ink font-mono outline-none focus:border-amber"
          />
        </div>

        <div>
          <label for="late-rem-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Motivo / Justificación</label>
          <input
            id="late-rem-input"
            type="text"
            bind:value={lateRemarks}
            placeholder="Transporte, trámite justificado..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs text-ink outline-none focus:border-amber"
          />
        </div>
      </div>

      <div class="tg-modal-foot">
        <button onclick={() => lateModalDancer = null} class="tg-btn">Cancelar</button>
        <button onclick={submitLate} class="tg-btn tg-btn-primary">
          Registrar Demora
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ======================================================================= -->
<!-- MODAL DE VISOR / ZOOM DE CERTIFICADO MÉDICO                              -->
<!-- ======================================================================= -->
{#if zoomCertificateModal}
  <div class="fixed inset-0 z-50 bg-black/90 flex flex-col items-center justify-center p-4" use:modalBehavior={{ onclose: () => zoomCertificateModal = false }}>
    <div class="tg-modal max-w-4xl">
      <ModalHeader title={zoomCertificateTitle} onclose={() => zoomCertificateModal = false} />
      <div class="flex-1 overflow-auto p-4 flex items-center justify-center bg-void/50">
        {#if zoomCertificateUrl}
          {#if zoomCertificateUrl.startsWith('data:image/')}
            <img
              src={zoomCertificateUrl}
              alt="Certificado Médico Adjunto"
              class="max-w-full max-h-[75vh] object-contain rounded border border-grid"
            />
          {:else if zoomCertificateUrl.startsWith('data:application/pdf')}
            <iframe
              src={zoomCertificateUrl}
              title="Certificado Médico PDF"
              class="w-full h-[75vh] rounded border border-grid"
            ></iframe>
          {:else}
            <div class="text-center p-8 text-ink space-y-3">
              <p class="text-sm font-semibold">Documento adjunto en formato no previsualizable directamente.</p>
              <a
                href={zoomCertificateUrl}
                download="certificado_medico"
                class="inline-block px-4 py-2 bg-raised hover:bg-grid text-ink border border-grid font-medium text-xs rounded transition-colors"
              >
                Descargar Archivo
              </a>
            </div>
          {/if}
        {:else}
          <div class="text-center p-8 text-dim">
            <p class="text-sm">No se adjuntó contenido base64 en la base de datos para este certificado.</p>
          </div>
        {/if}
      </div>
      <div class="tg-modal-foot">
        <button
          onclick={() => zoomCertificateModal = false}
          class="tg-btn"
        >
          Cerrar
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ======================================================================= -->
<!-- MODAL DE RECTIFICACIÓN ADMINISTRATIVA DE FICHADA / TC                   -->
<!-- ======================================================================= -->
{#if rectifyModalDancer}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4" use:modalBehavior={{ onclose: () => rectifyModalDancer = null, backdrop: false }}>
    <div class="tg-modal max-w-md">
      <ModalHeader title="Rectificar Asistencia / Estado" subtitle={rectifyModalDancer.dancer_name} onclose={() => rectifyModalDancer = null} />

      <div class="tg-modal-body space-y-3">
        <div>
          <label for="rect-status-select" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Estado Operativo</label>
          <select
            id="rect-status-select"
            bind:value={rectifyStatus}
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-medium text-ink outline-none focus:border-amber"
          >
            <option value="PRESENT">Está (Presente)</option>
            <option value="NOT_IN_CLASS">No en clase (Eximido/a de barra)</option>
            <option value="IN_PHYSIOTHERAPY">En Fisioterapia</option>
            <option value="LATE">Llegó tarde (Con demora computada)</option>
            <option value="MEDICAL_LEAVE">Licencia Médica</option>
            <option value="ABSENT">No está (Ausente)</option>
            <option value="NO_PART">Sin parte (Sin convocar)</option>
            <option value="JUSTIFIED_ABSENCE">Inasistencia justificada (IJ, máx. 2/mes y 10/año)</option>
            <option value="TASK_ADJUSTMENT">Adecuación de tareas (trabaja en otra área)</option>
            <option value="PENDING">? (Pendiente)</option>
          </select>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="rect-time-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Hora de Llegada / Aviso</label>
            <input
              id="rect-time-input"
              type="time"
              bind:value={rectifyCheckinTime}
              class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-mono font-semibold text-ink outline-none focus:border-amber"
            />
          </div>

          <div>
            <label for="rect-min-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Minutos Tardanza (TC)</label>
            <input
              id="rect-min-input"
              type="number"
              min="0"
              max="240"
              bind:value={rectifyMinutesLate}
              class="w-full h-10 px-3 bg-void border border-grid rounded text-xs font-mono font-semibold text-ink text-center outline-none focus:border-amber"
            />
          </div>
        </div>

        <div>
          <label for="rect-diag-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Diagnóstico / Afección</label>
          <input
            id="rect-diag-input"
            type="text"
            bind:value={rectifyDiagnosis}
            placeholder="Kinesiología, contractura, reposo..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs text-ink outline-none focus:border-amber"
          />
        </div>

        <div>
          <label for="rect-rem-input" class="block text-[10px] font-semibold uppercase tracking-wider text-dim mb-1">Motivo de la Rectificación / Justificación</label>
          <input
            id="rect-rem-input"
            type="text"
            bind:value={rectifyRemarks}
            placeholder="Ajuste de reloj, justificado por Inspección..."
            class="w-full h-10 px-3 bg-void border border-grid rounded text-xs text-ink outline-none focus:border-amber"
          />
        </div>
      </div>

      <div class="tg-modal-foot">
        <button onclick={() => rectifyModalDancer = null} class="tg-btn">Cancelar</button>
        <button onclick={submitRectification} class="tg-btn tg-btn-primary">
          Guardar Rectificación
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL DE PLANILLAS MENSUALES DE AGENTES (RRHH) -->
<AgentMonthlyAttendanceModal
  isOpen={showMonthlyModal}
  initialPersonnelId={monthlyInitialPersonnelId}
  onClose={() => showMonthlyModal = false}
/>

<!-- MODAL MATRIZ MENSUAL DE AUSENTISMO BOP (EXCEL DE CARINA LION) -->
<BopAttendanceMatrixModal
  isOpen={showBopMatrixModal}
  onClose={() => {
    showBopMatrixModal = false;
    loadLogs();
  }}
/>

<PrintPreviewModal
  open={printPreviewOpen}
  title="Vista previa de impresión"
  subtitle="Asistencia del día — ediciones solo para la copia impresa"
  source={printableEl}
  onclose={() => (printPreviewOpen = false)}
/>
