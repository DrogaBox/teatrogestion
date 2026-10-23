<script lang="ts">
  import { CHANGELOG } from '../../changelog';
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
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-5 bg-black/80 backdrop-blur-sm animate-fade-in">
    <div class="relative w-full max-w-5xl max-h-[92vh] flex flex-col bg-[#141416] border border-[#2d2d38] rounded-xl shadow-2xl overflow-hidden text-[#e0e0e0]">
      
      <!-- Encabezado Modal -->
      <div class="flex items-center justify-between px-6 py-4 bg-[#18181e] border-b border-[#2d2d38]">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded bg-[#ffb31a]/10 border border-[#ffb31a]/30 flex items-center justify-center text-[#ffb31a] font-mono font-bold text-sm">
            ?
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-bold text-white tracking-wide">CENTRO DE AYUDA Y MANUAL OFICIAL</h2>
              <span class="px-2 py-0.5 rounded text-[10px] font-mono font-semibold bg-[#262630] text-[#00e676] border border-[#343440]">
                v{CHANGELOG[0].version} Producción 2026
              </span>
            </div>
            <p class="text-xs text-[#8e8e9f]">Ballet Oficial de Córdoba — Teatro del Libertador San Martín</p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <button
            onclick={onClose}
            class="w-8 h-8 rounded bg-[#24242c] hover:bg-[#343440] text-[#8e8e9f] hover:text-white flex items-center justify-center text-sm font-bold transition-colors"
            title="Cerrar ventana"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Barra de Estado Institucional & Búsqueda -->
      <div class="px-6 py-2.5 bg-[#121214] border-b border-[#22222a] flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <span class="text-[11px] font-medium text-[#ffb31a] italic tracking-wide">
          ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
        </span>

        <div class="relative w-full sm:w-72">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Buscar tema (ej: solísticos, bóveda, S/P, DNI)..."
            class="w-full bg-[#1c1c22] border border-[#33333f] focus:border-[#ffb31a] rounded px-3 py-1.5 text-xs text-white placeholder-[#606070] focus:outline-none transition-colors"
          />
          {#if searchQuery}
            <button
              onclick={() => searchQuery = ''}
              class="absolute right-2 top-1.5 text-xs text-[#808090] hover:text-white"
            >
              ✕
            </button>
          {/if}
        </div>
      </div>

      <!-- Cuerpo del Manual: Menú Lateral + Contenido Detallado -->
      <div class="flex-1 flex flex-col md:flex-row overflow-hidden">
        
        <!-- Menú Lateral -->
        <div class="w-full md:w-64 bg-[#16161a] border-b md:border-b-0 md:border-r border-[#262630] overflow-y-auto p-3 space-y-1">
          <p class="text-[10px] font-mono uppercase tracking-wider text-[#606070] px-2 py-1">Capítulos Oficiales</p>
          {#each topics as t}
            <button
              onclick={() => activeTopic = t.id as any}
              class="w-full text-left px-3 py-2 rounded-lg text-xs font-medium transition-colors flex items-center justify-between {activeTopic === t.id ? 'bg-[#ffb31a] text-[#121214] font-bold shadow-sm' : 'text-[#a0a0b0] hover:bg-[#202026] hover:text-white'}"
            >
              <span class="truncate">{t.label}</span>
              <span class="text-[9px] px-1.5 py-0.5 rounded font-mono font-normal {activeTopic === t.id ? 'bg-[#121214]/20 text-[#121214]' : 'bg-[#22222a] text-[#707080]'}">
                {t.badge}
              </span>
            </button>
          {/each}

          <div class="pt-4 px-2">
            <div class="p-3 bg-[#1c1c24] border border-[#2d2d38] rounded-lg text-[11px] space-y-2">
              <p class="font-bold text-[#ffb31a]">Manual Impreso en PDF</p>
              <p class="text-[#888898] leading-relaxed">
                El pendrive de instalación incluye el documento oficial de 8 páginas:
              </p>
              <p class="font-mono text-[10px] text-white bg-[#101012] p-1.5 rounded border border-[#2a2a34]">
                Manual de Usuario TeatroGestion.pdf
              </p>
            </div>
          </div>
        </div>

        <!-- Panel de Lectura / Detalle -->
        <div class="flex-1 overflow-y-auto p-6 space-y-6 bg-[#141416] text-sm leading-relaxed">

          <!-- CAPÍTULO 1 -->
          {#if activeTopic === 'intro' && matchesSearch('inicio instalador grafico consola licencia offline windows 11')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#ffb31a] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">1. Inicio y Arquitectura Offline en Windows 11</h3>
                <p class="text-xs text-[#8e8e9f]">Soberanía absoluta, cero servidores en la nube y licencias Ed25519.</p>
              </div>

              <div class="p-3.5 bg-[#1a1a22] border border-[#2d2d38] rounded-lg space-y-2 text-xs">
                <p class="font-bold text-[#ffb31a]">Filosofía Técnica Teatral</p>
                <p class="text-[#c0c0d0]">
                  TeatroGestión está diseñado para funcionar en cabinas de sonido, salas de ensayo y depósitos sin requerir conexión a internet. La base de datos es un archivo local SQLite con modo WAL (Write-Ahead Logging) que garantiza protección contra cortes imprevistos de electricidad.
                </p>
              </div>

              <div class="space-y-2">
                <h4 class="text-xs font-bold text-white uppercase tracking-wider text-[#a0a0b0]">Opciones Duales de Instalador en el Pendrive</h4>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  <div class="p-3 bg-[#18181e] border border-[#2d2d38] rounded-lg space-y-1.5">
                    <span class="font-bold text-[#00e676] font-mono text-[11px]">Instalar_TeatroGestion_Grafico.bat</span>
                    <p class="text-[#9090a0]">
                      Abre la ventana gráfica moderna en Windows 11 con pestañas para seleccionar destino, duración de licencia, puesto, contraseña maestra y clave de la inspectora.
                    </p>
                  </div>
                  <div class="p-3 bg-[#18181e] border border-[#2d2d38] rounded-lg space-y-1.5">
                    <span class="font-bold text-[#ffb31a] font-mono text-[11px]">Instalar_TeatroGestion_Consola.bat</span>
                    <p class="text-[#9090a0]">
                      Ejecuta el asistente interactivo en terminal para administradores y técnicos con menú numérico paso a paso.
                    </p>
                  </div>
                </div>
              </div>

              <div class="p-3 bg-[#241a1a] border border-[#ff5252]/40 rounded-lg text-xs space-y-1 text-[#ff8a80]">
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
              <div class="border-l-4 border-[#ffb31a] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">2. Padrón del Personal y DNI Oficial</h3>
                <p class="text-xs text-[#8e8e9f]">Control unificado de bailarines, maestros, kinesiólogos e inspectora.</p>
              </div>

              <div class="space-y-3 text-xs">
                <p class="text-[#c0c0d0]">
                  El Padrón del Personal centraliza la nómina de todos los integrantes del Ballet Oficial de Córdoba. Cada ficha cuenta con:
                </p>
                <ul class="list-disc pl-5 space-y-1 text-[#a0a0b0]">
                  <li><strong class="text-white">DNI (Documento Nacional de Identidad):</strong> Campo obligatorio para partes legales y liquidaciones de plus por función.</li>
                  <li><strong class="text-white">Nombre Completo y Alias:</strong> Identificación artística y formal.</li>
                  <li><strong class="text-white">Cargos Oficiales:</strong> Bailarín/a, Maestro/a de Baile, Kinesiólogo/a, Inspector/a de Ballet, Director/a Artístico/a.</li>
                  <li><strong class="text-white">Estado Activo / Licencia:</strong> Permite pausar temporalmente a bailarines en reposo médico prolongado sin borrarlos de los históricos.</li>
                </ul>
              </div>

              <div class="p-3 bg-[#181820] border border-[#2d2d3c] rounded-lg text-xs space-y-1">
                <p class="font-bold text-[#ffb31a]">Integración con Módulos de Reparto y Ausentismo</p>
                <p class="text-[#9090a0]">
                  Al editar cualquier solístico o tomar asistencia, el sistema consulta en tiempo real este padrón para que no tenga que escribir nombres ni documentos a mano.
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 3 -->
          {#if activeTopic === 'regime' && matchesSearch('regimen descanso 15 minutos francos compensatorios direccion supervisora')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#ffb31a] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">3. Régimen Diario & Francos Compensatorios</h3>
                <p class="text-xs text-[#8e8e9f]">Normativa de descanso entre clase y ensayo, y arreglos de francos post función.</p>
              </div>

              <div class="p-3 bg-[#181820] border border-[#2d2d38] rounded-lg space-y-2 text-xs">
                <p class="font-bold text-[#ffb31a]">Intervalo de Descanso Obligatorio (15 minutos)</p>
                <p class="text-[#c0c0d0]">
                  Entre el cierre de la clase matutina de barra/centro y el comienzo del primer bloque de ensayo general, el sistema exige un receso mínimo de 15 minutos para recuperación física e hidratación del cuerpo de baile.
                </p>
              </div>

              <div class="space-y-2 text-xs">
                <h4 class="font-bold text-white">Selector de Francos Compensatorios (1 a 4 Días)</h4>
                <p class="text-[#9090a0]">
                  Dado que los días de franco posterior a una función oficial o gira provincial surgen de arreglos internos entre la Supervisora / Inspectora y la Dirección Artística, la inspectora puede elegir directamente la cantidad de días:
                </p>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 pt-1 font-mono text-center">
                  <div class="p-2 bg-[#202028] border border-[#333344] rounded text-white">1 Día de Franco</div>
                  <div class="p-2 bg-[#202028] border border-[#333344] rounded text-white">2 Días de Franco</div>
                  <div class="p-2 bg-[#202028] border border-[#333344] rounded text-white">3 Días de Franco</div>
                  <div class="p-2 bg-[#202028] border border-[#333344] rounded text-white">4 Días de Franco</div>
                </div>
                <p class="text-[#707080] text-[11px] pt-1">
                  Se activa desde la barra superior de "Régimen Diario" con un simple clic y queda registrado en el parte.
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 4 -->
          {#if activeTopic === 'attendance' && matchesSearch('ausentismo sin parte sp fisioterapia matriz bop')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#ffb31a] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">4. Control de Ausentismo & Estado "Sin Parte" (S/P)</h3>
                <p class="text-xs text-[#8e8e9f]">Toma durante la clase matutina y códigos oficiales en la Matriz BOP.</p>
              </div>

              <div class="p-3 bg-[#1c1424] border border-[#b388ff]/30 rounded-lg space-y-2 text-xs">
                <div class="flex items-center gap-2">
                  <span class="px-2 py-0.5 bg-[#b388ff] text-[#121214] font-bold rounded font-mono text-[10px]">S/P</span>
                  <span class="font-bold text-[#b388ff]">El Estado Especial: "Sin Parte" (Sin Convocar)</span>
                </div>
                <p class="text-[#d0c0e0]">
                  En jornadas donde ciertos bailarines del padrón no fueron convocados para el repertorio de ensayo, se les asigna el estado <strong>Sin parte (S/P)</strong>. Esto evita que figuren como inasistencia injustificada o ausente, garantizando un parte fidedigno y transparente.
                </p>
              </div>

              <div class="space-y-2 text-xs">
                <h4 class="font-bold text-white uppercase tracking-wider text-[#a0a0b0]">Códigos de la Matriz Mensual BOP</h4>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#00e67633;color:#00e676">P</span>
                    <span class="text-[#c0c0d0]">Presente (también cuenta si llegó tarde)</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#2ed6e833;color:#2ed6e8">EI</span>
                    <span class="text-[#c0c0d0]">Enfermedad inculpable / carpeta médica</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#f43f5e33;color:#f43f5e">SP</span>
                    <span class="text-[#c0c0d0]">Sin presencia: inasistencia injustificada</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#b388ff33;color:#b388ff">S/P</span>
                    <span class="text-[#c0c0d0]">Sin parte: no fue convocado, no es falta</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#ffb31a33;color:#ffb31a">LC</span>
                    <span class="text-[#c0c0d0]">Licencia compensatoria / franco</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#ff7b0033;color:#ff7b00">ART</span>
                    <span class="text-[#c0c0d0]">Accidente laboral</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#c084fc33;color:#c084fc">LA</span>
                    <span class="text-[#c0c0d0]">Licencia anual / vacaciones</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#5fa8ff33;color:#5fa8ff">IJ</span>
                    <span class="text-[#c0c0d0]">Inasistencia justificada: máximo 2 por mes y 10 por año</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#5fa8ff33;color:#5fa8ff">AT</span>
                    <span class="text-[#c0c0d0]">Adecuación de tareas: trabaja en otra área, cuenta como presente</span>
                  </div>
                  <div class="p-2 bg-[#18181e] border border-[#2d2d38] rounded flex items-center gap-2">
                    <span class="min-w-6 h-6 px-1 rounded font-bold flex items-center justify-center font-mono" style="background:#82828a33;color:#82828a">ES</span>
                    <span class="text-[#c0c0d0]">Ensayo suspendido</span>
                  </div>
                </div>
                <p class="text-[#9090a0]">En la PC: clic en la celda para pasar de código en código, clic derecho para elegir uno. En el celular: tocá la celda y elegí de la lista. Tocá el número de un día para anotar feriado, gira o ensayo suspendido.</p>
              </div>

              <div class="p-3 bg-[#14202a] border border-[#5fa8ff]/30 rounded-lg space-y-1 text-xs">
                <p class="font-bold text-[#5fa8ff]">Regla de las IJ (Inasistencias Justificadas)</p>
                <p class="text-[#b8d4ee]">Cada agente puede tener hasta <strong>2 IJ por mes</strong> y <strong>10 por año</strong>. Si se intenta cargar una más, el sistema no la guarda y avisa qué tope se alcanzó. La regla vale tanto en la planilla BOP como en el botón «Justificada» de la asistencia diaria. En Inicio aparece un aviso cuando alguien llega al tope del mes.</p>
              </div>

              <div class="p-3 bg-[#14202a] border border-[#5fa8ff]/30 rounded-lg space-y-1 text-xs">
                <p class="font-bold text-[#5fa8ff]">Adecuación de tareas (AT)</p>
                <p class="text-[#b8d4ee]">Para bailarines que trabajan en otra área del teatro. Se marca con el botón «Adecuación» de la asistencia diaria o con el código AT en la planilla. Cuenta como día presente: no es falta ni baja el presentismo.</p>
              </div>

              <div class="p-3 bg-[#181820] border border-[#2d2d38] rounded-lg text-xs space-y-1">
                <p class="font-bold text-[#ffb31a]">Fisioterapia y Control de Lesiones</p>
                <p class="text-[#9090a0]">
                  Los kinesiólogos del teatro pueden registrar sesiones terapéuticas, dolor articular, zonas afectadas y aptitud física sin alterar el parte de la inspectora.
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 5 -->
          {#if activeTopic === 'casting' && matchesSearch('repartos obras solisticos padron certificados pasajes')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#ffb31a] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">5. Planificación de Repartos & Certificaciones de Solísticos</h3>
                <p class="text-xs text-[#8e8e9f]">Vinculación directa con el padrón y antecedentes curriculares.</p>
              </div>

              <div class="space-y-3 text-xs text-[#c0c0d0]">
                <p>
                  El sistema permite desglosar las obras de la temporada en actos y pasajes coreográficos (Pas de Deux, Variaciones, Cuerpo de Baile).
                </p>
                <div class="p-3 bg-[#181822] border border-[#2d2d38] rounded-lg space-y-2">
                  <p class="font-bold text-[#ffb31a]">Asignación Rápida desde el Padrón</p>
                  <p class="text-[#9090a0]">
                    Al dar de alta o editar un rol solístico o reparto de estudio, se despliega automáticamente la lista de bailarines del padrón. Con solo seleccionarlo, se copian sus datos oficiales (nombre y DNI) asegurando coherencia documental.
                  </p>
                </div>
                <div class="p-3 bg-[#181822] border border-[#2d2d38] rounded-lg space-y-2">
                  <p class="font-bold text-[#ffb31a]">Certificaciones Oficiales de Solistas</p>
                  <p class="text-[#9090a0]">
                    Permite emitir certificados firmados digitalmente para que los bailarines acrediten sus roles principales ante la Agencia Córdoba Cultura o en concursos de carrera artística.
                  </p>
                </div>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 6 -->
          {#if activeTopic === 'vault' && matchesSearch('boveda documental carina lion certificados caja chica pesos ars memos')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#b388ff] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">6. Bóveda Documental de Inspección (Carina Lion)</h3>
                <p class="text-xs text-[#8e8e9f]">Apartado seguro para certificados médicos, facturas de caja chica y memos.</p>
              </div>

              <div class="space-y-3 text-xs">
                <div class="p-3.5 bg-[#1a1424] border border-[#b388ff]/30 rounded-lg space-y-2">
                  <p class="font-bold text-[#b388ff]">Seguridad Reforzada con Clave y Bloqueo</p>
                  <p class="text-[#d0c0e0]">
                    La Bóveda está protegida por una contraseña salteada criptográficamente (SHA-256) exclusiva de la inspectora. Cuenta con un temporizador automático de inactividad (1 min, 5 min, 15 min, 30 min o Nunca) que oculta los documentos de inmediato si la inspectora se retira de la computadora.
                  </p>
                </div>

                <div class="space-y-2">
                  <h4 class="font-bold text-white uppercase tracking-wider text-[#a0a0b0]">Documentación Respaldable</h4>
                  <ul class="list-disc pl-5 space-y-1.5 text-[#a0a0b0]">
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
              <div class="border-l-4 border-[#ff9800] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">7. Recuperación Administrativa de Contraseña</h3>
                <p class="text-xs text-[#8e8e9f]">Reseteo de la Bóveda desde el pendrive USB mediante la llave maestra.</p>
              </div>

              <div class="space-y-3 text-xs text-[#c0c0d0]">
                <p>
                  Si la inspectora olvida su contraseña, el administrador que posee el pendrive USB puede restablecerla sin perder ninguno de los documentos almacenados:
                </p>
                <div class="p-3.5 bg-[#181820] border border-[#2d2d38] rounded-lg space-y-2">
                  <p class="font-bold text-[#ff9800]">Pasos en Modo Gráfico (Recomendado):</p>
                  <ol class="list-decimal pl-5 space-y-1 text-[#a0a0b0]">
                    <li>Inserte el pendrive USB en la computadora del teatro.</li>
                    <li>Ejecute <span class="font-mono text-white">Instalar_TeatroGestion_Grafico.bat</span>.</li>
                    <li>Haga clic en la pestaña <strong class="text-white">"Recuperar Contraseña de Bóveda"</strong>.</li>
                    <li>Ingrese la Contraseña Maestra del Administrador (valida <span class="font-mono text-white">master.key.enc</span>).</li>
                    <li>Escriba la nueva contraseña para la inspectora y confirme.</li>
                    <li>Presione <strong class="text-white">[ RESTABLECER CONTRASEÑA DE BÓVEDA ]</strong>.</li>
                  </ol>
                </div>
                <p class="text-[#808090] text-[11px]">
                  También disponible en modo consola ejecutando <span class="font-mono text-white">Instalar_TeatroGestion_Consola.bat</span> y seleccionando la opción [3].
                </p>
              </div>
            </div>
          {/if}

          <!-- CAPÍTULO 8 -->
          {#if activeTopic === 'sync_mobile' && matchesSearch('sincronizacion red lan wifi android celular tablet gaynor')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#00e676] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">8. Sincronización en Red Local & Versión Android</h3>
                <p class="text-xs text-[#8e8e9f]">Operación móvil en sala de ensayo y escaneo de códigos de barra.</p>
              </div>

              <div class="space-y-3 text-xs text-[#c0c0d0]">
                <div class="p-3 bg-[#142018] border border-[#00e676]/30 rounded-lg space-y-2">
                  <p class="font-bold text-[#00e676]">Sincronización P2P sin Internet</p>
                  <p class="text-[#b0d8c0]">
                    Si la PC de cabina y el teléfono o tablet de inspección están conectados a la misma red Wi-Fi o switch local, pueden sincronizarse en un clic desde el menú <strong>Más → Sincronizar por Wi-Fi</strong> (en el celular: <strong>Menú → Sincronizar por Wi-Fi</strong>). Los dos equipos deben tener la <strong>misma clave de sincronización</strong>: se define en esa misma ventana y evita que otras personas de la red lean o modifiquen los datos.
                  </p>
                </div>

                <div class="p-3 bg-[#181820] border border-[#2d2d38] rounded-lg space-y-2">
                  <p class="font-bold text-[#ffb31a]">App Móvil para Teléfonos Samsung Galaxy A53 y Tablets</p>
                  <p class="text-[#9090a0]">
                    El pendrive contiene el paquete instalador <span class="font-mono text-white">TeatroGestion.apk</span>. Permite tomar asistencia caminando por la barra y utilizar la cámara del teléfono para escanear las etiquetas de las cajas de zapatillas Gaynor Minden del depósito.
                  </p>
                </div>
              </div>
            </div>
          {/if}


          <!-- CAPÍTULO 9 -->
          {#if activeTopic === 'home' && matchesSearch('inicio resumen dia avisos alertas agenda asistencia')}
            <div class="space-y-4">
              <div class="border-l-4 border-[#ffb31a] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">9. Pantalla de Inicio</h3>
                <p class="text-xs text-[#8e8e9f]">Lo importante del día en una sola vista.</p>
              </div>
              <div class="space-y-3 text-xs text-[#c0c0d0]">
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
              <div class="border-l-4 border-[#00e676] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">10. Actualizaciones automáticas</h3>
                <p class="text-xs text-[#8e8e9f]">La aplicación avisa cuando hay una versión nueva y muestra qué cambia.</p>
              </div>
              <div class="space-y-3 text-xs text-[#c0c0d0]">
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
              <div class="border-l-4 border-[#00e676] pl-3 py-0.5">
                <h3 class="text-base font-bold text-white">11. Novedades de cada versión</h3>
                <p class="text-xs text-[#8e8e9f]">Qué se agregó y qué se arregló. También está en el menú Más → Novedades.</p>
              </div>
              {#each CHANGELOG as v}
                <div class="space-y-2 text-xs text-[#c0c0d0]">
                  <p class="font-bold text-[#ffb31a]">Versión {v.version} · {v.date} — {v.title}</p>
                  <p class="font-bold text-[#00e676] uppercase tracking-wider">Nuevo</p>
                  <ul class="list-disc pl-5 space-y-1">{#each v.added as t}<li>{t}</li>{/each}</ul>
                  <p class="font-bold text-[#2ed6e8] uppercase tracking-wider">Arreglado y mejorado</p>
                  <ul class="list-disc pl-5 space-y-1">{#each v.fixed as t}<li>{t}</li>{/each}</ul>
                </div>
              {/each}
            </div>
          {/if}

        </div>
      </div>

      <!-- Pie de Modal -->
      <div class="px-6 py-3 bg-[#18181e] border-t border-[#2d2d38] flex items-center justify-between text-xs">
        <span class="text-[#707080] font-mono">
          Inspección del Ballet Oficial de Córdoba — Teatro del Libertador San Martín
        </span>
        <button
          onclick={onClose}
          class="px-4 py-1.5 bg-[#262630] hover:bg-[#343440] text-white font-bold rounded transition-colors"
        >
          Cerrar
        </button>
      </div>

    </div>
  </div>
{/if}
