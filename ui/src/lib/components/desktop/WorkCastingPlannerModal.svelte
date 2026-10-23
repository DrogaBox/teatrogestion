<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type PersonnelWithQuotaDto,
    type WorkCastingPlanDto,
    type WorkPassageDto,
    type WorkWithPassagesAndCastsDto,
    type UpsertWorkCastingPlanPayload,
    type UpsertWorkCastPlanMemberPayload,
    type CastAssignmentCondition
  } from '../../ipc/client';

  let {
    isOpen,
    onClose
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let works = $state<WorkWithPassagesAndCastsDto[]>([]);
  let selectedWorkId = $state<string>('');
  let isLoading = $state(false);

  // Padrón de Bailarines
  let personnelList = $state<PersonnelWithQuotaDto[]>([]);
  let dancers = $derived(personnelList.filter(p => p.role === 'DANCER' && p.is_active));

  // Vista activa dentro de la obra
  let currentSection = $state<'casts' | 'passages' | 'print'>('casts');

  // Modal de Edición de Obra
  let showWorkModal = $state(false);
  let editWorkId = $state<string | null>(null);
  let editWorkTitle = $state('');
  let editWorkComposer = $state('');
  let editWorkChoreographer = $state('');

  // Modal de Edición de Pasaje
  let showPassageModal = $state(false);
  let editPassageId = $state<string | null>(null);
  let editPassageName = $state('');
  let editPassageActScene = $state('');
  let editPassageDuration = $state(60);
  let editPassageSoundCue = $state('PISTA TeatroCue');

  // Modal de Edición de Plan de Reparto (Elenco)
  let showPlanModal = $state(false);
  let editPlanId = $state<string | null>(null);
  let editPlanPassageId = $state<string>('');
  let editPlanTier = $state('1° Elenco');
  let editPlanLabel = $state('');
  let editPlanSoundCue = $state('PISTA TeatroCue');
  let editPlanNotes = $state('');
  
  // Miembros configurados en el modal de plan
  interface TempMember {
    personnel_id: string;
    dancer_name: string;
    character_role: string;
    partner_personnel_id: string;
    condition: CastAssignmentCondition;
  }
  let planMembers = $state<TempMember[]>([]);
  let dancerSearch = $state('');

  async function loadData() {
    isLoading = true;
    try {
      const [wList, pList] = await Promise.all([
        api.listWorksWithCasts(),
        api.listPersonnel(null, true)
      ]);
      works = wList;
      personnelList = pList;
      if (!selectedWorkId && works.length > 0) {
        selectedWorkId = works[0].id;
      }
    } catch (e: any) {
      console.error('Error cargando repertorio y repartos:', e);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      loadData();
    }
  });

  const selectedWork = $derived(works.find(w => w.id === selectedWorkId) || null);

  // Guardar Obra
  async function saveWork() {
    if (!editWorkTitle.trim()) return;
    try {
      const res = await api.upsertWork({
        id: editWorkId,
        title: editWorkTitle.trim(),
        composer: editWorkComposer.trim() || null,
        choreographer: editWorkChoreographer.trim() || null,
        is_active: true,
      });
      await loadData();
      selectedWorkId = res.id;
      showWorkModal = false;
    } catch (e: any) {
      alert(`Error al guardar obra: ${e?.message || e}`);
    }
  }

  // Guardar Pasaje
  async function savePassage() {
    if (!selectedWorkId || !editPassageName.trim()) return;
    try {
      await api.upsertWorkPassage({
        id: editPassageId,
        work_id: selectedWorkId,
        name: editPassageName.trim(),
        act_scene: editPassageActScene.trim() || null,
        default_duration_min: editPassageDuration,
        default_sound_cue: editPassageSoundCue.trim() || null,
      });
      await loadData();
      showPassageModal = false;
    } catch (e: any) {
      alert(`Error al guardar pasaje: ${e?.message || e}`);
    }
  }

  // Eliminar Pasaje
  async function deletePassage(id: string) {
    if (!confirm('¿Deseas eliminar este pasaje y sus repartos asociados?')) return;
    try {
      await api.deleteWorkPassage(id);
      await loadData();
    } catch (e: any) {
      alert(`Error al eliminar pasaje: ${e?.message || e}`);
    }
  }

  // Abrir Modal de Plan
  function openNewPlan() {
    editPlanId = null;
    editPlanPassageId = selectedWork?.passages[0]?.id || '';
    editPlanTier = '1° Elenco';
    editPlanLabel = '';
    editPlanSoundCue = selectedWork?.passages[0]?.default_sound_cue || 'PISTA TeatroCue';
    editPlanNotes = '';
    planMembers = [];
    showPlanModal = true;
  }

  function openEditPlan(plan: WorkCastingPlanDto) {
    editPlanId = plan.id;
    editPlanPassageId = plan.passage_id || '';
    editPlanTier = plan.tier_name;
    editPlanLabel = plan.cast_label;
    editPlanSoundCue = plan.default_sound_cue || 'PISTA TeatroCue';
    editPlanNotes = plan.notes || '';
    planMembers = plan.members.map(m => ({
      personnel_id: m.personnel_id,
      dancer_name: m.dancer_name,
      character_role: m.character_role || '',
      partner_personnel_id: m.partner_personnel_id || '',
      condition: m.condition,
    }));
    showPlanModal = true;
  }

  function addDancerToPlan(d: PersonnelWithQuotaDto) {
    if (planMembers.some(m => m.personnel_id === d.id)) return;
    const name = `${d.last_name}, ${d.first_name} ${d.nickname ? `("${d.nickname}")` : ''}`;
    planMembers = [
      ...planMembers,
      {
        personnel_id: d.id,
        dancer_name: name,
        character_role: '',
        partner_personnel_id: '',
        condition: 'TITULAR',
      }
    ];
    updateAutoPlanLabel();
  }

  function removeDancerFromPlan(id: string) {
    planMembers = planMembers.filter(m => m.personnel_id !== id);
    updateAutoPlanLabel();
  }

  function updateAutoPlanLabel() {
    if (editPlanLabel && editPlanLabel !== 'Auto') return;
    const dancerNames = planMembers.map(m => {
      const parts = m.dancer_name.split(',');
      return parts[0].trim();
    });
    if (dancerNames.length > 0) {
      editPlanLabel = dancerNames.join(' & ');
    }
  }

  // Guardar Plan de Reparto
  async function saveCastingPlan() {
    if (!selectedWorkId || !editPlanLabel.trim()) {
      alert('Por favor ingresa una etiqueta para el elenco.');
      return;
    }
    try {
      const payload: UpsertWorkCastingPlanPayload = {
        id: editPlanId,
        work_id: selectedWorkId,
        passage_id: editPlanPassageId || null,
        tier_name: editPlanTier,
        cast_label: editPlanLabel.trim(),
        default_sound_cue: editPlanSoundCue.trim() || null,
        notes: editPlanNotes.trim() || null,
        members: planMembers.map(m => ({
          personnel_id: m.personnel_id,
          character_role: m.character_role.trim() || null,
          partner_personnel_id: m.partner_personnel_id || null,
          condition: m.condition,
        }))
      };
      await api.upsertWorkCastingPlan(payload);
      await loadData();
      showPlanModal = false;
    } catch (e: any) {
      alert(`Error al guardar reparto de obra: ${e?.message || e}`);
    }
  }

  // Eliminar Plan de Reparto
  async function deleteCastingPlan(planId: string) {
    if (!confirm('¿Deseas eliminar este reparto planificado?')) return;
    try {
      await api.deleteWorkCastingPlan(planId);
      await loadData();
    } catch (e: any) {
      alert(`Error al eliminar reparto: ${e?.message || e}`);
    }
  }

  // Exportar Excel de Reparto
  function exportExcel() {
    if (!selectedWork) return;
    let csv = '\uFEFF';
    csv += 'TEATRO DEL LIBERTADOR SAN MARTIN - BALLET OFICIAL DE CORDOBA\n';
    csv += `PLANILLA OFICIAL DE REPARTO DE ESTUDIO - ${selectedWork.title.toUpperCase()}\n`;
    csv += `Coreografía: ${selectedWork.choreographer || 'S/D'} • Música: ${selectedWork.composer || 'S/D'}\n\n`;
    csv += 'PASAJE / CUADRO;ELENCO / TIER;REPARTO;BAILARIN;ROL / PERSONAJE;PAREJA;CONDICION;SONIDO TEATROCUE\n';

    selectedWork.casting_plans.forEach(plan => {
      if (plan.members.length === 0) {
        csv += `"${plan.passage_name || 'General'}";"${plan.tier_name}";"${plan.cast_label}";"Sin asignar";"";"";"TITULAR";"${plan.default_sound_cue || ''}"\n`;
      } else {
        plan.members.forEach(m => {
          csv += `"${plan.passage_name || 'General'}";"${plan.tier_name}";"${plan.cast_label}";"${m.dancer_name}";"${m.character_role || ''}";"${m.partner_name || ''}";"${m.condition}";"${plan.default_sound_cue || ''}"\n`;
        });
      }
    });

    csv += '\n"★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★"\n';

    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', `Reparto_${selectedWork.title.replace(/\s+/g, '_')}_Ballet_Oficial.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
</script>

{#if isOpen}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-2 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0"
    onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
  >
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-6xl max-h-[92vh] flex flex-col overflow-hidden shadow-2xl relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">
      
      <!-- Barra Superior -->
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 shrink-0 flex items-center justify-between print:hidden">
        <div class="flex items-center gap-2.5">
          <span class="text-[10px] font-mono font-bold text-[#ffb31a] bg-[#121214] px-2 py-0.5 rounded border border-[#34343c]">REPARTOS</span>
          <div>
            <h2 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider flex items-center gap-2">
              Planificación de Repartos para Obras
              <span class="text-[10px] font-mono text-[#82828a]">
                ({works.length} obras)
              </span>
            </h2>
            <p class="text-[11px] text-[#82828a]">
              Ballet Oficial de Córdoba • Teatro del Libertador San Martín
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => {
              editWorkId = null;
              editWorkTitle = '';
              editWorkComposer = '';
              editWorkChoreographer = '';
              showWorkModal = true;
            }}
            class="px-2.5 py-1 bg-[#1a1a1e] hover:bg-[#2e2e36] text-[#ffb31a] border border-[#34343c] hover:border-[#ffb31a]/50 rounded text-xs font-medium transition-colors"
          >
            + Nueva Obra
          </button>
          <button
            type="button"
            onclick={(e) => { e.stopPropagation(); onClose(); }}
            class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#34343c] text-[#82828a] hover:text-white font-mono text-base transition-colors shrink-0"
            title="Cerrar modal"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Selector de Obras en Pestañas Superiores -->
      <div class="bg-[#17171b] border-b border-[#34343c] px-4 py-2 flex items-center gap-1.5 overflow-x-auto shrink-0 print:hidden">
        {#each works as w}
          <button
            onclick={() => selectedWorkId = w.id}
            class="px-3 py-1.5 rounded text-xs font-medium transition-colors shrink-0 flex items-center gap-2 {selectedWorkId === w.id ? 'bg-[#24242a] text-[#ffb31a] border border-[#ffb31a]/50 font-bold' : 'bg-[#121214] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
          >
            <span>{w.title}</span>
            <span class="text-[10px] px-1.5 py-0.2 rounded font-mono {selectedWorkId === w.id ? 'bg-[#ffb31a]/20 text-[#ffb31a]' : 'bg-[#1a1a1e] text-[#82828a]'}">
              {w.casting_plans.length}
            </span>
          </button>
        {/each}
      </div>

      {#if selectedWork}
        <!-- Encabezado de la Obra Seleccionada -->
        <div class="bg-[#1a1a1e] px-4 py-2.5 border-b border-[#34343c] flex flex-wrap items-center justify-between gap-3 shrink-0 print:hidden">
          <div>
            <h3 class="text-sm font-bold text-[#ffb31a] uppercase tracking-wide">
              {selectedWork.title}
            </h3>
            <p class="text-[11px] text-[#82828a] font-mono">
              <span>Música: {selectedWork.composer || 'S/D'}</span>
              <span class="mx-1.5">•</span>
              <span>Coreografía: {selectedWork.choreographer || 'S/D'}</span>
            </p>
          </div>

          <!-- Conmutador de Sección Interna -->
          <div class="flex items-center gap-2 flex-wrap">
            <div class="inline-flex bg-[#121214] p-0.5 rounded border border-[#34343c]">
              <button
                onclick={() => currentSection = 'casts'}
                class="px-2.5 py-1 rounded text-xs font-medium transition-colors {currentSection === 'casts' ? 'bg-[#ffb31a] text-[#121214] font-bold' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
              >
                Repartos ({selectedWork.casting_plans.length})
              </button>
              <button
                onclick={() => currentSection = 'passages'}
                class="px-2.5 py-1 rounded text-xs font-medium transition-colors {currentSection === 'passages' ? 'bg-[#ffb31a] text-[#121214] font-bold' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
              >
                Pasajes ({selectedWork.passages.length})
              </button>
              <button
                onclick={() => currentSection = 'print'}
                class="px-2.5 py-1 rounded text-xs font-medium transition-colors {currentSection === 'print' ? 'bg-[#ffb31a] text-[#121214] font-bold' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
              >
                Planilla Cartelera
              </button>
            </div>

            {#if currentSection === 'casts'}
              <button
                onclick={openNewPlan}
                class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#ffb31a] border border-[#ffb31a]/40 rounded text-xs font-bold transition-colors"
              >
                + Nuevo Reparto
              </button>
            {:else if currentSection === 'passages'}
              <button
                onclick={() => {
                  editPassageId = null;
                  editPassageName = '';
                  editPassageActScene = '';
                  editPassageDuration = 60;
                  editPassageSoundCue = 'PISTA TeatroCue';
                  showPassageModal = true;
                }}
                class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#2ed6e8] border border-[#2ed6e8]/40 rounded text-xs font-bold transition-colors"
              >
                + Nuevo Pasaje
              </button>
            {:else}
              <button
                onclick={exportExcel}
                class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#3de06a] border border-[#3de06a]/40 rounded text-xs font-bold transition-colors"
              >
                Exportar Excel
              </button>
              <button
                onclick={() => window.print()}
                class="px-2.5 py-1 bg-[#24242a] hover:bg-[#2e2e36] text-[#d8d8dc] border border-[#34343c] rounded text-xs font-bold transition-colors"
              >
                Imprimir / PDF
              </button>
            {/if}
          </div>
        </div>

        <!-- CONTENIDO PRINCIPAL POR SECCIÓN -->
        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <!-- 1. VISTA DE REPARTOS PLANIFICADOS -->
          {#if currentSection === 'casts'}
            {#if selectedWork.casting_plans.length === 0}
              <div class="py-12 text-center bg-[#17171b] rounded border border-[#34343c] text-[#82828a] text-xs space-y-3">
                <p>No hay repartos predefinidos para esta obra todavía.</p>
                <button
                  onclick={openNewPlan}
                  class="px-3.5 py-1.5 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] font-bold rounded text-xs"
                >
                  + Crear Primer Reparto de Obra
                </button>
              </div>
            {:else}
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each selectedWork.casting_plans as plan}
                  <div class="bg-[#17171b] border border-[#34343c] hover:border-[#ffb31a]/40 rounded p-3 shadow transition-colors space-y-2.5">
                    <div class="flex items-start justify-between gap-2">
                      <div>
                        <div class="flex items-center gap-2">
                          <span class="px-1.5 py-0.5 rounded text-[10px] font-mono uppercase {plan.tier_name.includes('1°') ? 'bg-[#ffb31a]/20 text-[#ffb31a] border border-[#ffb31a]/40' : plan.tier_name.includes('2°') ? 'bg-[#2ed6e8]/20 text-[#2ed6e8] border border-[#2ed6e8]/40' : 'bg-[#24242a] text-[#82828a] border border-[#34343c]'}">
                            {plan.tier_name}
                          </span>
                          {#if plan.passage_name}
                            <span class="text-xs font-bold text-[#d8d8dc]">{plan.passage_name}</span>
                          {/if}
                        </div>
                        <h4 class="text-xs font-bold text-[#ffb31a] mt-1 font-mono">{plan.cast_label}</h4>
                      </div>

                      <div class="flex items-center gap-1">
                        <button
                          onclick={() => openEditPlan(plan)}
                          class="px-2 py-0.5 rounded bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] text-xs font-mono border border-[#34343c]"
                          title="Editar reparto"
                        >
                          Editar
                        </button>
                        <button
                          onclick={() => deleteCastingPlan(plan.id)}
                          class="px-2 py-0.5 rounded bg-[#24242a] hover:bg-[#ff4a4a]/20 text-[#82828a] hover:text-[#ff4a4a] text-xs font-mono border border-[#34343c]"
                          title="Eliminar reparto"
                        >
                          ✕
                        </button>
                      </div>
                    </div>

                    <!-- Miembros del Reparto -->
                    <div class="space-y-1 bg-[#121214] p-2 rounded border border-[#34343c] text-xs">
                      {#if plan.members.length === 0}
                        <p class="text-[#82828a] text-[11px] italic">Sin bailarines nominales asignados.</p>
                      {:else}
                        {#each plan.members as mem}
                          <div class="flex items-center justify-between text-[11px]">
                            <div class="flex items-center gap-1.5">
                              <span class="text-[#ffb31a]">•</span>
                              <span class="font-bold text-[#d8d8dc]">{mem.dancer_name}</span>
                              {#if mem.character_role}
                                <span class="text-[#82828a] font-mono text-[10px]">({mem.character_role})</span>
                              {/if}
                            </div>
                            <div>
                              {#if mem.partner_name}
                                <span class="text-[10px] text-[#2ed6e8] bg-[#2ed6e8]/10 px-1.5 py-0.5 rounded border border-[#2ed6e8]/30">
                                  c/ {mem.partner_name.split(',')[0]}
                                </span>
                              {/if}
                              <span class="text-[9px] font-bold uppercase ml-1 {mem.condition === 'COVER' ? 'text-[#ffb31a]' : 'text-[#3de06a]'}">
                                {mem.condition === 'COVER' ? 'Estudia' : mem.condition}
                              </span>
                            </div>
                          </div>
                        {/each}
                      {/if}
                    </div>

                    <!-- Metadatos de Sonido y Notas -->
                    {#if plan.default_sound_cue || plan.notes}
                      <div class="flex items-center justify-between text-[10px] font-mono text-[#82828a] pt-1 border-t border-[#34343c]">
                        <span>[CUE] {plan.default_sound_cue || 'PISTA'}</span>
                        {#if plan.notes}
                          <span class="truncate max-w-[200px]" title={plan.notes}>[NOTA] {plan.notes}</span>
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

          <!-- 2. VISTA DE PASAJES Y CUADROS -->
          {:else if currentSection === 'passages'}
            <div class="space-y-1.5">
              {#each selectedWork.passages as pass}
                <div class="bg-[#17171b] border border-[#34343c] p-2.5 rounded flex items-center justify-between gap-3 text-xs">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-[#d8d8dc] text-xs">{pass.name}</span>
                      {#if pass.act_scene}
                        <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-[#24242a] text-[#82828a] font-mono border border-[#34343c]">
                          {pass.act_scene}
                        </span>
                      {/if}
                    </div>
                    <div class="text-[11px] text-[#82828a] font-mono mt-0.5 flex items-center gap-3">
                      <span>Duración: {pass.default_duration_min} min</span>
                      <span>Cue Sonido: {pass.default_sound_cue || 'Sin cue'}</span>
                    </div>
                  </div>

                  <div class="flex items-center gap-1.5">
                    <button
                      onclick={() => {
                        editPassageId = pass.id;
                        editPassageName = pass.name;
                        editPassageActScene = pass.act_scene || '';
                        editPassageDuration = pass.default_duration_min;
                        editPassageSoundCue = pass.default_sound_cue || '';
                        showPassageModal = true;
                      }}
                      class="px-2 py-0.5 rounded bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] text-xs font-mono border border-[#34343c]"
                    >
                      Editar
                    </button>
                    <button
                      onclick={() => deletePassage(pass.id)}
                      class="px-2 py-0.5 rounded bg-[#24242a] hover:bg-[#ff4a4a]/20 text-[#82828a] hover:text-[#ff4a4a] text-xs font-mono border border-[#34343c]"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              {/each}
            </div>

          <!-- 3. PLANILLA OFICIAL DE CARTELERA (IMPRIMIBLE / PDF) -->
          {:else}
            <div class="bg-white text-black p-6 rounded shadow-xl print:p-0 print:shadow-none print:rounded-none">
              <!-- Encabezado Institucional -->
              <div class="text-center pb-4 mb-4 border-b-2 border-black">
                <h1 class="text-lg font-black tracking-tight uppercase">
                  Teatro del Libertador San Martín • Ballet Oficial de Córdoba
                </h1>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-800 mt-0.5">
                  Planilla Oficial de Reparto de Obra: {selectedWork.title}
                </h2>
                <div class="flex justify-between items-center text-xs font-mono mt-2 pt-1 border-t border-slate-400">
                  <span>Coreografía: {selectedWork.choreographer || 'S/D'}</span>
                  <span>Música: {selectedWork.composer || 'S/D'}</span>
                  <span>Fecha: {new Date().toLocaleDateString('es-AR')}</span>
                </div>
              </div>

              <!-- Tabla de Reparto General -->
              <table class="w-full text-left border-collapse text-xs border border-black mb-6">
                <thead>
                  <tr class="bg-slate-200 border-b border-black text-black uppercase font-black text-[10px]">
                    <th class="p-2 border-r border-black">Pasaje / Cuadro</th>
                    <th class="p-2 border-r border-black">Rol / Personaje</th>
                    <th class="p-2 border-r border-black">1° Elenco (Titular)</th>
                    <th class="p-2 border-r border-black">2° Elenco / Alternante</th>
                    <th class="p-2 border-r border-black">Estudian (por las dudas)</th>
                    <th class="p-2">Sonido / Cue</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-black font-medium">
                  {#each selectedWork.passages as pass}
                    {@const plansForPass = selectedWork.casting_plans.filter(p => p.passage_id === pass.id)}
                    {@const tier1 = plansForPass.find(p => p.tier_name.includes('1°'))}
                    {@const tier2 = plansForPass.find(p => p.tier_name.includes('2°'))}
                    {@const covers = plansForPass.find(p => p.tier_name.toLowerCase().includes('cover') || p.tier_name.toLowerCase().includes('estudia') || p.tier_name.includes('3°'))}
                    <tr class="break-inside-avoid">
                      <td class="p-2 font-bold border-r border-black bg-slate-50">
                        <div>{pass.name}</div>
                        <div class="text-[9px] text-slate-600 font-mono">{pass.act_scene || ''}</div>
                      </td>
                      <td class="p-2 border-r border-black font-mono text-[11px]">
                        {#if tier1 && tier1.members.length > 0}
                          {tier1.members.map(m => m.character_role || 'Solista').join(', ')}
                        {:else}
                          —
                        {/if}
                      </td>
                      <td class="p-2 border-r border-black font-bold">
                        {tier1 ? tier1.cast_label : '—'}
                      </td>
                      <td class="p-2 border-r border-black text-slate-800">
                        {tier2 ? tier2.cast_label : '—'}
                      </td>
                      <td class="p-2 border-r border-black text-slate-700 italic text-[11px]">
                        {covers ? covers.cast_label : '—'}
                      </td>
                      <td class="p-2 font-mono text-[10px]">
                        {pass.default_sound_cue || 'PISTA'}
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>

              <!-- Invariante Legal Institucional Obligatorio -->
              <div class="text-center pt-3 border-t-2 border-black">
                <p class="text-xs font-black tracking-widest uppercase font-mono">
                  ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
                </p>
                <p class="text-[9px] text-slate-600 font-mono mt-1">
                  Dirección y Ensayos del Ballet Oficial de Córdoba • Teatro del Libertador San Martín
                </p>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<!-- MODAL ALTA / EDICIÓN DE OBRA -->
{#if showWorkModal}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4"
    onclick={(e) => { if (e.target === e.currentTarget) showWorkModal = false; }}
  >
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-md shadow-2xl overflow-hidden flex flex-col">
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 flex items-center justify-between">
        <h3 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider">
          {editWorkId ? 'Editar Obra' : 'Nueva Obra de Repertorio'}
        </h3>
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); showWorkModal = false; }}
          class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#34343c] text-[#82828a] hover:text-white font-mono text-base transition-colors shrink-0"
          title="Cerrar modal"
        >
          ✕
        </button>
      </div>

      <div class="p-4 space-y-3">
        <div>
          <label for="work-title-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Título de la Obra</label>
          <input
            id="work-title-input"
            type="text"
            bind:value={editWorkTitle}
            placeholder="Ej: Giselle, El Lago de los Cisnes, Paquita..."
            class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
          />
        </div>

        <div>
          <label for="work-composer-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Compositor / Música</label>
          <input
            id="work-composer-input"
            type="text"
            bind:value={editWorkComposer}
            placeholder="Ej: Piotr Ilich Tchaikovsky, Adolphe Adam..."
            class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
          />
        </div>

        <div>
          <label for="work-choreographer-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Coreógrafo / Versión</label>
          <input
            id="work-choreographer-input"
            type="text"
            bind:value={editWorkChoreographer}
            placeholder="Ej: Marius Petipa / Jean Coralli..."
            class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
          />
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-[#34343c]">
          <button onclick={() => showWorkModal = false} class="px-3 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] rounded text-xs border border-[#34343c]">Cancelar</button>
          <button onclick={saveWork} class="px-3 py-1 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] font-bold rounded text-xs">
            Guardar Obra
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL ALTA / EDICIÓN DE PASAJE -->
{#if showPassageModal}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4"
    onclick={(e) => { if (e.target === e.currentTarget) showPassageModal = false; }}
  >
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-md shadow-2xl overflow-hidden flex flex-col">
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 flex items-center justify-between">
        <h3 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider">
          {editPassageId ? 'Editar Pasaje / Cuadro' : 'Nuevo Pasaje / Cuadro'}
        </h3>
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); showPassageModal = false; }}
          class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#34343c] text-[#82828a] hover:text-white font-mono text-base transition-colors shrink-0"
          title="Cerrar modal"
        >
          ✕
        </button>
      </div>

      <div class="p-4 space-y-3">
        <div>
          <label for="passage-name-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Nombre del Pasaje / Cuadro</label>
          <input
            id="passage-name-input"
            type="text"
            bind:value={editPassageName}
            placeholder="Ej: Pas de deux Cisne Blanco, Danza Española..."
            class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
          />
        </div>

        <div>
          <label for="passage-scene-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Acto / Escena</label>
          <input
            id="passage-scene-input"
            type="text"
            bind:value={editPassageActScene}
            placeholder="Ej: Acto II, Escena 1..."
            class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
          />
        </div>

        <div class="grid grid-cols-2 gap-2">
          <div>
            <label for="passage-duration-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Duración Habitual (min)</label>
            <input
              id="passage-duration-input"
              type="number"
              min="15"
              max="180"
              step="15"
              bind:value={editPassageDuration}
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] font-mono outline-none focus:border-[#ffb31a]"
            />
          </div>
          <div>
            <label for="passage-sound-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Cue de Sonido TeatroCue</label>
            <input
              id="passage-sound-input"
              type="text"
              bind:value={editPassageSoundCue}
              placeholder="PISTA TeatroCue #08 / Piano"
              class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc] font-mono outline-none focus:border-[#ffb31a]"
            />
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-[#34343c]">
          <button onclick={() => showPassageModal = false} class="px-3 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] rounded text-xs border border-[#34343c]">Cancelar</button>
          <button onclick={savePassage} class="px-3 py-1 bg-[#2ed6e8] hover:bg-[#26b4c4] text-[#121214] font-bold rounded text-xs">
            Guardar Pasaje
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL ALTA / EDICIÓN DE REPARTO (ELENCO Y ASIGNACIÓN DE BAILARINES) -->
{#if showPlanModal}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3"
    onclick={(e) => { if (e.target === e.currentTarget) showPlanModal = false; }}
  >
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-2xl max-h-[92vh] flex flex-col shadow-2xl overflow-hidden">
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 shrink-0 flex items-center justify-between">
        <h3 class="text-xs font-bold text-[#d8d8dc] uppercase tracking-wider">
          {editPlanId ? 'Editar Reparto de Obra' : 'Nuevo Reparto Planificado'}
        </h3>
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); showPlanModal = false; }}
          class="w-8 h-8 rounded flex items-center justify-center hover:bg-[#34343c] text-[#82828a] hover:text-white font-mono text-base transition-colors shrink-0"
          title="Cerrar modal"
        >
          ✕
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
          <!-- Pasaje -->
          <div>
            <label for="plan-passage-select" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Pasaje Asociado</label>
            <select
              id="plan-passage-select"
              bind:value={editPlanPassageId}
              class="w-full h-8 px-2 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
            >
              <option value="">(Toda la Obra / General)</option>
              {#each selectedWork?.passages || [] as pass}
                <option value={pass.id}>{pass.name}</option>
              {/each}
            </select>
          </div>

          <!-- Tier / Elenco -->
          <div>
            <label for="plan-tier-select" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Elenco / Nivel</label>
            <select
              id="plan-tier-select"
              bind:value={editPlanTier}
              class="w-full h-8 px-2 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
            >
              <option value="1° Elenco">1° Elenco (Titular)</option>
              <option value="2° Elenco">2° Elenco (Alternante)</option>
              <option value="3° Elenco">3° Elenco</option>
              <option value="Estudia (por las dudas)">Estudia (por las dudas)</option>
              <option value="Cuerpo de Baile">Cuerpo de Baile</option>
              <option value="Estudio General">Estudio General</option>
            </select>
          </div>

          <!-- Etiqueta del Elenco -->
          <div>
            <label for="plan-label-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Etiqueta Formateada</label>
            <input
              id="plan-label-input"
              type="text"
              bind:value={editPlanLabel}
              placeholder="Ej: Vera & Morchio C."
              class="w-full h-8 px-2 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#ffb31a] font-mono outline-none focus:border-[#ffb31a]"
            />
          </div>
        </div>

        <!-- Buscador y Selector Rápido de Bailarines -->
        <div class="border-t border-[#34343c] pt-2">
          <label for="dancer-search-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1 flex items-center justify-between">
            <span>Seleccionar Bailarines del Padrón Activo ({dancers.length})</span>
            <span class="text-[#ffb31a]">{planMembers.length} seleccionados</span>
          </label>
          
          <input
            id="dancer-search-input"
            type="text"
            bind:value={dancerSearch}
            placeholder="Buscar bailarín por apellido o apodo (ej: Popi, Brenda, Álvarez, Gil)..."
            class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs text-[#d8d8dc] outline-none focus:border-[#ffb31a] mb-2"
          />

          <div class="flex items-center gap-1 overflow-x-auto pb-1">
            {#each dancers.filter(d => (d.last_name + d.first_name + (d.nickname || '')).toLowerCase().includes(dancerSearch.toLowerCase())) as d}
              {@const isSelected = planMembers.some(m => m.personnel_id === d.id)}
              <button
                type="button"
                onclick={() => isSelected ? removeDancerFromPlan(d.id) : addDancerToPlan(d)}
                class="px-2 py-1 rounded text-[11px] font-medium transition-colors shrink-0 flex items-center gap-1 {isSelected ? 'bg-[#ffb31a] text-[#121214] font-bold' : 'bg-[#121214] text-[#82828a] hover:text-[#d8d8dc] border border-[#34343c]'}"
              >
                <span>{isSelected ? '✓' : '+'}</span>
                <span>{d.last_name} {d.nickname ? `("${d.nickname}")` : ''}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Lista de Bailarines Asignados con Rol y Pareja -->
        <div class="space-y-1.5 max-h-48 overflow-y-auto pr-1 border border-[#34343c] rounded p-2 bg-[#121214]">
          {#if planMembers.length === 0}
            <p class="text-xs text-[#82828a] text-center py-4">Haz clic en los bailarines de arriba para agregarlos al reparto.</p>
          {:else}
            {#each planMembers as mem, idx}
              <div class="bg-[#17171b] border border-[#34343c] p-2 rounded flex flex-wrap items-center justify-between gap-2 text-xs">
                <div class="flex items-center gap-2">
                  <span class="w-4 h-4 rounded bg-[#24242a] text-[#ffb31a] flex items-center justify-center font-bold text-[10px] font-mono border border-[#34343c]">
                    {idx + 1}
                  </span>
                  <span class="font-bold text-[#d8d8dc]">{mem.dancer_name}</span>
                </div>

                <div class="flex items-center gap-1.5 flex-wrap">
                  <input
                    type="text"
                    bind:value={mem.character_role}
                    placeholder="Rol (ej: Odette)..."
                    class="h-7 px-2 bg-[#121214] border border-[#34343c] rounded text-[11px] text-[#d8d8dc] w-28 outline-none focus:border-[#ffb31a]"
                  />

                  <select
                    bind:value={mem.partner_personnel_id}
                    class="h-7 px-1.5 bg-[#121214] border border-[#34343c] rounded text-[11px] text-[#82828a] outline-none focus:border-[#ffb31a]"
                  >
                    <option value="">(Sin pareja)</option>
                    {#each dancers.filter(d => d.id !== mem.personnel_id) as partner}
                      <option value={partner.id}>c/ {partner.last_name}</option>
                    {/each}
                  </select>

                  <select
                    bind:value={mem.condition}
                    class="h-7 px-1.5 bg-[#121214] border border-[#34343c] rounded text-[11px] text-[#ffb31a] outline-none focus:border-[#ffb31a]"
                  >
                    <option value="TITULAR">Titular</option>
                    <option value="COVER">Estudia (por las dudas)</option>
                    <option value="ALTERNATE_SLASH">Barra (/)</option>
                    <option value="REINFORCEMENT">Refuerzo</option>
                  </select>

                  <button
                    type="button"
                    onclick={() => removeDancerFromPlan(mem.personnel_id)}
                    class="text-[#82828a] hover:text-[#ff4a4a] px-1 text-sm font-mono"
                    title="Quitar"
                  >
                    ✕
                  </button>
                </div>
              </div>
            {/each}
          {/if}
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-[#34343c]">
          <button onclick={() => showPlanModal = false} class="px-3 py-1 bg-[#24242a] hover:bg-[#34343c] text-[#d8d8dc] rounded text-xs border border-[#34343c]">Cancelar</button>
          <button onclick={saveCastingPlan} class="px-3 py-1 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] font-bold rounded text-xs">
            Guardar Reparto
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
