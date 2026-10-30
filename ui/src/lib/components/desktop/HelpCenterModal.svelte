<script lang="ts">
  import { CHANGELOG } from '../../changelog';
  import { modalBehavior } from '../../actions/modalBehavior';
  let {
    isOpen,
    onClose,
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  // Búsqueda y Navegación
  let searchQuery = $state('');
  let activeTopic = $state<'intro' | 'personnel' | 'regime' | 'attendance' | 'casting' | 'vault' | 'admin_reset' | 'sync_mobile' | 'home' | 'news' | 'updates'>('intro');

  const topics = [
    { id: 'intro', label: '1. Inicio & Licencia Offline', badge: 'Windows 11' },
    { id: 'personnel', label: '2. Padrón del Personal & DNI', badge: 'Oficial' },
    { id: 'regime', label: '3. Régimen & Francos Compensatorios', badge: '15 min / 1-4 días' },
    { id: 'attendance', label: '4. Ausentismo & "Sin Parte" (S/P)', badge: 'Matriz BOP' },
    { id: 'casting', label: '5. Repartos & Certificación Solísticos', badge: 'Padrón vinculable' },
    { id: 'vault', label: '6. Bóveda Documental de Inspección', badge: 'Carina Lion' },
    { id: 'admin_reset', label: '7. Recuperar Contraseña (Admin)', badge: 'master.key.enc' },
    { id: 'sync_mobile', label: '8. Sincronización LAN & Android', badge: 'P2P / APK' },
    { id: 'home', label: '9. Pantalla de Inicio', badge: 'Resumen del día' },
    { id: 'updates', label: '10. Actualizaciones automáticas', badge: 'Más > Actualizaciones' },
    { id: 'news', label: '11. Novedades de la versión', badge: 'v' + CHANGELOG[0].version },
  ];

  function matchesSearch(text: string): boolean {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase().trim();
    return text.toLowerCase().includes(q);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-5 bg-black/80" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-5xl relative">

      <!-- Encabezado Modal -->
      <div class="flex px-6 py-4 bg-panel border-b border-raised items-start gap-3">
        <!-- cierre fijo en la esquina: el resto de la barra puede partirse en filas -->
        <div class="flex flex-1 min-w-0 flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded bg-amber/10 border border-amber/30 flex items-center justify-center text-amber font-mono font-bold text-sm">
              ?
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-base font-bold text-white tracking-wide">CENTRO DE AYUDA Y MANUAL OFICIAL</h2>
                <span class="px-2 py-0.5 rounded text-[10px] font-mono font-semibold bg-raised text-green border border-raised">
                  v{CHANGELOG[0].version} Producción 2026
                </span>
              </div>
              <p class="text-xs text-dim">Ballet Oficial de Córdoba — Teatro del Libertador San Martín</p>
            </div>
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

      <!-- Barra de Estado Institucional & Búsqueda -->
      <div class="px-6 py-2.5 bg-void border-b border-raised flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <span class="text-[11px] font-medium text-amber italic tracking-wide">
          ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
        </span>

        <div class="relative w-full sm:w-72">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Buscar tema (ej: solísticos, bóveda, S/P, DNI)..."
            class="w-full bg-void border border-raised focus:border-amber rounded px-3 py-1.5 text-xs text-white placeholder-dim focus:outline-none transition-colors"
          />
          {#if searchQuery}
            <button
              onclick={() => searchQuery = ''}
              class="absolute right-2 top-1.5 text-xs text-dim hover:text-white"
            >
              ✕
            </button>
          {/if}
        </div>
      </div>

      <!-- Cuerpo del Manual: Menú Lateral + Contenido Detallado -->
      <div class="flex-1 flex flex-col md:flex-row overflow-hidden">
        
        <!-- Menú Lateral -->
        <div class="w-full md:w-64 bg-void border-b md:border-b-0 md:border-r border-raised overflow-y-auto p-3 space-y-1">
          <p class="text-[10px] font-mono uppercase tracking-wider text-dim px-2 py-1">Capítulos Oficiales</p>
          {#each topics as t}
            <button
              onclick={() => activeTopic = t.id as any}
              class="w-full text-left px-3 py-2 rounded-lg text-xs font-medium transition-colors flex items-center justify-between {activeTopic === t.id ? 'bg-amber text-void font-bold shadow-sm' : 'text-dim hover:bg-raised hover:text-white'}"
            >
              <span class="truncate">{t.label}</span>
              <span class="text-[9px] px-1.5 py-0.5 rounded font-mono font-normal {activeTopic === t.id ? 'bg-void/20 text-void' : 'bg-raised text-dim'}">
                {t.badge}
              </span>
            </button>
          {/each}

          <div class="pt-4 px-2">
            <div class="p-3 bg-void border border-raised rounded-lg text-[11px] space-y-2">
              <p class="font-bold text-amber">Manual Impreso en PDF</p>
              <p class="text-dim leading-relaxed">
                El pendrive de instalación incluye el documento oficial de 8 páginas:
              </p>
              <p class="font-mono text-[10px] text-white bg-void p-1.5 rounded border border-raised">
                Manual de Usuario TeatroGestion.pdf
              </p>
            </div>
          </div>
        </div>

        <!-- Panel de Lectura / Detalle -->
        <div class="flex-1 overflow-y-auto p-6 space-y-6 bg-void text-sm leading-relaxed">

          <!-- CAPÍTULO 1 -->
          {#if activeTopic === 'intro' && matchesSearch('inicio instalador grafico consola licencia offline windows 11')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">1. Inicio y Arquitectura Offline en Windows 11</h3>
                <p class="text-xs text-dim">Soberanía absoluta, cero servidores en la nube y licencias Ed25519.</p>
              </div>

              <div class="p-3.5 bg-void border border-raised rounded-lg space-y-2 text-xs">
                <p class="font-bold text-amber">Filosofía Técnica Teatral</p>
                <p class="text-ink">
                  TeatroGestión está diseñado para funcionar en cabinas de sonido, salas de ensayo y depósitos sin requerir conexión a internet. La base de datos es un archivo local SQLite con modo WAL (Write-Ahead Logging) que garantiza protección contra cortes imprevistos de electricidad.
                </p>
              </div>

              <div class="space-y-2">
                <h4 class="text-xs font-bold text-white uppercase tracking-wider text-dim">Opciones Duales de Instalador en el Pendrive</h4>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  <div class="p-3 bg-panel border border-raised rounded-lg space-y-1.5">
                    <span class="font-bold text-green font-mono text-[11px]">Instalar_TeatroGestion_Grafico.bat</span>
                    <p class="text-dim">
                      Abre la ventana gráfica moderna en Windows 11 con pestañas para seleccionar destino, duración de licencia, puesto, contraseña maestra y clave de la inspectora.
                    </p>
                  </div>
                  <div class="p-3 bg-panel border border-raised rounded-lg space-y-1.5">
                    <span class="font-bold text-amber font-mono text-[11px]">Instalar_TeatroGestion_Consola.bat</span>
                    <p class="text-dim">
                      Ejecuta el asistente interactivo en terminal para administradores y técnicos con menú numérico paso a paso.
                    </p>
                  </div>
                </div>
              </div>

              <div class="p-3 bg-void border border-red/40 rounded-lg text-xs space-y-1 text-red">
                <p class="font-bold">Reloj Antifraude Monotónico</p>
                <p>
                  El sistema detecta manipulaciones intencionales en el reloj de la computadora. No modifique la fecha del sistema para evitar que el software pase a modo de bloqueo por seguridad.
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 2 -->
          {#if activeTopic === 'personnel' && matchesSearch('padron personal dni bailarines cargos altas')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">2. Padrón del Personal y DNI Oficial</h3>
                <p class="text-xs text-dim">Control unificado de bailarines, maestros, kinesiólogos e inspectora.</p>
              </div>

              <div class="space-y-3 text-xs">
                <p class="text-ink">
                  El Padrón del Personal centraliza la nómina de todos los integrantes del Ballet Oficial de Córdoba. Cada ficha cuenta con:
                </p>
                <ul class="list-disc pl-5 space-y-1 text-dim">
                  <li><strong class="text-white">DNI (Documento Nacional de Identidad):</strong> Campo obligatorio para partes legales y liquidaciones de plus por función.</li>
                  <li><strong class="text-white">Nombre Completo y Alias:</strong> Identificación artística y formal.</li>
                  <li><strong class="text-white">Cargos Oficiales:</strong> Bailarín/a, Maestro/a de Baile, Kinesiólogo/a, Inspector/a de Ballet, Director/a Artístico/a.</li>
                  <li><strong class="text-white">Estado Activo / Licencia:</strong> Permite pausar temporalmente a bailarines en reposo médico prolongado sin borrarlos de los históricos.</li>
                </ul>
              </div>

              <div class="p-3 bg-void border border-raised rounded-lg text-xs space-y-1">
                <p class="font-bold text-amber">Integración con Módulos de Reparto y Ausentismo</p>
                <p class="text-dim">
                  Al editar cualquier solístico o tomar asistencia, el sistema consulta en tiempo real este padrón para que no tenga que escribir nombres ni documentos a mano.
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 3 -->
          {#if activeTopic === 'regime' && matchesSearch('regimen descanso 15 minutos francos compensatorios direccion supervisora')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">3. Régimen Diario & Francos Compensatorios</h3>
                <p class="text-xs text-dim">Normativa de descanso entre clase y ensayo, y arreglos de francos post función.</p>
              </div>

              <div class="p-3 bg-void border border-raised rounded-lg space-y-2 text-xs">
                <p class="font-bold text-amber">Intervalo de Descanso Obligatorio (15 minutos)</p>
                <p class="text-ink">
                  Entre el cierre de la clase matutina de barra/centro y el comienzo del primer bloque de ensayo general, el sistema exige un receso mínimo de 15 minutos para recuperación física e hidratación del cuerpo de baile.
                </p>
              </div>

              <div class="space-y-2 text-xs">
                <h4 class="font-bold text-white">Selector de Francos Compensatorios (1 a 4 Días)</h4>
                <p class="text-dim">
                  Dado que los días de franco posterior a una función oficial o gira provincial surgen de arreglos internos entre la Supervisora / Inspectora y la Dirección Artística, la inspectora puede elegir directamente la cantidad de días:
                </p>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 pt-1 font-mono text-center">
                  <div class="p-2 bg-raised border border-raised rounded text-white">1 Día de Franco</div>
                  <div class="p-2 bg-raised border border-raised rounded text-white">2 Días de Franco</div>
                  <div class="p-2 bg-raised border border-raised rounded text-white">3 Días de Franco</div>
                  <div class="p-2 bg-raised border border-raised rounded text-white">4 Días de Franco</div>
                </div>
                <p class="text-dim text-[11px] pt-1">
                  Se activa desde la barra superior de "Régimen Diario" con un simple clic y queda registrado en el parte.
                </p>
              </div>
            </div>
          {/if}

          {#if activeTopic === 'regime' && matchesSearch('sonido sonidista cabina clases ensayos funciones planillas')}
            <div class="p-3 bg-void border border-cyan/30 rounded-lg space-y-1 text-xs">
              <p class="font-bold text-cyan">Sonidista a cargo</p>
              <p class="text-blue">Al crear o editar una clase, un ensayo o una función se puede elegir el <strong>sonidista a cargo</strong> (lista de personal con rol «Sonido» del padrón; si no hay ninguno, se da de alta en el Padrón). Se muestra en la tarjeta, en el parte oficial, en el plan semanal impreso y en la agenda de Inicio. Si el mismo sonidista queda en dos lugares a la vez, el cronograma lo avisa como conflicto.</p>
            </div>
          {/if}

          <!-- CAPÍTULO 4 -->
          {#if activeTopic === 'attendance' && matchesSearch('ausentismo sin parte sp fisioterapia matriz bop')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">4. Control de Ausentismo & Estado "Sin Parte" (S/P)</h3>
                <p class="text-xs text-dim">Toma durante la clase matutina y códigos oficiales en la Matriz BOP.</p>
              </div>

              <div class="p-3 bg-void border border-violet/30 rounded-lg space-y-2 text-xs">
                <div class="flex items-center gap-2">
                  <span class="px-2 py-0.5 bg-violet text-void font-bold rounded font-mono text-[10px]">S/P</span>
                  <span class="font-bold text-violet">El Estado Especial: "Sin Parte" (Sin Convocar)</span>
                </div>
                <p class="text-violet">
                  En jornadas donde ciertos bailarines del padrón no fueron convocados para el repertorio de ensayo, se les asigna el estado <strong>Sin parte (S/P)</strong>. Esto evita que figuren como inasistencia injustificada o ausente, garantizando un parte fidedigno y transparente.
                </p>
              </div>

              <div class="space-y-2 text-xs">
                <h4 class="font-bold text-white uppercase tracking-wider text-dim">Códigos de la Matriz Mensual BOP</h4>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#00e67633;color:#00e676">P</span>
                    <span class="text-ink">Presente (también cuenta si llegó tarde)</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#2ed6e833;color:#2ed6e8">EI</span>
                    <span class="text-ink">Enfermedad inculpable / carpeta médica</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#f43f5e33;color:#f43f5e">SP</span>
                    <span class="text-ink">Sin presencia: inasistencia injustificada</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#b388ff33;color:#b388ff">S/P</span>
                    <span class="text-ink">Sin parte: no fue convocado, no es falta</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#ffb31a33;color:#ffb31a">LC</span>
                    <span class="text-ink">Licencia compensatoria / franco</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#ff7b0033;color:#ff7b00">ART</span>
                    <span class="text-ink">Accidente laboral</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#c084fc33;color:#c084fc">LA</span>
                    <span class="text-ink">Licencia anual / vacaciones</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#5fa8ff33;color:#5fa8ff">IJ</span>
                    <span class="text-ink">Inasistencia justificada: máximo 2 por mes y 10 por año</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#5fa8ff33;color:#5fa8ff">AT</span>
                    <span class="text-ink">Adecuación de tareas: trabaja en otra área, cuenta como presente</span>
                  </div>
                  <div class="p-2 bg-panel border border-raised rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#82828a33;color:#82828a">ES</span>
                    <span class="text-ink">Ensayo suspendido</span>
                  </div>
                </div>
                <p class="text-dim">En la PC: clic en la celda para pasar de código en código, clic derecho para elegir uno. En el celular: tocá la celda y elegí de la lista. Tocá el número de un día para anotar feriado, gira o ensayo suspendido.</p>
              </div>

              <div class="p-3 bg-void border border-blue/30 rounded-lg space-y-1 text-xs">
                <p class="font-bold text-blue">Regla de las IJ (Inasistencias Justificadas)</p>
                <p class="text-blue">Cada agente puede tener hasta <strong>2 IJ por mes</strong> y <strong>10 por año</strong>. Si se intenta cargar una más, el sistema no la guarda y avisa qué tope se alcanzó. La regla vale tanto en la planilla BOP como en el botón «Justificada» de la asistencia diaria. En Inicio aparece un aviso cuando alguien llega al tope del mes.</p>
              </div>

              <div class="p-3 bg-void border border-blue/30 rounded-lg space-y-1 text-xs">
                <p class="font-bold text-blue">Adecuación de tareas (AT)</p>
                <p class="text-blue">Para bailarines que trabajan en otra área del teatro. Se marca con el botón «Adecuación» de la asistencia diaria o con el código AT en la planilla. Cuenta como día presente: no es falta ni baja el presentismo.</p>
              </div>

              <div class="p-3 bg-void border border-raised rounded-lg text-xs space-y-1">
                <p class="font-bold text-amber">Fisioterapia y Control de Lesiones</p>
                <p class="text-dim">
                  Los kinesiólogos del teatro pueden registrar sesiones terapéuticas, dolor articular, zonas afectadas y aptitud física sin alterar el parte de la inspectora.
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 5 -->
          {#if activeTopic === 'casting' && matchesSearch('repartos obras solisticos padron certificados pasajes')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">5. Planificación de Repartos & Certificaciones de Solísticos</h3>
                <p class="text-xs text-dim">Vinculación directa con el padrón y antecedentes curriculares.</p>
              </div>

              <div class="space-y-3 text-xs text-ink">
                <p>
                  El sistema permite desglosar las obras de la temporada en actos y pasajes coreográficos (Pas de Deux, Variaciones, Cuerpo de Baile).
                </p>
                <div class="p-3 bg-void border border-raised rounded-lg space-y-2">
                  <p class="font-bold text-amber">Asignación Rápida desde el Padrón</p>
                  <p class="text-dim">
                    Al dar de alta o editar un rol solístico o reparto de estudio, se despliega automáticamente la lista de bailarines del padrón. Con solo seleccionarlo, se copian sus datos oficiales (nombre y DNI) asegurando coherencia documental.
                  </p>
                </div>
                <div class="p-3 bg-void border border-raised rounded-lg space-y-2">
                  <p class="font-bold text-amber">Certificaciones Oficiales de Solistas</p>
                  <p class="text-dim">
                    Permite emitir certificados firmados digitalmente para que los bailarines acrediten sus roles principales ante la Agencia Córdoba Cultura o en concursos de carrera artística.
                  </p>
                </div>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 6 -->
          {#if activeTopic === 'vault' && matchesSearch('boveda documental carina lion certificados caja chica pesos ars memos')}
            <div class="space-y-4">
              <div class="border-l-4 border-violet pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">6. Bóveda Documental de Inspección (Carina Lion)</h3>
                <p class="text-xs text-dim">Apartado seguro para certificados médicos, facturas de caja chica y memos.</p>
              </div>

              <div class="space-y-3 text-xs">
                <div class="p-3.5 bg-void border border-violet/30 rounded-lg space-y-2">
                  <p class="font-bold text-violet">Seguridad Reforzada con Clave y Bloqueo</p>
                  <p class="text-violet">
                    La Bóveda está protegida por una contraseña salteada criptográficamente (SHA-256) exclusiva de la inspectora. Cuenta con un temporizador automático de inactividad (1 min, 5 min, 15 min, 30 min o Nunca) que oculta los documentos de inmediato si la inspectora se retira de la computadora.
                  </p>
                </div>

                <div class="space-y-2">
                  <h4 class="font-bold text-white uppercase tracking-wider text-dim">Documentación Respaldable</h4>
                  <ul class="list-disc pl-5 space-y-1.5 text-dim">
                    <li><strong class="text-white">Certificados Médicos:</strong> Reposos por desgarros, cirugías o bajas traumatológicas con fotografía o PDF adjunto.</li>
                    <li><strong class="text-white">Caja Chica ($ ARS) y Comprobantes:</strong> Registro de compras menores (panes de colofonia, cintas kinesiológicas, botiquín urgente) con monto exacto en pesos argentinos y nombre del comercio proveedor.</li>
                    <li><strong class="text-white">Memos de Dirección y Acuerdos:</strong> Disposiciones y notas de ensayo emitidas a horas intempestivas (ej: 22:52 hs).</li>
                    <li><strong class="text-white">Totalizador Automático:</strong> Suma dinámicamente los gastos de caja chica del mes para facilitar la rendición administrativa.</li>
                  </ul>
                </div>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 7 -->
          {#if activeTopic === 'admin_reset' && matchesSearch('recuperar contrasena admin master key enc pendrive')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">7. Recuperación Administrativa de Contraseña</h3>
                <p class="text-xs text-dim">Reseteo de la Bóveda desde el pendrive USB mediante la llave maestra.</p>
              </div>

              <div class="space-y-3 text-xs text-ink">
                <p>
                  Si la inspectora olvida su contraseña, el administrador que posee el pendrive USB puede restablecerla sin perder ninguno de los documentos almacenados:
                </p>
                <div class="p-3.5 bg-void border border-raised rounded-lg space-y-2">
                  <p class="font-bold text-amber">Pasos en Modo Gráfico (Recomendado):</p>
                  <ol class="list-decimal pl-5 space-y-1 text-dim">
                    <li>Inserte el pendrive USB en la computadora del teatro.</li>
                    <li>Ejecute <span class="font-mono text-white">Instalar_TeatroGestion_Grafico.bat</span>.</li>
                    <li>Haga clic en la pestaña <strong class="text-white">"Recuperar Contraseña de Bóveda"</strong>.</li>
                    <li>Ingrese la Contraseña Maestra del Administrador (valida <span class="font-mono text-white">master.key.enc</span>).</li>
                    <li>Escriba la nueva contraseña para la inspectora y confirme.</li>
                    <li>Presione <strong class="text-white">[ RESTABLECER CONTRASEÑA DE BÓVEDA ]</strong>.</li>
                  </ol>
                </div>
                <p class="text-dim text-[11px]">
                  También disponible en modo consola ejecutando <span class="font-mono text-white">Instalar_TeatroGestion_Consola.bat</span> y seleccionando la opción [3].
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 8 -->
          {#if activeTopic === 'sync_mobile' && matchesSearch('sincronizacion red lan wifi android celular tablet gaynor')}
            <div class="space-y-4">
              <div class="border-l-4 border-green pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">8. Sincronización en Red Local & Versión Android</h3>
                <p class="text-xs text-dim">Operación móvil en sala de ensayo y escaneo de códigos de barra.</p>
              </div>

              <div class="space-y-3 text-xs text-ink">
                <div class="p-3 bg-void border border-green/30 rounded-lg space-y-2">
                  <p class="font-bold text-green">Sincronización P2P sin Internet</p>
                  <p class="text-green">
                    Si la PC de cabina y el teléfono o tablet de inspección están conectados a la misma red Wi-Fi o switch local, pueden sincronizarse en un clic desde el menú <strong>Más → Sincronizar por Wi-Fi</strong> (en el celular: <strong>Menú → Sincronizar por Wi-Fi</strong>). Los dos equipos deben tener la <strong>misma clave de sincronización</strong>: se define en esa misma ventana y evita que otras personas de la red lean o modifiquen los datos.
                  </p>
                </div>

                <div class="p-3 bg-void border border-raised rounded-lg space-y-2">
                  <p class="font-bold text-amber">App Móvil para Teléfonos Samsung Galaxy A53 y Tablets</p>
                  <p class="text-dim">
                    El pendrive contiene el paquete instalador <span class="font-mono text-white">TeatroGestion.apk</span>. Permite tomar asistencia caminando por la barra y utilizar la cámara del teléfono para escanear las etiquetas de las cajas de zapatillas Gaynor Minden del depósito.
                  </p>
                </div>
              </div>
            </div>
          {/if}


          <!-- CAPÍTULO 9 -->
          {#if activeTopic === 'home' && matchesSearch('inicio resumen dia avisos alertas agenda asistencia')}
            <div class="space-y-4">
              <div class="border-l-4 border-amber pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">9. Pantalla de Inicio</h3>
                <p class="text-xs text-dim">Lo importante del día en una sola vista.</p>
              </div>
              <div class="space-y-3 text-xs text-ink">
                <p><strong class="text-white">Encabezado:</strong> fecha, régimen horario (Ordinario, Pre-función o Extraordinario), hora de fichaje y de cierre.</p>
                <p><strong class="text-white">Avisos:</strong> solo aparecen si hay algo que atender: conflictos del cronograma, agentes que llegaron al tope de IJ del mes, personas sin marcar en la asistencia de hoy, stock bajo y vencimiento de licencia. Cada aviso tiene un botón que lleva directo al lugar para resolverlo.</p>
                <p><strong class="text-white">Asistencia:</strong> cuántos llegaron, tardanzas, licencias, ausentes y fisioterapia, con los nombres de quienes necesitan atención.</p>
                <p><strong class="text-white">Agenda del día:</strong> clases y ensayos en orden de hora; si estás mirando el día de hoy se marca lo que está en curso (AHORA) y lo siguiente (PRÓXIMO).</p>
                <p><strong class="text-white">Accesos rápidos:</strong> tomar asistencia, planilla BOP, parte oficial e inventario.</p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 10 -->
          {#if activeTopic === 'updates' && matchesSearch('actualizaciones actualizar version nueva token github descargar instalar')}
            <div class="space-y-4">
              <div class="border-l-4 border-green pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">10. Actualizaciones automáticas</h3>
                <p class="text-xs text-dim">La aplicación avisa cuando hay una versión nueva y muestra qué cambia.</p>
              </div>
              <div class="space-y-3 text-xs text-ink">
                <p><strong class="text-white">Primera vez:</strong> menú <strong>Más → Actualizaciones</strong> y escribir el token de actualización que entrega el desarrollador. Se carga una sola vez en cada equipo; la aplicación comprueba que funcione antes de guardarlo.</p>
                <p><strong class="text-white">Después:</strong> la aplicación busca sola una vez por día. Si hay una versión nueva se abre una ventana con las notas y el botón «Descargar e instalar». También se puede buscar a mano desde el mismo menú.</p>
                <p><strong class="text-white">En la PC:</strong> se descarga el actualizador, se verifica y se ejecuta. TeatroGestión se cierra, se hace una copia de seguridad de los datos y se reemplaza solo el programa. La base de datos y la licencia no se tocan.</p>
                <p><strong class="text-white">En el celular:</strong> se descarga el APK y Android pide confirmar la instalación. La primera vez puede pedir permiso para instalar desde esta aplicación.</p>
                <p><strong class="text-white">Sin internet:</strong> no pasa nada, simplemente no busca. El resto del programa funciona igual.</p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 11 -->
          {#if activeTopic === 'news' && matchesSearch('novedades cambios version changelog arreglos')}
            <div class="space-y-5">
              <div class="border-l-4 border-green pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">11. Novedades de cada versión</h3>
                <p class="text-xs text-dim">Qué se agregó y qué se arregló. También está en el menú Más → Novedades.</p>
              </div>
              {#each CHANGELOG as v}
                <div class="space-y-2 text-xs text-ink">
                  <p class="font-bold text-amber">Versión {v.version} · {v.date} — {v.title}</p>
                  <p class="font-bold text-green uppercase tracking-wider">Nuevo</p>
                  <ul class="list-disc pl-5 space-y-1">{#each v.added as t}<li>{t}</li>{/each}</ul>
                  <p class="font-bold text-cyan uppercase tracking-wider">Arreglado y mejorado</p>
                  <ul class="list-disc pl-5 space-y-1">{#each v.fixed as t}<li>{t}</li>{/each}</ul>
                </div>
              {/each}
            </div>
          {/if}

        </div>
      </div>

      <!-- Pie de Modal -->
      <div class="px-6 py-3 bg-panel border-t border-raised flex items-center justify-between text-xs">
        <span class="text-dim font-mono">
          Inspección del Ballet Oficial de Córdoba — Teatro del Libertador San Martín
        </span>
        <button
          onclick={onClose}
          class="px-4 py-1.5 bg-raised hover:bg-raised text-white font-bold rounded transition-colors"
        >
          Cerrar
        </button>
      </div>

    </div>
  </div>
{/if}
