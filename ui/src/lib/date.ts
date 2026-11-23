/**
 * Devuelve la fecha local en formato ISO `YYYY-MM-DD` (sin desplazamiento UTC).
 * Evita el salto al día siguiente después de las 21:00 ART (UTC-3).
 */
export function toLocalIso(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/**
 * Devuelve la fecha local de hoy en formato ISO `YYYY-MM-DD`.
 */
export function todayLocalIso(): string {
  return toLocalIso(new Date());
}
