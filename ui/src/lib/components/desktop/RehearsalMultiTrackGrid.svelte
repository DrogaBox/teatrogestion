<script lang="ts">
  import { roomLabel } from '../../labels';
  import { onMount } from 'svelte';
  import {
    api,
    type ActivityType,
    type ConfirmationState,
    type PersonnelWithQuotaDto,
    type RehearsalBlock,
    type RoomCode,
    type ScheduleConflict,
    type UpsertCastGroupPayload,
    type UpsertCastMemberPayload,
    type UpsertRehearsalBlockPayload,
    type WorkWithPassagesAndCastsDto,
    type WorkCastingPlanDto
  } from '../../ipc/client';

  let {
    scheduleId,
    blocks,
    conflicts = [],
    onUpdated
  } = $props<{
    scheduleId: string;
    blocks: RehearsalBlock[];
    conflicts?: ScheduleConflict[];
    onUpdated: () => void;
  }>();

  // Definición de salas oficiales del Teatro y Logística de Gira
  const allRooms: { code: RoomCode; name: string; tag: string; isTour?: boolean; color: string }[] = [
    { code: 'SALA_1', name: 'Sala 1', tag: 'Ensayos de Estudio', color: 'border-amber-400/40 text-amber-400' },
    { code: 'SALA_2', name: 'Sala 2', tag: 'Ensayos de Estudio', color: 'border-emerald-400/40 text-emerald-400' },
    { code: 'SALA_3', name: 'Sala 3', tag: 'Ensayos de Estudio', color: 'border-blue-400/40 text-blue-400' },
    { code: 'ESCENARIO', name: 'Escenario Principal', tag: 'Puesta en Escena', color: 'border-purple-400/40 text-purple-400' },
    { code: 'GIRA_TRASLADO', name: 'Gira: Traslado / Salida', tag: 'Logística de Micros', isTour: true, color: 'border-cyan-400/40 text-cyan-400' },
    { code: 'GIRA_ESCENARIO', name: 'Gira: Escenario Externo', tag: 'Teatro en Gira', isTour: true, color: 'border-indigo-400/40 text-indigo-400' },
    { code: 'GIRA_SALA', name: 'Gira: Sala de Ensayo', tag: 'Estudio de Gira', isTour: true, color: 'border-rose-400/40 text-rose-400' },
  ];

  // Filtro de salas visibles
  let roomFilter = $state<'ALL' | 'THEATER' | 'TOUR'>('THEATER');
  const visibleRooms = $derived(
    roomFilter === 'THEATER'
      ? allRooms.filter(r => !r.isTour)
      : roomFilter === 'TOUR'
      ? allRooms.filter(r => r.isTour)
      : allRooms
  );

  // Padrón de Personal de la Compañía y Repartos de Obra
  let personnelList = $state<PersonnelWithQuotaDto[]>([]);
  let worksWithCasts = $state<WorkWithPassagesAndCastsDto[]>([]);
  let dancers = $derived(personnelList.filter(p => p.role === 'DANCER' && p.is_active));
  let masters = $derived(personnelList.filter(p => p.role === 'MASTER' && p.is_active));
  let soundTechs = $derived(personnelList.filter(p => p.role === 'SOUND_TECH' && p.is_active));
  let pianists = $derived(personnelList.filter(p => p.role === 'PIANIST' && p.is_active));

  async function loadPersonnelAndWorks() {
    try {
      const [pList, wList] = await Promise.all([
        api.listPersonnel(null, true),
        api.listWorksWithCasts()
      ]);
      personnelList = pList;
      worksWithCasts = wList;
    } catch (e) {
      console.error('Error cargando padrón y obras:', e);
    }
  }

  onMount(() => {
    loadPersonnelAndWorks();
  });

  // Estado del Modal (Alta / Edición)
  let showModal = $state(false);
  let editingBlockId = $state<string | null>(null);

  let selectedRoom = $state<RoomCode>('SALA_1');
  let startTime = $state('10:30');
  let endTime = $state('11:30');
  let passageName = $state('Princesas y Bufón');
  let soundType = $state<'teatrocue' | 'pianist' | 'orchestra' | 'silence'>('teatrocue');
  let soundReq = $state('PISTA TeatroCue #04');
  let selectedPianistId = $state<string>('');
  let masterId = $state<string>('');
  let soundTechId = $state<string>('');
  // Se conservan al editar para no perder la obra vinculada
  let editingWorkId = $state<string | null>(null);
  let editingPassageId = $state<string | null>(null);
  let isToConfirm = $state(false);
  let allowSharedRoom = $state(false);
  let notes = $state('');

  // Reparto y Control de Bailarines Citados
  let castLine = $state('');
  let selectedDancerIds = $state<string[]>([]);
  let dancerSearch = $state('');

  let isSubmitting = $state(false);

  // Presets habituales de repertorio y gira
  const worksPresets = [
    { name: 'Princesas y Bufón', cue: 'PISTA TeatroCue #04', soundType: 'teatrocue' as const },
    { name: 'Danza Española', cue: 'PISTA TeatroCue #11', soundType: 'teatrocue' as const },
    { name: 'Pas de deux Odette y Sigfrido', cue: 'PISTA TeatroCue #08 / Piano', soundType: 'pianist' as const },
    { name: 'Cisne Negro (Grand Pas)', cue: 'PISTA TeatroCue #14', soundType: 'teatrocue' as const },
    { name: 'Diana y Acteón', cue: 'PISTA TeatroCue #21', soundType: 'teatrocue' as const },
    { name: 'Danza Napolitana', cue: 'PISTA TeatroCue #18', soundType: 'teatrocue' as const },
    { name: 'Danza Mazurca', cue: 'PISTA TeatroCue #09', soundType: 'teatrocue' as const },
    { name: 'Gira: Salida Micro Terminal / Explanada', cue: 'SIN MUSICA - LOGISTICA', soundType: 'silence' as const },
    { name: 'Gira: Ensayo General en Escenario Externo', cue: 'ORQUESTA EN VIVO / FOSO', soundType: 'orchestra' as const },
  ];

  function selectPreset(preset: typeof worksPresets[0]) {
    passageName = preset.name;
    soundType = preset.soundType;
    if (preset.soundType === 'teatrocue') {
      soundReq = preset.cue;
    } else if (preset.soundType === 'pianist') {
      soundReq = 'PIANO ACÚSTICO';
      if (pianists.length > 0 && !selectedPianistId) selectedPianistId = pianists[0].id;
    } else if (preset.soundType === 'orchestra') {
      soundReq = 'ORQUESTA EN VIVO - Foso';
    } else {
      soundReq = 'CONTEO DE MAESTRO (Sin sonido)';
    }
  }

  function applyPlannedCasting(work: WorkWithPassagesAndCastsDto, plan: WorkCastingPlanDto) {
    passageName = plan.passage_name ? `${work.title} - ${plan.passage_name}` : `${work.title} (${plan.tier_name})`;
    if (plan.default_sound_cue) {
      soundReq = plan.default_sound_cue;
      if (plan.default_sound_cue.toLowerCase().includes('piano')) {
        soundType = 'pianist';
        if (!selectedPianistId && pianists.length > 0) selectedPianistId = pianists[0].id;
      } else if (plan.default_sound_cue.toLowerCase().includes('orquesta')) {
        soundType = 'orchestra';
      } else {
        soundType = 'teatrocue';
      }
    }
    castLine = plan.cast_label;
    if (plan.members.length > 0) {
      selectedDancerIds = plan.members.map(m => m.personnel_id);
    }
  }

  function handleSoundTypeChange(type: 'teatrocue' | 'pianist' | 'orchestra' | 'silence') {
    soundType = type;
    if (type === 'teatrocue') {
      soundReq = 'PISTA TeatroCue #04';
      selectedPianistId = '';
    } else if (type === 'pianist') {
      soundReq = 'PIANO ACÚSTICO';
      if (!selectedPianistId && pianists.length > 0) selectedPianistId = pianists[0].id;
    } else if (type === 'orchestra') {
      soundReq = 'ORQUESTA EN VIVO - Foso';
      selectedPianistId = '';
    } else {
      soundReq = 'CONTEO DE MAESTRO (Sin sonido)';
      selectedPianistId = '';
    }
  }

  function toggleDancer(dId: string) {
    if (selectedDancerIds.includes(dId)) {
      selectedDancerIds = selectedDancerIds.filter(id => id !== dId);
    } else {
      selectedDancerIds = [...selectedDancerIds, dId];
    }
    updateCastLineFromDancers();
  }

  function updateCastLineFromDancers() {
    if (selectedDancerIds.length === 0) return;
    const names = selectedDancerIds
      .map(id => {
        const d = dancers.find(p => p.id === id);
        return d ? d.last_name : id;
      })
      .filter(Boolean);
    castLine = names.join(' - ');
  }

  function openCreateModal(initialRoom: RoomCode = 'SALA_1') {
    editingBlockId = null;
    selectedRoom = initialRoom;
    startTime = '10:30';
    endTime = '11:30';
    passageName = '';
    soundType = 'teatrocue';
    soundReq = '';
    selectedPianistId = '';
    masterId = '';
    soundTechId = '';
    editingWorkId = null;
    editingPassageId = null;
    isToConfirm = false;
    allowSharedRoom = false;
    notes = '';
    selectedDancerIds = [];
    castLine = '';
    showModal = true;
  }

  function openEditModal(block: RehearsalBlock) {
    editingBlockId = block.id;
    selectedRoom = block.room;
    startTime = block.start_time;
    endTime = block.end_time;
    passageName = block.passage_name;
    masterId = block.master_id || '';
    soundTechId = block.sound_tech_id || '';
    editingWorkId = block.work_id;
    editingPassageId = block.passage_id;
    selectedPianistId = block.pianist_id || '';
    isToConfirm = block.confirmation_state === 'REHEARSAL_TO_CONFIRM';
    allowSharedRoom = block.allow_shared_room;
    notes = '';

    if (block.pianist_id) {
      soundType = 'pianist';
      soundReq = block.sound_requirement || 'PIANO ACÚSTICO';
    } else if (block.sound_requirement && block.sound_requirement.toUpperCase().includes('ORQUESTA')) {
      soundType = 'orchestra';
      soundReq = block.sound_requirement;
    } else if (block.sound_requirement && block.sound_requirement.toUpperCase().includes('CONTEO')) {
      soundType = 'silence';
      soundReq = block.sound_requirement;
    } else {
      soundType = 'teatrocue';
      soundReq = block.sound_requirement || 'PISTA TeatroCue #04';
    }

    // Reconstruir bailarines seleccionados desde cast_groups
    const memberIds: string[] = [];
    if (block.cast_groups && block.cast_groups.length > 0) {
      for (const grp of block.cast_groups) {
        if (grp.formatted_line) {
          castLine = grp.formatted_line;
        }
        for (const mem of grp.members) {
          memberIds.push(mem.personnel_id);
          if (mem.partner_personnel_id) {
            memberIds.push(mem.partner_personnel_id);
          }
        }
      }
    }
    selectedDancerIds = Array.from(new Set(memberIds));
    if (selectedDancerIds.length > 0 && !castLine) {
      updateCastLineFromDancers();
    }
    showModal = true;
  }

  // Chequeo reactivo de colisiones en tiempo real dentro del modal
  function parseTimeToMinutes(t: string): number {
    const [h, m] = t.split(':').map(Number);
    return (h || 0) * 60 + (m || 0);
  }

  const liveCollisionWarnings = $derived.by(() => {
    if (!showModal) return [];
    const warnings: string[] = [];
    const sMin = parseTimeToMinutes(startTime);
    const eMin = parseTimeToMinutes(endTime);

    if (sMin >= eMin) {
      warnings.push('El horario de inicio debe ser menor al horario de fin.');
      return warnings;
    }

    // 1. Colisión de sala
    if (!allowSharedRoom) {
      const roomConflict = blocks.find(b =>
        b.id !== editingBlockId &&
        b.room === selectedRoom &&
        !b.allow_shared_room &&
        parseTimeToMinutes(b.start_time) < eMin &&
        parseTimeToMinutes(b.end_time) > sMin
      );
      if (roomConflict) {
        warnings.push(`Colisión de Sala: ${selectedRoom} ya está ocupada por "${roomConflict.passage_name}" (${roomConflict.start_time} - ${roomConflict.end_time}).`);
      }
    }

    // 2. Colisión de Maestro
    if (masterId) {
      const mConflict = blocks.find(b =>
        b.id !== editingBlockId &&
        b.master_id === masterId &&
        parseTimeToMinutes(b.start_time) < eMin &&
        parseTimeToMinutes(b.end_time) > sMin
      );
      if (mConflict) {
        const mObj = masters.find(m => m.id === masterId);
        const mName = mObj ? `${mObj.last_name}` : 'El maestro/a seleccionado/a';
        warnings.push(`Colisión de Maestro: ${mName} ya está citado/a en ${roomLabel(mConflict.room)} ("${mConflict.passage_name}", ${mConflict.start_time} - ${mConflict.end_time}).`);
      }
    }

    // 3. Colisión de Pianista
    if (soundType === 'pianist' && selectedPianistId) {
      const pConflict = blocks.find(b =>
        b.id !== editingBlockId &&
        b.pianist_id === selectedPianistId &&
        parseTimeToMinutes(b.start_time) < eMin &&
        parseTimeToMinutes(b.end_time) > sMin
      );
      if (pConflict) {
        const pObj = pianists.find(p => p.id === selectedPianistId);
        const pName = pObj ? `${pObj.last_name}` : 'El pianista seleccionado';
        warnings.push(`Colisión de Pianista: ${pName} ya está acompañando en ${roomLabel(pConflict.room)} ("${pConflict.passage_name}", ${pConflict.start_time} - ${pConflict.end_time}).`);
      }
    }

    // 4. Colisión de Bailarines
    for (const dId of selectedDancerIds) {
      const dConflict = blocks.find(b => {
        if (b.id === editingBlockId) return false;
        const overlaps = parseTimeToMinutes(b.start_time) < eMin && parseTimeToMinutes(b.end_time) > sMin;
        if (!overlaps) return false;
        return b.cast_groups.some(g =>
          g.members.some(m => m.personnel_id === dId || m.partner_personnel_id === dId)
        );
      });
      if (dConflict) {
        const dObj = dancers.find(d => d.id === dId);
        const dName = dObj ? `${dObj.last_name}, ${dObj.first_name}` : dId;
        warnings.push(`Colisión de Bailarín/a: ${dName} ya está citado/a en ${roomLabel(dConflict.room)} ("${dConflict.passage_name}", ${dConflict.start_time} - ${dConflict.end_time}).`);
      }
    }

    return warnings;
  });

  async function saveBlock() {
    isSubmitting = true;
    try {
      // Determinar activity_type
      let actType: ActivityType = 'STUDIO_REHEARSAL';
      if (selectedRoom === 'ESCENARIO') actType = 'STAGE_TECH_REHEARSAL';
      else if (selectedRoom === 'GIRA_TRASLADO') actType = 'TOUR_TRAVEL';
      else if (selectedRoom === 'GIRA_ESCENARIO') actType = 'TOUR_STAGE_REHEARSAL';

      // Construir miembros del reparto
      const members: UpsertCastMemberPayload[] = selectedDancerIds.map((dId, idx) => ({
        personnel_id: dId,
        character_role: 'Reparto Oficial',
        partner_personnel_id: null,
        condition: idx % 2 === 1 ? 'ALTERNATE_SLASH' : 'TITULAR',
      }));

      const castGroup: UpsertCastGroupPayload = {
        group_order: 1,
        group_label: castLine || 'Reparto de Estudio',
        is_to_confirm: false,
        members,
      };

      // Obra y pasaje: los que ya tenía el ensayo, o los que coinciden con el nombre elegido
      let linkedWork = editingWorkId;
      let linkedPassage = editingPassageId;
      const known = worksWithCasts.flatMap((w) => w.passages.map((p) => ({ w, p }))).find(({ p }) => p.name === passageName);
      if (known) {
        linkedWork = known.w.id;
        linkedPassage = known.p.id;
      }

      const payload: UpsertRehearsalBlockPayload = {
        id: editingBlockId || null,
        schedule_id: scheduleId,
        room: selectedRoom,
        start_time: startTime,
        end_time: endTime,
        activity_type: actType,
        work_id: linkedWork,
        passage_id: linkedPassage,
        custom_passage_label: passageName,
        confirmation_state: isToConfirm ? 'REHEARSAL_TO_CONFIRM' : 'CONFIRMED',
        sound_requirement: soundReq,
        teatrocue_cue_ref: soundType === 'teatrocue' ? soundReq : null,
        master_id: masterId || null,
        pianist_id: soundType === 'pianist' ? selectedPianistId || null : null,
        sound_tech_id: soundTechId || null,
        allow_shared_room: allowSharedRoom,
        notes: notes || null,
        cast_groups: members.length > 0 ? [castGroup] : []
      };

      await api.upsertRehearsalBlock(payload);
      showModal = false;
      onUpdated();
    } catch (err: any) {
      console.error('Error al guardar bloque de ensayo:', err);
      alert(`Error al guardar citación de ensayo:\n${err?.message || err}`);
    } finally {
      isSubmitting = false;
    }
  }

  async function removeBlock(blockId: string) {
    if (!confirm('¿Confirma eliminar este bloque de citación de ensayo?')) return;
    try {
      await api.deleteRehearsalBlock(blockId);
      onUpdated();
    } catch (err: any) {
      console.error(err);
      alert(`No se pudo eliminar el bloque: ${err?.message || err}`);
    }
  }

  // Verifica si un bloque tiene conflictos auditados
  function getBlockConflicts(bId: string): ScheduleConflict[] {
    return conflicts.filter(c => {
      if (c.conflict_type === 'PERSON_DOUBLE_BOOKED') {
        return c.first_block_id === bId || c.second_block_id === bId;
      }
      if (c.conflict_type === 'ROOM_DOUBLE_BOOKED') {
        return c.first_block_id === bId || c.second_block_id === bId;
      }
      if (c.conflict_type === 'SHIFT_LIMIT_EXCEEDED') {
        return c.block_id === bId;
      }
      return false;
    });
  }
</script>

<div class="bg-[#1a1a1e] border border-[#34343c] rounded p-3 shadow-md">
  <!-- Encabezado de la Grilla y Acciones -->
  <div class="flex flex-wrap items-center justify-between pb-2.5 mb-3 border-b border-[#34343c] gap-2">
    <div>
      <h3 class="text-sm font-bold uppercase text-[#d8d8dc] tracking-wide">
        Grilla Diaria de Ensayos
      </h3>
      <div class="text-[11px] text-[#82828a] mt-0.5">
        Distribución por salas, control de repartos nominales y referencia para TeatroCue
      </div>
    </div>

    <div class="flex items-center gap-2 flex-wrap">
      <!-- Filtro de Salas / Gira -->
      <div class="bg-[#141416] p-0.5 rounded border border-[#34343c] inline-flex text-xs">
        <button
          onclick={() => roomFilter = 'THEATER'}
          class="px-2.5 py-1 rounded font-bold transition-colors {roomFilter === 'THEATER' ? 'bg-[#ffb31a] text-[#121214]' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
        >
          Salas del Teatro
        </button>
        <button
          onclick={() => roomFilter = 'TOUR'}
          class="px-2.5 py-1 rounded font-bold transition-colors {roomFilter === 'TOUR' ? 'bg-[#2ed6e8] text-[#121214]' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
        >
          Gira / Logística
        </button>
        <button
          onclick={() => roomFilter = 'ALL'}
          class="px-2.5 py-1 rounded font-bold transition-colors {roomFilter === 'ALL' ? 'bg-[#24242a] text-[#d8d8dc]' : 'text-[#82828a] hover:text-[#d8d8dc]'}"
        >
          Todas ({allRooms.length})
        </button>
      </div>

      <button
        onclick={() => openCreateModal('SALA_1')}
        class="h-8 px-3 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] font-bold text-xs uppercase rounded flex items-center gap-1.5 transition-colors"
      >
        <span>+ Citar Ensayo</span>
      </button>
    </div>
  </div>

  <!-- Columnas Multipista por Sala -->
  <div class="grid grid-cols-1 md:grid-cols-2 {visibleRooms.length >= 4 ? 'lg:grid-cols-4' : 'lg:grid-cols-3'} gap-3">
    {#each visibleRooms as rm}
      {@const roomBlocks = blocks.filter(b => b.room === rm.code)}
      <div class="bg-[#141416] border border-[#34343c] rounded p-2.5 flex flex-col min-h-[220px]">
        <!-- Encabezado de Columna de Sala -->
        <div class="pb-2 mb-2.5 border-b border-[#34343c] flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="w-2.5 h-2.5 rounded-full {rm.code === 'SALA_1' ? 'bg-[#3de06a]' : rm.code === 'SALA_2' ? 'bg-[#2ed6e8]' : rm.code === 'SALA_3' ? 'bg-[#ffb31a]' : rm.code === 'ESCENARIO' ? 'bg-[#ff7070]' : 'bg-[#82828a]'}"></span>
            <span class="font-bold text-xs text-[#d8d8dc]">{rm.name}</span>
          </div>
          <button
            onclick={() => openCreateModal(rm.code)}
            class="text-[10px] text-[#82828a] hover:text-[#ffb31a] bg-[#1a1a1e] hover:bg-[#24242a] px-2 py-0.5 rounded font-mono border border-[#34343c] transition-colors"
            title="Citar en esta sala"
          >
            + Citar
          </button>
        </div>

        <!-- Bloques en esta sala -->
        <div class="space-y-2 flex-1">
          {#if roomBlocks.length === 0}
            <div class="text-center py-8 text-xs text-[#82828a] font-mono italic">
              Sin ensayos en {rm.name}
            </div>
          {:else}
            {#each roomBlocks as b}
              {@const blockConflicts = getBlockConflicts(b.id)}
              {@const hasConflict = blockConflicts.length > 0}
              
              <div class="bg-[#1a1a1e] border {hasConflict ? 'border-[#ff4a4a] bg-[#2a1616]' : 'border-[#34343c] hover:border-[#82828a]'} rounded p-2.5 transition-colors">
                
                <!-- Horario & Acciones -->
                <div class="flex items-center justify-between mb-1.5">
                  <div class="font-mono text-xs font-bold {hasConflict ? 'text-[#ff4a4a]' : 'text-[#ffb31a]'} px-1.5 py-0.5 rounded bg-[#121214] border border-[#34343c]">
                    {b.start_time} - {b.end_time}
                  </div>
                  
                  <div class="flex items-center gap-1">
                    <button
                      onclick={() => openEditModal(b)}
                      class="text-[#82828a] hover:text-[#ffb31a] text-xs px-1.5 py-0.5 rounded hover:bg-[#24242a] transition-colors font-medium"
                      title="Modificar citación"
                    >
                      Editar
                    </button>
                    <button
                      onclick={() => removeBlock(b.id)}
                      class="text-[#82828a] hover:text-[#ff4a4a] text-xs px-1.5 py-0.5 rounded hover:bg-[#24242a] transition-colors font-mono"
                      title="Eliminar citación"
                    >
                      ✕
                    </button>
                  </div>
                </div>

                <!-- Pasaje / Obra -->
                <div class="font-black text-sm text-white mb-1.5 flex items-start justify-between gap-1">
                  <span>{b.passage_name}</span>
                  {#if b.allow_shared_room}
                    <span class="text-[9px] bg-blue-900/40 text-blue-300 border border-blue-700/50 px-1.5 py-0.2 rounded font-mono shrink-0">
                      Sala compartida
                    </span>
                  {/if}
                </div>

                <!-- Reparto Formateado -->
                <div class="bg-slate-950 px-2.5 py-1.5 rounded-lg border border-slate-800/80 text-xs font-bold text-slate-300 mb-2">
                  <span class="text-slate-500 text-[10px] uppercase font-bold block mb-0.5">Reparto de Estudio:</span>
                  {#if b.cast_groups.length > 0 && b.cast_groups[0].formatted_line}
                    <span class="text-amber-300 font-mono text-[11px] leading-tight block">{b.cast_groups[0].formatted_line}</span>
                  {:else}
                    <span class="text-slate-400 text-[11px] block">Reparto a confirmar</span>
                  {/if}
                </div>

                <!-- Columna SONIDO para TeatroCue y Acompañamiento -->
                <div class="space-y-1 text-xs pt-1.5 border-t border-[#34343c]">
                  <div class="flex items-center justify-between text-[11px]">
                    <div class="flex items-center gap-1.5 text-[#2ed6e8] font-mono font-medium truncate">
                      <span class="text-[9px] uppercase px-1 py-0.2 rounded bg-[#24242a] text-[#2ed6e8] border border-[#34343c] font-bold">
                        {b.pianist_name ? 'PIANO' : 'SONIDO'}
                      </span>
                      <span class="truncate">{b.sound_requirement}</span>
                    </div>
                  </div>

                  <div class="flex items-center justify-between text-[10px] text-[#82828a]">
                    <div>
                      Mtro: <span class="text-[#d8d8dc] font-semibold">{b.master_name ? b.master_name.split(',')[0] : 'Sin asignar'}</span>
                    </div>
                    {#if b.pianist_name}
                      <div>
                        Pno: <span class="text-[#d8d8dc] font-semibold">{b.pianist_name.split(',')[0]}</span>
                      </div>
                    {/if}
                  </div>
                  {#if b.sound_tech_name}
                    <div class="text-[10px] text-[#82828a]">
                      Sonido: <span class="text-[#2ed6e8] font-semibold">{b.sound_tech_name.split(',')[0]}</span>
                    </div>
                  {/if}
                </div>

                <!-- Alerta Visual si hay Cruce / Colisión -->
                {#if hasConflict}
                  <div class="mt-2 p-2 rounded bg-[#ff4a4a]/10 border border-[#ff4a4a]/40 text-[10px] text-[#ffd0d0] space-y-1">
                    {#each blockConflicts as c}
                      <div class="flex items-center gap-1.5 font-semibold text-[#ff4a4a]">
                        <span class="w-1.5 h-1.5 rounded-full bg-[#ff4a4a] shrink-0"></span>
                        {#if c.conflict_type === 'PERSON_DOUBLE_BOOKED'}
                          <span>Cruce: {c.person_name} ({c.overlap_start}-{c.overlap_end})</span>
                        {:else if c.conflict_type === 'ROOM_DOUBLE_BOOKED'}
                          <span>Colisión de Sala: {roomLabel(c.room)} ({c.overlap_start}-{c.overlap_end})</span>
                        {:else if c.conflict_type === 'SHIFT_LIMIT_EXCEEDED'}
                          <span>Excede límite {c.allowed_limit_time} h</span>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/if}

                <!-- Badge Condicional si aplica -->
                {#if b.confirmation_state !== 'CONFIRMED'}
                  <div class="mt-2 text-[10px] font-semibold px-2 py-0.5 rounded bg-amber-500/15 text-amber-400 border border-amber-500/30 text-center">
                    A confirmar: {b.confirmation_state === 'REHEARSAL_TO_CONFIRM' ? 'Ensayo' : 'Reparto'}
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- ========================================================================= -->
<!-- MODAL PARA CITAR O EDITAR BLOQUE DE ENSAYO                                 -->
<!-- ========================================================================= -->
{#if showModal}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3 sm:p-4">
    <div class="w-full max-w-2xl max-h-[92vh] flex flex-col bg-[#1a1a1e] border border-[#34343c] rounded-lg shadow-2xl overflow-hidden text-xs">
      
      <!-- Encabezado Modal (shrink-0) -->
      <div class="p-4 border-b border-[#34343c] flex items-center justify-between shrink-0 bg-[#17171b]">
        <div>
          <h3 class="text-sm font-bold uppercase text-[#d8d8dc] tracking-wide">
            {editingBlockId ? 'Modificar Citación de Ensayo' : 'Citar Nuevo Bloque de Ensayo'}
          </h3>
          <p class="text-[11px] text-[#82828a]">Control de repartos de estudio y chequeo de colisión en tiempo real</p>
        </div>
        <button onclick={() => showModal = false} class="text-[#82828a] hover:text-white font-mono text-sm">✕</button>
      </div>

      <!-- Alertas de Colisión en Tiempo Real dentro del Modal -->
      {#if liveCollisionWarnings.length > 0}
        <div class="m-3 mb-0 p-2.5 bg-[#2a1616] border border-[#ff4a4a] rounded text-xs text-[#ffd0d0] space-y-1 shrink-0">
          <div class="font-bold text-[#ff4a4a] uppercase text-[11px]">
            Colisión Detectada
          </div>
          {#each liveCollisionWarnings as w}
            <div class="pl-2 border-l-2 border-[#ff4a4a] text-[11px]">{w}</div>
          {/each}
        </div>
      {/if}

      <!-- Cuerpo Scrolleable -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3.5">
        
        <!-- Fila 1: Sala, Horario Inicio y Horario Fin -->
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
          <div>
            <label for="modal-block-room" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Sala / Ubicación</label>
            <select id="modal-block-room" bind:value={selectedRoom} class="w-full h-9 px-2.5 bg-[#141416] border border-[#34343c] rounded text-xs text-[#d8d8dc]">
              <optgroup label="Salas del Teatro">
                <option value="SALA_1">Sala 1</option>
                <option value="SALA_2">Sala 2</option>
                <option value="SALA_3">Sala 3</option>
                <option value="ESCENARIO">Escenario Principal</option>
              </optgroup>
              <optgroup label="Logística de Gira">
                <option value="GIRA_TRASLADO">Gira: Traslado / Salida</option>
                <option value="GIRA_ESCENARIO">Gira: Escenario Externo</option>
                <option value="GIRA_SALA">Gira: Sala de Ensayo</option>
              </optgroup>
            </select>
          </div>

          <div>
            <label for="modal-block-start" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Hora Inicio</label>
            <input id="modal-block-start" type="text" bind:value={startTime} placeholder="HH:MM" class="w-full h-9 px-2.5 bg-[#141416] border border-[#34343c] rounded text-xs font-mono font-bold text-[#d8d8dc] text-center" />
          </div>

          <div>
            <label for="modal-block-end" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Hora Fin</label>
            <input id="modal-block-end" type="text" bind:value={endTime} placeholder="HH:MM" class="w-full h-9 px-2.5 bg-[#141416] border border-[#34343c] rounded text-xs font-mono font-bold text-[#d8d8dc] text-center" />
          </div>
        </div>

        <!-- Selección Rápida de Reparto Planificado de Obra (1-Touch Casting) -->
        {#if worksWithCasts.length > 0}
          <div class="bg-[#141416] p-2.5 rounded border border-[#34343c]">
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-[#ffb31a]">
                Repartos Planificados de Obras (Auto-Completar Citación)
              </span>
              <span class="text-[9px] text-[#82828a] font-mono">1-Touch: pasaje, sonido y elenco</span>
            </div>
            <div class="flex flex-wrap gap-1.5 max-h-28 overflow-y-auto pr-1">
              {#each worksWithCasts as w}
                {#each w.casting_plans as plan}
                  <button
                    type="button"
                    onclick={() => applyPlannedCasting(w, plan)}
                    class="text-[11px] bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] hover:text-[#ffb31a] border border-[#34343c] px-2 py-0.5 rounded transition-colors flex items-center gap-1 font-mono"
                    title="{w.title} - {plan.tier_name}: {plan.cast_label}"
                  >
                    <span class="text-[#ffb31a] font-bold">{w.title}:</span>
                    <span>{plan.passage_name || plan.tier_name}</span>
                    <span class="text-[9px] text-[#82828a]">[{plan.members.length} citados]</span>
                  </button>
                {/each}
              {/each}
            </div>
          </div>
        {/if}

        <!-- Fila 2: Pasaje / Obra y Presets Rápidos -->
        <div>
          <label for="modal-block-passage" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Pasaje / Obra / Detalle</label>
          <input id="modal-block-passage" type="text" bind:value={passageName} class="w-full h-9 px-2.5 bg-[#141416] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] mb-1.5" />
          <div class="flex flex-wrap gap-1">
            {#each worksPresets as wp}
              <button
                type="button"
                onclick={() => selectPreset(wp)}
                class="text-[10px] bg-[#24242a] hover:bg-[#34343e] text-[#82828a] hover:text-[#d8d8dc] px-2 py-0.5 rounded border border-[#34343c] transition-colors"
              >
                {wp.name}
              </button>
            {/each}
          </div>
        </div>

        <!-- Fila 3: Acompañamiento Musical (Pianistas, Sonidistas TeatroCue, Orquesta) -->
        <div class="bg-[#141416] p-2.5 rounded border border-[#34343c]">
          <span class="block text-[10px] font-bold uppercase text-[#82828a] mb-2">Acompañamiento Musical & Sonido</span>
          
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-1.5 mb-2.5">
            <button
              type="button"
              onclick={() => handleSoundTypeChange('teatrocue')}
              class="py-1.5 px-2 rounded text-xs font-bold transition-colors border {soundType === 'teatrocue' ? 'bg-[#18262a] text-[#2ed6e8] border-[#2ed6e8]' : 'bg-[#24242a] text-[#82828a] border-[#34343c]'}"
            >
              Pista TeatroCue
            </button>
            <button
              type="button"
              onclick={() => handleSoundTypeChange('pianist')}
              class="py-1.5 px-2 rounded text-xs font-bold transition-colors border {soundType === 'pianist' ? 'bg-[#2a2218] text-[#ffb31a] border-[#ffb31a]' : 'bg-[#24242a] text-[#82828a] border-[#34343c]'}"
            >
              Pianista
            </button>
            <button
              type="button"
              onclick={() => handleSoundTypeChange('orchestra')}
              class="py-1.5 px-2 rounded text-xs font-bold transition-colors border {soundType === 'orchestra' ? 'bg-[#22182a] text-[#9d72ff] border-[#9d72ff]' : 'bg-[#24242a] text-[#82828a] border-[#34343c]'}"
            >
              Orquesta
            </button>
            <button
              type="button"
              onclick={() => handleSoundTypeChange('silence')}
              class="py-1.5 px-2 rounded text-xs font-bold transition-colors border {soundType === 'silence' ? 'bg-[#24242a] text-[#d8d8dc] border-[#82828a]' : 'bg-[#24242a] text-[#82828a] border-[#34343c]'}"
            >
              Silencio / Conteo
            </button>
          </div>

          {#if soundType === 'teatrocue'}
            <div>
              <label for="modal-block-cue" class="block text-[10px] font-mono uppercase text-[#2ed6e8] mb-1">Referencia de Cabina TeatroCue / Pista</label>
              <input id="modal-block-cue" type="text" bind:value={soundReq} class="w-full h-8 px-2.5 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-mono font-bold text-[#2ed6e8]" />
            </div>
          {:else if soundType === 'pianist'}
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <div>
                <label for="modal-block-pianist" class="block text-[10px] font-bold uppercase text-[#ffb31a] mb-1">Pianista a Cargo</label>
                <select id="modal-block-pianist" bind:value={selectedPianistId} class="w-full h-8 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc]">
                  {#each pianists as p}
                    <option value={p.id}>{p.last_name}, {p.first_name}</option>
                  {/each}
                </select>
              </div>
              <div>
                <label for="modal-block-piano-req" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Detalle / Piano</label>
                <input id="modal-block-piano-req" type="text" bind:value={soundReq} class="w-full h-8 px-2.5 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc]" />
              </div>
            </div>
          {:else if soundType === 'orchestra'}
            <div>
              <label for="modal-block-orch-req" class="block text-[10px] font-bold uppercase text-[#9d72ff] mb-1">Requerimiento de Orquesta en Foso</label>
              <input id="modal-block-orch-req" type="text" bind:value={soundReq} class="w-full h-8 px-2.5 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc]" />
            </div>
          {:else}
            <div>
              <label for="modal-block-mod-req" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Modalidad de Ensayo</label>
              <input id="modal-block-mod-req" type="text" bind:value={soundReq} class="w-full h-8 px-2.5 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc]" />
            </div>
          {/if}
        </div>

        <!-- Responsable de sonido -->
        <div>
          <label for="modal-block-soundtech" class="block text-[10px] font-bold uppercase text-[#2ed6e8] mb-1">Sonidista a cargo</label>
          <select id="modal-block-soundtech" bind:value={soundTechId} class="w-full h-9 px-2.5 bg-[#141416] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc]">
            <option value="">Sin asignar</option>
            {#each soundTechs as t}
              <option value={t.id}>{t.last_name}, {t.first_name}</option>
            {/each}
          </select>
        </div>

        <!-- Fila 4: Maestro/a a Cargo -->
        <div>
          <label for="modal-block-master" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1">Maestro/a a Cargo</label>
          <select id="modal-block-master" bind:value={masterId} class="w-full h-9 px-2.5 bg-[#141416] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc]">
            {#each masters as m}
              <option value={m.id}>{m.last_name}, {m.first_name}</option>
            {/each}
            <option value="">Sin maestro/a asignado/a</option>
          </select>
        </div>

        <!-- Fila 5: Control de Reparto y Citados (Bailarines) -->
        <div class="bg-[#141416] p-2.5 rounded border border-[#34343c]">
          <div class="flex items-center justify-between mb-1.5">
            <label for="modal-block-castline" class="text-[10px] font-bold uppercase text-[#ffb31a]">
              Línea Visible de Reparto de Estudio
            </label>
            <span class="text-[10px] text-[#82828a] font-mono">
              {selectedDancerIds.length} citados
            </span>
          </div>

          <input
            id="modal-block-castline"
            type="text"
            bind:value={castLine}
            placeholder="ej. Álvarez/Marcato - Morchio C/Gil"
            class="w-full h-9 px-2.5 bg-[#1a1a1e] border border-[#34343c] rounded text-xs font-mono font-bold text-[#ffb31a] mb-2"
          />

          <!-- Selector Rápido de Bailarines del Padrón -->
          <div class="mt-2 pt-2 border-t border-[#34343c]">
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-[10px] font-bold uppercase text-[#82828a]">Seleccionar del Padrón:</span>
              <input
                type="text"
                placeholder="Buscar bailarín..."
                bind:value={dancerSearch}
                class="h-7 px-2 bg-[#1a1a1e] border border-[#34343c] rounded text-[10px] text-[#d8d8dc] w-40 font-mono"
              />
            </div>

            <div class="flex flex-wrap gap-1 max-h-28 overflow-y-auto pr-1">
              {#each dancers.filter(d => d.last_name.toLowerCase().includes(dancerSearch.toLowerCase()) || d.first_name.toLowerCase().includes(dancerSearch.toLowerCase())) as d}
                {@const isSelected = selectedDancerIds.includes(d.id)}
                <button
                  type="button"
                  onclick={() => toggleDancer(d.id)}
                  class="text-[10px] px-2 py-0.5 rounded font-mono transition-colors border {isSelected ? 'bg-[#ffb31a] text-[#121214] border-[#ffb31a] font-bold' : 'bg-[#24242a] text-[#82828a] hover:text-[#d8d8dc] border-[#34343c]'}"
                >
                  {d.last_name}
                </button>
              {/each}
            </div>
          </div>
        </div>

        <!-- Opciones Adicionales -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-1">
          <label class="flex items-center gap-2 text-xs text-[#d8d8dc] cursor-pointer bg-[#141416] p-2 rounded border border-[#34343c]">
            <input type="checkbox" bind:checked={isToConfirm} class="h-3.5 w-3.5 rounded accent-[#ffb31a]" />
            <span>Ensayo a confirmar</span>
          </label>

          <label class="flex items-center gap-2 text-xs text-[#d8d8dc] cursor-pointer bg-[#141416] p-2 rounded border border-[#34343c]">
            <input type="checkbox" bind:checked={allowSharedRoom} class="h-3.5 w-3.5 rounded accent-[#2ed6e8]" />
            <span>Permitir sala compartida</span>
          </label>
        </div>
      </div>

      <!-- Botonera Modal (shrink-0) -->
      <div class="p-3 border-t border-[#34343c] bg-[#17171b] shrink-0 flex gap-2">
        <button
          type="button"
          onclick={() => showModal = false}
          class="flex-1 h-9 bg-[#24242a] hover:bg-[#34343e] text-[#82828a] hover:text-[#d8d8dc] font-bold rounded text-xs transition-colors"
        >
          Cancelar
        </button>
        <button
          type="button"
          disabled={isSubmitting}
          onclick={saveBlock}
          class="flex-1 h-9 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] font-black rounded text-xs uppercase transition-colors"
        >
          {isSubmitting ? 'Guardando...' : editingBlockId ? 'Actualizar y Auditar' : 'Guardar y Auditar'}
        </button>
      </div>
    </div>
  </div>
{/if}
