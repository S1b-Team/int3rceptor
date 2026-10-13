import { createApp } from 'vue';
import App from './App.vue';
import './styles/tokens.css';
import { initSentry } from './telemetry/sentry';

initSentry();
// Sentry.captureException(new Error("Sentry UI test"));

createApp(App).mount('#app');
