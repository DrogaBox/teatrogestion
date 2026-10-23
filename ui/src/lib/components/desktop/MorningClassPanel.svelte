<script lang="ts">
  import { roomLabel } from '../../labels';
  import { onMount } from 'svelte';
  import {
    api,
    type GenderGroup,
    type MorningClassSlot,
    type PersonnelWithQuotaDto,
    type RoomCode,
    type UpsertMorningClassPayload
  } from '../../ipc/client';

  let {
    classes,
    scheduleId = '',
    onUpdated = () => {}
  } = $props<{
    classes: MorningClassSlot[];
    scheduleId?: string;
    onUpdated?: () => void;
  }>();

  // Padrón de Personal
  let personnel = $state<PersonnelWithQuotaDto[]>([]);
  let masters = $derived(personnel.filter(p => p.role === 'MASTER' && p.is_active));
  let pianists = $derived(personnel.filter(p => p.role === 'PIANIST' && p.is_active));
  let soundTechs = $derived(personnel.filter(p => p.role === 'SOUND_TECH' && p.is_active));
  let soundTechId = $state('');

  async function loadPersonnel() {
    try {
      personnel = await api.listPersonnel(null, true);
    } catch (e) {
      console.error(e);
    }
  }

  onMount(() => {
    loadPersonnel();
  });

  // Estado del Modal de Edición de Clase
  let showModal = $state(false);
  let editingClassId = $state<string | null>(null);
  let room = $state<RoomCode>('SALA_1');
  let genderDivision = $state<GenderGroup>('FEMALE');
  let startTime = $state('09:00');
  let endTime = $state('10:15');
  let masterId = $state('PER_MASTER_ANANIEV');
  let accompanimentType = $state<'pianist' | 'teatrocue'>('pianist');
  let pianistId = $state('PER_PIANIST_ROMERO');
  let isSaving = $state(false);

  function openCreateModal() {
    editingClassId = null;
    room = 'SALA_2';
    genderDivision = 'FEMALE';
    startTime = '09:00';
    endTime = '10:15';
    masterId = masters.length > 0 ? masters[0].id : 'PER_MASTER_ANANIEV';
    pianistId = pianists.length > 0 ? pianists[0].id : 'PER_PIANIST_ROMERO';
    accompanimentType = 'pianist';
    soundTechId = '';
    showModal = true;
  }

  function openEditModal(c: MorningClassSlot) {
    editingClassId = c.id;
    room = c.room;
    genderDivision = c.gender_division;
    startTime = c.start_time;
    endTime = c.end_time;
    masterId = c.master_id;
    pianistId = c.pianist_id;
    soundTechId = c.sound_tech_id || '';
    accompanimentType = c.pianist_id === 'PER_SOUND_TEATROCUE' ? 'teatrocue' : 'pianist';
    showModal = true;
  }

  async function saveClass() {
    if (!scheduleId) return;
    isSaving = true;
    try {
      const payload: UpsertMorningClassPayload = {
        id: editingClassId || null,
        schedule_id: scheduleId,
        room,
        gender_division: genderDivision,
        start_time: startTime,
        end_time: endTime,
        master_id: masterId,
        pianist_id: accompanimentType === 'teatrocue' ? 'PER_SOUND_TEATROCUE' : pianistId,
        sound_tech_id: soundTechId || null,
      };

      await api.upsertMorningClass(payload);
      showModal = false;
      onUpdated();
    } catch (err: any) {
      console.error(err);
      alert(`Error al guardar clase técnica:\n${err?.message || err}`);
    } finally {
      isSaving = false;
    }
  }

  async function deleteClass(classId: string) {
    if (!confirm('¿Confirma eliminar esta clase matutina?')) return;
    try {
      await api.deleteMorningClass(classId);
      onUpdated();
    } catch (err: any) {
      console.error(err);
      alert(`No se pudo eliminar la clase: ${err?.message || err}`);
    }
  }
</script>

<div class="bg-[#1a1a1e] border border-[#34343c] rounded p-3 shadow-md mb-3">
  <div class="flex items-center justify-between pb-2 mb-2.5 border-b border-[#34343c]">
    <div>
      <h3 class="text-xs font-bold uppercase tracking-wider text-[#d8d8dc]">
        Clases Técnicas Matutinas ({classes.length > 0 ? `${classes[0].start_time} a ${classes[0].end_time} h` : '09:00 a 10:15 h'})
      </h3>
    </div>
    
    <div class="flex items-center gap-2">
      <span class="text-[11px] font-mono text-[#ffb31a] bg-[#141416] border border-[#34343c] px-2 py-0.5 rounded">
        {classes.length} {classes.length === 1 ? 'Clase' : 'Clases Simultáneas'}
      </span>
      {#if scheduleId}
        <button
          onclick={openCreateModal}
          class="text-xs font-bold text-[#d8d8dc] hover:text-[#ffb31a] bg-[#24242a] hover:bg-[#34343e] px-2.5 py-1 rounded border border-[#34343c] transition-colors"
        >
          + Añadir Clase
        </button>
      {/if}
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2.5">
    {#each classes as c}
      <div class="bg-[#141416] border border-[#34343c] rounded p-2.5 flex items-center justify-between relative group hover:border-[#82828a] transition-colors">
        <div class="flex items-center gap-2.5">
          <div class="w-10 h-10 rounded bg-[#1a1a1e] border border-[#34343c] flex flex-col items-center justify-center font-mono shrink-0">
            <span class="text-[8px] text-[#82828a] font-bold uppercase">Sala</span>
            <span class="text-xs font-bold text-[#ffb31a]">
              {c.room.replace('SALA_', '').replace('ESCENARIO', 'ESC').replace('GIRA_', 'G-')}
            </span>
          </div>

          <div>
            <div class="text-[11px] font-bold uppercase text-[#82828a] flex items-center gap-1.5 flex-wrap">
              <span>{roomLabel(c.room)} ({c.gender_division === 'FEMALE' ? 'Mujeres' : c.gender_division === 'MALE' ? 'Varones' : 'General'})</span>
              <span class="text-[10px] bg-[#1a1a1e] text-[#d8d8dc] px-1.5 py-0.2 rounded font-mono">
                {c.start_time} - {c.end_time}
              </span>
            </div>
            <div class="text-xs font-bold text-[#d8d8dc] mt-0.5">
              Mtro: <span class="text-[#ffb31a]">{c.master_name || 'Sin maestro'}</span>
            </div>
            <div class="text-[11px] text-[#82828a] flex items-center gap-1 font-mono">
              <span>{c.pianist_id === 'PER_SOUND_TEATROCUE' ? 'Pista:' : 'Piano:'}</span>
              <span class="text-[#d8d8dc]">{c.pianist_name || 'Sin pianista'}</span>
              {#if c.sound_tech_name}
                <span class="text-[#82828a]">· Sonido:</span>
                <span class="text-[#2ed6e8]">{c.sound_tech_name.split(',')[0]}</span>
              {/if}
            </div>
          </div>
        </div>

        {#if scheduleId}
          <div class="flex flex-col items-end gap-1 shrink-0">
            <button
              onclick={() => openEditModal(c)}
              class="text-xs text-[#82828a] hover:text-[#ffb31a] px-1.5 py-0.5 rounded hover:bg-[#24242a] transition-colors font-medium"
              title="Modificar maestro, pianista o sala"
            >
              Editar
            </button>
            <button
              onclick={() => deleteClass(c.id)}
              class="text-xs text-[#82828a] hover:text-[#ff4a4a] px-1.5 py-0.5 rounded hover:bg-[#24242a] transition-colors font-mono"
              title="Eliminar clase"
            >
              ✕
            </button>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<!-- Modal para Crear / Editar Clase Técnica Matutina -->
{#if showModal}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-3">
    <div class="w-full max-w-lg max-h-[92vh] flex flex-col bg-[#1a1a1e] border border-[#34343c] rounded-md shadow-2xl overflow-hidden">
      <div class="bg-[#24242a] border-b border-[#34343c] px-4 py-2.5 flex items-center justify-between">
        <div>
          <h3 class="text-xs font-bold uppercase tracking-wider text-[#d8d8dc]">
            {editingClassId ? 'Modificar Clase Técnica' : 'Añadir Clase Técnica'}
          </h3>
          <p class="text-[10px] text-[#82828a]">Asignación de sala, maestro, pianista o sonido</p>
        </div>
        <button onclick={() => showModal = false} class="text-[#82828a] hover:text-[#d8d8dc] text-sm font-mono px-2 py-0.5 rounded hover:bg-[#34343c]">✕</button>
      </div>

      <div class="p-4 space-y-3 overflow-y-auto">
        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="class-room-select" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1 font-mono">Sala</label>
            <select id="class-room-select" bind:value={room} class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]">
              <option value="SALA_1">Sala 1</option>
              <option value="SALA_2">Sala 2</option>
              <option value="SALA_3">Sala 3</option>
              <option value="ESCENARIO">Escenario</option>
              <option value="GIRA_SALA">Gira: Sala de Ensayo</option>
              <option value="GIRA_ESCENARIO">Gira: Escenario</option>
            </select>
          </div>

          <div>
            <label for="class-division-select" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1 font-mono">División</label>
            <select id="class-division-select" bind:value={genderDivision} class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]">
              <option value="FEMALE">Mujeres</option>
              <option value="MALE">Varones</option>
              <option value="NOT_APPLICABLE">General Compañía</option>
            </select>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="class-start-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1 font-mono">Hora Inicio</label>
            <input id="class-start-input" type="text" bind:value={startTime} placeholder="09:00" class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-mono font-bold text-[#d8d8dc] text-center outline-none focus:border-[#ffb31a]" />
          </div>
          <div>
            <label for="class-end-input" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1 font-mono">Hora Fin</label>
            <input id="class-end-input" type="text" bind:value={endTime} placeholder="10:15" class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-mono font-bold text-[#d8d8dc] text-center outline-none focus:border-[#ffb31a]" />
          </div>
        </div>

        <div>
          <label for="class-master-select" class="block text-[10px] font-bold uppercase text-[#82828a] mb-1 font-mono">Maestro/a a Cargo</label>
          <select id="class-master-select" bind:value={masterId} class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]">
            {#each masters as m}
              <option value={m.id}>{m.last_name}, {m.first_name}</option>
            {/each}
          </select>
        </div>

        <!-- Selector de Acompañamiento: Pianista vs Cabina TeatroCue -->
        <div class="bg-[#17171b] p-2.5 rounded border border-[#34343c]">
          <span class="block text-[10px] font-bold uppercase text-[#82828a] mb-2 font-mono">Acompañamiento</span>
          <div class="grid grid-cols-2 gap-2 mb-2">
            <button
              type="button"
              onclick={() => accompanimentType = 'pianist'}
              class="h-8 px-2.5 rounded text-xs font-bold border transition-colors {accompanimentType === 'pianist' ? 'bg-[#2a2218] text-[#ffb31a] border-[#ffb31a]' : 'bg-[#24242a] text-[#82828a] border-[#34343c]'}"
            >
              Pianista
            </button>
            <button
              type="button"
              onclick={() => accompanimentType = 'teatrocue'}
              class="h-8 px-2.5 rounded text-xs font-bold border transition-colors {accompanimentType === 'teatrocue' ? 'bg-[#18262a] text-[#2ed6e8] border-[#2ed6e8]' : 'bg-[#24242a] text-[#82828a] border-[#34343c]'}"
            >
              Pista TeatroCue
            </button>
          </div>

          <div class="mb-2">
            <label for="class-soundtech-select" class="block text-[10px] font-bold uppercase text-[#2ed6e8] mb-1 font-mono">Sonidista a cargo</label>
            <select id="class-soundtech-select" bind:value={soundTechId} class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]">
              <option value="">Sin asignar</option>
              {#each soundTechs as t}
                <option value={t.id}>{t.last_name}, {t.first_name}</option>
              {/each}
            </select>
          </div>

          {#if accompanimentType === 'pianist'}
            <div>
              <label for="class-pianist-select" class="block text-[10px] font-bold uppercase text-[#ffb31a] mb-1 font-mono">Pianista Acompañante</label>
              <select id="class-pianist-select" bind:value={pianistId} class="w-full h-8 px-2.5 bg-[#121214] border border-[#34343c] rounded text-xs font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]">
                {#each pianists as p}
                  <option value={p.id}>{p.last_name}, {p.first_name}</option>
                {/each}
              </select>
            </div>
          {:else}
            <div class="text-[11px] text-[#2ed6e8] font-mono py-1">
              [AUDIO] Reproducción de cabina TeatroCue / Pistas técnicas
            </div>
          {/if}
        </div>
      </div>

      <div class="bg-[#24242a] px-4 py-2.5 border-t border-[#34343c] flex items-center justify-end gap-2">
        <button onclick={() => showModal = false} class="h-8 px-3 bg-[#1a1a1e] hover:bg-[#34343c] text-[#82828a] hover:text-[#d8d8dc] font-bold rounded text-xs transition-colors border border-[#34343c]">
          Cancelar
        </button>
        <button
          disabled={isSaving}
          onclick={saveClass}
          class="h-8 px-4 bg-[#ffb31a] hover:bg-[#e69f14] text-[#121214] font-bold rounded text-xs transition-colors disabled:opacity-50"
        >
          {isSaving ? 'Guardando...' : 'Guardar Clase'}
        </button>
      </div>
    </div>
  </div>
{/if}
