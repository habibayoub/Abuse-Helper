import React from 'react';
import './index.css';
import App from './App';
import * as serviceWorker from './serviceWorker';
import { NextUIProvider } from '@nextui-org/react';
import { createRoot } from 'react-dom/client';

const root = createRoot(document.getElementById("root")!);
root.render(
  <React.StrictMode>
    <NextUIProvider>
      <main className="dark text-foreground bg-background">
        <App />
      </main>
    </NextUIProvider>
  </React.StrictMode>,
);

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
