<script lang="ts">
  import {
    api,
    type QuotaBucket,
    type StockItemAvailabilityDto,
    type UpsertStockItemPayload
  } from '../../ipc/client';
  import ModalHeader from '../shared/ModalHeader.svelte';
  import { modalBehavior } from '../../actions/modalBehavior';

  let {
    isOpen,
    initialItem = null,
    onClose,
    onSaved
  } = $props<{
    isOpen: boolean;
    initialItem?: StockItemAvailabilityDto | null;
    onClose: () => void;
    onSaved: (item: StockItemAvailabilityDto) => void;
  }>();

  let subcategory = $state('CALZA_MASCULINA');
  let family = $state('APPAREL');
  let quotaBucket = $state<QuotaBucket>('TIGHTS');
  let brand = $state('Capezio');
  let modelName = $state('');
  let rawFactoryCode = $state('');
  let color = $state('Negro');
  let material = $state('ALGODON_LYCRA');
  let sizeSystem = $state('ALPHA');
  let sizeValue = $state('M');
  let widthValue = $state('');
  let physicalQty = $state(10);
  let minAlertQty = $state(2);
  let storageBinLocation = $state('');
  let isSubmitting = $state(false);
  let errorMessage = $state<string | null>(null);

  const subcategoryPresets: Record<string, { family: string; quota: QuotaBucket; sizeSys: string; defaultMat: string }> = {
    'CALZA_MASCULINA': { family: 'APPAREL', quota: 'TIGHTS', sizeSys: 'ALPHA', defaultMat: 'ALGODON_LYCRA' },
    'SUSPENSOR_MASCULINO': { family: 'APPAREL', quota: 'SUPPORTER', sizeSys: 'ALPHA', defaultMat: 'ALGODON_LYCRA' },
    'CANCAN': { family: 'APPAREL', quota: 'TIGHTS', sizeSys: 'ALPHA', defaultMat: 'MICROFIBRA' },
    'MEDIAS_DANZA': { family: 'APPAREL', quota: 'TIGHTS', sizeSys: 'ALPHA', defaultMat: 'MICROFIBRA' },
    'MEDIA_PUNTA_CUERO': { family: 'WOMENS_FLAT', quota: 'FLAT', sizeSys: 'US', defaultMat: 'CUERO' },
    'MEDIA_PUNTA_TELA': { family: 'WOMENS_FLAT', quota: 'FLAT', sizeSys: 'US', defaultMat: 'TELA_ELASTICA' },
    'PUNTA_OTRAS': { family: 'POINTE', quota: 'POINTE', sizeSys: 'US', defaultMat: 'SATEN' },
    'ZAPATOS_JAZZ': { family: 'WOMENS_FLAT', quota: 'FLAT', sizeSys: 'US', defaultMat: 'CUERO' },
    'ZAPATOS_CARACTER': { family: 'WOMENS_FLAT', quota: 'FLAT', sizeSys: 'US', defaultMat: 'CUERO' },
    'BOMBACHA_ESCENA': { family: 'APPAREL', quota: 'BRIEFS', sizeSys: 'ALPHA', defaultMat: 'ALGODON_LYCRA' },
    'OTRO_INDUMENTARIA': { family: 'APPAREL', quota: 'EXEMPT', sizeSys: 'ALPHA', defaultMat: 'MICROFIBRA' },
  };

  function onSubcategoryChange(newSub: string) {
    subcategory = newSub;
    const preset = subcategoryPresets[newSub];
    if (preset) {
      family = preset.family;
      quotaBucket = preset.quota;
      sizeSystem = preset.sizeSys;
      material = preset.defaultMat;
    }
  }

  $effect(() => {
    if (initialItem) {
      subcategory = initialItem.subcategory;
      family = initialItem.family;
      brand = initialItem.brand;
      modelName = initialItem.model_name;
      rawFactoryCode = initialItem.raw_factory_code || '';
      color = initialItem.color;
      material = initialItem.material;
      sizeSystem = initialItem.size_system;
      sizeValue = initialItem.size_value;
      widthValue = initialItem.width_value || '';
      physicalQty = Number(initialItem.physical_qty) || 0;
      storageBinLocation = initialItem.storage_bin_location || '';
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!modelName.trim()) {
      errorMessage = 'Por favor ingresa el nombre del modelo o descripción.';
      return;
    }
    if (!sizeValue.trim()) {
      errorMessage = 'El talle es obligatorio.';
      return;
    }

    isSubmitting = true;
    errorMessage = null;

    try {
      const payload: UpsertStockItemPayload = {
        id: initialItem ? initialItem.id : null,
        sku_code: initialItem ? initialItem.sku_code : null,
        family,
        subcategory,
        quota_bucket: quotaBucket,
        brand: brand.trim(),
        model_name: modelName.trim(),
        raw_factory_code: rawFactoryCode.trim() || null,
        color: color.trim(),
        material: material.trim(),
        size_system: sizeSystem,
        size_value: sizeValue.trim(),
        width_value: widthValue.trim() || null,
        physical_qty: Math.max(0, physicalQty),
        min_alert_qty: Math.max(0, minAlertQty),
        storage_bin_location: storageBinLocation.trim() || null,
      };

      const saved = await api.upsertStockItem(payload);
      onSaved(saved);
      onClose();
    } catch (err: any) {
      console.error(err);
      errorMessage = err?.message || 'Error al guardar el ítem en la base local';
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-3 md:p-6 overflow-y-auto"
    use:modalBehavior={{ onclose: onClose, backdrop: false }}
  >
    <div class="tg-modal max-w-xl text-ink">

      <!-- Encabezado del Modal -->
      <ModalHeader
        title={initialItem ? 'Modificar Ítem de Inventario' : 'Alta de Nuevo Ítem de Inventario'}
        subtitle="Ballet Oficial de Córdoba • Depósito de Calzado e Indumentaria"
        onclose={onClose}
      />

      {#if errorMessage}
        <div class="mb-4 p-3 rounded bg-red/10 border border-red/40 text-red text-xs font-medium">
          {errorMessage}
        </div>
      {/if}

      <!-- Formulario de Alta / Edición -->
      <form onsubmit={handleSubmit} class="tg-modal-body space-y-4 text-xs">
        
        <!-- Rubro / Subcategoría -->
        <div>
          <label class="block text-dim font-semibold mb-1 uppercase tracking-wider text-[10px]" for="subcategory-select">
            Rubro / Subcategoría *
          </label>
          <select
            id="subcategory-select"
            value={subcategory}
            onchange={(e) => onSubcategoryChange((e.target as HTMLSelectElement).value)}
            class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded font-medium text-ink outline-none"
          >
            <option value="CALZA_MASCULINA">Calzas de Varón (Con/Sin Pie)</option>
            <option value="SUSPENSOR_MASCULINO">Suspensores de Varón (Dance Belts)</option>
            <option value="CANCAN">Cancán / Medias Mallas (Entero / Convertible / Red)</option>
            <option value="MEDIAS_DANZA">Medias de Danza / Soquetes</option>
            <option value="MEDIA_PUNTA_CUERO">Media Puntas de Cuero (Piel/Blanco/Negro)</option>
            <option value="MEDIA_PUNTA_TELA">Media Puntas de Tela Elastizada</option>
            <option value="ZAPATOS_JAZZ">Zapatos de Jazz (Suela Partida)</option>
            <option value="ZAPATOS_CARACTER">Zapatos de Carácter (Taco Cubano / Bajo)</option>
            <option value="PUNTA_OTRAS">Zapatillas de Punta (Otras Marcas)</option>
            <option value="BOMBACHA_ESCENA">Bombachas de Escena</option>
            <option value="OTRO_INDUMENTARIA">Otro Rubro / Indumentaria Escénica</option>
          </select>
        </div>

        <!-- Marca y Modelo -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="brand-input">Marca *</label>
            <input
              id="brand-input"
              type="text"
              bind:value={brand}
              placeholder="Capezio, Bloch, Sansha, So Danca..."
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink font-medium outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="model-input">Modelo / Descripción *</label>
            <input
              id="model-input"
              type="text"
              bind:value={modelName}
              placeholder="Ej: M9000, Pro 1C, Neo-Flex, Ultra Soft..."
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink font-medium outline-none"
              required
            />
          </div>
        </div>

        <!-- Color y Material -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="color-input">Color *</label>
            <input
              id="color-input"
              type="text"
              bind:value={color}
              placeholder="Blanco, Negro, Piel, Rosa, Gris, Marrón..."
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink font-medium outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="material-select">Material *</label>
            <select
              id="material-select"
              bind:value={material}
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded font-medium text-ink outline-none"
            >
              <option value="ALGODON_LYCRA">Algodón con Lycra</option>
              <option value="MICROFIBRA">Microfibra</option>
              <option value="CUERO">Cuero Genuino</option>
              <option value="TELA_ELASTICA">Tela Elástica (Canvas Stretch)</option>
              <option value="CANVAS">Lona / Canvas Clásica</option>
              <option value="SATEN">Satén Tradicional</option>
              <option value="SATEN_VEGAN_LYRA">Satén Vegano Lyra</option>
              <option value="RED_FISHNET">Red Profesional (Fishnet)</option>
              <option value="OTRO">Otro Material</option>
            </select>
          </div>
        </div>

        <!-- Sistema de Talle, Talle y Ancho -->
        <div class="grid grid-cols-3 gap-2">
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="size-system-select">Sistema</label>
            <select
              id="size-system-select"
              bind:value={sizeSystem}
              class="w-full h-10 px-2 bg-void border border-grid focus:border-amber rounded font-medium text-ink outline-none text-xs"
            >
              <option value="ALPHA">ALPHA (S, M, L)</option>
              <option value="US">US (Calzado USA)</option>
              <option value="EU">EU (35 a 45)</option>
              <option value="OTRO">Otro / Único</option>
            </select>
          </div>
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="size-value-input">Talle *</label>
            <input
              id="size-value-input"
              type="text"
              bind:value={sizeValue}
              placeholder="M, 8, 39, L/XL..."
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink font-mono outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="width-value-input">Ancho</label>
            <input
              id="width-value-input"
              type="text"
              bind:value={widthValue}
              placeholder="M, W, C, X..."
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink font-mono outline-none"
            />
          </div>
        </div>

        <!-- Cantidades y Ubicación -->
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-1">
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="physical-qty-input">Stock Físico</label>
            <input
              id="physical-qty-input"
              type="number"
              min="0"
              bind:value={physicalQty}
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-amber font-mono font-bold outline-none text-sm"
              required
            />
          </div>
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="min-alert-qty-input">Alerta Mínima</label>
            <input
              id="min-alert-qty-input"
              type="number"
              min="0"
              bind:value={minAlertQty}
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink font-mono outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-dim font-semibold mb-1 text-[10px] uppercase tracking-wider" for="storage-bin-input">Estante / Gaveta</label>
            <input
              id="storage-bin-input"
              type="text"
              bind:value={storageBinLocation}
              placeholder="Ej: Gaveta Calzas 01..."
              class="w-full h-10 px-3 bg-void border border-grid focus:border-amber rounded text-ink outline-none text-xs"
            />
          </div>
        </div>

        <!-- Botones de Acción -->
        <div class="tg-modal-foot">
          <button
            type="button"
            onclick={onClose}
            class="tg-btn"
          >
            Cancelar
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            class="tg-btn tg-btn-primary uppercase tracking-wider disabled:opacity-50"
          >
            {isSubmitting ? 'Guardando...' : 'Guardar en Depósito'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
