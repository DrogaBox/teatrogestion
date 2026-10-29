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
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

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

<div class="bg-panel] border border-grid] rounded p-3 shadow-md mb-3">
  <div class="flex items-center justify-between pb-2 mb-2.5 border-b border-grid]">
    <div>
      <h3 class="text-xs font-bold uppercase tracking-wider text-ink]">
        Clases Técnicas Matutinas ({classes.length > 0 ? `${classes[0].start_time} a ${classes[0].end_time} h` : '09:00 a 10:15 h'})
      </h3>
    </div>
    
    <div class="flex items-center gap-2">
      <span class="text-[11px] font-mono text-amber] bg-void] border border-grid] px-2 py-0.5 rounded">
        {classes.length} {classes.length === 1 ? 'Clase' : 'Clases Simultáneas'}
      </span>
      {#if scheduleId}
        <button
          onclick={openCreateModal}
          class="text-xs font-bold text-ink] hover:text-amber] bg-raised] hover:bg-raised-hover] px-2.5 py-1 rounded border border-grid] transition-colors"
        >
          + Añadir Clase
        </button>
      {/if}
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2.5">
    {#each classes as c}
      <div class="bg-void] border border-grid] rounded p-2.5 flex items-center justify-between relative group hover:border-dim] transition-colors">
        <div class="flex items-center gap-2.5">
          <div class="w-10 h-10 rounded bg-panel] border border-grid] flex flex-col items-center justify-center font-mono shrink-0">
            <span class="text-[8px] text-dim] font-bold uppercase">Sala</span>
            <span class="text-xs font-bold text-amber]">
              {c.room.replace('SALA_', '').replace('ESCENARIO', 'ESC').replace('GIRA_', 'G-')}
            </span>
          </div>

          <div>
            <div class="text-[11px] font-bold uppercase text-dim] flex items-center gap-1.5 flex-wrap">
              <span>{roomLabel(c.room)} ({c.gender_division === 'FEMALE' ? 'Mujeres' : c.gender_division === 'MALE' ? 'Varones' : 'General'})</span>
              <span class="text-[10px] bg-panel] text-ink] px-1.5 py-0.2 rounded font-mono">
                {c.start_time} - {c.end_time}
              </span>
            </div>
            <div class="text-xs font-bold text-ink] mt-0.5">
              Mtro: <span class="text-amber]">{c.master_name || 'Sin maestro'}</span>
            </div>
            <div class="text-[11px] text-dim] flex items-center gap-1 font-mono">
              <span>{c.pianist_id === 'PER_SOUND_TEATROCUE' ? 'Pista:' : 'Piano:'}</span>
              <span class="text-ink]">{c.pianist_name || 'Sin pianista'}</span>
              {#if c.sound_tech_name}
                <span class="text-dim]">· Sonido:</span>
                <span class="text-cyan]">{c.sound_tech_name.split(',')[0]}</span>
              {/if}
            </div>
          </div>
        </div>

        {#if scheduleId}
          <div class="flex flex-col items-end gap-1 shrink-0">
            <button
              onclick={() => openEditModal(c)}
              class="text-xs text-dim] hover:text-amber] px-1.5 py-0.5 rounded hover:bg-raised] transition-colors font-medium"
              title="Modificar maestro, pianista o sala"
            >
              Editar
            </button>
            <button
              onclick={() => deleteClass(c.id)}
              class="text-xs text-dim] hover:text-red] px-1.5 py-0.5 rounded hover:bg-raised] transition-colors font-mono"
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
  <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3" use:modalBehavior={{ onclose: () => showModal = false, backdrop: false }}>
    <div class="tg-modal max-w-lg">
      <ModalHeader
        title={editingClassId ? 'Modificar Clase Técnica' : 'Añadir Clase Técnica'}
        subtitle="Asignación de sala, maestro, pianista o sonido"
        onclose={() => showModal = false}
      />

      <div class="tg-modal-body space-y-3">
        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="class-room-select" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Sala</label>
            <select id="class-room-select" bind:value={room} class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]">
              <option value="SALA_1">Sala 1</option>
              <option value="SALA_2">Sala 2</option>
              <option value="SALA_3">Sala 3</option>
              <option value="ESCENARIO">Escenario</option>
              <option value="GIRA_SALA">Gira: Sala de Ensayo</option>
              <option value="GIRA_ESCENARIO">Gira: Escenario</option>
            </select>
          </div>

          <div>
            <label for="class-division-select" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">División</label>
            <select id="class-division-select" bind:value={genderDivision} class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]">
              <option value="FEMALE">Mujeres</option>
              <option value="MALE">Varones</option>
              <option value="NOT_APPLICABLE">General Compañía</option>
            </select>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="class-start-input" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Hora Inicio</label>
            <input id="class-start-input" type="text" bind:value={startTime} placeholder="09:00" class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-mono font-bold text-ink] text-center outline-none focus:border-amber]" />
          </div>
          <div>
            <label for="class-end-input" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Hora Fin</label>
            <input id="class-end-input" type="text" bind:value={endTime} placeholder="10:15" class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-mono font-bold text-ink] text-center outline-none focus:border-amber]" />
          </div>
        </div>

        <div>
          <label for="class-master-select" class="block text-[10px] font-bold uppercase text-dim] mb-1 font-mono">Maestro/a a Cargo</label>
          <select id="class-master-select" bind:value={masterId} class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]">
            {#each masters as m}
              <option value={m.id}>{m.last_name}, {m.first_name}</option>
            {/each}
          </select>
        </div>

        <!-- Selector de Acompañamiento: Pianista vs Cabina TeatroCue -->
        <div class="bg-alt] p-2.5 rounded border border-grid]">
          <span class="block text-[10px] font-bold uppercase text-dim] mb-2 font-mono">Acompañamiento</span>
          <div class="grid grid-cols-2 gap-2 mb-2">
            <button
              type="button"
              onclick={() => accompanimentType = 'pianist'}
              class="h-8 px-2.5 rounded text-xs font-bold border transition-colors {accompanimentType === 'pianist' ? 'bg-raised] text-amber] border-amber]' : 'bg-raised] text-dim] border-grid]'}"
            >
              Pianista
            </button>
            <button
              type="button"
              onclick={() => accompanimentType = 'teatrocue'}
              class="h-8 px-2.5 rounded text-xs font-bold border transition-colors {accompanimentType === 'teatrocue' ? 'bg-void] text-cyan] border-cyan]' : 'bg-raised] text-dim] border-grid]'}"
            >
              Pista TeatroCue
            </button>
          </div>

          <div class="mb-2">
            <label for="class-soundtech-select" class="block text-[10px] font-bold uppercase text-cyan] mb-1 font-mono">Sonidista a cargo</label>
            <select id="class-soundtech-select" bind:value={soundTechId} class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]">
              <option value="">Sin asignar</option>
              {#each soundTechs as t}
                <option value={t.id}>{t.last_name}, {t.first_name}</option>
              {/each}
            </select>
          </div>

          {#if accompanimentType === 'pianist'}
            <div>
              <label for="class-pianist-select" class="block text-[10px] font-bold uppercase text-amber] mb-1 font-mono">Pianista Acompañante</label>
              <select id="class-pianist-select" bind:value={pianistId} class="w-full h-8 px-2.5 bg-void] border border-grid] rounded text-xs font-bold text-ink] outline-none focus:border-amber]">
                {#each pianists as p}
                  <option value={p.id}>{p.last_name}, {p.first_name}</option>
                {/each}
              </select>
            </div>
          {:else}
            <div class="text-[11px] text-cyan] font-mono py-1">
              [AUDIO] Reproducción de cabina TeatroCue / Pistas técnicas
            </div>
          {/if}
        </div>
      </div>

      <div class="tg-modal-foot">
        <button onclick={() => showModal = false} class="tg-btn">
          Cancelar
        </button>
        <button
          disabled={isSaving}
          onclick={saveClass}
          class="tg-btn tg-btn-primary disabled:opacity-50"
        >
          {isSaving ? 'Guardando...' : 'Guardar Clase'}
        </button>
      </div>
    </div>
  </div>
{/if}
