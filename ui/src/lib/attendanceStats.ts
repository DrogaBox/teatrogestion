import type { AttendanceLogDto, AttendanceStatus } from './ipc/client';

const PRESENT_STATUSES = new Set<AttendanceStatus>([
  'PRESENT',
  'LATE',
  'NOT_IN_CLASS',
  'IN_PHYSIOTHERAPY',
  'TASK_ADJUSTMENT',
]);

const EXCLUDED_FROM_DENOMINATOR = new Set<AttendanceStatus>([
  'RETIRED_EXEMPT',
  'NO_PART',
]);

export interface AttendanceSummaryStats {
  totalActive: number;
  presentCount: number;
  lateCount: number;
  medicalLeaveCount: number;
  physioCount: number;
  absentCount: number;
  retiredCount: number;
  noPartCount: number;
  pendingCount: number;
  presencePct: number;
}

/**
 * Fórmula unificada de % Presentismo compartida entre Dashboard y Asistencia:
 * - Excluye RETIRED_EXEMPT (jubilados) y NO_PART (sin parte) del denominador activo.
 * - Cuenta como presentes: PRESENT, LATE, NOT_IN_CLASS, IN_PHYSIOTHERAPY y TASK_ADJUSTMENT.
 */
export function computeAttendanceStats(logs: AttendanceLogDto[]): AttendanceSummaryStats {
  let totalActive = 0;
  let presentCount = 0;
  let lateCount = 0;
  let medicalLeaveCount = 0;
  let physioCount = 0;
  let absentCount = 0;
  let retiredCount = 0;
  let noPartCount = 0;
  let pendingCount = 0;

  for (const log of logs) {
    const st = log.attendance_status;
    if (st === 'RETIRED_EXEMPT' || log.is_retired) {
      retiredCount += 1;
      continue;
    }
    if (st === 'NO_PART') {
      noPartCount += 1;
      continue;
    }
    if (!EXCLUDED_FROM_DENOMINATOR.has(st)) {
      totalActive += 1;
    }
    if (PRESENT_STATUSES.has(st)) {
      presentCount += 1;
    }
    if (st === 'LATE') lateCount += 1;
    if (st === 'MEDICAL_LEAVE') medicalLeaveCount += 1;
    if (st === 'IN_PHYSIOTHERAPY') physioCount += 1;
    if (st === 'ABSENT' || st === 'JUSTIFIED_ABSENCE') absentCount += 1;
    if (st === 'PENDING') pendingCount += 1;
  }

  const presencePct = totalActive > 0 ? Math.round((presentCount / totalActive) * 100) : 0;

  return {
    totalActive,
    presentCount,
    lateCount,
    medicalLeaveCount,
    physioCount,
    absentCount,
    retiredCount,
    noPartCount,
    pendingCount,
    presencePct,
  };
}
