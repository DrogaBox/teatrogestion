import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('Elemento raíz #app no encontrado en el DOM');
}

const app = mount(App, { target });

export default app;

// Botones de cierre (✕) más grandes para el dedo en el celular.
{
  const mark = () => {
    document.querySelectorAll<HTMLButtonElement>('button:not(.tg-close)').forEach((b) => {
      const t = (b.textContent ?? '').trim();
      if (t === '✕' || t === '×' || t === '✖') b.classList.add('tg-close');
    });
  };
  new MutationObserver(mark).observe(document.body, { childList: true, subtree: true });
  mark();
}
