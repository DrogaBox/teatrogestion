// Importar con una foto: lee el texto de la imagen sin internet (Tesseract, en public/ocr) y reconoce qué documento es:
// tipo, fecha, título, persona e importe. Solo propone: quien importa lo revisa antes de guardar.
export type VaultCategory =
  | 'RECIBO_COMPRA'
  | 'CERTIFICADO_MEDICO'
  | 'MEMO_DISPOSICION'
  | 'ACUERDO_DIRECCION'
  | 'SOLISTICO_FIRMA'
  | 'PARTE'
  | 'INFORME'
  | 'OTRO';

export interface DetectedDocument {
  category: VaultCategory;
  /** Nombre del tipo para el título, «Parte», «Informe»… */
  label: string;
  /** La lectura encontró señales claras del tipo (si no, queda como «Otro»). */
  confident: boolean;
  date: string | null;
  title: string | null;
  personnelId: string | null;
  amount: number | null;
}

interface Person {
  id: string;
  first_name: string;
  last_name: string;
}

const LABELS: Record<VaultCategory, string> = {
  RECIBO_COMPRA: 'Recibo',
  CERTIFICADO_MEDICO: 'Certificado médico',
  MEMO_DISPOSICION: 'Memo',
  ACUERDO_DIRECCION: 'Acuerdo',
  SOLISTICO_FIRMA: 'Solístico',
  PARTE: 'Parte',
  INFORME: 'Informe',
  OTRO: 'Documento',
};

/// Señales de cada tipo, sin acentos y en mayúsculas, con su peso: las de nuestros formatos pesan más.
const SIGNALS: [VaultCategory, string, number][] = [
  ['PARTE', 'PARTE OFICIAL', 6],
  ['PARTE', 'PARTE DIARIO', 6],
  ['PARTE', 'PARTE DE ENSAYO', 6],
  ['PARTE', 'PARTE DE CLASES', 6],
  ['PARTE', 'CLASES TECNICAS', 4],
  ['PARTE', 'CITACIONES', 3],
  ['PARTE', 'REPARTO DE ESTUDIO', 4],
  ['PARTE', 'MARCADA OBLIGATORIA', 3],
  ['PARTE', 'PLAN SEMANAL', 3],
  ['INFORME', 'INFORME', 5],
  ['INFORME', 'ELEVO', 1],
  ['CERTIFICADO_MEDICO', 'CERTIFICADO MEDICO', 6],
  ['CERTIFICADO_MEDICO', 'CERTIFICO', 3],
  ['CERTIFICADO_MEDICO', 'DIAGNOSTICO', 3],
  ['CERTIFICADO_MEDICO', 'REPOSO', 3],
  ['CERTIFICADO_MEDICO', 'MATRICULA', 2],
  ['CERTIFICADO_MEDICO', 'KINESIO', 1],
  ['RECIBO_COMPRA', 'FACTURA', 5],
  ['RECIBO_COMPRA', 'TICKET', 4],
  ['RECIBO_COMPRA', 'RECIBO', 4],
  ['RECIBO_COMPRA', 'CUIT', 3],
  ['RECIBO_COMPRA', 'IVA', 1],
  ['RECIBO_COMPRA', 'TOTAL', 1],
  ['SOLISTICO_FIRMA', 'SOLISTICO', 6],
  ['SOLISTICO_FIRMA', 'ME DIRIJO A UD', 3],
  ['MEMO_DISPOSICION', 'MEMORANDUM', 6],
  ['MEMO_DISPOSICION', 'MEMO', 3],
  ['MEMO_DISPOSICION', 'DISPOSICION', 5],
  ['MEMO_DISPOSICION', 'RESOLUCION', 5],
  ['ACUERDO_DIRECCION', 'ACUERDO', 5],
  ['ACUERDO_DIRECCION', 'FRANCO', 2],
];

const MONTHS = ['ENERO', 'FEBRERO', 'MARZO', 'ABRIL', 'MAYO', 'JUNIO', 'JULIO', 'AGOSTO', 'SEPTIEMBRE', 'OCTUBRE', 'NOVIEMBRE', 'DICIEMBRE'];

/// Encabezados institucionales de todos los papeles: no sirven de título.
const LETTERHEAD = ['TEATRO DEL LIBERTADOR', 'BALLET OFICIAL', 'AGENCIA CORDOBA', 'GOBIERNO DE'];

function normalize(text: string): string {
  return text.normalize('NFD').replace(/[̀-ͯ]/g, '').toUpperCase();
}

function hasWord(haystack: string, word: string): boolean {
  return new RegExp(`(^|[^A-Z])${word.replace(/\s+/g, '\\s+')}`).test(haystack);
}

function isoDate(year: number, month: number, day: number): string | null {
  if (year < 100) year += 2000;
  if (year < 2000 || year > 2100 || month < 1 || month > 12 || day < 1 || day > 31) return null;
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

function findDate(upper: string): string | null {
  const iso = upper.match(/\b(20\d{2})-(\d{1,2})-(\d{1,2})\b/);
  if (iso) return isoDate(+iso[1], +iso[2], +iso[3]);
  const written = upper.match(new RegExp(`\\b(\\d{1,2})\\s+DE\\s+(${MONTHS.join('|')})\\s+(?:DE|DEL)\\s+(\\d{4})\\b`));
  if (written) return isoDate(+written[3], MONTHS.indexOf(written[2]) + 1, +written[1]);
  const numeric = upper.match(/\b(\d{1,2})[/.-](\d{1,2})[/.-](\d{4}|\d{2})\b/);
  return numeric ? isoDate(+numeric[3], +numeric[2], +numeric[1]) : null;
}

function findAmount(upper: string): number | null {
  const found = upper.match(/\bTOTAL[^0-9\n]{0,15}(\d[\d.,]*)/);
  if (!found) return null;
  const value = Number(found[1].replace(/\.(?=\d{3}\b)/g, '').replace(',', '.').replace(/[.,]$/, ''));
  return Number.isFinite(value) && value > 0 ? value : null;
}

/// El título: de los renglones en mayúsculas del principio, el que nombra al documento; si no, el primero.
function findTitle(text: string, category: VaultCategory): string | null {
  const lines = text.split('\n').map((l) => l.replace(/\s+/g, ' ').trim()).slice(0, 15);
  const headings = lines.filter((line) => {
    const letters = line.replace(/[^\p{L}]/gu, '');
    if (letters.length < 6 || line.length > 90) return false;
    const upper = letters.replace(/[^\p{Lu}]/gu, '').length;
    return upper / letters.length >= 0.7 && !LETTERHEAD.some((h) => normalize(line).includes(h));
  });
  const signals = SIGNALS.filter(([c]) => c === category).map(([, signal]) => signal);
  const heading = headings.find((line) => signals.some((signal) => hasWord(normalize(line), signal))) ?? headings[0];
  return heading ? heading.replace(/^[^\p{L}\d]+|[^\p{L}\d)]+$/gu, '') : null;
}

function findPerson(upper: string, people: Person[]): string | null {
  const found = people.filter((p) => {
    const words = normalize(`${p.last_name} ${p.first_name}`).split(/\s+/).filter((w) => w.length > 2);
    return words.length >= 2 && words.every((w) => hasWord(upper, w));
  });
  return found.length === 1 ? found[0].id : null;
}

export function detectDocument(text: string, people: Person[] = []): DetectedDocument {
  const upper = normalize(text);
  const scores = new Map<VaultCategory, number>();
  for (const [category, signal, weight] of SIGNALS) {
    if (hasWord(upper, signal)) scores.set(category, (scores.get(category) ?? 0) + weight);
  }
  const [best, score] = [...scores].sort((a, b) => b[1] - a[1])[0] ?? ['OTRO' as VaultCategory, 0];
  const confident = score >= 3;
  const category = confident ? best : 'OTRO';
  return {
    category,
    label: LABELS[category],
    confident,
    date: findDate(upper),
    title: findTitle(text, category),
    // Un parte nombra a mucha gente: la persona solo cuenta en un papel individual.
    personnelId: category === 'PARTE' ? null : findPerson(upper, people),
    amount: category === 'RECIBO_COMPRA' ? findAmount(upper) : null,
  };
}

/// Lee el archivo como imagen, sin pasarse de `maxSide` de lado (la orientación de la cámara ya viene aplicada).
async function drawImage(file: Blob, maxSide: number, grayscale: boolean): Promise<HTMLCanvasElement> {
  const bitmap = await createImageBitmap(file);
  const scale = Math.min(1, maxSide / Math.max(bitmap.width, bitmap.height));
  const canvas = document.createElement('canvas');
  canvas.width = Math.round(bitmap.width * scale);
  canvas.height = Math.round(bitmap.height * scale);
  const context = canvas.getContext('2d')!;
  if (grayscale) context.filter = 'grayscale(1) contrast(1.25)';
  context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  bitmap.close();
  return canvas;
}

/// Una foto de la cámara pesa varios MB: se guarda achicada, de sobra para leerla e imprimirla.
export async function compressPhoto(file: File): Promise<File> {
  if (!file.type.startsWith('image/') || file.size < 1_000_000) return file;
  const canvas = await drawImage(file, 2000, false);
  const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/jpeg', 0.85));
  if (!blob || blob.size >= file.size) return file;
  return new File([blob], file.name.replace(/\.[^.]+$/, '') + '.jpg', { type: 'image/jpeg' });
}

export interface PhotoReader {
  read(file: Blob, onProgress: (fraction: number) => void): Promise<string>;
  close(): Promise<void>;
}

/// Prepara el lector (tarda unos segundos la primera vez). El motor se carga recién cuando hace falta.
export async function openPhotoReader(): Promise<PhotoReader> {
  const { createWorker } = await import('tesseract.js');
  const base = new URL('ocr/', document.baseURI).href;
  let report: (fraction: number) => void = () => {};
  const worker = await createWorker('spa', 1, {
    workerPath: `${base}worker.min.js`,
    corePath: base,
    langPath: base,
    gzip: true,
    cacheMethod: 'none',
    workerBlobURL: false,
    logger: (m: { status: string; progress: number }) => {
      if (m.status === 'recognizing text') report(m.progress);
    },
  });
  return {
    async read(file, onProgress) {
      report = onProgress;
      const canvas = await drawImage(file, 2600, true);
      const { data } = await worker.recognize(canvas, { rotateAuto: true });
      return data.text;
    },
    async close() {
      await worker.terminate();
    },
  };
}
