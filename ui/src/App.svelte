<script lang="ts">
  import { regimeLabel, roomLabel } from './lib/labels';
  import { onMount } from 'svelte';
  import {
    api,
    type UpdateInfo,
    type DailyScheduleAggregate,
    type ScheduleConflict,
    type WorkRegime
  } from './lib/ipc/client';

  // Componentes de Escritorio
  import DailyRegimeBar from './lib/components/desktop/DailyRegimeBar.svelte';
  import { modalBehavior } from './lib/actions/modalBehavior';
  import ModalHeader from './lib/components/shared/ModalHeader.svelte';
  import MorningClassPanel from './lib/components/desktop/MorningClassPanel.svelte';
  import RehearsalMultiTrackGrid from './lib/components/desktop/RehearsalMultiTrackGrid.svelte';
  import ConflictAuditPanel from './lib/components/desktop/ConflictAuditPanel.svelte';
  import AttendancePhysioPanel from './lib/components/desktop/AttendancePhysioPanel.svelte';
  import OfficialPartReportModal from './lib/components/desktop/OfficialPartReportModal.svelte';
  import InventoryReportModal from './lib/components/desktop/InventoryReportModal.svelte';
  import PersonnelRosterModal from './lib/components/desktop/PersonnelRosterModal.svelte';
  import WorkCastingPlannerModal from './lib/components/desktop/WorkCastingPlannerModal.svelte';
  import FirstRunSetupWizardModal from './lib/components/desktop/FirstRunSetupWizardModal.svelte';
  import NewStockItemModal from './lib/components/desktop/NewStockItemModal.svelte';
  import WeeklySchedulePlanner from './lib/components/desktop/WeeklySchedulePlanner.svelte';
  import MonthlySchedulePlanner from './lib/components/desktop/MonthlySchedulePlanner.svelte';
  import LicenseLockScreen from './lib/components/desktop/LicenseLockScreen.svelte';
  import LicenseInfoModal from './lib/components/desktop/LicenseInfoModal.svelte';
  import SoloistCertificationModal from './lib/components/desktop/SoloistCertificationModal.svelte';
  import BopAttendanceMatrixModal from './lib/components/desktop/BopAttendanceMatrixModal.svelte';
  import InspectionVaultModal from './lib/components/desktop/InspectionVaultModal.svelte';
  import HelpCenterModal from './lib/components/desktop/HelpCenterModal.svelte';

  // Componentes Móviles y de Sincronización
  import DancerDepositSearch from './lib/components/mobile/DancerDepositSearch.svelte';
  import MobileStockCatalogView from './lib/components/mobile/MobileStockCatalogView.svelte';
  import GaynorDecoder from './lib/components/mobile/GaynorDecoder.svelte';
  import UpdateModal from './lib/components/UpdateModal.svelte';
  import ChangelogModal from './lib/components/ChangelogModal.svelte';
  import { CURRENT_VERSION } from './lib/changelog';
  import MenuButton from './lib/components/MenuButton.svelte';
  import Dashboard from './lib/components/Dashboard.svelte';
  import LanSyncModal from './lib/components/LanSyncModal.svelte';
  import MobileBarcodeScannerModal from './lib/components/mobile/MobileBarcodeScannerModal.svelte';
  import type { LicenseInfo, StockItemAvailabilityDto } from './lib/ipc/client';

  // Estado reactivo principal
  const isAndroidDevice = typeof navigator !== 'undefined' && /Android/i.test(navigator.userAgent);
  let viewMode = $state<'desktop' | 'mobile'>('desktop');
  let mobileTab = $state<'home' | 'stock' | 'inventory' | 'attendance' | 'gaynor' | 'schedule'>('home');
  let desktopTab = $state<'home' | 'schedule' | 'attendance'>('home');
  let scheduleViewMode = $state<'day' | 'week' | 'month'>('day');
  let showDesktopStockModal = $state(false);
  let showOfficialPartsModal = $state(false);
  let showInventoryReportModal = $state(false);
  let showPersonnelRosterModal = $state(false);
  let showLanSyncModal = $state(false);
  let showWorkCastingModal = $state(false);
  let showSetupWizardModal = $state(false);
  let showBarcodeScannerModal = $state(false);
  let showNewStockModal = $state(false);
  let showLicenseModal = $state(false);
  let showSoloistModal = $state(false);
  let showBopMatrixModal = $state(false);
  let showVaultModal = $state(false);
  let showHelpModal = $state(false);
  let showChangelogModal = $state(false);
  let showUpdateModal = $state(false);
  let foundUpdate = $state<UpdateInfo | null>(null);
  let licenseInfo = $state<LicenseInfo | null>(null);
  let prefilledStockItem = $state<StockItemAvailabilityDto | null>(null);

  // Busca versiones nuevas como máximo una vez cada 24 horas, solo si ya hay token cargado.
  async function autoCheckForUpdate() {
    try {
      const last = Number(localStorage.getItem('tg_last_update_check') ?? 0);
      if (Date.now() - last < 24 * 3600 * 1000) return;
      const status = await api.getUpdateStatus();
      if (!status.platform || !status.has_token) return;
      const info = await api.checkForUpdate();
      localStorage.setItem('tg_last_update_check', String(Date.now()));
      if (info) {
        foundUpdate = info;
        showUpdateModal = true;
      }
    } catch {
      /* sin conexión o sin acceso: no se molesta a la usuaria */
    }
  }

  async function loadLicenseInfo() {
    try {
      licenseInfo = await api.getLicenseInfo();
    } catch (e) {
      console.error('Error cargando información de licencia:', e);
    }
  }

  function getLocalDateIsoString(d = new Date()): string {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  function changeDate(deltaDays: number) {
    const parts = currentDate.split('-').map(Number);
    const d = new Date(parts[0], parts[1] - 1, parts[2] + deltaDays);
    currentDate = getLocalDateIsoString(d);
    loadSchedule(currentDate);
  }

  function goToToday() {
    currentDate = getLocalDateIsoString();
    loadSchedule(currentDate);
  }

  // Fecha de trabajo (hora local del dispositivo en Argentina, no UTC)
  let currentDate = $state(getLocalDateIsoString());
  let schedule = $state<DailyScheduleAggregate | null>(null);
  let conflicts = $state<ScheduleConflict[]>([]);
  let isLoading = $state(true);
  let errorMessage = $state<string | null>(null);

  async function loadSchedule(dateStr: string) {
    isLoading = true;
    errorMessage = null;
    try {
      const res = await api.getOrCreateDailySchedule(dateStr);
      schedule = res.schedule;
      conflicts = res.conflicts;
      currentDate = dateStr;
    } catch (err: any) {
      console.error('Error cargando cronograma:', err);
      errorMessage = err?.message || 'Error al conectar con gestion-core';
    } finally {
      isLoading = false;
    }
  }

  async function handleRegimeChange(newRegime: WorkRegime) {
    if (!schedule) return;
    try {
      const res = await api.setScheduleRegime(schedule.id, newRegime);
      schedule = res.schedule;
      conflicts = res.conflicts;
    } catch (err: any) {
      alert(`No se pudo cambiar el régimen: ${err?.message || err}`);
    }
  }

  onMount(() => {
    loadLicenseInfo();
    // Detección automática de factor de forma inicial (Desktop vs Mobile)
    if (window.innerWidth <= 768 || /Android|iPhone|iPad|iPod/i.test(navigator.userAgent)) {
      viewMode = 'mobile';
    } else {
      viewMode = 'desktop';
      // Asistente Paso por Paso al iniciar por primera vez en PC
      const setupDone = localStorage.getItem('tg_onboarding_completed');
      if (!setupDone) {
        showSetupWizardModal = true;
      }
    }
    try {
      if (localStorage.getItem('tg_last_seen_version') !== CURRENT_VERSION) {
        // En una instalación nueva de PC el asistente inicial ya explica todo: no se superpone
        showChangelogModal = viewMode === 'mobile' || !!localStorage.getItem('tg_onboarding_completed');
        localStorage.setItem('tg_last_seen_version', CURRENT_VERSION);
      }
    } catch { /* sin almacenamiento: no se muestra */ }
    autoCheckForUpdate();
    loadSchedule(currentDate);
  });
</script>

{#if licenseInfo?.is_locked}
  <LicenseLockScreen info={licenseInfo} onActivated={loadLicenseInfo} />
{:else}
  <main class="app-shell h-screen overflow-hidden bg-void text-ink flex flex-col selection:bg-amber selection:text-void">
    
    {#if licenseInfo?.warning}
      <div class="shrink-0 bg-amber/15 border-b border-amber/30 text-amber px-4 py-2 text-xs flex items-center justify-between z-40">
        <span>{licenseInfo.warning}</span>
        <button
          type="button"
          onclick={() => showLicenseModal = true}
          class="underline font-bold hover:text-white ml-2 uppercase text-[11px]"
        >
          [Ver Licencia]
        </button>
      </div>
    {/if}

    <!-- Encabezado Estilo TeatroCue -->
    <header class="shrink-0 bg-panel border-b border-grid z-30 px-3 md:px-5 py-2">
      <div class="max-w-7xl mx-auto flex items-center gap-2 md:gap-4">

        <!-- Marca -->
        <div class="flex items-center gap-2.5 shrink-0">
          <div class="w-9 h-9 rounded bg-raised border border-amber flex items-center justify-center font-mono font-bold text-amber text-sm shrink-0">TG</div>
          <div class="leading-tight">
            <h1 class="text-[15px] font-bold tracking-tight text-ink">TeatroGestión</h1>
            <p class="text-[11px] text-dim hidden sm:block">Ballet Oficial de Córdoba</p>
          </div>
        </div>

        {#if viewMode === 'desktop'}
          <!-- Navegación principal (escritorio) -->
          <nav class="hidden md:flex items-center gap-1 ml-2" aria-label="Secciones">
            {#each [['home', 'Inicio'], ['schedule', 'Planificación'], ['attendance', 'Asistencia']] as [key, label]}
              <button
                type="button"
                onclick={() => (desktopTab = key as typeof desktopTab)}
                class="h-8 px-3 rounded text-[13px] font-semibold transition-colors {desktopTab === key ? 'bg-amber text-void' : 'text-ink hover:bg-raised hover:text-white'}"
              >
                {label}
              </button>
            {/each}
          </nav>
        {/if}

        <div class="flex-1"></div>

        <div class="flex items-center gap-1.5">
          {#if viewMode === 'desktop'}
            <div class="hidden md:flex items-center gap-1.5">
              <MenuButton
                label="Documentos"
                items={[
                  { label: 'Parte diario y semanal', hint: 'Cartelera oficial, Excel y PDF', onSelect: () => (showOfficialPartsModal = true) },
                  { label: 'Planilla de ausentismo BOP', hint: 'Matriz mensual de asistencia', onSelect: () => (showBopMatrixModal = true) },
                  { label: 'Solísticos y plus por rol', hint: 'Certificación para Agencia Córdoba Cultura', onSelect: () => (showSoloistModal = true) },
                  { label: 'Repartos de obras', hint: 'Elencos, pasajes y planilla de cartelera', onSelect: () => (showWorkCastingModal = true) },
                  { label: 'Bóveda de inspección', hint: 'Certificados, recibos y memos', separatorBefore: true, onSelect: () => (showVaultModal = true) },
                ]}
              />
              <MenuButton
                label="Depósito y personal"
                items={[
                  { label: 'Inventario', hint: 'Stock oficial, PDF y Excel', onSelect: () => (showInventoryReportModal = true) },
                  { label: 'Entregas de calzado', hint: 'Vales y entregas a bailarines', onSelect: () => (showDesktopStockModal = true) },
                  { label: 'Escáner de códigos', hint: 'Leer códigos de barras con la cámara', onSelect: () => (showBarcodeScannerModal = true) },
                  { label: 'Padrón del personal', hint: 'Datos, talles y estado jubilatorio', separatorBefore: true, onSelect: () => (showPersonnelRosterModal = true) },
                ]}
              />
            </div>
          {/if}

          <MenuButton
            label={viewMode === 'desktop' ? 'Más' : 'Menú'}
            align="right"
            items={[
              { label: 'Sincronizar por Wi-Fi', hint: 'PC ↔ celular en la misma red', onSelect: () => (showLanSyncModal = true) },
              ...(viewMode === 'mobile' ? [{ label: 'Escáner de códigos', hint: 'Leer códigos de barras con la cámara', onSelect: () => (showBarcodeScannerModal = true) }] : []),
              { label: 'Licencia', hint: 'Estado y vencimiento', onSelect: () => (showLicenseModal = true) },
              { label: 'Asistente de configuración', hint: 'Parámetros de jornada y salas', onSelect: () => (showSetupWizardModal = true) },
              { label: 'Actualizaciones', hint: 'Buscar e instalar versiones nuevas', separatorBefore: true, onSelect: () => { foundUpdate = null; showUpdateModal = true; } },
              { label: 'Novedades', hint: 'Qué se agregó y se arregló en esta versión', onSelect: () => (showChangelogModal = true) },
              { label: 'Ayuda y manual', hint: 'Guías paso a paso', onSelect: () => (showHelpModal = true) },
            ]}
          />

          <!-- Selector Vista PC / Móvil -->
          <div class="{isAndroidDevice ? 'hidden' : 'inline-flex'} bg-void p-0.5 rounded border border-grid ml-1" role="group" aria-label="Tipo de pantalla">
            <button
              type="button"
              onclick={() => (viewMode = 'desktop')}
              class="h-8 px-2.5 rounded text-xs font-bold transition-colors {viewMode === 'desktop' ? 'bg-amber text-void' : 'text-dim hover:text-ink'}"
              title="Vista de computadora"
            >PC</button>
            <button
              type="button"
              onclick={() => (viewMode = 'mobile')}
              class="h-8 px-2.5 rounded text-xs font-bold transition-colors {viewMode === 'mobile' ? 'bg-amber text-void' : 'text-dim hover:text-ink'}"
              title="Vista de celular"
            >Móvil</button>
          </div>
        </div>
      </div>
    </header>

  <!-- Contenedor Principal según Modo de Visualización -->
  <!-- La pantalla tiene su propia zona de desplazamiento: el contenido llega hasta el final y nada lo tapa (el pie ocupa su lugar). -->
  <div class="app-scroll flex-1 min-h-0 overflow-y-auto">
  <div class="max-w-7xl w-full mx-auto p-3 md:p-6 {viewMode === 'mobile' ? 'pb-32' : 'pb-8'}">
    {#if isLoading && !schedule}
      <div class="h-64 flex flex-col items-center justify-center gap-3 text-dim">
        <div class="w-8 h-8 border-2 border-amber border-t-transparent rounded-full animate-spin"></div>
        <p class="text-xs font-mono">Conectando con base de datos SQLite WAL local...</p>
      </div>
    {:else if errorMessage}
      <div class="p-6 rounded bg-void border border-red text-red text-center my-8">
        <p class="text-xs font-mono font-bold mb-2">[ERROR DE INICIALIZACIÓN]</p>
        <p class="text-xs font-mono mb-4 text-ink">{errorMessage}</p>
        <button
          onclick={() => loadSchedule(currentDate)}
          class="px-4 py-2 bg-red hover:bg-red text-white text-xs font-bold rounded transition-colors"
        >
          Reintentar conexión
        </button>
      </div>
    {:else if schedule}
      
      <!-- ================================================================= -->
      <!-- VISTA DE ESCRITORIO (PLANIFICACIÓN DE CITACIONES Y CRONOGRAMAS)     -->
      <!-- ================================================================= -->
      {#if viewMode === 'desktop'}
        <div class="space-y-4">
          <!-- Selector de vista del cronograma -->
          {#if desktopTab === 'schedule'}
          <div class="flex items-center justify-between gap-2 bg-panel p-2 rounded border border-grid">
            <!-- Selector de Vista Diaria / Semanal / Mensual -->
            {#if true}
              <div class="inline-flex bg-void p-0.5 rounded border border-grid">
                <button
                  onclick={() => scheduleViewMode = 'day'}
                  class="px-3 py-1 rounded text-xs font-bold transition-colors {scheduleViewMode === 'day' ? 'bg-raised text-amber border border-amber/40' : 'text-dim hover:text-ink'}"
                >
                  Vista Diaria
                </button>
                <button
                  onclick={() => scheduleViewMode = 'week'}
                  class="px-3 py-1 rounded text-xs font-bold transition-colors {scheduleViewMode === 'week' ? 'bg-raised text-amber border border-amber/40' : 'text-dim hover:text-ink'}"
                >
                  Vista Semanal
                </button>
                <button
                  onclick={() => scheduleViewMode = 'month'}
                  class="px-3 py-1 rounded text-xs font-bold transition-colors {scheduleViewMode === 'month' ? 'bg-raised text-amber border border-amber/40' : 'text-dim hover:text-ink'}"
                >
                  Vista Mensual
                </button>
              </div>
            {/if}
          </div>
          {/if}

          {#if desktopTab === 'home'}
            <Dashboard
  {schedule}
  {conflicts}
  {currentDate}
  {licenseInfo}
  onGoAttendance={() => { desktopTab = 'attendance'; mobileTab = 'attendance'; }}
  onGoSchedule={() => { desktopTab = 'schedule'; scheduleViewMode = 'day'; mobileTab = 'schedule'; }}
  onOpenBop={() => showBopMatrixModal = true}
  onOpenParts={() => showOfficialPartsModal = true}
  onOpenInventory={() => { showInventoryReportModal = true; }}
  onOpenLicense={() => showLicenseModal = true}
/>
          {:else if desktopTab === 'schedule'}
            {#if scheduleViewMode === 'day'}
              <!-- Barra de Régimen, Fecha y Descanso -->
              <DailyRegimeBar
                {schedule}
                onRegimeChange={handleRegimeChange}
                onDateChange={(d) => loadSchedule(d)}
              />

              <!-- Auditoría Anti-Cruces en Tiempo Real -->
              <ConflictAuditPanel {conflicts} />

              <!-- Panel de Clases Técnicas Matutinas (09:00 a 10:15 h) -->
              <MorningClassPanel
                classes={schedule.morning_classes}
                scheduleId={schedule.id}
                onUpdated={() => loadSchedule(currentDate)}
              />

              <!-- Grilla Multipista de Ensayos (Sala 1, Sala 2, Sala 3, Escenario, Gira) -->
              <RehearsalMultiTrackGrid
                scheduleId={schedule.id}
                blocks={schedule.rehearsal_blocks}
                {conflicts}
                onUpdated={() => loadSchedule(currentDate)}
              />
            {:else if scheduleViewMode === 'week'}
              <WeeklySchedulePlanner
                {currentDate}
                onSelectDate={(d) => { currentDate = d; loadSchedule(d); }}
                onOpenCitationModal={(d) => { currentDate = d; loadSchedule(d).then(() => { scheduleViewMode = 'day'; }); }}
                onSwitchToDailyView={(d) => { currentDate = d; loadSchedule(d); scheduleViewMode = 'day'; }}
              />
            {:else if scheduleViewMode === 'month'}
              <MonthlySchedulePlanner
                {currentDate}
                onSelectDate={(d) => { currentDate = d; loadSchedule(d); }}
                onSwitchToDailyView={(d) => { currentDate = d; loadSchedule(d); scheduleViewMode = 'day'; }}
                onSwitchToWeeklyView={(d) => { currentDate = d; loadSchedule(d); scheduleViewMode = 'week'; }}
              />
            {/if}
          {:else}
            <!-- Panel Completo de Asistencia Diaria & Fisioterapia -->
            <AttendancePhysioPanel
              scheduleDate={currentDate}
              onDateChange={(d) => {
                currentDate = d;
                loadSchedule(d);
              }}
            />
          {/if}
        </div>

      <!-- ================================================================= -->
      <!-- VISTA MÓVIL (OPTIMIZADA PARA SAMSUNG GALAXY A53 5G - ONE UI)     -->
      <!-- ================================================================= -->
      {:else}
        <div class="space-y-4 max-w-xl mx-auto">
          
          <!-- Pestaña 1: Entregas de Calzado y Asignación Nominal Rápida -->
          {#if mobileTab === 'home'}
            <Dashboard
  {schedule}
  {conflicts}
  {currentDate}
  {licenseInfo}
  onGoAttendance={() => { desktopTab = 'attendance'; mobileTab = 'attendance'; }}
  onGoSchedule={() => { desktopTab = 'schedule'; scheduleViewMode = 'day'; mobileTab = 'schedule'; }}
  onOpenBop={() => showBopMatrixModal = true}
  onOpenParts={() => showOfficialPartsModal = true}
  onOpenInventory={() => { showInventoryReportModal = true; }}
  onOpenLicense={() => showLicenseModal = true}
/>
          {:else if mobileTab === 'stock'}
            <div>
              <DancerDepositSearch />
            </div>

          <!-- Pestaña 2: Inventario Integral Móvil y Reportes -->
          {:else if mobileTab === 'inventory'}
            <div class="space-y-3">
              
              <!-- Tarjeta de Acciones Rápidas de Inventario -->
              <div class="bg-panel border border-grid rounded p-4 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <h2 class="text-sm font-bold text-ink uppercase tracking-wide">Inventario del Ballet Oficial</h2>
                    <p class="text-[11px] text-dim">Puntas, zapatillas, indumentaria y stock del depósito</p>
                  </div>
                  <span class="text-[10px] font-mono font-bold text-amber bg-raised px-2 py-0.5 rounded border border-grid">INVENTARIO</span>
                </div>

                <div class="grid grid-cols-2 gap-2 pt-1">
                  <button
                    onclick={() => showBarcodeScannerModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-grid rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-ink">Escanear Código</div>
                      <div class="text-[10px] text-dim">Cámara / Ingreso / Pedido</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showWorkCastingModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-grid rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-ink">Repartos de Obra</div>
                      <div class="text-[10px] text-dim">Elencos y Pasajes</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showSoloistModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-amber/40 rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-amber">Solísticos (Rol)</div>
                      <div class="text-[10px] text-dim">Plus / Certificación</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showBopMatrixModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-cyan/40 rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-cyan">Ausentismo BOP</div>
                      <div class="text-[10px] text-dim">Planilla Mensual Excel</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showVaultModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-red/40 rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-red">Bóveda de Inspección</div>
                      <div class="text-[10px] text-dim">Fotos, Recibos y Memos</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showInventoryReportModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-grid rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-ink">Inventario Completo</div>
                      <div class="text-[10px] text-dim">Exportar PDF / Excel</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showPersonnelRosterModal = true}
                    class="p-2.5 bg-raised hover:bg-raised-hover border border-grid rounded text-left transition-colors flex flex-col justify-between"
                  >
                    <div>
                      <div class="text-xs font-bold text-ink">Padrón de Personal</div>
                      <div class="text-[10px] text-dim">DNI, Talles y Revista</div>
                    </div>
                  </button>

                  <button
                    onclick={() => showHelpModal = true}
                    class="p-2.5 bg-void hover:bg-raised-hover border border-green/40 rounded text-left transition-colors flex flex-col justify-between col-span-2"
                  >
                    <div class="flex items-center justify-between">
                      <div>
                        <div class="text-xs font-bold text-green">Centro de Ayuda & Manual</div>
                        <div class="text-[10px] text-dim">Manual oficial en PDF (8 págs) y guías operativas</div>
                      </div>
                      <span class="text-[10px] text-green font-mono font-bold px-2 py-0.5 rounded bg-void border border-green/30">PDF</span>
                    </div>
                  </button>
                </div>
              </div>

              <!-- Catálogo Completo de Depósito Móvil -->
              <div class="border-t border-grid pt-2">
                <MobileStockCatalogView onScanCode={() => showBarcodeScannerModal = true} />
              </div>
            </div>

          <!-- Pestaña 3: Control de Asistencia & Fisioterapia Móvil -->
          {:else if mobileTab === 'attendance'}
            <div>
              <AttendancePhysioPanel
                scheduleDate={currentDate}
                onDateChange={(d) => {
                  currentDate = d;
                  loadSchedule(d);
                }}
              />
            </div>

          <!-- Pestaña 4: Decodificador Gaynor Visual Táctil -->
          {:else if mobileTab === 'gaynor'}
            <div>
              <GaynorDecoder />
            </div>

          <!-- Pestaña 5: Vista Rápida del Parte Diario Móvil -->
          {:else if mobileTab === 'schedule'}
            <div class="space-y-3">
              <!-- Navegador de Fecha Móvil -->
              <div class="flex items-center justify-between gap-1.5 bg-panel border border-grid rounded p-2">
                <button
                  onclick={() => changeDate(-1)}
                  class="h-8 px-2.5 bg-raised hover:bg-grid text-ink rounded text-xs font-mono font-bold transition-colors"
                  title="Día Anterior"
                >
                  ◀ Ayer
                </button>
                <div class="flex items-center gap-1.5">
                  <input
                    type="date"
                    bind:value={currentDate}
                    onchange={() => loadSchedule(currentDate)}
                    class="h-8 px-2 bg-void border border-grid rounded text-xs font-mono font-bold text-amber outline-none"
                  />
                  <button
                    onclick={goToToday}
                    class="h-8 px-2.5 bg-raised hover:bg-grid text-amber rounded text-xs font-mono font-bold transition-colors"
                    title="Ir a la fecha de hoy"
                  >
                    Hoy
                  </button>
                </div>
                <button
                  onclick={() => changeDate(1)}
                  class="h-8 px-2.5 bg-raised hover:bg-grid text-ink rounded text-xs font-mono font-bold transition-colors"
                  title="Día Siguiente"
                >
                  Mañana ▶
                </button>
              </div>

              <button
                onclick={() => showOfficialPartsModal = true}
                class="w-full py-2.5 px-3 bg-raised hover:bg-raised-hover border border-amber/40 text-amber rounded text-xs font-bold transition-colors flex items-center justify-center gap-2"
              >
                <span>Ver Parte Oficial Semanal (Excel / PDF)</span>
              </button>

              <div class="bg-panel border border-grid rounded p-3">
                <div class="flex items-center justify-between text-xs mb-1.5">
                  <span class="text-dim font-medium uppercase tracking-wider text-[10px]">Fecha de Citación</span>
                  <span class="font-mono text-amber font-bold">{schedule.schedule_date}</span>
                </div>
                <div class="flex items-center justify-between text-xs mb-1.5">
                  <span class="text-dim font-medium uppercase tracking-wider text-[10px]">Régimen Laboral</span>
                  <span class="bg-raised px-2 py-0.5 rounded text-ink text-[11px] font-medium border border-grid">{regimeLabel(schedule.regime)}</span>
                </div>
                <div class="flex items-center justify-between text-xs">
                  <span class="text-dim font-medium uppercase tracking-wider text-[10px]">Límite Horario</span>
                  <span class="font-mono text-green font-bold">{schedule.nominal_end_time} h</span>
                </div>
              </div>

              <!-- Clases matutinas compactas -->
              <MorningClassPanel
                classes={schedule.morning_classes}
                scheduleId={schedule.id}
                onUpdated={() => loadSchedule(currentDate)}
              />

              <!-- Auditoría compacta de cruces -->
              <ConflictAuditPanel {conflicts} />

              <!-- Lista compacta de bloques de ensayo -->
              <div class="bg-panel border border-grid rounded p-3">
                <h3 class="text-[10px] font-bold uppercase tracking-wider text-dim mb-2">Bloques de Ensayo Citados</h3>
                {#if schedule.rehearsal_blocks.length === 0}
                  <p class="text-xs text-dim py-2">No hay bloques de ensayo agendados para esta fecha.</p>
                {:else}
                  <div class="space-y-1.5">
                    {#each schedule.rehearsal_blocks as b}
                      <div class="p-2.5 rounded bg-alt border border-grid text-xs">
                        <div class="flex items-center justify-between font-mono text-amber font-bold">
                          <span>{b.start_time} - {b.end_time} h</span>
                          <span class="bg-raised text-dim px-1.5 py-0.5 rounded text-[10px] border border-grid">{roomLabel(b.room)}</span>
                        </div>
                        <div class="text-ink font-bold text-xs mt-1">{b.passage_name}</div>
                        <div class="text-[11px] text-dim mt-0.5">Mtro: {b.master_name || 'Sin maestro'} • Pno: {b.pianist_name || 'Sin piano'}</div>
                        {#if b.sound_requirement}
                          <div class="text-[10px] font-mono text-cyan mt-1">[CUE] {b.sound_requirement}</div>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {/if}

        </div>
      {/if}
    {/if}
  </div>
  </div>

  <!-- Modal Desplegable de Stock / Entregas en Escritorio -->
  {#if showDesktopStockModal}
    <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4" use:modalBehavior={{ onclose: () => showDesktopStockModal = false }}>
      <div class="tg-modal max-w-2xl">
        <ModalHeader title="Entregas de Calzado y Vales a Bailarines" onclose={() => showDesktopStockModal = false} />

        <div class="tg-modal-body space-y-4">
          <DancerDepositSearch />
          <div class="pt-3 border-t border-grid">
            <GaynorDecoder />
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal de Partes Oficiales Semanales (Clases & Ensayos) en PDF / Excel -->
  {#if schedule}
    <OfficialPartReportModal
      {schedule}
      isOpen={showOfficialPartsModal}
      onClose={() => showOfficialPartsModal = false}
    />
  {/if}

  <!-- Modal de Inventario Oficial de Ballet (PDF / Excel) -->
  <InventoryReportModal
    isOpen={showInventoryReportModal}
    onClose={() => showInventoryReportModal = false}
  />

  <!-- Modal de Padrón del Personal del Sector -->
  <PersonnelRosterModal
    isOpen={showPersonnelRosterModal}
    onClose={() => showPersonnelRosterModal = false}
  />

  <!-- Modal de Sincronización Wi-Fi LAN P2P -->
  <LanSyncModal
    isOpen={showLanSyncModal}
    onClose={() => showLanSyncModal = false}
    onSyncCompleted={() => {
      loadSchedule(currentDate);
    }}
  />

  <!-- Modal de Planificación de Repartos de Obras y Elencos -->
  <WorkCastingPlannerModal
    isOpen={showWorkCastingModal}
    onClose={() => {
      showWorkCastingModal = false;
      loadSchedule(currentDate);
    }}
  />

  <!-- Modal de Certificación Oficial de Solísticos y Plus por Rol -->
  <SoloistCertificationModal
    isOpen={showSoloistModal}
    onClose={() => showSoloistModal = false}
  />

  <!-- Modal de Matriz Mensual de Ausentismo BOP (Planilla Histórica de Carina) -->
  <BopAttendanceMatrixModal
    isOpen={showBopMatrixModal}
    onClose={() => showBopMatrixModal = false}
  />

  <!-- Modal de Bóveda Documental de Inspección (Carina Lion) -->
  <InspectionVaultModal
    isOpen={showVaultModal}
    onClose={() => showVaultModal = false}
  />

  <!-- Modal de Asistente de Configuración Inicial (Walkthrough Primer Inicio en PC) -->
  <FirstRunSetupWizardModal
    isOpen={showSetupWizardModal}
    onClose={() => showSetupWizardModal = false}
    onFinished={() => {
      showSetupWizardModal = false;
      loadSchedule(currentDate);
    }}
  />

  <!-- Modal de Escáner de Códigos de Barras con Cámara Móvil / PC -->
  <MobileBarcodeScannerModal
    isOpen={showBarcodeScannerModal}
    onClose={() => showBarcodeScannerModal = false}
    onDeliverItem={(item) => {
      showBarcodeScannerModal = false;
      showInventoryReportModal = true;
    }}
    onNewProductWithBarcode={(barcode) => {
      showBarcodeScannerModal = false;
      prefilledStockItem = {
        id: '',
        subcategory: 'MEDIA_PUNTA_TELA',
        family: 'WOMENS_FLAT',
        quota_bucket: 'FLAT',
        brand: 'Bloch',
        model_name: `Producto Escaneado (${barcode})`,
        raw_factory_code: barcode,
        color: 'Rosa',
        material: 'TELA_ELASTICA',
        size_system: 'US',
        size_value: '6',
        width_value: 'B',
        physical_qty: 10,
        min_alert_qty: 2,
        storage_bin_location: 'ESTANTERIA_A',
        is_active: true,
        created_at: '',
        updated_at: ''
      };
      showNewStockModal = true;
    }}
  />

  <!-- Modal de Alta o Edición de Producto con Código Escaneado -->
  <NewStockItemModal
    isOpen={showNewStockModal}
    initialItem={prefilledStockItem}
    onClose={() => {
      showNewStockModal = false;
      prefilledStockItem = null;
    }}
    onSaved={() => {
      showNewStockModal = false;
      prefilledStockItem = null;
      showInventoryReportModal = true;
    }}
  />

  <!-- ===================================================================== -->
  <!-- BARRA DE NAVEGACIÓN INFERIOR MÓVIL (MATERIAL 3 / ONE UI - GALAXY A53)  -->
  <!-- ===================================================================== -->
  {#if viewMode === 'mobile'}
    <nav class="fixed bottom-0 left-0 right-0 z-40 bg-panel border-t border-grid ">
      <!-- 5 Pestañas Ergonómicas Táctiles -->
      <div class="grid grid-cols-6 h-16 max-w-lg mx-auto px-1">

        <button
          onclick={() => mobileTab = 'home'}
          class="flex flex-col items-center justify-center gap-0.5 transition-colors {mobileTab === 'home' ? 'text-amber' : 'text-dim hover:text-ink'}"
        >
          <svg class="w-[22px] h-[22px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 11.5 12 4l9 7.5"/><path d="M5 10v10h14V10"/><path d="M10 20v-6h4v6"/></svg>
          <span class="text-[10px] leading-none tracking-tight">Inicio</span>
        </button>

        <button
          onclick={() => mobileTab = 'stock'}
          class="flex flex-col items-center justify-center gap-0.5 transition-colors {mobileTab === 'stock' ? 'text-amber' : 'text-dim hover:text-ink'}"
        >
          <svg class="w-[22px] h-[22px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 8l9-5 9 5v8l-9 5-9-5z"/><path d="M3 8l9 5 9-5"/><path d="M12 13v8"/></svg>
          <span class="text-[10px] leading-none tracking-tight">Entregas</span>
        </button>

        <button
          onclick={() => mobileTab = 'inventory'}
          class="flex flex-col items-center justify-center gap-0.5 transition-colors {mobileTab === 'inventory' ? 'text-amber' : 'text-dim hover:text-ink'}"
        >
          <svg class="w-[22px] h-[22px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="5" rx="1"/><path d="M5 9v10h14V9"/><path d="M10 13h4"/></svg>
          <span class="text-[10px] leading-none tracking-tight">Inventario</span>
        </button>

        <button
          onclick={() => mobileTab = 'attendance'}
          class="flex flex-col items-center justify-center gap-0.5 transition-colors {mobileTab === 'attendance' ? 'text-amber' : 'text-dim hover:text-ink'}"
        >
          <svg class="w-[22px] h-[22px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="5" y="4" width="14" height="17" rx="2"/><path d="M9 4h6v3H9z"/><path d="m9 14 2 2 4-4"/></svg>
          <span class="text-[10px] leading-none tracking-tight">Asistencia</span>
        </button>

        <button
          onclick={() => mobileTab = 'gaynor'}
          class="flex flex-col items-center justify-center gap-0.5 transition-colors {mobileTab === 'gaynor' ? 'text-amber' : 'text-dim hover:text-ink'}"
        >
          <svg class="w-[22px] h-[22px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"/></svg>
          <span class="text-[10px] leading-none tracking-tight">Gaynor</span>
        </button>

        <button
          onclick={() => mobileTab = 'schedule'}
          class="flex flex-col items-center justify-center gap-0.5 transition-colors {mobileTab === 'schedule' ? 'text-amber' : 'text-dim hover:text-ink'}"
        >
          <!-- Badge dinámico con el día real y mes -->
          <div class="w-6 h-6 rounded border {mobileTab === 'schedule' ? 'border-amber' : 'border-grid'} flex flex-col overflow-hidden bg-void">
            <div class="bg-amber text-void text-[7px] font-black leading-tight text-center uppercase">
              {currentDate ? new Date(currentDate + 'T12:00:00').toLocaleDateString('es-AR', { month: 'short' }).slice(0, 3) : 'HOY'}
            </div>
            <div class="text-[10px] font-bold text-ink text-center leading-tight flex-1 flex items-center justify-center font-mono">
              {currentDate ? currentDate.split('-')[2] : '•'}
            </div>
          </div>
          <span class="text-[10px] leading-none tracking-tight">Parte</span>
        </button>

      </div>
    </nav>

  <!-- Pie Institucional de Escritorio -->
  {:else}
    <footer class="shrink-0 z-40 bg-void border-t border-grid px-4 py-2">
      <div class="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-1 text-center sm:text-left">
        <p class="text-[11px] font-mono text-dim">
          Teatro del Libertador San Martín · Ballet Oficial de Córdoba
        </p>
        <p class="text-[10px] text-dim font-mono hidden md:block">
          Datos guardados en este equipo · Sincroniza por Wi-Fi con el celular
        </p>
      </div>
    </footer>
  {/if}

</main>
{/if}

{#if showLicenseModal && licenseInfo}
  <LicenseInfoModal
    info={licenseInfo}
    onClose={() => (showLicenseModal = false)}
    onUpdated={loadLicenseInfo}
  />
{/if}

<UpdateModal isOpen={showUpdateModal} initialInfo={foundUpdate} onClose={() => (showUpdateModal = false)} />
<ChangelogModal isOpen={showChangelogModal} onClose={() => (showChangelogModal = false)} />

<HelpCenterModal
  isOpen={showHelpModal}
  onClose={() => (showHelpModal = false)}
/>

