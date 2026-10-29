<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    api,
    type InspectionVaultDocumentDto,
    type CreateInspectionDocumentPayload,
    type InspectionVaultFilterPayload,
    type VaultSecurityStatusDto,
    type VaultMonthlyTotalsDto,
    type PersonnelDto,
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let {
    isOpen,
    onClose,
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  // Estados de Seguridad y Bloqueo
  let securityStatus = $state<VaultSecurityStatusDto>({ has_password: false, lock_timeout_minutes: 5 });
  let isUnlocked = $state(false);
  let unlockPassword = $state('');
  let unlockError = $state('');
  let isAuthenticating = $state(false);
  let showPasswordEye = $state(false);

  // Modal de Configuración Inicial / Cambio de Clave
  let showSecurityConfigModal = $state(false);
  let oldPasswordInput = $state('');
  let newPasswordInput = $state('');
  let confirmPasswordInput = $state('');
  let timeoutMinutesInput = $state(5);
  let securityConfigError = $state('');
  let isSavingSecurityConfig = $state(false);

  // Temporizador de Bloqueo por Inactividad
  let lastActivityTimestamp = $state(Date.now());
  let inactivityInterval: any = null;

  // Filtros y Búsqueda
  let selectedCategory = $state<string>('ALL');
  let searchQuery = $state<string>('');
  let selectedPersonnelId = $state<string>('');
  let selectedMonth = $state<number>(new Date().getMonth() + 1);
  let selectedYear = $state<number>(new Date().getFullYear());

  // Datos
  let documents = $state<InspectionVaultDocumentDto[]>([]);
  let monthlyTotals = $state<VaultMonthlyTotalsDto | null>(null);
  let personnelList = $state<PersonnelDto[]>([]);
  let isLoading = $state(false);

  // Modal de Alta / Carga de Documento
  let showUploadModal = $state(false);
  let uploadTitle = $state('');
  let uploadCategory = $state<'RECIBO_COMPRA' | 'CERTIFICADO_MEDICO' | 'MEMO_DISPOSICION' | 'ACUERDO_DIRECCION' | 'SOLISTICO_FIRMA' | 'OTRO'>('RECIBO_COMPRA');
  let uploadDate = $state(new Date().toISOString().split('T')[0]);
  let uploadPersonnelId = $state('');
  let uploadAmountArs = $state<number | null>(null);
  let uploadSupplier = $state('');
  let uploadNotes = $state('');
  let uploadFileName = $state('');
  let uploadFileType = $state('');
  let uploadFileData = $state('');
  let uploadFileSize = $state(0);
  let uploadError = $state('');
  let isSavingDocument = $state(false);

  // Visor de Documento a Pantalla Completa
  let viewingDocument = $state<InspectionVaultDocumentDto | null>(null);

  const CATEGORIES = [
    { key: 'ALL', label: 'Todos los Documentos', badge: 'bg-raised] text-ink] border-grid]' },
    { key: 'RECIBO_COMPRA', label: 'Recibos Caja Chica', badge: 'bg-amber]/15 text-amber] border-amber]/30' },
    { key: 'CERTIFICADO_MEDICO', label: 'Certificados Médicos', badge: 'bg-cyan]/15 text-cyan] border-cyan]/30' },
    { key: 'ACUERDO_DIRECCION', label: 'Acuerdos Francos / Dirección', badge: 'bg-green]/15 text-green] border-green]/30' },
    { key: 'MEMO_DISPOSICION', label: 'Memos y Disposiciones', badge: 'bg-violet]/15 text-violet] border-violet]/30' },
    { key: 'SOLISTICO_FIRMA', label: 'Solísticos Firmados', badge: 'bg-amber-500/15 text-amber-300 border-amber-500/30' },
    { key: 'OTRO', label: 'Otros / Notas', badge: 'bg-zinc-700/20 text-zinc-300 border-zinc-600/30' },
  ];

  function resetActivityTimer() {
    lastActivityTimestamp = Date.now();
  }

  function checkInactivityLock() {
    if (!isUnlocked || !securityStatus.has_password) return;
    if (securityStatus.lock_timeout_minutes <= 0) return; // 0 = Nunca bloquear

    const elapsed = Date.now() - lastActivityTimestamp;
    const timeoutMs = securityStatus.lock_timeout_minutes * 60 * 1000;
    if (elapsed >= timeoutMs) {
      lockVault();
    }
  }

  function lockVault() {
    isUnlocked = false;
    unlockPassword = '';
    unlockError = '';
    showUploadModal = false;
    showSecurityConfigModal = false;
    viewingDocument = null;
  }

  async function loadSecurityStatus() {
    try {
      securityStatus = await api.getVaultSecurityStatus();
      timeoutMinutesInput = securityStatus.lock_timeout_minutes;
      if (!securityStatus.has_password) {
        // Sin clave previa -> desbloqueado por defecto
        isUnlocked = true;
      }
    } catch (e) {
      console.error('Error cargando estado de seguridad:', e);
    }
  }

  async function unlockVault() {
    unlockError = '';
    isAuthenticating = true;
    try {
      const ok = await api.verifyVaultPassword({ password: unlockPassword });
      if (ok) {
        isUnlocked = true;
        unlockPassword = '';
        resetActivityTimer();
        await loadVaultData();
      } else {
        unlockError = 'Contraseña incorrecta. Verifique mayúsculas o consulte al administrador.';
      }
    } catch (e: any) {
      unlockError = e?.message || 'Error al verificar contraseña';
    } finally {
      isAuthenticating = false;
    }
  }

  async function loadVaultData() {
    if (!isUnlocked) return;
    isLoading = true;
    try {
      const filter: InspectionVaultFilterPayload = {
        category: selectedCategory === 'ALL' ? undefined : selectedCategory,
        personnel_id: selectedPersonnelId ? selectedPersonnelId : undefined,
        query: searchQuery.trim() || undefined,
      };

      const [docs, totals] = await Promise.all([
        api.listVaultDocuments(filter),
        api.getVaultMonthlyTotals(selectedMonth, selectedYear),
      ]);

      documents = docs;
      monthlyTotals = totals;
    } catch (e) {
      console.error('Error cargando bóveda de inspección:', e);
    } finally {
      isLoading = false;
    }
  }

  async function loadPersonnel() {
    try {
      personnelList = await api.listPersonnel();
    } catch (e) {
      console.error('Error cargando personal:', e);
    }
  }

  function handleFileSelected(event: Event) {
    const target = event.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];
    uploadFileName = file.name;
    uploadFileType = file.type || 'application/octet-stream';
    uploadFileSize = file.size;

    const reader = new FileReader();
    reader.onload = () => {
      uploadFileData = reader.result as string;
    };
    reader.readAsDataURL(file);
  }

  async function submitDocument() {
    uploadError = '';
    if (!uploadTitle.trim()) {
      uploadError = 'Ingrese un título descriptivo del documento o compra.';
      return;
    }
    if (!uploadFileData) {
      uploadError = 'Adjunte una foto del comprobante o archivo PDF.';
      return;
    }

    isSavingDocument = true;
    try {
      const payload: CreateInspectionDocumentPayload = {
        title: uploadTitle.trim(),
        category: uploadCategory,
        document_date: uploadDate,
        personnel_id: uploadPersonnelId ? uploadPersonnelId : null,
        file_name: uploadFileName,
        file_type: uploadFileType,
        file_size_bytes: uploadFileSize,
        file_data: uploadFileData,
        amount_ars: uploadCategory === 'RECIBO_COMPRA' && uploadAmountArs !== null ? Number(uploadAmountArs) : null,
        supplier_vendor: uploadSupplier.trim() || null,
        notes: uploadNotes.trim() || null,
      };

      await api.createVaultDocument(payload);
      showUploadModal = false;
      resetUploadForm();
      await loadVaultData();
    } catch (e: any) {
      uploadError = e?.message || 'Error guardando documento en la bóveda';
    } finally {
      isSavingDocument = false;
    }
  }

  function resetUploadForm() {
    uploadTitle = '';
    uploadCategory = 'RECIBO_COMPRA';
    uploadDate = new Date().toISOString().split('T')[0];
    uploadPersonnelId = '';
    uploadAmountArs = null;
    uploadSupplier = '';
    uploadNotes = '';
    uploadFileName = '';
    uploadFileType = '';
    uploadFileData = '';
    uploadFileSize = 0;
    uploadError = '';
  }

  async function handleDeleteDocument(doc: InspectionVaultDocumentDto) {
    if (!confirm(`¿Eliminar definitivamente "${doc.title}" de la bóveda de inspección? Esta acción no se puede deshacer.`)) return;
    try {
      await api.deleteVaultDocument(doc.id);
      await loadVaultData();
    } catch (e: any) {
      alert(`No se pudo eliminar el documento: ${e?.message || e}`);
    }
  }

  async function saveSecurityConfig() {
    securityConfigError = '';
    if (!newPasswordInput) {
      securityConfigError = 'La contraseña no puede estar vacía.';
      return;
    }
    if (newPasswordInput !== confirmPasswordInput) {
      securityConfigError = 'Las contraseñas no coinciden.';
      return;
    }

    isSavingSecurityConfig = true;
    try {
      await api.setVaultPassword({
        old_password: securityStatus.has_password ? oldPasswordInput : null,
        new_password: newPasswordInput,
        lock_timeout_minutes: timeoutMinutesInput,
      });

      securityStatus = await api.getVaultSecurityStatus();
      showSecurityConfigModal = false;
      oldPasswordInput = '';
      newPasswordInput = '';
      confirmPasswordInput = '';
      alert('Configuración de seguridad y contraseña guardada con éxito.');
    } catch (e: any) {
      securityConfigError = e?.message || 'Error al actualizar seguridad';
    } finally {
      isSavingSecurityConfig = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      loadSecurityStatus();
      loadPersonnel();
      if (isUnlocked) {
        loadVaultData();
      }
    }
  });

  onMount(() => {
    inactivityInterval = setInterval(checkInactivityLock, 10000); // Chequea cada 10s
    window.addEventListener('mousemove', resetActivityTimer);
    window.addEventListener('keydown', resetActivityTimer);
  });

  onDestroy(() => {
    if (inactivityInterval) clearInterval(inactivityInterval);
    window.removeEventListener('mousemove', resetActivityTimer);
    window.removeEventListener('keydown', resetActivityTimer);
  });

  function formatArs(amount: number | null | undefined): string {
    if (amount === null || amount === undefined) return '$ 0,00';
    return '$ ' + amount.toLocaleString('es-AR', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / 1048576).toFixed(1) + ' MB';
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-3" use:modalBehavior={{ onclose: onClose }}>
    <div class="tg-modal max-w-6xl text-ink]">
      
      <!-- ===================================================================== -->
      <!-- CABECERA DE LA BÓVEDA DOCUMENTAL                                      -->
      <!-- ===================================================================== -->
      <div class="bg-void] border-b border-grid] px-4 py-3 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded bg-amber]/15 border border-amber]/30 flex items-center justify-center text-amber]">
            <!-- Icono Caja Fuerte / Blindaje -->
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
            </svg>
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-sm font-black tracking-wide uppercase font-sans text-white">
                Bóveda de Documentación de Inspección
              </h2>
              <span class="text-[10px] font-mono font-bold px-2 py-0.5 rounded bg-amber]/15 text-amber] border border-amber]/30">
                CARINA LION
              </span>
              {#if isUnlocked}
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-green]/15 text-green] border border-green]/30">
                  DESBLOQUEADA ({securityStatus.lock_timeout_minutes > 0 ? `${securityStatus.lock_timeout_minutes}m inactividad` : 'Sin límite'})
                </span>
              {:else}
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-red]/15 text-red] border border-red]/30">
                  BLOQUEADA
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-dim] mt-0.5">
              Certificados médicos, facturas de caja chica (colofonia, botiquín), memos de dirección y acuerdos.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          {#if isUnlocked}
            <button
              onclick={() => showSecurityConfigModal = true}
              class="h-8 px-2.5 bg-raised] hover:bg-grid] text-ink] hover:text-amber] border border-grid] rounded text-xs font-semibold flex items-center gap-1.5 transition-colors"
              title="Cambiar contraseña de inspectora o tiempo de bloqueo automático"
            >
              <span>Seguridad</span>
            </button>
            <button
              onclick={lockVault}
              class="h-8 px-2.5 bg-void] hover:bg-void] text-red] border border-red]/30 rounded text-xs font-semibold flex items-center gap-1.5 transition-colors"
              title="Bloquear la bóveda inmediatamente"
            >
              <span>Bloquear</span>
            </button>
          {/if}

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
      </div>

      <!-- ===================================================================== -->
      <!-- PANTALLA DE BLOQUEO / AUTENTICACIÓN                                    -->
      <!-- ===================================================================== -->
      {#if !isUnlocked}
        <div class="flex-1 flex flex-col items-center justify-center p-8 bg-void] text-center">
          <div class="w-16 h-16 rounded-full bg-panel] border-2 border-amber]/40 flex items-center justify-center text-amber] mb-4">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
            </svg>
          </div>

          <h3 class="text-base font-bold text-white uppercase tracking-wide">
            Bóveda de Inspección Protegida
          </h3>
          <p class="text-xs text-dim] max-w-md mt-1 mb-6">
            Para ver o modificar certificados médicos, facturas de caja chica y memos reservados, ingrese la contraseña de la inspectora Carina Lion.
          </p>

          <form onsubmit={(e) => { e.preventDefault(); unlockVault(); }} class="w-full max-w-sm space-y-3">
            <div class="relative">
              <input
                type={showPasswordEye ? "text" : "password"}
                bind:value={unlockPassword}
                placeholder="Ingrese contraseña de inspectora..."
                class="w-full h-11 px-3.5 pr-10 bg-panel] border border-grid] rounded-md text-sm text-ink] outline-none focus:border-amber] font-mono"
              />
              <button
                type="button"
                onclick={() => showPasswordEye = !showPasswordEye}
                class="absolute right-3 top-3 text-dim] hover:text-ink] text-xs font-mono"
              >
                {showPasswordEye ? 'Ocultar' : 'Ver'}
              </button>
            </div>

            {#if unlockError}
              <div class="p-2.5 rounded bg-red]/15 border border-red]/30 text-xs text-red] text-left font-sans">
                {unlockError}
              </div>
            {/if}

            <div class="flex items-center gap-2 pt-1">
              <button
                type="submit"
                disabled={isAuthenticating || !unlockPassword}
                class="flex-1 h-11 bg-amber] hover:bg-amber] disabled:opacity-40 text-void] font-bold rounded-md text-sm transition-colors flex items-center justify-center gap-2"
              >
                <span>{isAuthenticating ? 'Verificando...' : 'Desbloquear Bóveda'}</span>
              </button>
              <button
                type="button"
                onclick={onClose}
                class="h-11 px-4 bg-raised] hover:bg-grid] text-dim] hover:text-white rounded-md text-xs font-semibold transition-colors"
              >
                Cancelar
              </button>
            </div>

            {#if !securityStatus.has_password}
              <p class="text-[11px] text-amber] mt-3">
                ★ No hay contraseña creada aún. Haga clic en desbloquear para acceder y configurar su clave de inspectora.
              </p>
            {/if}
          </form>
        </div>

      {:else}

        <!-- =================================================================== -->
        <!-- PANEL DE TOTALES / KPIS Y CAJA CHICA                                 -->
        <!-- =================================================================== -->
        <div class="bg-void] border-b border-grid] px-4 py-3 shrink-0">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
            
            <!-- KPI 1: Total Gastos Caja Chica -->
            <div class="bg-void] p-3 rounded border border-amber]/30">
              <div class="text-[10px] font-bold uppercase tracking-wider text-amber] font-mono">
                Gastos Caja Chica ({selectedMonth}/{selectedYear})
              </div>
              <div class="text-lg md:text-xl font-mono font-bold text-white mt-0.5">
                {formatArs(monthlyTotals?.total_amount_ars)}
              </div>
              <div class="text-[10px] text-dim] mt-0.5 font-mono">
                {monthlyTotals?.receipts_count || 0} recibos de compra
              </div>
            </div>

            <!-- KPI 2: Certificados Médicos -->
            <div class="bg-void] p-3 rounded border border-cyan]/30">
              <div class="text-[10px] font-bold uppercase tracking-wider text-cyan] font-mono">
                Certificados Médicos
              </div>
              <div class="text-lg md:text-xl font-mono font-bold text-white mt-0.5">
                {monthlyTotals?.certificates_count || 0}
              </div>
              <div class="text-[10px] text-dim] mt-0.5 font-mono">
                Carpetas y partes médicos
              </div>
            </div>

            <!-- KPI 3: Total Documentos -->
            <div class="bg-void] p-3 rounded border border-grid]">
              <div class="text-[10px] font-bold uppercase tracking-wider text-ink] font-mono">
                Documentos en Bóveda
              </div>
              <div class="text-lg md:text-xl font-mono font-bold text-white mt-0.5">
                {documents.length}
              </div>
              <div class="text-[10px] text-dim] mt-0.5 font-mono">
                Mostrados en filtro actual
              </div>
            </div>

            <!-- Botón de Alta Rápida de Documento / Recibo -->
            <div class="bg-void] p-3 rounded border border-grid] flex flex-col justify-center">
              <button
                onclick={() => { resetUploadForm(); showUploadModal = true; }}
                class="h-full py-2 px-3 bg-amber] hover:bg-amber] text-void] font-bold rounded text-xs transition-colors flex items-center justify-center gap-1.5 shadow"
              >
                <span>+ Cargar Documento / Recibo</span>
              </button>
            </div>
          </div>
        </div>

        <!-- =================================================================== -->
        <!-- BARRA DE FILTROS & BÚSQUEDA                                         -->
        <!-- =================================================================== -->
        <div class="bg-void] border-b border-grid] px-4 py-2.5 flex flex-wrap items-center justify-between gap-3 shrink-0">
          
          <!-- Filtro de Categorías -->
          <div class="flex items-center gap-1 flex-wrap">
            {#each CATEGORIES as cat}
              <button
                type="button"
                onclick={() => { selectedCategory = cat.key; loadVaultData(); }}
                class="px-2.5 py-1 rounded text-xs font-semibold border transition-colors {selectedCategory === cat.key ? 'bg-amber] text-void] border-amber]' : 'bg-raised] text-dim] border-grid] hover:text-ink]'}"
              >
                {cat.label}
              </button>
            {/each}
          </div>

          <!-- Búsqueda y Filtro de Personal -->
          <div class="flex items-center gap-2 flex-wrap">
            <input
              type="text"
              bind:value={searchQuery}
              oninput={() => loadVaultData()}
              placeholder="Buscar título, comercio, notas..."
              class="h-8 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber] w-48 sm:w-64"
            />

            <!-- Selector de Bailarín / Personal -->
            <select
              bind:value={selectedPersonnelId}
              onchange={() => loadVaultData()}
              class="h-8 px-2 bg-void] border border-grid] rounded text-xs font-semibold text-ink] outline-none focus:border-amber] max-w-[160px]"
            >
              <option value="">Personal: Todos</option>
              {#each personnelList as p}
                <option value={p.id}>{p.last_name}, {p.first_name}</option>
              {/each}
            </select>
          </div>
        </div>

        <!-- =================================================================== -->
        <!-- GRILLA DE DOCUMENTOS                                                -->
        <!-- =================================================================== -->
        <div class="flex-1 overflow-auto p-4 bg-void]">
          {#if isLoading}
            <div class="py-20 text-center text-xs font-mono text-dim]">
              Cargando documentación de la bóveda...
            </div>
          {:else if documents.length === 0}
            <div class="py-16 text-center text-dim]">
              <div class="w-12 h-12 rounded-full bg-panel] border border-grid] flex items-center justify-center mx-auto mb-2 text-dim]">
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                </svg>
              </div>
              <p class="text-xs font-semibold text-ink]">No hay documentos que coincidan con el filtro.</p>
              <p class="text-[11px] text-dim] mt-0.5">Haga clic en "+ Cargar Documento / Recibo" para subir comprobantes o certificados.</p>
            </div>
          {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {#each documents as doc}
                {@const catMeta = CATEGORIES.find(c => c.key === doc.category) || CATEGORIES[CATEGORIES.length - 1]}
                <div class="bg-panel] border border-grid] hover:border-amber]/50 rounded-lg p-3.5 flex flex-col justify-between transition-colors shadow-sm">
                  
                  <div>
                    <div class="flex items-start justify-between gap-2 mb-2">
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold font-mono border {catMeta.badge}">
                        {catMeta.label}
                      </span>
                      <span class="text-[10px] font-mono text-dim]">
                        {doc.document_date}
                      </span>
                    </div>

                    <h4 class="text-xs font-bold text-white font-sans line-clamp-2 leading-snug">
                      {doc.title}
                    </h4>

                    {#if doc.personnel_name}
                      <div class="text-[11px] font-semibold text-cyan] mt-1 flex items-center gap-1">
                        <span>Bailarín/a:</span>
                        <span>{doc.personnel_name}</span>
                      </div>
                    {/if}

                    {#if doc.category === 'RECIBO_COMPRA' && doc.amount_ars !== null}
                      <div class="mt-2 p-2 rounded bg-void] border border-grid] flex items-center justify-between">
                        <span class="text-[10px] font-mono text-dim]">Importe Pagado:</span>
                        <span class="text-sm font-mono font-bold text-amber]">
                          {formatArs(doc.amount_ars)}
                        </span>
                      </div>
                      {#if doc.supplier_vendor}
                        <div class="text-[11px] text-ink] mt-1 font-mono">
                          Comercio: <span class="font-bold">{doc.supplier_vendor}</span>
                        </div>
                      {/if}
                    {/if}

                    {#if doc.notes}
                      <p class="text-[11px] text-dim] mt-2 line-clamp-2 italic bg-void] p-1.5 rounded border border-raised]">
                        "{doc.notes}"
                      </p>
                    {/if}
                  </div>

                  <!-- Miniatura / Datos de Archivo & Acciones -->
                  <div class="mt-3 pt-2.5 border-t border-grid] flex items-center justify-between">
                    <div class="text-[10px] font-mono text-dim] truncate max-w-[140px]" title={doc.file_name}>
                      {doc.file_name} ({formatBytes(doc.file_size_bytes)})
                    </div>

                    <div class="flex items-center gap-1.5">
                      <button
                        onclick={() => viewingDocument = doc}
                        class="px-2.5 py-1 bg-raised] hover:bg-amber] hover:text-void] text-ink] rounded text-[10px] font-bold transition-colors"
                        title="Ver documento completo en alta resolución"
                      >
                        Ver
                      </button>
                      <button
                        onclick={() => handleDeleteDocument(doc)}
                        class="px-2 py-1 bg-void] hover:bg-red] text-red] hover:text-white rounded text-[10px] font-bold transition-colors"
                        title="Eliminar documento"
                      >
                        ✕
                      </button>
                    </div>
                  </div>

                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<!-- ========================================================================= -->
<!-- MODAL DE CARGA DE DOCUMENTO / FOTO DE COMPRA O CERTIFICADO               -->
<!-- ========================================================================= -->
{#if showUploadModal}
  <div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-3" use:modalBehavior={{ onclose: () => showUploadModal = false, backdrop: false }}>
    <div class="tg-modal max-w-lg">
      <ModalHeader
        title="Cargar Documento / Recibo en Bóveda"
        subtitle="Inspección del Ballet Oficial de Córdoba"
        onclose={() => showUploadModal = false}
      />

      <div class="tg-modal-body space-y-3">
        <!-- Título -->
        <div>
          <label for="vault-doc-title" class="block text-[10px] font-bold uppercase tracking-wider text-dim] mb-1 font-mono">Título del Documento *</label>
          <input
            id="vault-doc-title"
            type="text"
            bind:value={uploadTitle}
            placeholder="Ej: Factura Pez Rubia, Certificado Popi Gómez, Acuerdo Francos..."
            class="w-full h-9 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
          />
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <!-- Categoría -->
          <div>
            <label for="vault-doc-cat" class="block text-[10px] font-bold uppercase tracking-wider text-dim] mb-1 font-mono">Categoría</label>
            <select
              id="vault-doc-cat"
              bind:value={uploadCategory}
              class="w-full h-9 px-2 bg-void] border border-grid] rounded text-xs font-semibold text-ink] outline-none focus:border-amber]"
            >
              <option value="RECIBO_COMPRA">Recibo de Compra (Caja Chica)</option>
              <option value="CERTIFICADO_MEDICO">Certificado Médico</option>
              <option value="ACUERDO_DIRECCION">Acuerdo de Francos / Dirección</option>
              <option value="MEMO_DISPOSICION">Memo / Disposición Oficial</option>
              <option value="SOLISTICO_FIRMA">Solístico / Planilla Firmada</option>
              <option value="OTRO">Otro Documento</option>
            </select>
          </div>

          <!-- Fecha -->
          <div>
            <label for="vault-doc-date" class="block text-[10px] font-bold uppercase tracking-wider text-dim] mb-1 font-mono">Fecha del Comprobante</label>
            <input
              id="vault-doc-date"
              type="date"
              bind:value={uploadDate}
              class="w-full h-9 px-2 bg-void] border border-grid] rounded text-xs font-mono font-semibold text-ink] outline-none focus:border-amber]"
            />
          </div>
        </div>

        <!-- Campos Específicos para Recibo de Compra -->
        {#if uploadCategory === 'RECIBO_COMPRA'}
          <div class="p-3 rounded bg-void] border border-amber]/30 space-y-2.5">
            <span class="text-[10px] font-bold uppercase tracking-wider text-amber] font-mono block">
              Datos de Caja Chica / Compra
            </span>
            <div class="grid grid-cols-2 gap-2.5">
              <div>
                <label for="vault-doc-amount" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Monto ($ ARS)</label>
                <input
                  id="vault-doc-amount"
                  type="number"
                  step="0.01"
                  bind:value={uploadAmountArs}
                  placeholder="Ej: 45000.00"
                  class="w-full h-9 px-2.5 bg-void] border border-grid] rounded text-xs font-mono font-bold text-amber] outline-none focus:border-amber]"
                />
              </div>
              <div>
                <label for="vault-doc-vendor" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Proveedor / Comercio</label>
                <input
                  id="vault-doc-vendor"
                  type="text"
                  bind:value={uploadSupplier}
                  placeholder="Ej: Danza Córdoba, Farmacia..."
                  class="w-full h-9 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
                />
              </div>
            </div>
          </div>
        {/if}

        <!-- Bailarín Asociado (Opcional) -->
        <div>
          <label for="vault-doc-personnel" class="block text-[10px] font-bold uppercase tracking-wider text-dim] mb-1 font-mono">
            Bailarín / Agente Asociado (Opcional)
          </label>
          <select
            id="vault-doc-personnel"
            bind:value={uploadPersonnelId}
            class="w-full h-9 px-2 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
          >
            <option value="">Ninguno / Documento General</option>
            {#each personnelList as p}
              <option value={p.id}>{p.last_name}, {p.first_name}</option>
            {/each}
          </select>
        </div>

        <!-- Archivo / Cámara -->
        <div>
          <label for="vault-file-input" class="block text-[10px] font-bold uppercase tracking-wider text-dim] mb-1 font-mono">
            Foto del Comprobante o Archivo PDF *
          </label>
          <input
            id="vault-file-input"
            type="file"
            accept="image/*,application/pdf"
            capture="environment"
            onchange={handleFileSelected}
            class="w-full text-xs text-dim] file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-xs file:font-semibold file:bg-amber] file:text-void] hover:file:bg-amber] cursor-pointer"
          />
          {#if uploadFileName}
            <div class="mt-1 text-[11px] font-mono text-green]">
              Archivo seleccionado: {uploadFileName} ({formatBytes(uploadFileSize)})
            </div>
          {/if}
        </div>

        <!-- Notas -->
        <div>
          <label for="vault-doc-notes" class="block text-[10px] font-bold uppercase tracking-wider text-dim] mb-1 font-mono">Observaciones / Detalle</label>
          <textarea
            id="vault-doc-notes"
            bind:value={uploadNotes}
            rows="2"
            placeholder="Detalle de elementos comprados, motivo del certificado..."
            class="w-full p-2 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
          ></textarea>
        </div>

        {#if uploadError}
          <div class="p-2 rounded bg-red]/15 border border-red]/30 text-xs text-red]">
            {uploadError}
          </div>
        {/if}
      </div>

      <div class="tg-modal-foot">
        <button
          onclick={() => showUploadModal = false}
          class="tg-btn"
        >
          Cancelar
        </button>
        <button
          onclick={submitDocument}
          disabled={isSavingDocument}
          class="tg-btn tg-btn-primary disabled:opacity-50"
        >
          {isSavingDocument ? 'Guardando...' : 'Guardar en Bóveda'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ========================================================================= -->
<!-- MODAL DE VISOR DE DOCUMENTO A PANTALLA COMPLETA                           -->
<!-- ========================================================================= -->
{#if viewingDocument}
  <div class="fixed inset-0 z-50 bg-black/95 flex flex-col p-4" use:modalBehavior={{ onclose: () => viewingDocument = null }}>
    <div class="flex items-center justify-between pb-3 border-b border-grid] shrink-0">
      <div class="flex items-center gap-3">
        <span class="text-xs font-bold font-mono px-2 py-0.5 rounded bg-amber]/15 text-amber] border border-amber]/30">
          {viewingDocument.category}
        </span>
        <h3 class="text-sm font-bold text-white truncate max-w-xl">
          {viewingDocument.title}
        </h3>
        <span class="text-xs font-mono text-dim]">({viewingDocument.document_date})</span>
      </div>

      <div class="flex items-center gap-2">
        <a
          href={viewingDocument.file_data}
          download={viewingDocument.file_name}
          class="h-8 px-3 bg-raised] hover:bg-grid] text-green] border border-grid] rounded text-xs font-semibold flex items-center gap-1 transition-colors"
        >
          Descargar
        </a>
        <button
          onclick={() => window.print()}
          class="h-8 px-3 bg-raised] hover:bg-grid] text-ink] border border-grid] rounded text-xs font-semibold transition-colors"
        >
          Imprimir
        </button>
        <button
          onclick={() => viewingDocument = null}
          class="tg-modal-close"
          aria-label="Cerrar"
          title="Cerrar"
        >
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
            <path d="M2 2l10 10M12 2L2 12" />
          </svg>
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-auto flex items-center justify-center p-4">
      {#if viewingDocument.file_type.includes('pdf')}
        <iframe
          src={viewingDocument.file_data}
          class="w-full h-full rounded border border-grid] bg-white"
          title={viewingDocument.title}
        ></iframe>
      {:else}
        <img
          src={viewingDocument.file_data}
          alt={viewingDocument.title}
          class="max-w-full max-h-[85vh] object-contain rounded  border border-grid]"
        />
      {/if}
    </div>
  </div>
{/if}

<!-- ========================================================================= -->
<!-- MODAL DE CONFIGURACIÓN DE SEGURIDAD Y CAMBIO DE CLAVE                    -->
<!-- ========================================================================= -->
{#if showSecurityConfigModal}
  <div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-3" use:modalBehavior={{ onclose: () => showSecurityConfigModal = false, backdrop: false }}>
    <div class="tg-modal max-w-sm">
      <ModalHeader
        title="Seguridad de la Bóveda de Inspección"
        onclose={() => showSecurityConfigModal = false}
      />

      <div class="tg-modal-body space-y-3">
        {#if securityStatus.has_password}
          <div>
            <label for="sec-old-pass" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Contraseña Actual</label>
            <input
              id="sec-old-pass"
              type="password"
              bind:value={oldPasswordInput}
              class="w-full h-9 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber] font-mono"
            />
          </div>
        {/if}

        <div>
          <label for="sec-new-pass" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Nueva Contraseña</label>
          <input
            id="sec-new-pass"
            type="password"
            bind:value={newPasswordInput}
            class="w-full h-9 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber] font-mono"
          />
        </div>

        <div>
          <label for="sec-conf-pass" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Confirmar Nueva Contraseña</label>
          <input
            id="sec-conf-pass"
            type="password"
            bind:value={confirmPasswordInput}
            class="w-full h-9 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber] font-mono"
          />
        </div>

        <div>
          <label for="sec-timeout-select" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Tiempo de Bloqueo por Inactividad</label>
          <select
            id="sec-timeout-select"
            bind:value={timeoutMinutesInput}
            class="w-full h-9 px-2 bg-void] border border-grid] rounded text-xs font-semibold text-ink] outline-none focus:border-amber]"
          >
            <option value={1}>1 minuto</option>
            <option value={5}>5 minutos (Recomendado)</option>
            <option value={15}>15 minutos</option>
            <option value={30}>30 minutos</option>
            <option value={0}>Nunca bloquear automáticamente</option>
          </select>
        </div>

        {#if securityConfigError}
          <div class="p-2 rounded bg-red]/15 border border-red]/30 text-xs text-red]">
            {securityConfigError}
          </div>
        {/if}
      </div>

      <div class="tg-modal-foot">
        <button
          onclick={() => showSecurityConfigModal = false}
          class="tg-btn"
        >
          Cancelar
        </button>
        <button
          onclick={saveSecurityConfig}
          disabled={isSavingSecurityConfig}
          class="tg-btn tg-btn-primary disabled:opacity-50"
        >
          {isSavingSecurityConfig ? 'Guardando...' : 'Guardar Clave'}
        </button>
      </div>
    </div>
  </div>
{/if}
