// Imprimir y guardar archivos. En la PC alcanza con el navegador; el visor web de Android no imprime
// ni descarga, así que en el celular lo hace el puente nativo de MainActivity.kt (`TGOutput`):
// abre el diálogo de impresión del sistema (que también ofrece «Guardar como PDF») y el menú de
// compartir de Android (guardar en Archivos o Drive, mandar por WhatsApp o correo).
interface OutputBridge {
  print(jobName: string): void;
  saveFile(fileName: string, mimeType: string, base64: string): string;
}

const bridge = (): OutputBridge | undefined =>
  typeof window !== 'undefined' ? (window as unknown as { TGOutput?: OutputBridge }).TGOutput : undefined;

export function printDocument(jobName = 'TeatroGestion') {
  const native = bridge();
  if (native) native.print(jobName);
  else window.print();
}

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve((reader.result as string).split(',')[1] ?? '');
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}

/// Guarda un archivo generado por la app (planillas CSV, respaldos, documentos de la Bóveda).
export async function saveBlob(blob: Blob, fileName: string) {
  const native = bridge();
  if (native) {
    const result = native.saveFile(fileName, blob.type || 'application/octet-stream', await blobToBase64(blob));
    if (result !== 'ok') throw new Error(`No se pudo guardar ${fileName}: ${result}`);
    return;
  }
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

/// Lo mismo para un archivo guardado como `data:` (los adjuntos de la Bóveda y los certificados).
export async function saveDataUrl(dataUrl: string, fileName: string) {
  await saveBlob(await (await fetch(dataUrl)).blob(), fileName);
}
