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
    version: '1.5',
    date: '01/11/2026',
    title: 'Celular más cómodo, sincronización más segura y planillas corregidas',
    added: [
      'Celular: la barra de abajo usa íconos, el padrón y el inventario se ven como tarjetas, y desde el menú se puede usar a pantalla completa con la barra del teléfono escondida.',
      'La categoría de revista se escribe libremente (por ejemplo, contratos de extras): se crea la primera vez que se usa y viaja en la sincronización. El formulario de personal permite cambiarla.',
      'Planilla de ausentismo BOP: abre en el mes en curso, la semana tiene siempre 7 días (lunes a domingo) aunque cambie el mes, y el miércoles se marca con X para no repetir la M del martes.',
      'La vista previa de impresión se lee en fondo claro en todas las planillas, y el manual tiene letra más grande y más aire entre párrafos.',
    ],
    fixed: [
      'La sincronización por Wi-Fi ya no dice «exitosa» cuando el otro equipo no pudo guardar los datos: muestra el motivo.',
      'Si la PC y el celular abrieron el parte del mismo día por separado, al sincronizar se unen en uno en lugar de trabar la sincronización.',
      'La contraseña de la Bóveda ya no se puede cambiar desde la aplicación sin credenciales. El reseteo administrativo queda en el instalador, con la llave maestra del pendrive.',
      'Importar el padrón desde un archivo volvió a funcionar.',
      'El catálogo de stock bajo del celular volvió a funcionar.',
      'Las entregas de valeras usan la temporada en curso (antes quedaban siempre en 2026).',
      'El actualizador de Windows ya no rechaza bases creadas por la versión publicada en GitHub.',
      'El distintivo de jubilado desaparece en cuanto se carga el estado del día.',
      'En la planilla de asistencia, un nombre de categoría largo ya no corre el borde; la planilla de reparto de obra vuelve a abrir su vista previa al imprimir.',
    ],
  },
  {
    version: '1.4',
    date: '29/10/2026',
    title: 'Planilla de ausentismo por semana, mes y año; vista previa editable',
    added: [
      'La planilla de ausentismo BOP tiene tres vistas: Semana, Mes y Año. La semana va de lunes a domingo y se puede avanzar de una a otra aunque cambie el mes; los totales se calculan sobre lo que se ve. El Año resume los 12 meses con los días de ausencia de cada bailarín y sus totales, y un clic en el mes lo abre. Exportar Excel e Imprimir respetan la vista elegida.',
      'Al imprimir un parte se abre una vista previa donde se puede corregir cualquier dato antes de mandarlo a la impresora; la certificación de solistas guarda los cambios en el registro.',
    ],
    fixed: [
      'Se recuperaron los colores de la interfaz (acento ámbar, estados en verde, celeste y rojo) y el fondo de los menús desplegables, que se veían transparentes.',
      'La cruz de cerrar queda siempre en la esquina superior derecha de todas las ventanas, y el título de la planilla de ausentismo ya no se comprime.',
    ],
  },
  {
    version: '1.3',
    date: '23/10/2026',
    title: 'Sonidista a cargo en clases, ensayos y funciones',
    added: [
      'Cada clase, ensayo o función puede indicar el sonidista a cargo. Aparece en las tarjetas, en el parte oficial, en el plan semanal impreso y en la agenda de Inicio, para pasarle las planillas al jefe de sonido.',
      'Si el mismo sonidista queda asignado a dos lugares al mismo tiempo, la auditoría del cronograma lo marca como conflicto.',
      'Las clases matutinas ahora también se sincronizan entre la PC y el celular.',
      'Al duplicar un día se copia también el sonidista.',
    ],
    fixed: [
      'Un ensayo nuevo ya no viene con bailarines, maestro y pasaje de ejemplo precargados: si no se elige reparto, queda sin reparto en lugar de asignar a la primera bailarina de la lista.',
      'Guardar un ensayo ya no le cambia la obra por «El lago de los cisnes»: se conserva la obra y el pasaje que tenía.',
      'El parte oficial ya no muestra un reparto de ejemplo cuando el ensayo no tiene reparto cargado: dice «Reparto a confirmar».',
    ],
  },
  {
    version: '1.2',
    date: '23/10/2026',
    title: 'Actualización automática',
    added: [
      'La aplicación busca versiones nuevas por sí sola (una vez por día) y muestra las notas de cada versión antes de instalar.',
      'Menú Más > Actualizaciones: buscar a mano, cargar el token de acceso una sola vez y quitarlo si hace falta.',
      'En Windows se descarga el actualizador, se verifica su huella SHA-256 y se ejecuta con copia de seguridad previa. En el celular se descarga el APK y se abre el instalador de Android.',
    ],
    fixed: [],
  },
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
