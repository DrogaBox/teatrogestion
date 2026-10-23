<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type PersonnelWithQuotaDto,
    type SaveSoloistCertificationPayload,
    type SaveSoloistItemPayload,
    type SoloistCertificationDto,
    type WorkWithPassagesAndCastsDto,
  } from '../../ipc/client';

  let {
    isOpen,
    onClose,
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let activeTab = $state<'document' | 'editor'>('document');
  let certifications = $state<SoloistCertificationDto[]>([]);
  let currentCert = $state<SoloistCertificationDto | null>(null);
  let selectedCertId = $state<string>('');
  let isLoading = $state(false);

  // Obras disponibles para autogeneración
  let worksList = $state<WorkWithPassagesAndCastsDto[]>([]);
  let selectedWorkId = $state<string>('');
  let autoDatesLabel = $state<string>('01, 02, 03 y 04 de octubre de 2026');
  let autoPerformancesCount = $state<number>(2);

  // Personal disponible para selector rápido
  let personnelList = $state<PersonnelWithQuotaDto[]>([]);

  // Datos editables del documento actual
  let editProgramTitle = $state('');
  let editHallName = $state('Sala Mayor del Teatro del Libertador Gral. San Martín');
  let editDocumentDate = $state(new Date().toISOString().slice(0, 10));
  let editRecipientName = $state('Hadrián Ávila Arzuza');
  let editRecipientTitle = $state('Director del Teatro del Libertador');
  let editIntroBody = $state('');
  let editClosingText = $state('Sin otro particular saludo a Ud. Atte.');
  let editItems = $state<SaveSoloistItemPayload[]>([]);
  let isSaving = $state(false);

  async function loadData() {
    isLoading = true;
    try {
      const [certs, works, pers] = await Promise.all([
        api.listSoloistCertifications(),
        api.listWorksWithCasts(),
        api.listPersonnel(null, true),
      ]);
      certifications = certs;
      worksList = works;
      personnelList = pers;

      if (!selectedWorkId && works.length > 0) {
        selectedWorkId = works[0].id;
      }

      if (certs.length > 0) {
        if (!selectedCertId || !certs.some(c => c.id === selectedCertId)) {
          selectedCertId = certs[0].id;
        }
        selectCertification(selectedCertId);
      } else {
        createNewCertification();
      }
    } catch (e: any) {
      console.error('Error cargando solísticos:', e);
    } finally {
      isLoading = false;
    }
  }

  function selectCertification(id: string) {
    selectedCertId = id;
    const found = certifications.find(c => c.id === id);
    if (found) {
      currentCert = found;
      editProgramTitle = found.program_title;
      editHallName = found.hall_name;
      editDocumentDate = found.document_date;
      editRecipientName = found.recipient_name;
      editRecipientTitle = found.recipient_title;
      editIntroBody = found.intro_body;
      editClosingText = found.closing_text;
      editItems = found.items.map(i => ({
        id: i.id,
        personnel_id: i.personnel_id,
        dancer_name: i.dancer_name,
        work_role: i.work_role,
        performance_dates: i.performance_dates,
        performance_count: i.performance_count,
        sort_order: i.sort_order,
      }));
    }
  }

  function resolvePersonnelId(item: SaveSoloistItemPayload): string {
    if (item.personnel_id) return item.personnel_id;
    if (!item.dancer_name) return '';
    const norm = item.dancer_name.toLowerCase().replace(/,/g, ' ');
    const found = personnelList.find(p => {
      const pNorm = `${p.last_name} ${p.first_name}`.toLowerCase();
      const pRev = `${p.first_name} ${p.last_name}`.toLowerCase();
      return (
        norm.includes(p.last_name.toLowerCase()) &&
        (norm.includes(p.first_name.toLowerCase()) || pNorm.includes(norm) || pRev.includes(norm))
      );
    });
    if (found) {
      item.personnel_id = found.id;
      return found.id;
    }
    return '';
  }

  function createNewCertification() {
    selectedCertId = '';
    currentCert = null;
    editProgramTitle = '';
    editHallName = 'Sala Mayor del Teatro del Libertador Gral. San Martín';
    editDocumentDate = new Date().toISOString().slice(0, 10);
    editRecipientName = 'Hadrián Ávila Arzuza';
    editRecipientTitle = 'Director del Teatro del Libertador';
    editIntroBody = 'Me dirijo a Ud. a los fines de informarle los solísticos correspondientes al Ballet Oficial de la Pcia. con respecto a las funciones programadas en la Sala Mayor del Teatro del Libertador Gral. San Martín se cumplió según el siguiente detalle:';
    editClosingText = 'Sin otro particular saludo a Ud. Atte.';
    editItems = [
      {
        dancer_name: '',
        work_role: '',
        performance_dates: '',
        performance_count: 1,
        sort_order: 1,
      },
    ];
    activeTab = 'editor';
  }

  function addItemRow() {
    editItems = [
      ...editItems,
      {
        dancer_name: '',
        work_role: '',
        performance_dates: '01 y 03/10',
        performance_count: 2,
        sort_order: editItems.length + 1,
      },
    ];
  }

  function removeItemRow(index: number) {
    editItems = editItems.filter((_, i) => i !== index);
    editItems.forEach((item, i) => { item.sort_order = i + 1; });
  }

  function moveRowUp(index: number) {
    if (index <= 0) return;
    const items = [...editItems];
    const temp = items[index];
    items[index] = items[index - 1];
    items[index - 1] = temp;
    items.forEach((item, i) => { item.sort_order = i + 1; });
    editItems = items;
  }

  function moveRowDown(index: number) {
    if (index >= editItems.length - 1) return;
    const items = [...editItems];
    const temp = items[index];
    items[index] = items[index + 1];
    items[index + 1] = temp;
    items.forEach((item, i) => { item.sort_order = i + 1; });
    editItems = items;
  }

  async function handleSave() {
    if (!editProgramTitle.trim()) {
      alert('Debe especificar el título de la obra o programa.');
      return;
    }
    isSaving = true;
    try {
      const payload: SaveSoloistCertificationPayload = {
        id: selectedCertId ? selectedCertId : null,
        program_title: editProgramTitle,
        hall_name: editHallName,
        document_date: editDocumentDate,
        recipient_name: editRecipientName,
        recipient_title: editRecipientTitle,
        intro_body: editIntroBody,
        closing_text: editClosingText,
        status: 'APPROVED',
        items: editItems,
      };

      const saved = await api.saveSoloistCertification(payload);
      await loadData();
      selectCertification(saved.id);
      activeTab = 'document';
    } catch (e: any) {
      console.error('Error guardando certificación:', e);
      alert(`Error al guardar: ${e?.message || e}`);
    } finally {
      isSaving = false;
    }
  }

  async function handleAutoGenerate() {
    if (!selectedWorkId) return;
    isSaving = true;
    try {
      const generated = await api.generateSoloistCertificationFromWorkCast(
        selectedWorkId,
        autoDatesLabel,
        autoPerformancesCount
      );
      await loadData();
      selectCertification(generated.id);
      activeTab = 'document';
    } catch (e: any) {
      console.error('Error autogenerando solísticos:', e);
      alert(`Error al autogenerar: ${e?.message || e}`);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete() {
    if (!selectedCertId) return;
    if (!confirm('¿Confirma eliminar esta certificación de solísticos?')) return;
    try {
      await api.deleteSoloistCertification(selectedCertId);
      await loadData();
    } catch (e: any) {
      alert(`Error al eliminar: ${e?.message || e}`);
    }
  }

  function formatSpanishDate(isoDate: string): string {
    if (!isoDate) return '';
    try {
      const [year, month, day] = isoDate.split('-');
      const monthNames = [
        'enero', 'febrero', 'marzo', 'abril', 'mayo', 'junio',
        'julio', 'agosto', 'septiembre', 'octubre', 'noviembre', 'diciembre'
      ];
      const mIdx = parseInt(month, 10) - 1;
      return `Córdoba, ${day} de ${monthNames[mIdx] || month} de ${year}.`;
    } catch {
      return isoDate;
    }
  }

  $effect(() => {
    if (isOpen) {
      loadData();
    }
  });

  const totalPrestaciones = $derived(
    editItems.reduce((acc, curr) => acc + (Number(curr.performance_count) || 0), 0)
  );
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-2 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0">
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-5xl max-h-[94vh] flex flex-col p-4 md:p-6 shadow-2xl relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">

      <!-- ================================================================= -->
      <!-- BARRA DE HERRAMIENTAS SUPERIOR (NO IMPRIMIBLE)                    -->
      <!-- ================================================================= -->
      <div class="flex flex-wrap items-center justify-between pb-3 mb-3 border-b border-[#34343c] print:hidden gap-3">
        <div>
          <div class="flex items-center gap-2">
            <h2 class="text-xs font-semibold text-[#ffb31a] uppercase tracking-wider font-mono">
              [SOLÍSTICOS & PLUS POR PRESTACIÓN DE ROL]
            </h2>
            <span class="text-[10px] text-[#82828a] font-mono border border-[#34343c] px-1.5 py-0.5 rounded">
              Ballet Oficial de Córdoba
            </span>
          </div>
          <p class="text-[11px] text-[#82828a] mt-0.5">
            Certificación formal para liquidación de solísticos y roles principales (Agencia Córdoba Cultura)
          </p>
        </div>

        <!-- Conmutador de Pestañas -->
        <div class="inline-flex bg-[#121214] p-1 rounded border border-[#34343c]">
          <button
            onclick={() => activeTab = 'document'}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeTab === 'document' ? 'bg-[#ffb31a] text-black font-semibold' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
          >
            Nota Oficial A4
          </button>
          <button
            onclick={() => activeTab = 'editor'}
            class="px-3 py-1 rounded text-xs font-medium transition-colors {activeTab === 'editor' ? 'bg-[#ffb31a] text-black font-semibold' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
          >
            Editar Datos / Items
          </button>
        </div>

        <!-- Acciones Principales -->
        <div class="flex items-center gap-2">
          {#if activeTab === 'document'}
            <button
              onclick={() => activeTab = 'editor'}
              class="px-3 py-1.5 bg-[#24242a] hover:bg-[#34343c] text-[#ffb31a] border border-[#ffb31a]/40 rounded text-xs font-semibold font-mono transition-colors flex items-center gap-1.5"
              title="Editar los datos, obra o bailarines de esta nota"
            >
              ✎ Editar Datos
            </button>
            <button
              onclick={() => window.print()}
              class="px-3 py-1.5 bg-[#24242a] hover:bg-[#34343c] text-[#3de06a] border border-[#34343c] rounded text-xs font-semibold font-mono transition-colors flex items-center gap-1.5"
            >
              Imprimir / Guardar PDF
            </button>
          {:else}
            <button
              onclick={() => activeTab = 'document'}
              class="px-3 py-1.5 bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] border border-[#34343c] rounded text-xs font-semibold font-mono transition-colors"
            >
              Ver Nota A4
            </button>
            <button
              onclick={handleSave}
              disabled={isSaving}
              class="px-3 py-1.5 bg-[#ffb31a] hover:bg-[#e6a117] text-black rounded text-xs font-semibold font-mono transition-colors disabled:opacity-50"
            >
              {isSaving ? 'Guardando...' : 'Guardar Certificación'}
            </button>
          {/if}

          <button
            onclick={onClose}
            class="w-7 h-7 flex items-center justify-center text-[#82828a] hover:text-white bg-[#24242a] border border-[#34343c] rounded text-sm transition-colors"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Selector de Expediente / Certificación Guardada (No imprimible) -->
      <div class="bg-[#121214] p-2.5 rounded border border-[#34343c] mb-3 flex flex-wrap items-center justify-between gap-3 print:hidden">
        <div class="flex items-center gap-2 flex-1 min-w-[280px]">
          <label for="select-cert" class="text-xs font-semibold text-[#82828a] uppercase font-mono shrink-0">Programa:</label>
          <select
            id="select-cert"
            bind:value={selectedCertId}
            onchange={() => {
              if (selectedCertId) {
                selectCertification(selectedCertId);
              } else {
                createNewCertification();
              }
            }}
            class="w-full h-8 px-2.5 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-medium text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
          >
            {#if !selectedCertId}
              <option value="">+ Nueva Nota (Borrador sin guardar)</option>
            {/if}
            {#each certifications as c}
              <option value={c.id}>
                {c.program_title} ({c.document_date}) — {c.total_performances} prestaciones
              </option>
            {/each}
          </select>
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={createNewCertification}
            class="px-3 py-1 bg-[#ffb31a] hover:bg-[#e6a117] text-black font-bold border border-[#ffb31a] rounded text-xs font-mono transition-colors flex items-center gap-1 shadow-sm"
            title="Crear nueva nota oficial en blanco"
          >
            <span>+</span>
            <span>Nueva Nota</span>
          </button>
          {#if selectedCertId}
            <button
              onclick={handleDelete}
              class="px-2.5 py-1 bg-[#24242a] hover:bg-[#ff4a4a]/20 text-[#ff4a4a] border border-[#34343c] rounded text-xs font-mono transition-colors"
            >
              Eliminar
            </button>
          {/if}
        </div>
      </div>

      <!-- ================================================================= -->
      <!-- CUERPO PRINCIPAL                                                  -->
      <!-- ================================================================= -->
      <div class="flex-1 overflow-y-auto pr-1">

        <!-- PESTAÑA 1: VISTA PREVIA DEL DOCUMENTO A4 OFICIAL -->
        {#if activeTab === 'document'}
          <div class="bg-white text-black p-8 md:p-12 shadow-md rounded border border-[#34343c] font-sans max-w-[210mm] mx-auto min-h-[297mm] flex flex-col justify-between print:border-none print:shadow-none print:p-0 print:m-0 print:max-w-none print:w-full print:min-h-0">
            
            <div>
              <!-- 1. ENCABEZADO INSTITUCIONAL OBLIGATORIO DE CÓRDOBA -->
              <div class="flex items-center justify-between pb-6 pt-2 border-b border-neutral-300">
                <!-- Isotipo Teatro del Libertador San Martín -->
                <div class="flex items-center gap-3">
                  <img
                    src="/assets/logo_teatro_libertador_clean.png"
                    alt="Teatro del Libertador San Martín"
                    class="h-14 w-auto object-contain"
                  />
                </div>

                <!-- Banner Unificado: Córdoba Cultura | Gobierno de la Provincia -->
                <div class="flex items-center">
                  <img
                    src="/assets/cordoba_cultura_gob_banner_clean.png"
                    alt="Córdoba Cultura • Gobierno de la Provincia de Córdoba"
                    class="h-11 w-auto object-contain"
                  />
                </div>
              </div>

              <!-- 2. FECHA AL MARGEN DERECHO -->
              <div class="text-right pt-6 pb-4 text-xs font-serif italic text-neutral-800">
                {formatSpanishDate(editDocumentDate)}
              </div>

              <!-- 3. DESTINATARIO PROTOCOLAR -->
              <div class="text-left text-xs leading-relaxed font-serif text-neutral-900 pb-6">
                <p>Al Sr.</p>
                <p class="font-bold">Director del</p>
                <p class="font-bold">Teatro del Libertador</p>
                <p class="font-bold text-sm tracking-wide">{editRecipientName}</p>
                <p class="tracking-widest font-mono text-[11px]">S................../.................D</p>
              </div>

              <!-- 4. TEXTO INTRODUCTORIO FORMAL -->
              <div class="text-justify text-xs font-serif leading-relaxed text-neutral-800 pb-5 indent-6">
                Me dirijo a Ud. a los fines de informarle los solísticos correspondientes al Ballet Oficial de la Pcia. con respecto a las funciones <strong class="font-sans font-bold">"{editProgramTitle}"</strong> programadas en la {editHallName} los días 01, 02, 03 y 04 de octubre de 2026 se cumplió según el siguiente detalle:
              </div>

              <!-- 5. TABLA OFICIAL DE PRESTACIONES SOLÍSTICAS -->
              <table class="w-full border-collapse border border-neutral-800 text-xs mb-4 font-serif">
                <thead>
                  <tr class="bg-neutral-100 border-b border-neutral-800 font-bold text-center text-[11px] uppercase tracking-wider">
                    <th class="border border-neutral-800 py-2 px-3 text-center w-[30%]">
                      BAILARÍN/A<br/><span class="text-[9px] font-normal font-sans">Apellido y Nombre</span>
                    </th>
                    <th class="border border-neutral-800 py-2 px-3 text-center w-[32%]">
                      OBRA
                    </th>
                    <th class="border border-neutral-800 py-2 px-2 text-center w-[16%]">
                      FECHA
                    </th>
                    <th class="border border-neutral-800 py-2 px-2 text-center w-[12%] leading-tight">
                      CANTIDAD PRESTACIONES
                    </th>
                    <th class="border border-neutral-800 py-2 px-3 text-center w-[10%]">
                      FIRMA
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {#each editItems as item, idx}
                    <tr class="border-b border-neutral-600 text-xs">
                      <td class="border border-neutral-800 py-1.5 px-3 font-sans font-semibold uppercase text-neutral-900 text-[11px]">
                        {item.dancer_name || '—'}
                      </td>
                      <td class="border border-neutral-800 py-1.5 px-3 font-serif text-[11px] text-neutral-800">
                        {item.work_role || '—'}
                      </td>
                      <td class="border border-neutral-800 py-1.5 px-2 text-center font-mono text-[11px]">
                        {item.performance_dates || '—'}
                      </td>
                      <td class="border border-neutral-800 py-1.5 px-2 text-center font-bold font-mono text-sm">
                        {item.performance_count}
                      </td>
                      <td class="border border-neutral-800 py-1.5 px-2 text-center">
                        <!-- Espacio de firma física manuscrita del bailarín -->
                        <div class="h-6"></div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
                <tfoot>
                  <tr class="border-t-2 border-neutral-900 bg-neutral-100 font-bold">
                    <td colspan="3" class="border border-neutral-800 py-2 px-4 text-right uppercase text-xs font-mono">
                      Total:
                    </td>
                    <td class="border border-neutral-800 py-2 px-2 text-center font-mono text-base font-black">
                      {totalPrestaciones}
                    </td>
                    <td class="border border-neutral-800"></td>
                  </tr>
                </tfoot>
              </table>

              <!-- 6. CIERRE PROTOCOLAR -->
              <div class="text-left text-xs font-serif text-neutral-800 pt-3 pb-8">
                {editClosingText}
              </div>
            </div>

            <!-- 7. PIE DE PÁGINA INSTITUCIONAL OFICIAL -->
            <div class="pt-6 border-t border-neutral-300 text-center text-[10px] font-sans text-neutral-600 tracking-wide mt-auto">
              Av. Vélez Sarsfield 365, Córdoba, Argentina 5000
            </div>

          </div>

        <!-- PESTAÑA 2: EDITOR DE DATOS Y AUTOGENERACIÓN -->
        {:else}
          <div class="space-y-4">
            
            <!-- Tarjeta de Autogeneración desde Repartos -->
            <div class="bg-[#121214] border border-[#34343c] p-4 rounded-md">
              <h3 class="text-xs font-semibold text-[#ffb31a] uppercase font-mono mb-2">
                [Autogenerar Certificación desde Reparto de Obras]
              </h3>
              <p class="text-xs text-[#82828a] mb-3">
                Extrae automáticamente los bailarines asignados a roles de 1° Elenco, 2° Elenco y solísticos programados.
              </p>
              <div class="grid grid-cols-1 md:grid-cols-4 gap-3 items-end">
                <div>
                  <label for="auto-work" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Obra / Ballet:</label>
                  <select
                    id="auto-work"
                    bind:value={selectedWorkId}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none"
                  >
                    {#each worksList as w}
                      <option value={w.id}>{w.title}</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label for="auto-dates" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Etiqueta de Fechas:</label>
                  <input
                    id="auto-dates"
                    type="text"
                    bind:value={autoDatesLabel}
                    placeholder="ej: 01 y 03/10"
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none"
                  />
                </div>
                <div>
                  <label for="auto-count" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Funciones por Rol:</label>
                  <input
                    id="auto-count"
                    type="number"
                    min="1"
                    bind:value={autoPerformancesCount}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none text-center font-mono"
                  />
                </div>
                <div>
                  <button
                    onclick={handleAutoGenerate}
                    disabled={isSaving}
                    class="w-full h-8 bg-[#24242a] hover:bg-[#34343c] text-[#ffb31a] border border-[#34343c] rounded text-xs font-semibold font-mono transition-colors"
                  >
                    Generar Tabla
                  </button>
                </div>
              </div>
            </div>

            <!-- Datos Generales de la Nota -->
            <div class="bg-[#121214] border border-[#34343c] p-4 rounded-md space-y-3">
              <h3 class="text-xs font-semibold text-[#d8d8dc] uppercase font-mono">
                Encabezado Protocolar & Destinatario
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                <div>
                  <label for="edit-prog" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Título del Programa / Obra:</label>
                  <input
                    id="edit-prog"
                    type="text"
                    bind:value={editProgramTitle}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none font-bold"
                  />
                </div>
                <div>
                  <label for="edit-hall" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Sala / Escenario:</label>
                  <input
                    id="edit-hall"
                    type="text"
                    bind:value={editHallName}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none"
                  />
                </div>
                <div>
                  <label for="edit-date" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Fecha del Documento:</label>
                  <input
                    id="edit-date"
                    type="date"
                    bind:value={editDocumentDate}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#ffb31a] outline-none font-mono"
                  />
                </div>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">
                <div>
                  <label for="edit-recip-name" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Nombre del Director:</label>
                  <input
                    id="edit-recip-name"
                    type="text"
                    bind:value={editRecipientName}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none"
                  />
                </div>
                <div>
                  <label for="edit-recip-title" class="block text-[10px] uppercase font-mono text-[#82828a] mb-1">Cargo Formal:</label>
                  <input
                    id="edit-recip-title"
                    type="text"
                    bind:value={editRecipientTitle}
                    class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none"
                  />
                </div>
              </div>
            </div>

            <!-- Tabla de Ítems / Solísticos Individuales -->
            <div class="bg-[#121214] border border-[#34343c] p-4 rounded-md">
              <div class="flex items-center justify-between mb-3">
                <h3 class="text-xs font-semibold text-[#d8d8dc] uppercase font-mono">
                  Bailarines y Prestaciones Certificadas ({editItems.length} registros • Total {totalPrestaciones} funciones)
                </h3>
                <button
                  onclick={addItemRow}
                  class="px-2.5 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#3de06a] border border-[#34343c] rounded text-xs font-mono transition-colors"
                >
                  + Agregar Fila
                </button>
              </div>

              <div class="space-y-2">
                {#each editItems as item, idx}
                  {@const resolvedId = resolvePersonnelId(item)}
                  <div class="flex flex-wrap items-center gap-2 p-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs">
                    <span class="w-6 text-center font-mono text-[#82828a]">{idx + 1}</span>
                    
                    <div class="flex-1 min-w-[220px]">
                      <select
                        class="w-full h-7 px-2 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc] font-bold outline-none focus:border-[#ffb31a]"
                        value={resolvedId}
                        onchange={(e) => {
                          const target = e.target as HTMLSelectElement;
                          const val = target.value;
                          if (val === '__MANUAL__') {
                            item.personnel_id = null;
                          } else if (val) {
                            const found = personnelList.find(p => p.id === val);
                            if (found) {
                              item.personnel_id = found.id;
                              item.dancer_name = `${found.last_name.toUpperCase()}, ${found.first_name}`;
                            }
                          } else {
                            item.personnel_id = null;
                            item.dancer_name = '';
                          }
                        }}
                      >
                        <option value="">-- Seleccionar Bailarín del Padrón --</option>
                        {#each personnelList as p}
                          <option value={p.id}>
                            {p.last_name.toUpperCase()}, {p.first_name} {p.legajo ? `[DNI: ${p.legajo}]` : ''} ({p.category_name})
                          </option>
                        {/each}
                        <option value="__MANUAL__">[+] Nombre manual / Invitado externo</option>
                      </select>

                      {#if !resolvedId}
                        <input
                          type="text"
                          bind:value={item.dancer_name}
                          placeholder="APELLIDO, Nombre (Invitado o Manual)"
                          class="w-full h-7 mt-1 px-2 bg-[#121214] border border-[#ffb31a]/40 rounded text-xs text-[#ffb31a] uppercase font-bold outline-none"
                        />
                      {/if}
                    </div>

                    <div class="flex-1 min-w-[200px]">
                      <input
                        type="text"
                        bind:value={item.work_role}
                        placeholder="Obra / Rol / Variación"
                        class="w-full h-7 px-2 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc]"
                      />
                    </div>

                    <div class="w-28">
                      <input
                        type="text"
                        bind:value={item.performance_dates}
                        placeholder="Fechas (ej: 01 y 03/10)"
                        class="w-full h-7 px-2 bg-[#121214] border border-[#34343c] rounded text-xs text-center font-mono text-[#82828a]"
                      />
                    </div>

                    <div class="w-16">
                      <input
                        type="number"
                        min="1"
                        bind:value={item.performance_count}
                        class="w-full h-7 px-1 bg-[#121214] border border-[#34343c] rounded text-xs text-center font-mono text-[#ffb31a] font-bold"
                      />
                    </div>

                    <div class="flex items-center gap-0.5">
                      <button
                        onclick={() => moveRowUp(idx)}
                        disabled={idx === 0}
                        class="w-6 h-7 flex items-center justify-center text-[#82828a] hover:text-[#d8d8dc] hover:bg-[#24242a] rounded transition-colors disabled:opacity-20 disabled:hover:bg-transparent"
                        title="Subir fila"
                      >
                        ▲
                      </button>
                      <button
                        onclick={() => moveRowDown(idx)}
                        disabled={idx === editItems.length - 1}
                        class="w-6 h-7 flex items-center justify-center text-[#82828a] hover:text-[#d8d8dc] hover:bg-[#24242a] rounded transition-colors disabled:opacity-20 disabled:hover:bg-transparent"
                        title="Bajar fila"
                      >
                        ▼
                      </button>
                      <button
                        onclick={() => removeItemRow(idx)}
                        class="w-6 h-7 flex items-center justify-center text-[#ff4a4a] hover:bg-[#ff4a4a]/20 rounded transition-colors font-bold text-sm ml-1"
                        title="Eliminar fila"
                      >
                        ×
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </div>

          </div>
        {/if}

      </div>
    </div>
  </div>
{/if}
