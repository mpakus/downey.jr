import { mount } from 'svelte'
import App from './App.svelte'
import './chunks.css'
import './styles/app.css'
import './styles/tokens/tokens.css'
import './styles/syntax.css'
import './styles/editor.css'
import './styles/gfm.css'

mount(App, {
  target: document.getElementById('app')!,
})
