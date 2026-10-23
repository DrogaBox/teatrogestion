<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../../ipc/client';

  let {
    isOpen,
    onClose,
    onFinished
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
    onFinished: () => void;
  }>();

  let currentStep = $state<number>(1);
  const totalSteps = 5;

  // Paso 1: Identidad & Horarios
  let companyName = $state('Ballet Oficial de Córdoba');
  let theaterName = $state('Teatro del Libertador San Martín');
  let checkinTime = $state('08:45');
  let classTimes = $state('09:00 - 10:15');
  let shiftLimitOrdinary = $state('13:45');
  let shiftLimitPreFunction = $state('14:45');
  let intershiftRestHours = $state(12);
  let legalClause = $state('★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★');

  // Paso 2: Salas y Logística
  let enableSala1 = $state(true);
  let enableSala2 = $state(true);
  let enableSala3 = $state(true);
  let enableEscenario = $state(true);
  let enableTourLogistics = $state(true);

  // Paso 3: Staff Docente y Técnico
  let inspectorName = $state('Carina Lion (Inspección General)');
  let enableAnaniev = $state(true);
  let enableBenites = $state(true);
  let enablePino = $state(true);
  let soundSystem = $state<'teatrocue' | 'pianist' | 'mixed'>('mixed');

  // Paso 4: Obras Activas
  let activeWorks = $state([
    { id: 'WORK_LAGO_CISNES', title: 'El Lago de los Cisnes', selected: true },
    { id: 'WORK_DON_QUIJOTE', title: 'Don Quijote', selected: true },
    { id: 'WORK_GALA_CLASICA', title: 'Gala de Ballet Clásico (Diana y Acteón)', selected: true },
  ]);

  // Paso 5: Depósito y Escáner
  let pointeQuotaFirstFig = $state(24);
  let pointeQuotaSolista = $state(18);
  let pointeQuotaCorps = $state(14);
  let enableCameraScanner = $state(true);

  function nextStep() {
    if (currentStep < totalSteps) {
      currentStep++;
    } else {
      finishSetup();
    }
  }

  function prevStep() {
    if (currentStep > 1) {
      currentStep--;
    }
  }

  function finishSetup() {
    try {
      localStorage.setItem('tg_onboarding_completed', 'true');
      localStorage.setItem('tg_config_cache', JSON.stringify({
        companyName,
        theaterName,
        checkinTime,
        classTimes,
        shiftLimitOrdinary,
        shiftLimitPreFunction,
        intershiftRestHours,
        legalClause,
        enableTourLogistics,
        soundSystem,
        enableCameraScanner,
      }));
    } catch (e) {
      console.warn('No se pudo guardar en localStorage:', e);
    }
    onFinished();
    onClose();
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6">
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-2xl max-h-[92vh] flex flex-col p-5 md:p-6 shadow-2xl relative text-[#d8d8dc]">
      
      <!-- Encabezado del Asistente -->
      <div class="flex items-center justify-between pb-3 mb-3 border-b border-[#34343c]">
        <div>
          <h2 class="text-sm font-semibold text-[#d8d8dc] tracking-wide uppercase flex items-center gap-2">
            Asistente de Configuración Inicial
            <span class="text-[10px] bg-[#121214] text-[#ffb31a] border border-[#34343c] px-2 py-0.5 rounded font-mono">
              Paso {currentStep} de {totalSteps}
            </span>
          </h2>
          <p class="text-xs text-[#82828a] mt-0.5">
            Configuración básica de parámetros operativos y salas del teatro
          </p>
        </div>

        <button
          onclick={onClose}
          class="text-[#82828a] hover:text-white font-mono text-sm"
          title="Cerrar asistente"
        >
          ✕
        </button>
      </div>

      <!-- Barra de Progreso Superior -->
      <div class="w-full bg-[#121214] h-1.5 rounded mb-4 overflow-hidden border border-[#34343c]">
        <div
          class="bg-[#ffb31a] h-full transition-all duration-300"
          style="width: {(currentStep / totalSteps) * 100}%"
        ></div>
      </div>

      <!-- Contenido de Cada Paso -->
      <div class="flex-1 overflow-y-auto pr-1 space-y-4">
        
        <!-- =============================================================== -->
        <!-- PASO 1: Identidad Institucional & Régimen Horario               -->
        <!-- =============================================================== -->
        {#if currentStep === 1}
          <div class="space-y-4">
            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Institución Teatral</span>
              <p class="text-xs text-[#82828a] mb-3">Nombre formal de la compañía oficial y el teatro sede.</p>
              
              <div class="space-y-3">
                <div>
                  <label for="step1-company-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Cuerpo Estable</label>
                  <input
                    id="step1-company-input"
                    type="text"
                    bind:value={companyName}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-medium text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
                  />
                </div>
                <div>
                  <label for="step1-theater-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Teatro Sede</label>
                  <input
                    id="step1-theater-input"
                    type="text"
                    bind:value={theaterName}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-medium text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
                  />
                </div>
              </div>
            </div>

            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Parámetros Reglamentarios de Jornada</span>
              <p class="text-xs text-[#82828a] mb-3">Límites horarios y cláusulas de convenio colectivo.</p>

              <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div>
                  <label for="step1-checkin-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Fichaje Obligatorio</label>
                  <input
                    id="step1-checkin-input"
                    type="time"
                    bind:value={checkinTime}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#d8d8dc] font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
                <div>
                  <label for="step1-class-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Clases Técnicas</label>
                  <input
                    id="step1-class-input"
                    type="text"
                    bind:value={classTimes}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#2ed6e8] font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
                <div>
                  <label for="step1-shift-ord-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Fin Ordinario</label>
                  <input
                    id="step1-shift-ord-input"
                    type="time"
                    bind:value={shiftLimitOrdinary}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#3de06a] font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
                <div>
                  <label for="step1-shift-pre-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Fin Pre-Función</label>
                  <input
                    id="step1-shift-pre-input"
                    type="time"
                    bind:value={shiftLimitPreFunction}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#ffb31a] font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
              </div>

              <div class="mt-3">
                <label for="step1-clause-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Cláusula Invariable Oficial</label>
                <input
                  id="step1-clause-input"
                  type="text"
                  bind:value={legalClause}
                  class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#ffb31a] font-mono outline-none"
                  readonly
                />
              </div>
            </div>
          </div>

        <!-- =============================================================== -->
        <!-- PASO 2: Salas de Ensayo y Logística                            -->
        <!-- =============================================================== -->
        {:else if currentStep === 2}
          <div class="space-y-4">
            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Salas Activas del Teatro</span>
              <p class="text-xs text-[#82828a] mb-3">Salas donde se llevan a cabo las clases matutinas y ensayos de estudio.</p>

              <div class="space-y-2">
                <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                  <div class="flex items-center gap-2">
                    <span class="text-[#ffb31a] font-semibold text-xs">Sala 1</span>
                    <span class="text-[11px] text-[#82828a]">• Estudio Principal (Clases Técnicas)</span>
                  </div>
                  <input type="checkbox" bind:checked={enableSala1} class="h-4 w-4 rounded accent-[#ffb31a]" />
                </label>

                <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                  <div class="flex items-center gap-2">
                    <span class="text-[#3de06a] font-semibold text-xs">Sala 2</span>
                    <span class="text-[11px] text-[#82828a]">• Estudio Secundario (Solos y Pas de Deux)</span>
                  </div>
                  <input type="checkbox" bind:checked={enableSala2} class="h-4 w-4 rounded accent-[#ffb31a]" />
                </label>

                <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                  <div class="flex items-center gap-2">
                    <span class="text-[#2ed6e8] font-semibold text-xs">Sala 3</span>
                    <span class="text-[11px] text-[#82828a]">• Estudio de Varones / Clases Paralelas</span>
                  </div>
                  <input type="checkbox" bind:checked={enableSala3} class="h-4 w-4 rounded accent-[#ffb31a]" />
                </label>

                <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                  <div class="flex items-center gap-2">
                    <span class="text-purple-400 font-semibold text-xs">Escenario Principal</span>
                    <span class="text-[11px] text-[#82828a]">• Puesta en Escena y Ensayos Generales</span>
                  </div>
                  <input type="checkbox" bind:checked={enableEscenario} class="h-4 w-4 rounded accent-[#ffb31a]" />
                </label>
              </div>
            </div>

            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Módulo de Logística de Gira</span>
              <p class="text-xs text-[#82828a] mb-3">Habilita salas virtuales para salidas de micros interurbanos, escenarios externos y salas de calentamiento en gira.</p>

              <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                <div>
                  <div class="text-xs font-semibold text-[#d8d8dc]">Activar Soporte de Gira & Traslados</div>
                  <div class="text-[10px] text-[#2ed6e8] font-mono">GIRA_TRASLADO • GIRA_ESCENARIO • GIRA_SALA</div>
                </div>
                <input type="checkbox" bind:checked={enableTourLogistics} class="h-4 w-4 rounded accent-[#ffb31a]" />
              </label>
            </div>
          </div>

        <!-- =============================================================== -->
        <!-- PASO 3: Staff Docente y Acompañamiento                         -->
        <!-- =============================================================== -->
        {:else if currentStep === 3}
          <div class="space-y-4">
            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Inspección y Maestros</span>
              <p class="text-xs text-[#82828a] mb-3">Equipo docente a cargo de las citaciones y clases técnicas.</p>

              <div class="space-y-2">
                <div class="p-3 bg-[#1a1a1e] rounded border border-[#34343c]">
                  <label for="step3-insp-input" class="text-[10px] font-semibold uppercase tracking-wider text-[#82828a] block">Inspección General</label>
                  <input
                    id="step3-insp-input"
                    type="text"
                    bind:value={inspectorName}
                    class="w-full mt-1 bg-transparent text-xs font-semibold text-[#d8d8dc] outline-none"
                  />
                </div>

                <div class="grid grid-cols-3 gap-2 text-xs">
                  <label class="p-2.5 rounded bg-[#1a1a1e] border border-[#34343c] flex items-center justify-between cursor-pointer">
                    <span class="text-[#d8d8dc] text-xs font-medium">Mtro. Ananiev</span>
                    <input type="checkbox" bind:checked={enableAnaniev} class="rounded accent-[#ffb31a]" />
                  </label>
                  <label class="p-2.5 rounded bg-[#1a1a1e] border border-[#34343c] flex items-center justify-between cursor-pointer">
                    <span class="text-[#d8d8dc] text-xs font-medium">Mtro. Benítez</span>
                    <input type="checkbox" bind:checked={enableBenites} class="rounded accent-[#ffb31a]" />
                  </label>
                  <label class="p-2.5 rounded bg-[#1a1a1e] border border-[#34343c] flex items-center justify-between cursor-pointer">
                    <span class="text-[#d8d8dc] text-xs font-medium">Mtra. Pino</span>
                    <input type="checkbox" bind:checked={enablePino} class="rounded accent-[#ffb31a]" />
                  </label>
                </div>
              </div>
            </div>

            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Sistema de Acompañamiento Musical</span>
              <p class="text-xs text-[#82828a] mb-3">Modalidad habitual para las salas de ensayo.</p>

              <div class="grid grid-cols-3 gap-2">
                <button
                  type="button"
                  onclick={() => soundSystem = 'teatrocue'}
                  class="p-3 rounded border text-center transition-colors {soundSystem === 'teatrocue' ? 'bg-[#ffb31a]/15 border-[#ffb31a] text-[#ffb31a]' : 'bg-[#1a1a1e] border-[#34343c] text-[#82828a]'}"
                >
                  <div class="text-xs font-bold">TeatroCue</div>
                  <div class="text-[9px] text-[#82828a]">Pistas y Cues</div>
                </button>

                <button
                  type="button"
                  onclick={() => soundSystem = 'pianist'}
                  class="p-3 rounded border text-center transition-colors {soundSystem === 'pianist' ? 'bg-[#ffb31a]/15 border-[#ffb31a] text-[#ffb31a]' : 'bg-[#1a1a1e] border-[#34343c] text-[#82828a]'}"
                >
                  <div class="text-xs font-bold">Pianistas</div>
                  <div class="text-[9px] text-[#82828a]">Músicos Titulares</div>
                </button>

                <button
                  type="button"
                  onclick={() => soundSystem = 'mixed'}
                  class="p-3 rounded border text-center transition-colors {soundSystem === 'mixed' ? 'bg-[#ffb31a]/15 border-[#ffb31a] text-[#ffb31a]' : 'bg-[#1a1a1e] border-[#34343c] text-[#82828a]'}"
                >
                  <div class="text-xs font-bold">Mixto</div>
                  <div class="text-[9px] text-[#82828a]">Piano + TeatroCue</div>
                </button>
              </div>
            </div>
          </div>

        <!-- =============================================================== -->
        <!-- PASO 4: Obras & Repartos de Estudio                            -->
        <!-- =============================================================== -->
        {:else if currentStep === 4}
          <div class="space-y-4">
            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Obras en Temporada Activa</span>
              <p class="text-xs text-[#82828a] mb-3">Obras precargadas con sus pasajes y repartos oficiales (1°, 2° elenco y quienes estudian).</p>

              <div class="space-y-2">
                {#each activeWorks as w}
                  <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                    <div>
                      <div class="text-xs font-semibold text-[#d8d8dc]">{w.title}</div>
                      <div class="text-[10px] text-[#82828a]">Pasajes y elencos preconfigurados en SQLite WAL</div>
                    </div>
                    <input type="checkbox" bind:checked={w.selected} class="h-4 w-4 rounded accent-[#ffb31a]" />
                  </label>
                {/each}
              </div>
            </div>

            <div class="bg-[#121214] border border-[#34343c] p-3 rounded text-xs space-y-1">
              <span class="font-semibold text-[#ffb31a] block">
                Repartos Planificados en 1 Toque
              </span>
              <p class="text-[#82828a] leading-relaxed">
                Al armar los partes diarios de ensayo, el botón "Cargar Reparto Planificado" te permitirá elegir la obra y elenco para completar el pasaje, sonido y todos los bailarines citados automáticamente.
              </p>
            </div>
          </div>

        <!-- =============================================================== -->
        <!-- PASO 5: Depósito, Calzado y Escáner de Código de Barras       -->
        <!-- =============================================================== -->
        {:else}
          <div class="space-y-4">
            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Cupos Anuales de Puntas</span>
              <p class="text-xs text-[#82828a] mb-3">Valores reglamentarios de pares anuales por categoría.</p>

              <div class="grid grid-cols-3 gap-3">
                <div>
                  <label for="step5-fig-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">1° Figuras</label>
                  <input
                    id="step5-fig-input"
                    type="number"
                    bind:value={pointeQuotaFirstFig}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#ffb31a] text-center font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
                <div>
                  <label for="step5-sol-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Solistas</label>
                  <input
                    id="step5-sol-input"
                    type="number"
                    bind:value={pointeQuotaSolista}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#ffb31a] text-center font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
                <div>
                  <label for="step5-corps-input" class="block text-[10px] font-semibold uppercase tracking-wider text-[#82828a] mb-1">Cuerpo de Baile</label>
                  <input
                    id="step5-corps-input"
                    type="number"
                    bind:value={pointeQuotaCorps}
                    class="w-full h-10 px-3 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-semibold text-[#ffb31a] text-center font-mono outline-none focus:border-[#ffb31a]"
                  />
                </div>
              </div>
            </div>

            <div class="bg-[#121214] p-4 rounded border border-[#34343c]">
              <span class="text-xs font-semibold uppercase tracking-wider text-[#ffb31a] block mb-1">Escáner de Códigos de Barra</span>
              <p class="text-xs text-[#82828a] mb-3">Lectura con la cámara del dispositivo móvil o archivo de imagen en PC para ingresar stock, entregar o pedir a proveedores.</p>

              <label class="flex items-center justify-between p-3 rounded bg-[#1a1a1e] border border-[#34343c] cursor-pointer">
                <div>
                  <div class="text-xs font-semibold text-[#d8d8dc]">Habilitar Escaneo con Cámara & Pedidos Multimarca</div>
                  <div class="text-[10px] text-[#3de06a] font-mono">Bloch • Capezio • Gaynor Minden • Sansha • Freed</div>
                </div>
                <input type="checkbox" bind:checked={enableCameraScanner} class="h-4 w-4 rounded accent-[#ffb31a]" />
              </label>
            </div>
          </div>
        {/if}

      </div>

      <!-- Botonera Inferior de Navegación del Asistente -->
      <div class="flex items-center justify-between pt-3 mt-3 border-t border-[#34343c]">
        <button
          disabled={currentStep === 1}
          onclick={prevStep}
          class="px-4 py-2 rounded text-xs font-medium bg-[#24242a] hover:bg-[#34343c] text-[#82828a] hover:text-[#d8d8dc] disabled:opacity-30 disabled:pointer-events-none transition-colors"
        >
          ◀ Anterior
        </button>

        <div class="flex items-center gap-1.5">
          {#each Array(totalSteps) as _, idx}
            <span class="w-2 h-2 rounded-full transition-all {currentStep === idx + 1 ? 'bg-[#ffb31a]' : 'bg-[#34343c]'}"></span>
          {/each}
        </div>

        <button
          onclick={nextStep}
          class="px-5 py-2 rounded text-xs font-semibold bg-[#ffb31a] hover:bg-[#e6a117] text-black transition-colors"
        >
          {#if currentStep === totalSteps}
            <span>Finalizar Configuración</span>
          {:else}
            <span>Siguiente ▶</span>
          {/if}
        </button>
      </div>

    </div>
  </div>
{/if}
