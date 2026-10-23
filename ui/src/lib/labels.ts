// Textos en español para códigos internos (que no se vean "SALA_1" ni "ORDINARY" en pantalla).
const ROOMS: Record<string, string> = {
  SALA_1: 'Sala 1', SALA_2: 'Sala 2', SALA_3: 'Sala 3', ESCENARIO: 'Escenario',
  GIRA_TRASLADO: 'Gira · Traslado', GIRA_ESCENARIO: 'Gira · Escenario', GIRA_SALA: 'Gira · Sala', SALA_AUX: 'Sala auxiliar',
};
const REGIMES: Record<string, string> = { ORDINARY: 'Ordinario', PRE_FUNCTION: 'Pre-función', EXTRAORDINARY: 'Extraordinario' };
const ROLES: Record<string, string> = {
  DANCER: 'Bailarín/a', MASTER: 'Maestro/a', PIANIST: 'Pianista', INSPECTOR: 'Inspección', SOUND_TECH: 'Sonido',
};
export const roomLabel = (c?: string | null) => (c ? ROOMS[c] ?? c : '');
export const regimeLabel = (c?: string | null) => (c ? REGIMES[c] ?? c : '');
export const roleLabel = (c?: string | null) => (c ? ROLES[c] ?? c : '');
