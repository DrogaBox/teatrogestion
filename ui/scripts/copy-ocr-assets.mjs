// Copia a public/ocr el motor de lectura de fotos y su idioma: la app no usa internet, así que todo viaja adentro.
import { copyFileSync, mkdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ui = join(dirname(fileURLToPath(import.meta.url)), '..');
const target = join(ui, 'public', 'ocr');
const require = createRequire(join(ui, 'package.json'));
const tesseract = dirname(require.resolve('tesseract.js/package.json'));
const core = dirname(createRequire(join(tesseract, 'package.json')).resolve('tesseract.js-core/package.json'));
const spanish = dirname(require.resolve('@tesseract.js-data/spa/package.json'));

mkdirSync(target, { recursive: true });
for (const [from, name] of [
  [join(tesseract, 'dist', 'worker.min.js'), 'worker.min.js'],
  [join(core, 'tesseract-core-lstm.wasm.js'), 'tesseract-core-lstm.wasm.js'],
  [join(core, 'tesseract-core-simd-lstm.wasm.js'), 'tesseract-core-simd-lstm.wasm.js'],
  [join(spanish, '4.0.0_best_int', 'spa.traineddata.gz'), 'spa.traineddata.gz'],
]) {
  copyFileSync(from, join(target, name));
}
