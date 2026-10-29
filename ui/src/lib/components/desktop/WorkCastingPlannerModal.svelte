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
  import ModalHeader from '../shared/ModalHeader.svelte';
  import PrintPreviewModal from '../shared/PrintPreviewModal.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let printPreviewOpen = $state(false);
  let printableEl: HTMLElement | null = $state(null);

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
    class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-2 md:p-6 print:p-0 print:bg-white print:fixed print:inset-0"
    use:modalBehavior={{ onclose: onClose }}
  >
    <div class="tg-modal max-w-6xl relative print:bg-white print:border-none print:shadow-none print:p-0 print:max-h-none print:overflow-visible">
      
      <!-- Barra Superior -->
      <div class="bg-raised] border-b border-grid] px-4 py-2.5 shrink-0 flex items-center justify-between print:hidden">
        <div class="flex items-center gap-2.5">
          <span class="text-[10px] font-mono font-bold text-amber] bg-void] px-2 py-0.5 rounded border border-grid]">REPARTOS</span>
          <div>
            <h2 class="text-xs font-bold text-ink] uppercase tracking-wider flex items-center gap-2">
              Planificación de Repartos para Obras
              <span class="text-[10px] font-mono text-dim]">
                ({works.length} obras)
              </span>
            </h2>
            <p class="text-[11px] text-dim]">
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
            class="px-2.5 py-1 bg-panel] hover:bg-raised-hover] text-amber] border border-grid] hover:border-amber]/50 rounded text-xs font-medium transition-colors"
          >
            + Nueva Obra
          </button>
          <button
            type="button"
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

      <!-- Selector de Obras en Pestañas Superiores -->
      <div class="bg-alt] border-b border-grid] px-4 py-2 flex items-center gap-1.5 overflow-x-auto shrink-0 print:hidden">
        {#each works as w}
          <button
            onclick={() => selectedWorkId = w.id}
            class="px-3 py-1.5 rounded text-xs font-medium transition-colors shrink-0 flex items-center gap-2 {selectedWorkId === w.id ? 'bg-raised] text-amber] border border-amber]/50 font-bold' : 'bg-void] text-dim] hover:text-ink] border border-grid]'}"
          >
            <span>{w.title}</span>
            <span class="text-[10px] px-1.5 py-0.2 rounded font-mono {selectedWorkId === w.id ? 'bg-amber]/20 text-amber]' : 'bg-panel] text-dim]'}">
              {w.casting_plans.length}
            </span>
          </button>
        {/each}
      </div>

      {#if selectedWork}
        <!-- Encabezado de la Obra Seleccionada -->
        <div class="bg-panel] px-4 py-2.5 border-b border-grid] flex flex-wrap items-center justify-between gap-3 shrink-0 print:hidden">
          <div>
            <h3 class="text-sm font-bold text-amber] uppercase tracking-wide">
              {selectedWork.title}
            </h3>
            <p class="text-[11px] text-dim] font-mono">
              <span>Música: {selectedWork.composer || 'S/D'}</span>
              <span class="mx-1.5">•</span>
              <span>Coreografía: {selectedWork.choreographer || 'S/D'}</span>
            </p>
          </div>

          <!-- Conmutador de Sección Interna -->
          <div class="flex items-center gap-2 flex-wrap">
            <div class="inline-flex bg-void] p-0.5 rounded border border-grid]">
              <button
                onclick={() => currentSection = 'casts'}
                class="px-2.5 py-1 rounded text-xs font-medium transition-colors {currentSection === 'casts' ? 'bg-amber] text-void] font-bold' : 'text-dim] hover:text-ink]'}"
              >
                Repartos ({selectedWork.casting_plans.length})
              </button>
              <button
                onclick={() => currentSection = 'passages'}
                class="px-2.5 py-1 rounded text-xs font-medium transition-colors {currentSection === 'passages' ? 'bg-amber] text-void] font-bold' : 'text-dim] hover:text-ink]'}"
              >
                Pasajes ({selectedWork.passages.length})
              </button>
              <button
                onclick={() => currentSection = 'print'}
                class="px-2.5 py-1 rounded text-xs font-medium transition-colors {currentSection === 'print' ? 'bg-amber] text-void] font-bold' : 'text-dim] hover:text-ink]'}"
              >
                Planilla Cartelera
              </button>
            </div>

            {#if currentSection === 'casts'}
              <button
                onclick={openNewPlan}
                class="px-2.5 py-1 bg-raised] hover:bg-raised-hover] text-amber] border border-amber]/40 rounded text-xs font-bold transition-colors"
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
                class="px-2.5 py-1 bg-raised] hover:bg-raised-hover] text-cyan] border border-cyan]/40 rounded text-xs font-bold transition-colors"
              >
                + Nuevo Pasaje
              </button>
            {:else}
              <button
                onclick={exportExcel}
                class="px-2.5 py-1 bg-raised] hover:bg-raised-hover] text-green] border border-green]/40 rounded text-xs font-bold transition-colors"
              >
                Exportar Excel
              </button>
              <button
                onclick={() => printPreviewOpen = true}
                class="px-2.5 py-1 bg-raised] hover:bg-raised-hover] text-ink] border border-grid] rounded text-xs font-bold transition-colors"
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
              <div class="py-12 text-center bg-alt] rounded border border-grid] text-dim] text-xs space-y-3">
                <p>No hay repartos predefinidos para esta obra todavía.</p>
                <button
                  onclick={openNewPlan}
                  class="px-3.5 py-1.5 bg-amber] hover:bg-amber] text-void] font-bold rounded text-xs"
                >
                  + Crear Primer Reparto de Obra
                </button>
              </div>
            {:else}
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each selectedWork.casting_plans as plan}
                  <div class="bg-alt] border border-grid] hover:border-amber]/40 rounded p-3 shadow transition-colors space-y-2.5">
                    <div class="flex items-start justify-between gap-2">
                      <div>
                        <div class="flex items-center gap-2">
                          <span class="px-1.5 py-0.5 rounded text-[10px] font-mono uppercase {plan.tier_name.includes('1°') ? 'bg-amber]/20 text-amber] border border-amber]/40' : plan.tier_name.includes('2°') ? 'bg-cyan]/20 text-cyan] border border-cyan]/40' : 'bg-raised] text-dim] border border-grid]'}">
                            {plan.tier_name}
                          </span>
                          {#if plan.passage_name}
                            <span class="text-xs font-bold text-ink]">{plan.passage_name}</span>
                          {/if}
                        </div>
                        <h4 class="text-xs font-bold text-amber] mt-1 font-mono">{plan.cast_label}</h4>
                      </div>

                      <div class="flex items-center gap-1">
                        <button
                          onclick={() => openEditPlan(plan)}
                          class="px-2 py-0.5 rounded bg-raised] hover:bg-grid] text-ink] text-xs font-mono border border-grid]"
                          title="Editar reparto"
                        >
                          Editar
                        </button>
                        <button
                          onclick={() => deleteCastingPlan(plan.id)}
                          class="px-2 py-0.5 rounded bg-raised] hover:bg-red]/20 text-dim] hover:text-red] text-xs font-mono border border-grid]"
                          title="Eliminar reparto"
                        >
                          ✕
                        </button>
                      </div>
                    </div>

                    <!-- Miembros del Reparto -->
                    <div class="space-y-1 bg-void] p-2 rounded border border-grid] text-xs">
                      {#if plan.members.length === 0}
                        <p class="text-dim] text-[11px] italic">Sin bailarines nominales asignados.</p>
                      {:else}
                        {#each plan.members as mem}
                          <div class="flex items-center justify-between text-[11px]">
                            <div class="flex items-center gap-1.5">
                              <span class="text-amber]">•</span>
                              <span class="font-bold text-ink]">{mem.dancer_name}</span>
                              {#if mem.character_role}
                                <span class="text-dim] font-mono text-[10px]">({mem.character_role})</span>
                              {/if}
                            </div>
                            <div>
                              {#if mem.partner_name}
                                <span class="text-[10px] text-cyan] bg-cyan]/10 px-1.5 py-0.5 rounded border border-cyan]/30">
                                  c/ {mem.partner_name.split(',')[0]}
                                </span>
                              {/if}
                              <span class="text-[9px] font-bold uppercase ml-1 {mem.condition === 'COVER' ? 'text-amber]' : 'text-green]'}">
                                {mem.condition === 'COVER' ? 'Estudia' : mem.condition}
                              </span>
                            </div>
                          </div>
                        {/each}
                      {/if}
                    </div>

                    <!-- Metadatos de Sonido y Notas -->
                    {#if plan.default_sound_cue || plan.notes}
                      <div class="flex items-center justify-between text-[10px] font-mono text-dim] pt-1 border-t border-grid]">
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
                <div class="bg-alt] border border-grid] p-2.5 rounded flex items-center justify-between gap-3 text-xs">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-ink] text-xs">{pass.name}</span>
                      {#if pass.act_scene}
                        <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-raised] text-dim] font-mono border border-grid]">
                          {pass.act_scene}
                        </span>
                      {/if}
                    </div>
                    <div class="text-[11px] text-dim] font-mono mt-0.5 flex items-center gap-3">
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
                      class="px-2 py-0.5 rounded bg-raised] hover:bg-grid] text-ink] text-xs font-mono border border-grid]"
                    >
                      Editar
                    </button>
                    <button
                      onclick={() => deletePassage(pass.id)}
                      class="px-2 py-0.5 rounded bg-raised] hover:bg-red]/20 text-dim] hover:text-red] text-xs font-mono border border-grid]"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              {/each}
            </div>

          <!-- 3. PLANILLA OFICIAL DE CARTELERA (IMPRIMIBLE / PDF) -->
          {:else}
            <div bind:this={printableEl} class="bg-white text-black p-6 rounded print:p-0 print:shadow-none print:rounded-none">
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
    class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4"
    use:modalBehavior={{ onclose: () => showWorkModal = false, backdrop: false }}
  >
    <div class="tg-modal max-w-md">
      <ModalHeader
        title={editWorkId ? 'Editar Obra' : 'Nueva Obra de Repertorio'}
        onclose={() => showWorkModal = false}
      />

      <div class="tg-modal-body space-y-3">
        <div>
          <label for="work-title-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Título de la Obra</label>
          <input
            id="work-title-input"
            type="text"
            bind:value={editWorkTitle}
            placeholder="Ej: Giselle, El Lago de los Cisnes, Paquita..."
            class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]"
          />
        </div>

        <div>
          <label for="work-composer-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Compositor / Música</label>
          <input
            id="work-composer-input"
            type="text"
            bind:value={editWorkComposer}
            placeholder="Ej: Piotr Ilich Tchaikovsky, Adolphe Adam..."
            class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
          />
        </div>

        <div>
          <label for="work-choreographer-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Coreógrafo / Versión</label>
          <input
            id="work-choreographer-input"
            type="text"
            bind:value={editWorkChoreographer}
            placeholder="Ej: Marius Petipa / Jean Coralli..."
            class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
          />
        </div>

        <div class="tg-modal-foot">
          <button onclick={() => showWorkModal = false} class="tg-btn">Cancelar</button>
          <button onclick={saveWork} class="tg-btn tg-btn-primary">
            Guardar Obra
          </button>
        </div>
      </div>

      <PrintPreviewModal
        open={printPreviewOpen}
        title="Vista previa de impresión"
        subtitle="Cartelera de obra — ediciones solo para la copia impresa"
        source={printableEl}
        onclose={() => (printPreviewOpen = false)}
      />
    </div>
  </div>
{/if}

<!-- MODAL ALTA / EDICIÓN DE PASAJE -->
{#if showPassageModal}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4"
    use:modalBehavior={{ onclose: () => showPassageModal = false, backdrop: false }}
  >
    <div class="tg-modal max-w-md">
      <ModalHeader
        title={editPassageId ? 'Editar Pasaje / Cuadro' : 'Nuevo Pasaje / Cuadro'}
        onclose={() => showPassageModal = false}
      />

      <div class="tg-modal-body space-y-3">
        <div>
          <label for="passage-name-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Nombre del Pasaje / Cuadro</label>
          <input
            id="passage-name-input"
            type="text"
            bind:value={editPassageName}
            placeholder="Ej: Pas de deux Cisne Blanco, Danza Española..."
            class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]"
          />
        </div>

        <div>
          <label for="passage-scene-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Acto / Escena</label>
          <input
            id="passage-scene-input"
            type="text"
            bind:value={editPassageActScene}
            placeholder="Ej: Acto II, Escena 1..."
            class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber]"
          />
        </div>

        <div class="grid grid-cols-2 gap-2">
          <div>
            <label for="passage-duration-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Duración Habitual (min)</label>
            <input
              id="passage-duration-input"
              type="number"
              min="15"
              max="180"
              step="15"
              bind:value={editPassageDuration}
              class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] font-mono outline-none focus:border-amber]"
            />
          </div>
          <div>
            <label for="passage-sound-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Cue de Sonido TeatroCue</label>
            <input
              id="passage-sound-input"
              type="text"
              bind:value={editPassageSoundCue}
              placeholder="PISTA TeatroCue #08 / Piano"
              class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] font-mono outline-none focus:border-amber]"
            />
          </div>
        </div>

        <div class="tg-modal-foot">
          <button onclick={() => showPassageModal = false} class="tg-btn">Cancelar</button>
          <button onclick={savePassage} class="tg-btn tg-btn-primary">
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
    class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-3"
    use:modalBehavior={{ onclose: () => showPlanModal = false, backdrop: false }}
  >
    <div class="tg-modal max-w-2xl">
      <ModalHeader
        title={editPlanId ? 'Editar Reparto de Obra' : 'Nuevo Reparto Planificado'}
        onclose={() => showPlanModal = false}
      />

      <div class="tg-modal-body flex-1 min-h-0 space-y-3">
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
          <!-- Pasaje -->
          <div>
            <label for="plan-passage-select" class="block text-[10px] font-bold uppercase text-dim] mb-1">Pasaje Asociado</label>
            <select
              id="plan-passage-select"
              bind:value={editPlanPassageId}
              class="w-full h-8 px-2 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]"
            >
              <option value="">(Toda la Obra / General)</option>
              {#each selectedWork?.passages || [] as pass}
                <option value={pass.id}>{pass.name}</option>
              {/each}
            </select>
          </div>

          <!-- Tier / Elenco -->
          <div>
            <label for="plan-tier-select" class="block text-[10px] font-bold uppercase text-dim] mb-1">Elenco / Nivel</label>
            <select
              id="plan-tier-select"
              bind:value={editPlanTier}
              class="w-full h-8 px-2 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]"
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
            <label for="plan-label-input" class="block text-[10px] font-bold uppercase text-dim] mb-1">Etiqueta Formateada</label>
            <input
              id="plan-label-input"
              type="text"
              bind:value={editPlanLabel}
              placeholder="Ej: Vera & Morchio C."
              class="w-full h-8 px-2 bg-void] border border-grid] rounded text-xs font-bold text-amber] font-mono outline-none focus:border-amber]"
            />
          </div>
        </div>

        <!-- Buscador y Selector Rápido de Bailarines -->
        <div class="border-t border-grid] pt-2">
          <label for="dancer-search-input" class="block text-[10px] font-bold uppercase text-dim] mb-1 flex items-center justify-between">
            <span>Seleccionar Bailarines del Padrón Activo ({dancers.length})</span>
            <span class="text-amber]">{planMembers.length} seleccionados</span>
          </label>
          
          <input
            id="dancer-search-input"
            type="text"
            bind:value={dancerSearch}
            placeholder="Buscar bailarín por apellido o apodo (ej: Popi, Brenda, Álvarez, Gil)..."
            class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs text-ink] outline-none focus:border-amber] mb-2"
          />

          <div class="flex items-center gap-1 overflow-x-auto pb-1">
            {#each dancers.filter(d => (d.last_name + d.first_name + (d.nickname || '')).toLowerCase().includes(dancerSearch.toLowerCase())) as d}
              {@const isSelected = planMembers.some(m => m.personnel_id === d.id)}
              <button
                type="button"
                onclick={() => isSelected ? removeDancerFromPlan(d.id) : addDancerToPlan(d)}
                class="px-2 py-1 rounded text-[11px] font-medium transition-colors shrink-0 flex items-center gap-1 {isSelected ? 'bg-amber] text-void] font-bold' : 'bg-void] text-dim] hover:text-ink] border border-grid]'}"
              >
                <span>{isSelected ? '✓' : '+'}</span>
                <span>{d.last_name} {d.nickname ? `("${d.nickname}")` : ''}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Lista de Bailarines Asignados con Rol y Pareja -->
        <div class="space-y-1.5 max-h-48 overflow-y-auto pr-1 border border-grid] rounded p-2 bg-void]">
          {#if planMembers.length === 0}
            <p class="text-xs text-dim] text-center py-4">Haz clic en los bailarines de arriba para agregarlos al reparto.</p>
          {:else}
            {#each planMembers as mem, idx}
              <div class="bg-alt] border border-grid] p-2 rounded flex flex-wrap items-center justify-between gap-2 text-xs">
                <div class="flex items-center gap-2">
                  <span class="w-4 h-4 rounded bg-raised] text-amber] flex items-center justify-center font-bold text-[10px] font-mono border border-grid]">
                    {idx + 1}
                  </span>
                  <span class="font-bold text-ink]">{mem.dancer_name}</span>
                </div>

                <div class="flex items-center gap-1.5 flex-wrap">
                  <input
                    type="text"
                    bind:value={mem.character_role}
                    placeholder="Rol (ej: Odette)..."
                    class="h-7 px-2 bg-void] border border-grid] rounded text-[11px] text-ink] w-28 outline-none focus:border-amber]"
                  />

                  <select
                    bind:value={mem.partner_personnel_id}
                    class="h-7 px-1.5 bg-void] border border-grid] rounded text-[11px] text-dim] outline-none focus:border-amber]"
                  >
                    <option value="">(Sin pareja)</option>
                    {#each dancers.filter(d => d.id !== mem.personnel_id) as partner}
                      <option value={partner.id}>c/ {partner.last_name}</option>
                    {/each}
                  </select>

                  <select
                    bind:value={mem.condition}
                    class="h-7 px-1.5 bg-void] border border-grid] rounded text-[11px] text-amber] outline-none focus:border-amber]"
                  >
                    <option value="TITULAR">Titular</option>
                    <option value="COVER">Estudia (por las dudas)</option>
                    <option value="ALTERNATE_SLASH">Barra (/)</option>
                    <option value="REINFORCEMENT">Refuerzo</option>
                  </select>

                  <button
                    type="button"
                    onclick={() => removeDancerFromPlan(mem.personnel_id)}
                    class="text-dim] hover:text-red] px-1 text-sm font-mono"
                    title="Quitar"
                  >
                    ✕
                  </button>
                </div>
              </div>
            {/each}
          {/if}
        </div>

        <div class="tg-modal-foot">
          <button onclick={() => showPlanModal = false} class="tg-btn">Cancelar</button>
          <button onclick={saveCastingPlan} class="tg-btn tg-btn-primary">
            Guardar Reparto
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
