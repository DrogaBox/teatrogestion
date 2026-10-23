<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    BrowserMultiFormatReader,
    DecodeHintType,
    BarcodeFormat,
  } from '@zxing/library';
  import {
    api,
    type StockItemAvailabilityDto,
    type UpsertStockItemPayload
  } from '../../ipc/client';

  let {
    isOpen,
    onClose,
    onDeliverItem,
    onNewProductWithBarcode
  } = $props<{
    isOpen: boolean;
    onClose: () => void;
    onDeliverItem?: (item: StockItemAvailabilityDto) => void;
    onNewProductWithBarcode?: (barcode: string) => void;
  }>();

  let videoElement = $state<HTMLVideoElement | null>(null);
  let currentStream: MediaStream | null = null;
  let currentTrack: MediaStreamTrack | null = null;

  let videoDevices = $state<MediaDeviceInfo[]>([]);
  let selectedDeviceId = $state<string>('');
  let isScanning = $state(false);
  let hasCameraError = $state(false);
  let cameraErrorMessage = $state('');

  // Capacidades del hardware
  let hasTorch = $state(false);
  let isTorchOn = $state(false);
  let isDetecting = $state(false);
  let scanSuccessFlash = $state(false);

  // Estados de escaneo y datos
  let manualCode = $state('');
  let scannedBarcode = $state('');
  let matchedItem = $state<StockItemAvailabilityDto | null>(null);
  let isSearching = $state(false);
  let copiedToast = $state(false);

  // Cantidad para sumar stock rápido
  let addQty = $state(5);
  let isUpdatingStock = $state(false);
  let stockUpdatedSuccess = $state(false);

  // Motores de decodificación
  let nativeDetector: any = null;
  let zxingReader: BrowserMultiFormatReader | null = null;
  let scanInterval: any = null;
  let isLoopActive = false;
  let offscreenCanvas: HTMLCanvasElement | null = null;
  let offscreenCtx: CanvasRenderingContext2D | null = null;

  function initDetectors() {
    // 1. BarcodeDetector nativo de Chromium / Android (acelerado por hardware MLKit)
    if (typeof window !== 'undefined' && 'BarcodeDetector' in window) {
      try {
        const formats = [
          'ean_13',
          'ean_8',
          'code_128',
          'code_39',
          'upc_a',
          'upc_e',
          'qr_code',
          'itf'
        ];
        nativeDetector = new (window as any).BarcodeDetector({ formats });
      } catch (e) {
        console.warn('BarcodeDetector nativo no disponible:', e);
        nativeDetector = null;
      }
    }

    // 2. ZXing MultiFormatReader con TRY_HARDER y formatos explícitos
    if (!zxingReader) {
      const hints = new Map();
      hints.set(DecodeHintType.TRY_HARDER, true);
      hints.set(DecodeHintType.POSSIBLE_FORMATS, [
        BarcodeFormat.EAN_13,
        BarcodeFormat.EAN_8,
        BarcodeFormat.CODE_128,
        BarcodeFormat.CODE_39,
        BarcodeFormat.UPC_A,
        BarcodeFormat.UPC_E,
        BarcodeFormat.QR_CODE,
        BarcodeFormat.ITF
      ]);
      zxingReader = new BrowserMultiFormatReader(hints);
    }
  }

  async function startCamera() {
    stopCamera();
    hasCameraError = false;
    cameraErrorMessage = '';
    isScanning = true;

    try {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        throw new Error('La interfaz de cámara no está disponible en este dispositivo.');
      }

      // Priorizar cámara trasera con resolución óptima y autofocus
      const videoConstraints: MediaTrackConstraints = selectedDeviceId
        ? {
            deviceId: { exact: selectedDeviceId },
            width: { ideal: 1920, min: 1280 },
            height: { ideal: 1080, min: 720 },
          }
        : {
            facingMode: { ideal: 'environment' },
            width: { ideal: 1920, min: 1280 },
            height: { ideal: 1080, min: 720 },
          };

      const stream = await navigator.mediaDevices.getUserMedia({
        video: videoConstraints,
        audio: false
      });

      currentStream = stream;
      const track = stream.getVideoTracks()[0];
      currentTrack = track;

      // Verificar capacidades (linterna / autofocus continuo)
      if (track && typeof track.getCapabilities === 'function') {
        const caps = track.getCapabilities() as any;
        hasTorch = !!caps?.torch;
        if (caps?.focusMode && caps.focusMode.includes('continuous')) {
          try {
            await track.applyConstraints({
              advanced: [{ focusMode: 'continuous' } as any]
            });
          } catch (e) {
            console.warn('Autofocus continuo no disponible:', e);
          }
        }
      } else {
        hasTorch = false;
      }
      isTorchOn = false;

      if (videoElement) {
        videoElement.srcObject = stream;
        await videoElement.play().catch(console.warn);
      }

      // Una vez concedido el permiso, consultar etiquetas de dispositivos
      await refreshDevicesList();

      // Iniciar bucle continuo y resiliente de escaneo
      startScanLoop();
    } catch (err: any) {
      console.error('Error al iniciar cámara:', err);
      hasCameraError = true;
      cameraErrorMessage = err?.message || 'No se pudo acceder a la cámara o el permiso fue denegado.';
      isScanning = false;
    }
  }

  async function refreshDevicesList() {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      videoDevices = devices.filter(d => d.kind === 'videoinput');
      if (!selectedDeviceId && videoDevices.length > 0) {
        const back = videoDevices.find(d =>
          d.label.toLowerCase().includes('back') ||
          d.label.toLowerCase().includes('trasera') ||
          d.label.toLowerCase().includes('rear') ||
          d.label.toLowerCase().includes('environment')
        );
        if (back) {
          selectedDeviceId = back.deviceId;
        }
      }
    } catch (e) {
      console.warn('Error al enumerar dispositivos:', e);
    }
  }

  async function toggleTorch() {
    if (!currentTrack || !hasTorch) return;
    try {
      const next = !isTorchOn;
      await currentTrack.applyConstraints({
        advanced: [{ torch: next } as any]
      });
      isTorchOn = next;
    } catch (e) {
      console.warn('Error cambiando linterna:', e);
    }
  }

  function stopCamera() {
    isLoopActive = false;
    if (scanInterval) {
      clearInterval(scanInterval);
      scanInterval = null;
    }

    if (currentTrack) {
      if (isTorchOn) {
        try {
          currentTrack.applyConstraints({ advanced: [{ torch: false } as any] });
        } catch (e) {}
      }
      currentTrack.stop();
      currentTrack = null;
    }

    if (currentStream) {
      currentStream.getTracks().forEach(t => t.stop());
      currentStream = null;
    }

    if (videoElement) {
      videoElement.srcObject = null;
    }

    isTorchOn = false;
    hasTorch = false;
    isScanning = false;
  }

  async function handleDeviceChange() {
    await startCamera();
  }

  function startScanLoop() {
    isLoopActive = true;
    if (scanInterval) clearInterval(scanInterval);

    scanInterval = setInterval(async () => {
      if (!isLoopActive || !videoElement || scannedBarcode) return;
      if (videoElement.readyState < 2 || videoElement.videoWidth === 0 || videoElement.videoHeight === 0) {
        return;
      }

      try {
        const code = await scanCurrentVideoFrame();
        if (code && code !== scannedBarcode) {
          handleBarcodeScanned(code);
        }
      } catch (e) {
        // Ignorar fallos de cuadro no decodificado
      }
    }, 125); // ~8 veces por segundo
  }

  async function scanCurrentVideoFrame(): Promise<string | null> {
    if (!videoElement || videoElement.videoWidth === 0) return null;

    // 1. Probar BarcodeDetector nativo de Chromium/Android
    if (nativeDetector) {
      try {
        const barcodes = await nativeDetector.detect(videoElement);
        if (barcodes && barcodes.length > 0) {
          const raw = barcodes[0].rawValue?.trim();
          if (raw) return raw;
        }
      } catch (e) {
        // Fallback a ZXing
      }
    }

    // 2. Probar ZXing en cuadro completo
    if (!zxingReader) return null;

    try {
      const result = zxingReader.decode(videoElement);
      if (result) {
        const text = result.getText()?.trim();
        if (text) return text;
      }
    } catch (e) {
      // NotFoundException habitual, continuar con recorte central
    }

    // 3. Probar ZXing en el recorte central (zoom óptico digital sobre la retícula)
    try {
      const vw = videoElement.videoWidth;
      const vh = videoElement.videoHeight;
      if (vw > 200 && vh > 200) {
        if (!offscreenCanvas) {
          offscreenCanvas = document.createElement('canvas');
        }
        const cropW = Math.floor(vw * 0.85);
        const cropH = Math.floor(vh * 0.55);
        const cropX = Math.floor((vw - cropW) / 2);
        const cropY = Math.floor((vh - cropH) / 2);

        offscreenCanvas.width = cropW;
        offscreenCanvas.height = cropH;
        if (!offscreenCtx) {
          offscreenCtx = offscreenCanvas.getContext('2d', { willReadFrequently: true });
        }
        if (offscreenCtx) {
          offscreenCtx.drawImage(videoElement, cropX, cropY, cropW, cropH, 0, 0, cropW, cropH);
          const result = zxingReader.decode(offscreenCanvas);
          if (result) {
            const text = result.getText()?.trim();
            if (text) return text;
          }
        }
      }
    } catch (e) {
      // Continuar
    }

    return null;
  }

  async function captureAndScanNow() {
    if (!videoElement || videoElement.videoWidth === 0) return;
    isDetecting = true;
    try {
      const code = await scanCurrentVideoFrame();
      if (code) {
        handleBarcodeScanned(code);
      } else {
        alert('No se detectó un código legible en este cuadro. Centra el código dentro de la retícula o usa el botón "Tomar foto con cámara del sistema".');
      }
    } finally {
      isDetecting = false;
    }
  }

  // Escaneo directo desde foto capturada o archivo de imagen
  async function handleFileUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const file = input.files[0];
    const imgUrl = URL.createObjectURL(file);

    try {
      initDetectors();
      if (!zxingReader) {
        const hints = new Map();
        hints.set(DecodeHintType.TRY_HARDER, true);
        zxingReader = new BrowserMultiFormatReader(hints);
      }
      const result = await zxingReader.decodeFromImageUrl(imgUrl);
      if (result) {
        handleBarcodeScanned(result.getText().trim());
      } else {
        alert('No se detectó ningún código de barras válido en la imagen seleccionada.');
      }
    } catch (err: any) {
      alert('No se detectó ningún código de barras válido en la foto tomada.');
    } finally {
      URL.revokeObjectURL(imgUrl);
    }
  }

  function triggerSuccessFeedback() {
    scanSuccessFlash = true;
    setTimeout(() => (scanSuccessFlash = false), 600);

    // Vibración háptica
    if (typeof navigator !== 'undefined' && typeof navigator.vibrate === 'function') {
      navigator.vibrate([80, 40, 80]);
    }

    // Beep audible idéntico al escáner de caja de supermercado (Zebra / Motorola Symbol / Datalogic)
    try {
      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      if (AudioCtx) {
        const ctx = new AudioCtx();
        if (ctx.state === 'suspended') {
          ctx.resume().catch(() => {});
        }
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        // 2093 Hz (Do 7 / C7 brillante y nítido de escáner láser de supermercado)
        osc.type = 'sine';
        osc.frequency.setValueAtTime(2093, ctx.currentTime);
        gain.gain.setValueAtTime(0.35, ctx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.075);
        osc.connect(gain);
        gain.connect(ctx.destination);
        osc.start(ctx.currentTime);
        osc.stop(ctx.currentTime + 0.08);
      }
    } catch (e) {
      // Ignorar restricciones de audio del sistema
    }
  }

  async function handleBarcodeScanned(code: string) {
    scannedBarcode = code;
    isSearching = true;
    matchedItem = null;
    stockUpdatedSuccess = false;

    triggerSuccessFeedback();

    try {
      const results = await api.listStockCatalog({ query: code });
      if (results.length > 0) {
        matchedItem = results[0];
      }
    } catch (e) {
      console.error('Error buscando código en catálogo:', e);
    } finally {
      isSearching = false;
    }
  }

  function handleManualSubmit(e: Event) {
    e.preventDefault();
    if (manualCode.trim()) {
      handleBarcodeScanned(manualCode.trim());
    }
  }

  async function addPhysicalStock(delta: number) {
    if (!matchedItem) return;
    isUpdatingStock = true;
    try {
      const newPhysical = Math.max(0, Number(matchedItem.physical_qty) + delta);
      const payload: UpsertStockItemPayload = {
        id: matchedItem.id,
        sku_code: matchedItem.sku_code,
        family: matchedItem.family,
        subcategory: matchedItem.subcategory,
        quota_bucket: 'EXEMPT',
        brand: matchedItem.brand,
        model_name: matchedItem.model_name,
        raw_factory_code: matchedItem.raw_factory_code,
        color: matchedItem.color,
        material: matchedItem.material,
        size_system: matchedItem.size_system,
        size_value: matchedItem.size_value,
        width_value: matchedItem.width_value,
        physical_qty: newPhysical,
        min_alert_qty: 2,
        storage_bin_location: matchedItem.storage_bin_location,
      };
      const updated = await api.upsertStockItem(payload);
      matchedItem = updated;
      stockUpdatedSuccess = true;
      setTimeout(() => (stockUpdatedSuccess = false), 3000);
    } catch (e: any) {
      alert(`Error sumando stock: ${e?.message || e}`);
    } finally {
      isUpdatingStock = false;
    }
  }

  function copySupplierOrder() {
    if (!matchedItem) return;
    const text = `PEDIDO DE REPOSICIÓN DE STOCK - BALLET OFICIAL DE CÓRDOBA
- Institución: Teatro del Libertador San Martín
- Marca / Proveedor: ${matchedItem.brand}
- Modelo: ${matchedItem.model_name}
- Código de Fábrica / SKU: ${matchedItem.raw_factory_code || matchedItem.sku_code}
- Código de Barras: ${scannedBarcode}
- Color / Material: ${matchedItem.color} - ${matchedItem.material}
- Talle / Medida: ${matchedItem.size_value} ${matchedItem.width_value ? `/${matchedItem.width_value}` : ''} (${matchedItem.size_system})
- Cantidad Solicitada: ${addQty} pares / unidades
• Destino: Depósito de Danza - Teatro del Libertador`;

    navigator.clipboard.writeText(text).then(() => {
      copiedToast = true;
      setTimeout(() => (copiedToast = false), 3000);
    });
  }

  $effect(() => {
    if (isOpen) {
      initDetectors();
      startCamera();
    } else {
      stopCamera();
      scannedBarcode = '';
      matchedItem = null;
    }
  });

  onDestroy(() => {
    stopCamera();
  });
</script>

{#if isOpen}
  <div
    role="presentation"
    class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-3 sm:p-4"
    onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
  >
    <div class="bg-[#1a1a1e] border border-[#34343c] rounded-md w-full max-w-lg max-h-[92vh] flex flex-col shadow-2xl text-xs overflow-hidden">
      
      <!-- Encabezado Modal (shrink-0) -->
      <div class="p-3.5 border-b border-[#34343c] flex items-center justify-between shrink-0 bg-[#17171b]">
        <div>
          <h2 class="text-sm font-bold text-[#d8d8dc] uppercase tracking-wide">
            Lector Óptico de Código de Barras
          </h2>
          <p class="text-[11px] text-[#82828a]">Decodificación en tiempo real (EAN-13, Code-128, QR) & Pedidos</p>
        </div>
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); onClose(); }}
          class="w-8 h-8 rounded bg-[#24242a] hover:bg-[#34343e] text-[#82828a] hover:text-white flex items-center justify-center font-mono text-base"
        >
          ✕
        </button>
      </div>

      <!-- Cuerpo del Modal -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        
        <!-- Visor de Cámara con Retícula TeatroCue -->
        <div class="relative bg-black rounded border {scanSuccessFlash ? 'border-[#3de06a] ring-2 ring-[#3de06a]' : 'border-[#34343c]'} aspect-video flex items-center justify-center overflow-hidden transition-all duration-300">
          <video
            bind:this={videoElement}
            autoplay
            playsinline
            muted
            class="w-full h-full object-cover"
          ></video>

          {#if hasCameraError}
            <div class="absolute inset-0 bg-[#121214]/95 flex flex-col items-center justify-center p-4 text-center space-y-2">
              <span class="text-xs text-[#ff4a4a] font-bold">{cameraErrorMessage}</span>
              <p class="text-[11px] text-[#82828a]">Puedes tomar una foto directamente con el botón de abajo o ingresar el código a mano.</p>
            </div>
          {:else}
            <!-- Retícula de Escaneo Láser TeatroCue -->
            <div class="absolute inset-x-8 inset-y-6 border border-[#ffb31a]/70 rounded pointer-events-none flex flex-col justify-between p-2">
              <div class="flex justify-between text-[10px] text-[#ffb31a] font-mono">
                <span>[SCANNER ACTIVO]</span>
                <span>EAN/SKU</span>
              </div>
              <div class="w-full h-0.5 bg-[#ffb31a] animate-pulse shadow-[0_0_8px_#ffb31a]"></div>
              <div class="text-center text-[10px] text-[#ffb31a] font-mono">
                Alinea el código dentro del marco
              </div>
            </div>

            <!-- Botón flotante de Linterna si el sensor lo permite -->
            {#if hasTorch}
              <button
                type="button"
                onclick={toggleTorch}
                class="absolute top-2 right-2 px-2 py-1 rounded text-[10px] font-mono font-bold transition-colors shadow-md {isTorchOn ? 'bg-[#ffb31a] text-[#121214]' : 'bg-[#24242a]/80 text-[#d8d8dc] border border-[#34343c]'}"
                title="Encender o apagar flash"
              >
                {isTorchOn ? '[LINTERNA: ON]' : '[LINTERNA: OFF]'}
              </button>
            {/if}
          {/if}
        </div>

        <!-- Botones de Acción Inmediata: Capturar Cuadro o Tomar Foto con Cámara Nativa -->
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            disabled={isDetecting || hasCameraError}
            onclick={captureAndScanNow}
            class="py-2 px-2 bg-[#24242a] hover:bg-[#34343e] text-[#ffb31a] border border-[#34343c] rounded text-xs font-mono font-bold transition-colors flex items-center justify-center gap-1.5"
            title="Fuerza el escaneo inmediato en máxima resolución del cuadro en pantalla"
          >
            <span>{isDetecting ? 'Analizando...' : 'Escanear Cuadro'}</span>
          </button>

          <!-- Disparador Directo de Cámara Nativa con capture="environment" -->
          <label class="py-2 px-2 bg-[#24242a] hover:bg-[#34343e] text-[#2ed6e8] border border-[#34343c] rounded text-xs font-mono font-bold transition-colors flex items-center justify-center gap-1.5 cursor-pointer text-center">
            <span>Tomar Foto Directa</span>
            <input
              type="file"
              accept="image/*"
              capture="environment"
              onchange={handleFileUpload}
              class="hidden"
            />
          </label>
        </div>

        <!-- Selector de Cámara si hay más de 1 dispositivo -->
        {#if videoDevices.length > 1}
          <div class="flex items-center gap-2">
            <span class="text-[10px] font-mono uppercase text-[#82828a] shrink-0">Cámara:</span>
            <select
              bind:value={selectedDeviceId}
              onchange={handleDeviceChange}
              class="flex-1 h-8 px-2 bg-[#141416] border border-[#34343c] rounded text-xs text-[#d8d8dc]"
            >
              {#each videoDevices as d}
                <option value={d.deviceId}>{d.label || `Cámara ${d.deviceId.slice(0, 5)}`}</option>
              {/each}
            </select>
          </div>
        {/if}

        <!-- Ingreso Manual de Respaldo -->
        <div class="space-y-1 pt-1">
          <form onsubmit={handleManualSubmit} class="flex items-center gap-2">
            <input
              type="text"
              bind:value={manualCode}
              placeholder="Escribe código (ej: 7790001001, GM-EUR...)"
              class="flex-1 h-9 px-3 bg-[#141416] border border-[#34343c] rounded text-xs font-mono font-bold text-[#d8d8dc] outline-none focus:border-[#ffb31a]"
            />
            <button
              type="submit"
              class="h-9 px-3 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] font-bold text-xs rounded border border-[#34343c]"
            >
              Buscar
            </button>
          </form>
        </div>

        <!-- RESULTADO DEL ESCANEO -->
        {#if isSearching}
          <div class="p-4 text-center text-[#82828a] font-mono text-xs">
            Buscando en base de datos SQLite WAL local...
          </div>
        {:else if scannedBarcode}
          {#if matchedItem}
            <!-- Ítem Encontrado -->
            <div class="bg-[#141416] border border-[#3de06a] rounded p-3 space-y-2.5">
              <div class="flex items-start justify-between gap-2">
                <div>
                  <span class="px-1.5 py-0.5 rounded text-[9px] font-mono font-bold uppercase bg-[#1e2a22] text-[#3de06a] border border-[#3de06a]/40">
                    Ítem Registrado en Depósito
                  </span>
                  <h3 class="text-sm font-bold text-[#d8d8dc] mt-1">{matchedItem.brand} {matchedItem.model_name}</h3>
                  <div class="text-xs text-[#ffb31a] font-mono">{matchedItem.raw_factory_code || matchedItem.sku_code}</div>
                </div>

                <div class="text-right font-mono">
                  <div class="text-[9px] text-[#82828a] uppercase font-bold">Stock Físico</div>
                  <div class="text-lg font-black text-[#d8d8dc]">{matchedItem.physical_qty}</div>
                  <div class="text-[10px] text-[#3de06a]">{matchedItem.free_available_qty} libres</div>
                </div>
              </div>

              <!-- Ficha Técnica Rápida -->
              <div class="grid grid-cols-3 gap-2 bg-[#1a1a1e] p-2 rounded border border-[#34343c] text-[11px]">
                <div>
                  <span class="text-[#82828a] block text-[9px] uppercase font-mono">Talle / Ancho</span>
                  <span class="font-bold text-[#d8d8dc] font-mono">{matchedItem.size_value} {matchedItem.width_value || ''}</span>
                </div>
                <div>
                  <span class="text-[#82828a] block text-[9px] uppercase font-mono">Color</span>
                  <span class="text-[#d8d8dc]">{matchedItem.color}</span>
                </div>
                <div>
                  <span class="text-[#82828a] block text-[9px] uppercase font-mono">Ubicación</span>
                  <span class="text-[#d8d8dc] font-mono truncate">{matchedItem.storage_bin_location || '—'}</span>
                </div>
              </div>

              {#if stockUpdatedSuccess}
                <div class="p-2 rounded bg-[#1e2a22] border border-[#3de06a] text-[#3de06a] text-xs text-center font-mono">
                  ✓ Stock físico actualizado correctamente en SQLite WAL.
                </div>
              {/if}

              <!-- ACCIONES INMEDIATAS -->
              <div class="space-y-1.5 pt-1 border-t border-[#34343c]">
                {#if onDeliverItem}
                  <button
                    onclick={() => {
                      if (matchedItem) onDeliverItem(matchedItem);
                      onClose();
                    }}
                    class="w-full py-2 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] rounded font-bold text-xs uppercase"
                  >
                    Entregar a Bailarín / Personal
                  </button>
                {/if}

                <div class="flex items-center gap-1.5">
                  <span class="text-xs text-[#82828a] font-mono shrink-0">Sumar Stock:</span>
                  <button
                    disabled={isUpdatingStock}
                    onclick={() => addPhysicalStock(1)}
                    class="flex-1 py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] rounded text-xs font-mono border border-[#34343c]"
                  >
                    +1
                  </button>
                  <button
                    disabled={isUpdatingStock}
                    onclick={() => addPhysicalStock(5)}
                    class="flex-1 py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] rounded text-xs font-mono border border-[#34343c]"
                  >
                    +5
                  </button>
                  <button
                    disabled={isUpdatingStock}
                    onclick={() => addPhysicalStock(10)}
                    class="flex-1 py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] rounded text-xs font-mono border border-[#34343c]"
                  >
                    +10
                  </button>
                </div>

                <button
                  onclick={copySupplierOrder}
                  class="w-full py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#2ed6e8] border border-[#34343c] rounded text-xs font-bold"
                >
                  Copiar Pedido al Proveedor (WhatsApp / Email)
                </button>

                {#if copiedToast}
                  <div class="p-2 rounded bg-[#18262a] border border-[#2ed6e8] text-[#2ed6e8] text-xs text-center font-mono">
                    ✓ Ficha de pedido copiada al portapapeles.
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <!-- Ítem NO Encontrado -->
            <div class="bg-[#141416] border border-[#ffb31a]/40 rounded p-4 text-center space-y-2">
              <h4 class="text-xs font-bold text-[#d8d8dc]">Código no registrado en el depósito</h4>
              <div class="text-xs font-mono text-[#ffb31a]">{scannedBarcode}</div>
              <p class="text-[11px] text-[#82828a]">
                Este código no coincide con ningún producto existente en el catálogo local.
              </p>
              {#if onNewProductWithBarcode}
                <button
                  onclick={() => {
                    onNewProductWithBarcode(scannedBarcode);
                    onClose();
                  }}
                  class="px-4 py-2 bg-[#ffb31a] hover:bg-[#ffb31a]/90 text-[#121214] font-bold rounded text-xs uppercase"
                >
                  + Dar de Alta Producto con este Código
                </button>
              {/if}
            </div>
          {/if}
        {/if}

      </div>

      <!-- Pie del Modal (shrink-0) -->
      <div class="p-3 border-t border-[#34343c] flex justify-end shrink-0 bg-[#17171b]">
        <button
          onclick={onClose}
          class="px-4 py-1.5 bg-[#24242a] hover:bg-[#34343e] text-[#d8d8dc] text-xs font-bold rounded border border-[#34343c]"
        >
          Cerrar
        </button>
      </div>

    </div>
  </div>
{/if}
