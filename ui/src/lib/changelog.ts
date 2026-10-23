// Registro de cambios visible para el usuario (ventana «Novedades» y capítulo de la Ayuda).
// Agregar siempre la versión más nueva ARRIBA.
export interface ChangelogEntry {
  version: string;
  date: string;
  title: string;
  added: string[];
  fixed: string[];
}

export const CHANGELOG: ChangelogEntry[] = [
  {
    version: '1.1',
    date: '23/10/2026',
    title: 'Pantalla de Inicio, nuevas reglas de asistencia y celular rediseñado',
    added: [
      'Pantalla de Inicio: resume el día (fichaje, asistencia, agenda de clases y ensayos) y avisa lo importante: conflictos del cronograma, agentes que llegaron al tope de IJ y stock bajo.',
      'Código IJ (Inasistencia Justificada) en la planilla BOP y en la asistencia diaria. Regla: máximo 2 por mes y 10 por año para cada agente; si se supera, el sistema no deja guardarla y explica por qué.',
      'Estado «Adecuación de tareas» (AT) para bailarines que trabajan en otra área: cuenta como presente, no como falta.',
      'Menús ordenados en la versión de PC: Documentos, Depósito y personal, y Más (sincronización, licencia, ayuda).',
      'Celular: menú «Menú» con Sincronizar por Wi-Fi, Escáner, Licencia y Ayuda (antes no se podía llegar a la sincronización desde el celular).',
      'Sincronización Wi-Fi con clave compartida entre PC y celular, para que nadie más de la red pueda leer ni modificar los datos.',
      'Planilla BOP: las anotaciones del mes (feriados, ensayo suspendido, gira) se leen completas arriba de la tabla y en la columna del día.',
      'Actualizador para Windows: permite instalar una nueva versión sin perder los datos, con copia de seguridad previa.',
    ],
    fixed: [
      'Lo cargado en la planilla BOP (EI, SP, LC, IJ…) ahora se ve igual en la asistencia diaria; antes aparecía como «?».',
      '«Sin parte» ya no baja el porcentaje de presentismo de la planilla mensual.',
      'Planilla BOP en celular: la columna de nombres queda fija, se desliza de costado y cada celda se toca para elegir el código.',
      'Textos en español en toda la aplicación (Sala 1, Ordinario, Bailarín/a) en lugar de códigos internos.',
      'Letras más grandes, botones más cómodos para el dedo y ventanas a pantalla completa en el celular.',
      'El planificador semanal ya no queda cortado ni tapado al final de la pantalla.',
    ],
  },
];

export const CURRENT_VERSION = CHANGELOG[0].version;
