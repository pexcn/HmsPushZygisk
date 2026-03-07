import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { enableEdgeToEdge } from 'kernelsu'
import './style.css'

enableEdgeToEdge(true);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
