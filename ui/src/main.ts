import { mount } from 'svelte'
import App from './App.svelte'
import './chunks.css'

mount(App, {
  target: document.getElementById('app')!,
})
