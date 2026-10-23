<script lang="ts">
  import { roleLabel } from '../../labels';
  import {
    api,
    type ImportPersonnelResult,
    type PersonnelRole,
    type PersonnelWithQuotaDto,
    type UpsertPersonnelPayload
  } from '../../ipc/client';

  let {
    isOpen,
    onClose
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let roster = $state<PersonnelWithQuotaDto[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let roleFilter = $state<string>('ALL');
  let filterRetiredOnly = $state(false);

  // Estado del formulario de edición / alta
  let showEditForm = $state(false);
  let formId = $state<string | null>(null);
  let formLegajo = $state('');
  let formFirstName = $state('');
  let formLastName = $state('');
  let formNickname = $state('');
  let formRole = $state<PersonnelRole>('DANCER');
  let formSpecificPosition = $state('');
  let formGender = $state<'FEMALE' | 'MALE'>('FEMALE');
  let formCategoryId = $state('CAT_CUERPO_BAILE');
  let formIsRetired = $state(false);
  let formRetirementNotes = $state('');
  let formIsActive = $state(true);
  let formNotes = $state('');

  // Estado del importador de CSV
  let showImportModal = $state(false);
  let importRawText = $state('');
  let importResult = $state<ImportPersonnelResult | null>(null);
  let isImporting = $state(false);

  async function loadRoster() {
    isLoading = true;
    try {
      roster = await api.listPersonnel(null, false);
    } catch (err) {
      console.error('Error cargando padrón:', err);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      loadRoster();
    }
  });

  let filteredRoster = $derived(
    roster.filter((p) => {
      if (filterRetiredOnly && !p.is_retired) return false;
      if (roleFilter !== 'ALL' && p.role !== roleFilter) return false;
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const full = `${p.first_name} ${p.last_name} ${p.nickname || ''} ${p.legajo || ''} ${p.specific_position || ''}`.toLowerCase();
        if (!full.includes(q)) return false;
      }
      return true;
    })
  );

  function openNewPersonnel() {
    formId = null;
    formLegajo = '';
    formFirstName = '';
    formLastName = '';
    formNickname = '';
    formRole = 'DANCER';
    formSpecificPosition = '';
    formGender = 'FEMALE';
    formCategoryId = 'CAT_CUERPO_BAILE';
    formIsRetired = false;
    formRetirementNotes = '';
    formIsActive = true;
    formNotes = '';
    showEditForm = true;
  }

  function openEditPersonnel(p: PersonnelWithQuotaDto) {
    formId = p.id;
    formLegajo = p.legajo || '';
    formFirstName = p.first_name;
    formLastName = p.last_name;
    formNickname = p.nickname || '';
    formRole = p.role;
    formSpecificPosition = p.specific_position || '';
    formGender = (p.gender_group === 'MALE' ? 'MALE' : 'FEMALE');
    formCategoryId = p.category_id;
    formIsRetired = p.is_retired;
    formRetirementNotes = p.retirement_notes || '';
    formIsActive = p.is_active;
    formNotes = '';
    showEditForm = true;
  }

  async function savePersonnel(e: Event) {
    e.preventDefault();
    if (!formFirstName.trim() || !formLastName.trim()) {
      alert('Nombre y Apellido son obligatorios.');
      return;
    }

    try {
      const payload: UpsertPersonnelPayload = {
        id: formId,
        legajo: formLegajo.trim() || null,
        first_name: formFirstName.trim(),
        last_name: formLastName.trim(),
        nickname: formNickname.trim() || null,
        role: formRole,
        gender_group: formGender,
        category_id: formCategoryId,
        default_pointe_raw_code: null,
        default_flat_size: null,
        is_active: formIsActive,
        is_retired: formIsRetired,
        retirement_notes: formRetirementNotes.trim() || null,
        specific_position: formSpecificPosition.trim() || null,
        notes: formNotes.trim() || null,
      };

      await api.upsertPersonnel(payload);
      showEditForm = false;
      loadRoster();
    } catch (err: any) {
      alert(`Error al guardar: ${err?.message || err}`);
    }
  }

  async function handleDeletePersonnel(p: { id: string; first_name: string; last_name: string }) {
    if (!confirm(`¿Está seguro de que desea eliminar a "${p.first_name} ${p.last_name}" del padrón?\n\nSi tiene vales entregados se dará de baja preservando la auditoría legal.`)) {
      return;
    }
    try {
      await api.deletePersonnel(p.id);
      if (formId === p.id) showEditForm = false;
      await loadRoster();
    } catch (err: any) {
      alert(`Error al eliminar personal: ${err?.message || err}`);
    }
  }

  // Importar desde archivo o texto plano CSV/TSV
  async function executeImport() {
    if (!importRawText.trim()) {
      alert('Pega o selecciona el contenido de la lista para importar.');
      return;
    }

    isImporting = true;
    importResult = null;

    try {
      const lines = importRawText.split(/\r?\n/).filter((l) => l.trim().length > 0);
      const batch: UpsertPersonnelPayload[] = [];

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        // Ignorar encabezado si existe
        if (i === 0 && (line.toLowerCase().includes('apellido') || line.toLowerCase().includes('nombre'))) {
          continue;
        }

        // Detección automática de separador (; o tabulador o coma)
        let parts: string[] = [];
        if (line.includes(';')) parts = line.split(';');
        else if (line.includes('\t')) parts = line.split('\t');
        else parts = line.split(',');

        if (parts.length >= 2) {
          const legajo = parts[0]?.trim() || null;
          const apellido = parts[1]?.trim() || '';
          const nombre = parts[2]?.trim() || '';
          const rolRaw = parts[3]?.trim().toUpperCase() || 'DANCER';
          const cargo = parts[4]?.trim() || null;
          const jubiladoRaw = parts[5]?.trim().toLowerCase() || 'no';

          let mappedRole: PersonnelRole = 'DANCER';
          if (rolRaw.includes('MAEST') || rolRaw === 'MASTER') mappedRole = 'MASTER';
          else if (rolRaw.includes('PIAN') || rolRaw === 'PIANIST') mappedRole = 'PIANIST';
          else if (rolRaw.includes('INSPEC') || rolRaw.includes('SUPERV') || rolRaw === 'INSPECTOR') mappedRole = 'INSPECTOR';
          else if (rolRaw.includes('TECN') || rolRaw.includes('SONID') || rolRaw === 'SOUND_TECH') mappedRole = 'SOUND_TECH';

          const isRetired = jubiladoRaw === 'si' || jubiladoRaw === 'sí' || jubiladoRaw === 'true' || jubiladoRaw === '1' || jubiladoRaw.includes('jubilad');

          batch.push({
            id: null,
            legajo,
            first_name: nombre || apellido,
            last_name: nombre ? apellido : 'Personal',
            nickname: null,
            role: mappedRole,
            gender_group: 'FEMALE',
            category_id: 'CAT_CUERPO_BAILE',
            default_pointe_raw_code: null,
            default_flat_size: null,
            is_active: true,
            is_retired: isRetired,
            retirement_notes: isRetired ? 'Importado en estado jubilatorio' : null,
            specific_position: cargo,
            notes: null,
          });
        }
      }

      if (batch.length === 0) {
        alert('No se detectaron filas válidas para importar. Revisa el formato.');
        return;
      }

      const res = await api.importPersonnelBatch(batch);
      importResult = res;
      loadRoster();
    } catch (err: any) {
      alert(`Fallo en la importación: ${err?.message || err}`);
    } finally {
      isImporting = false;
    }
  }

  function handleFileSelect(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      importRawText = (event.target?.result as string) || '';
    };
    reader.readAsText(file);
  }

  // Exportar Excel en formato compatible con Windows 11 Excel (BOM UTF-8 y delimitador ;)
  function exportExcel() {
    let csv = '\uFEFF';
    csv += 'TEATRO DEL LIBERTADOR SAN MARTIN - BALLET OFICIAL DE CORDOBA\n';
    csv += 'PADRON INTEGRAL DEL SECTOR Y ESTADO DE REVISTA\n';
    csv += `FECHA DE EMISION:;${new Date().toLocaleDateString('es-AR')} ${new Date().toLocaleTimeString('es-AR')}\n\n`;
    csv += 'DNI;APELLIDO;NOMBRE;APODO;ROL;CARGO_ESPECIFICO;CATEGORIA;ESTADO_JUBILATORIO;NOTAS_JUBILACION;CUPO_REST_PUNTAS;CUPO_REST_MEDIA_PUNTAS\n';

    filteredRoster.forEach((p) => {
      const jub = p.is_retired ? 'SI (NO CONCURRE)' : 'NO';
      csv += `"${p.legajo || ''}";"${p.last_name}";"${p.first_name}";"${p.nickname || ''}";"${p.role}";"${p.specific_position || ''}";"${p.category_name}";"${jub}";"${p.retirement_notes || ''}";${p.remaining_pointe_qty};${p.remaining_flat_qty}\n`;
    });

    csv += '\n"★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★"\n';

    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', `Padron_Oficial_Ballet_Oficial_Cordoba_${new Date().toISOString().split('T')[0]}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-2 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0">
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-6xl max-h-[92vh] flex flex-col overflow-hidden shadow-2xl relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">
      
      <!-- Barra Superior (No imprimible) -->
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 shrink-0 flex items-center justify-between print:hidden">
        <div class="flex items-center gap-2.5">
          <span class="text-[10px] font-mono font-bold text-[#ffb31a] bg-[#121214] px-2 py-0.5 rounded border border-[#34343c]">PADRÓN</span>
          <div>
            <h2 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider flex items-center gap-2">
              Padrón Integral del Sector
              <span class="text-[10px] font-mono text-[#82828a]">
                ({filteredRoster.length} miembros)
              </span>
            </h2>
            <p class="text-[11px] text-[#82828a]">
              Ballet Oficial de Córdoba • Bailarines, Maestros, Pianistas, Dirección e Inspección
            </p>
          </div>
        </div>

        <!-- Botones de Acción -->
        <div class="flex items-center gap-2 flex-wrap">
          <button
            onclick={openNewPersonnel}
            class="px-2.5 py-1 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] rounded text-xs font-bold transition-colors"
          >
            + Nuevo Personal
          </button>

          <button
            onclick={() => { showImportModal = true; importResult = null; }}
            class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#ffb31a] border border-[#ffb31a]/40 rounded text-xs font-bold transition-colors"
          >
            Importar Excel / CSV
          </button>

          <button
            onclick={exportExcel}
            class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#3de06a] border border-[#3de06a]/40 rounded text-xs font-bold transition-colors"
            title="Exportar archivo compatible con Microsoft Excel en Windows 11"
          >
            Exportar Excel
          </button>

          <button
            onclick={() => window.print()}
            class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#d8d8dc] border border-[#34343c] rounded text-xs font-bold transition-colors"
          >
            Exportar PDF / Imprimir
          </button>

          <button
            onclick={onClose}
            class="text-[#82828a] hover:text-white px-2 py-0.5 rounded hover:bg-[#34343c] text-sm font-mono"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Filtros Rápidos (No imprimibles) -->
      <div class="bg-[#17171b] px-4 py-2 border-b border-[#34343c] print:hidden">
        <div class="grid grid-cols-1 sm:grid-cols-12 gap-2 items-center">
          <div class="sm:col-span-5 relative">
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Buscar por nombre, apellido, DNI o apodo..."
              class="w-full h-8 pl-3 pr-8 bg-[#121214] border border-[#34343c] focus:border-[#ffb31a] rounded text-xs font-bold text-[#d8d8dc] outline-none placeholder-[#82828a]"
            />
            {#if searchQuery}
              <button
                onclick={() => searchQuery = ''}
                class="absolute right-2 top-1.5 text-[#82828a] hover:text-white text-xs"
              >
                ✕
              </button>
            {/if}
          </div>

          <div class="sm:col-span-4">
            <select
              bind:value={roleFilter}
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] focus:border-[#ffb31a] rounded text-xs font-bold text-[#d8d8dc] outline-none"
            >
              <option value="ALL">Todos los Roles y Cargos</option>
              <option value="DANCER">Bailarines</option>
              <option value="MASTER">Maestros Ensayistas</option>
              <option value="PIANIST">Músicos Pianistas</option>
              <option value="INSPECTOR">Inspección / Supervisión</option>
              <option value="SOUND_TECH">Asistencia Técnica / Sonido</option>
            </select>
          </div>

          <div class="sm:col-span-3">
            <label class="inline-flex items-center gap-2 cursor-pointer text-xs font-medium text-[#d8d8dc]">
              <input
                type="checkbox"
                bind:checked={filterRetiredOnly}
                class="w-4 h-4 text-[#ffb31a] rounded bg-[#121214] border-[#34343c]"
              />
              <span class="text-[#ffb31a] font-bold">
                Solo Estado Jubilatorio
              </span>
            </label>
          </div>
        </div>
      </div>

      <!-- ================================================================= -->
      <!-- TABLA OFICIAL DE PADRÓN / DOCUMENTO IMPRIMIBLE A4                  -->
      <!-- ================================================================= -->
      <div class="flex-1 overflow-y-auto p-4 text-[#d8d8dc] print:text-black print:overflow-visible">
        
        <!-- Encabezado Institucional Exclusivo de Impresión (PDF / Papel) -->
        <div class="hidden print:block text-center pb-4 mb-4 border-b-2 border-black">
          <h1 class="text-lg font-black tracking-tight uppercase">
            Teatro del Libertador San Martín • Ballet Oficial de Córdoba
          </h1>
          <h2 class="text-sm font-bold uppercase tracking-widest text-slate-700 mt-0.5">
            Padrón Oficial del Sector Artístico, Docente, Musical y Técnico
          </h2>
          <div class="flex justify-between items-center text-xs font-mono mt-2 pt-1 border-t border-slate-400">
            <span>Fecha y Hora de Emisión: {new Date().toLocaleDateString('es-AR')} - {new Date().toLocaleTimeString('es-AR')}</span>
            <span>Total Miembros: {filteredRoster.length}</span>
          </div>
        </div>

        {#if isLoading}
          <div class="py-12 text-center text-[#82828a] font-mono text-xs flex items-center justify-center gap-2">
            Consultando base de datos SQLite...
          </div>
        {:else if filteredRoster.length === 0}
          <div class="py-12 text-center bg-[#17171b] rounded border border-[#34343c] text-[#82828a] text-xs">
            No se encontraron miembros para el criterio de búsqueda.
          </div>
        {:else}
          <div class="overflow-x-auto">
            <table class="w-full text-left border-collapse text-xs print:text-[10px]">
              <thead>
                <tr class="bg-[#17171b] border-b border-[#34343c] text-[#82828a] uppercase tracking-wider font-mono text-[10px] print:bg-slate-200 print:text-black print:border-black">
                  <th class="py-2 px-3">DNI</th>
                  <th class="py-2 px-3">Apellido y Nombre</th>
                  <th class="py-2 px-3">Rol / Cargo Específico</th>
                  <th class="py-2 px-3">Categoría de Revista</th>
                  <th class="py-2 px-3 text-center">Estado</th>
                  <th class="py-2 px-2 text-center">Cupo Puntas</th>
                  <th class="py-2 px-2 text-center">Cupo Media P.</th>
                  <th class="py-2 px-2 text-center print:hidden">Acción</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-[#34343c]/60 print:divide-black">
                {#each filteredRoster as p}
                  <tr class="hover:bg-[#24242a] transition-colors print:hover:bg-transparent {p.is_retired ? 'bg-[#ffb31a]/5' : ''}">
                    <!-- Legajo -->
                    <td class="py-2 px-3 font-mono text-[#82828a] font-bold print:text-black">
                      {p.legajo || '—'}
                    </td>

                    <!-- Apellido, Nombre y Apodo -->
                    <td class="py-2 px-3">
                      <div class="font-bold text-[#d8d8dc] print:text-black flex items-center gap-1.5">
                        <span>{p.last_name}, {p.first_name}</span>
                        {#if p.nickname}
                          <span class="bg-[#ffb31a]/20 text-[#ffb31a] border border-[#ffb31a]/30 text-[10px] px-1.5 py-0.2 rounded font-mono">
                            "{p.nickname}"
                          </span>
                        {/if}
                      </div>
                    </td>

                    <!-- Rol y Cargo Específico -->
                    <td class="py-2 px-3 text-[#d8d8dc] print:text-black">
                      <div class="font-bold text-xs">{roleLabel(p.role)}</div>
                      {#if p.specific_position}
                        <div class="text-[11px] text-[#ffb31a] font-medium print:text-slate-700">
                          {p.specific_position}
                        </div>
                      {/if}
                    </td>

                    <!-- Categoría de Revista -->
                    <td class="py-2 px-3 text-[11px] text-[#82828a] print:text-black">
                      {p.category_name}
                    </td>

                    <!-- Estado Jubilatorio -->
                    <td class="py-2 px-3 text-center">
                      {#if p.is_retired}
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold bg-[#ffb31a]/15 text-[#ffb31a] border border-[#ffb31a]/30 print:border-black print:text-black" title={p.retirement_notes || 'En trámite jubilatorio'}>
                          Jubilatorio (No concurre)
                        </span>
                      {:else}
                        <span class="inline-flex px-2 py-0.5 rounded text-[10px] font-bold bg-[#3de06a]/10 text-[#3de06a] border border-[#3de06a]/30 print:border-none print:text-black">
                          En Actividad
                        </span>
                      {/if}
                    </td>

                    <!-- Cupo Puntas -->
                    <td class="py-2 px-2 text-center font-mono font-bold {p.remaining_pointe_qty > 0 ? 'text-[#ffb31a]' : 'text-[#82828a]'} print:text-black">
                      {p.remaining_pointe_qty} / {p.effective_pointe_quota}
                    </td>

                    <!-- Cupo Media Punta -->
                    <td class="py-2 px-2 text-center font-mono font-bold {p.remaining_flat_qty > 0 ? 'text-[#3de06a]' : 'text-[#82828a]'} print:text-black">
                      {p.remaining_flat_qty} / {p.effective_flat_quota}
                    </td>

                    <!-- Acciones (Oculto en Impresión) -->
                    <td class="py-2 px-2 text-center print:hidden">
                      <div class="flex items-center justify-center gap-1">
                        <button
                          onclick={() => openEditPersonnel(p)}
                          class="px-2 py-0.5 rounded bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] text-xs font-mono border border-[#34343c]"
                          title="Editar personal"
                        >
                          Editar
                        </button>
                        <button
                          onclick={() => handleDeletePersonnel(p)}
                          class="px-2 py-0.5 rounded bg-[#24242a] hover:bg-[#ff4a4a]/20 border border-[#34343c] text-[#82828a] hover:text-[#ff4a4a] text-xs font-mono"
                          title="Eliminar del padrón"
                        >
                          ✕
                        </button>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}

        <!-- Pie de Impresión Institucional -->
        <div class="hidden print:block mt-12 pt-6 border-t-2 border-black">
          <p class="text-center font-mono font-black text-xs tracking-wider uppercase mb-8">
            ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
          </p>
          <div class="grid grid-cols-2 gap-12 text-center text-xs pt-8">
            <div class="border-t border-black pt-2">
              <p class="font-extrabold uppercase">Inspectora / Supervisora General</p>
              <p class="text-[10px] text-slate-600">Ballet Oficial de Córdoba</p>
              <p class="text-[9px] text-slate-500 mt-1">Firma y Aclaración</p>
            </div>
            <div class="border-t border-black pt-2">
              <p class="font-extrabold uppercase">Dirección Artística</p>
              <p class="text-[10px] text-slate-600">Teatro del Libertador San Martín</p>
              <p class="text-[9px] text-slate-500 mt-1">Firma y Sello Oficial</p>
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>
{/if}

<!-- Sub-Modal: Formulario de Alta y Edición de Personal -->
{#if showEditForm}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3 md:p-6 overflow-y-auto">
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-lg shadow-2xl overflow-hidden flex flex-col text-[#d8d8dc]">
      
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 flex items-center justify-between">
        <h3 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider">
          {formId ? 'Modificar Registro de Personal' : 'Alta de Nuevo Personal'}
        </h3>
        <button
          onclick={() => showEditForm = false}
          class="text-[#82828a] hover:text-white px-2 py-0.5 rounded hover:bg-[#34343c] text-sm font-mono"
        >
          ✕
        </button>
      </div>

      <form onsubmit={savePersonnel} class="p-4 space-y-3 text-xs">
        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label class="block text-[#82828a] font-bold mb-1 text-[10px] uppercase" for="legajo-input">DNI</label>
            <input
              id="legajo-input"
              type="text"
              bind:value={formLegajo}
              placeholder="Ej: 36.412.890"
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-mono font-bold outline-none focus:border-[#ffb31a]"
            />
          </div>
          <div>
            <label class="block text-[#82828a] font-bold mb-1 text-[10px] uppercase" for="nickname-input">Apodo (Opcional)</label>
            <input
              id="nickname-input"
              type="text"
              bind:value={formNickname}
              placeholder="Ej: Popi, Brenda..."
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-bold outline-none focus:border-[#ffb31a]"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label class="block text-[#82828a] font-bold mb-1 text-[10px] uppercase" for="lastname-input">Apellido *</label>
            <input
              id="lastname-input"
              type="text"
              bind:value={formLastName}
              placeholder="Apellido"
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-bold outline-none focus:border-[#ffb31a]"
              required
            />
          </div>
          <div>
            <label class="block text-[#82828a] font-bold mb-1 text-[10px] uppercase" for="firstname-input">Nombre *</label>
            <input
              id="firstname-input"
              type="text"
              bind:value={formFirstName}
              placeholder="Nombre"
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-bold outline-none focus:border-[#ffb31a]"
              required
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label class="block text-[#82828a] font-bold mb-1 text-[10px] uppercase" for="role-select">Rol Institucional *</label>
            <select
              id="role-select"
              bind:value={formRole}
              class="w-full h-8 px-2 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-bold outline-none focus:border-[#ffb31a]"
            >
              <option value="DANCER">Bailarín / Bailarina</option>
              <option value="MASTER">Maestro / Directora de Ensayos</option>
              <option value="PIANIST">Músico Pianista</option>
              <option value="INSPECTOR">Supervisora / Inspectora</option>
              <option value="SOUND_TECH">Asistencia Técnica / Sonido</option>
            </select>
          </div>
          <div>
            <label class="block text-[#82828a] font-bold mb-1 text-[10px] uppercase" for="position-input">Cargo Específico</label>
            <input
              id="position-input"
              type="text"
              bind:value={formSpecificPosition}
              placeholder="Ej: Supervisora General, Solista..."
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-bold outline-none focus:border-[#ffb31a]"
            />
          </div>
        </div>

        <!-- Casillero de Estado Jubilatorio -->
        <div class="p-2.5 bg-[#17171b] border border-[#34343c] rounded space-y-1.5">
          <label class="inline-flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={formIsRetired}
              class="w-4 h-4 text-[#ffb31a] rounded bg-[#121214] border-[#34343c]"
            />
            <span class="text-[#ffb31a] font-bold text-xs">
              En Estado Jubilatorio (No concurre a la marcada)
            </span>
          </label>
          {#if formIsRetired}
            <div>
              <label class="block text-[10px] text-[#82828a] font-bold uppercase mb-1" for="retirement-notes-input">
                Expediente o Detalles de la Exención
              </label>
              <input
                id="retirement-notes-input"
                type="text"
                bind:value={formRetirementNotes}
                placeholder="Ej: Exp. previsional Caja de Jubilaciones de Cba..."
                class="w-full h-8 px-2.5 bg-[#121214] border border-[#ffb31a]/40 rounded text-[#d8d8dc] text-xs outline-none focus:border-[#ffb31a]"
              />
            </div>
          {/if}
        </div>

        <div class="flex items-center justify-between pt-2 border-t border-[#34343c]">
          <div>
            {#if formId}
              <button
                type="button"
                onclick={() => handleDeletePersonnel({ id: formId!, first_name: formFirstName, last_name: formLastName })}
                class="px-2.5 py-1 rounded bg-[#24242a] hover:bg-[#ff4a4a]/20 border border-[#34343c] text-[#82828a] hover:text-[#ff4a4a] text-xs font-mono"
              >
                Eliminar Registro
              </button>
            {/if}
          </div>
          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={() => showEditForm = false}
              class="px-3 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] rounded text-xs border border-[#34343c]"
            >
              Cancelar
            </button>
            <button
              type="submit"
              class="px-3 py-1 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] font-bold rounded text-xs"
            >
              Guardar Personal
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Sub-Modal: Importación Masiva desde Excel / CSV -->
{#if showImportModal}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3 md:p-6 overflow-y-auto">
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-xl shadow-2xl overflow-hidden flex flex-col text-[#d8d8dc]">
      
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 flex items-center justify-between">
        <h3 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider">
          Importación Masiva de Personal (Excel / CSV)
        </h3>
        <button
          onclick={() => showImportModal = false}
          class="text-[#82828a] hover:text-white px-2 py-0.5 rounded hover:bg-[#34343c] text-sm font-mono"
        >
          ✕
        </button>
      </div>

      <div class="p-4 space-y-3 text-xs">
        <p class="text-[#82828a]">
          Puedes seleccionar un archivo <strong class="text-[#ffb31a]">.csv</strong> o pegar las filas directamente desde Microsoft Excel.
        </p>

        <!-- Selector de archivo -->
        <div>
          <label class="block text-[#82828a] font-bold text-[10px] uppercase mb-1" for="file-upload">
            Subir archivo (.csv / .txt)
          </label>
          <input
            id="file-upload"
            type="file"
            accept=".csv,.txt,.tsv"
            onchange={handleFileSelect}
            class="w-full text-xs text-[#82828a] file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-xs file:font-bold file:bg-[#24242a] file:text-[#ffb31a] hover:file:bg-[#34343c]"
          />
        </div>

        <!-- Área de texto -->
        <div>
          <label class="block text-[#82828a] font-bold text-[10px] uppercase mb-1" for="raw-csv-textarea">
            O pega las líneas aquí (Formato: DNI;Apellido;Nombre;Rol;Cargo;Jubilado)
          </label>
          <textarea
            id="raw-csv-textarea"
            bind:value={importRawText}
            rows="6"
            placeholder="34.789.012;Gaitán;Matías;DANCER;Primer Bailarín;No&#10;28.456.123;Lion;Carina;INSPECTOR;Supervisora General;No&#10;32.901.234;Vázquez;Beatriz;DANCER;Cuerpo de Baile;Si"
            class="w-full p-2.5 bg-[#121214] border border-[#34343c] rounded text-[#d8d8dc] font-mono text-[11px] outline-none focus:border-[#ffb31a]"
          ></textarea>
        </div>

        {#if importResult}
          <div class="p-2.5 bg-[#17171b] border border-[#3de06a]/40 rounded text-[#3de06a]">
            <p class="font-bold text-xs">✓ Importación completada con éxito:</p>
            <p class="text-[11px] font-mono">Insertados: <strong>{importResult.inserted_count}</strong> | Actualizados: <strong>{importResult.updated_count}</strong></p>
          </div>
        {/if}

        <div class="flex items-center justify-end gap-2 pt-2 border-t border-[#34343c]">
          <button
            onclick={() => showImportModal = false}
            class="px-3 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] rounded text-xs border border-[#34343c]"
          >
            Cerrar
          </button>
          <button
            onclick={executeImport}
            disabled={isImporting}
            class="px-3 py-1 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] font-bold rounded text-xs transition-colors disabled:opacity-50"
          >
            {isImporting ? 'Procesando...' : 'Ejecutar Importación'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
