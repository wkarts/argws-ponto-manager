<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{
  name?: string;
  size?: number | string;
  strokeWidth?: number;
}>(), {
  name: "circle",
  size: 18,
  strokeWidth: 2,
});

const paths: Record<string, string> = {
  circle: "M12 12h.01",
  menu: "M4 6h16M4 12h16M4 18h16",
  search: "M21 21l-4.35-4.35M10.5 18a7.5 7.5 0 1 1 0-15 7.5 7.5 0 0 1 0 15Z",
  refresh: "M20 6v5h-5M4 18v-5h5M5.6 9A7 7 0 0 1 18 6.8L20 11M4 13l2 4.2A7 7 0 0 0 18.4 15",
  sun: "M12 4V2M12 22v-2M4.93 4.93 3.52 3.52M20.49 20.49l-1.41-1.41M4 12H2M22 12h-2M4.93 19.07l-1.41 1.41M20.49 3.51l-1.41 1.41M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z",
  moon: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79Z",
  bell: "M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9M13.73 21a2 2 0 0 1-3.46 0",
  user: "M20 21a8 8 0 1 0-16 0M12 13a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z",
  users: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75",
  home: "M3 11l9-8 9 8v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1v-9Z",
  building: "M4 21V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v16M3 21h18M8 7h4M8 11h4M8 15h4M18 9h1a1 1 0 0 1 1 1v11",
  department: "M4 7h16M4 12h16M4 17h16M6 7v10M12 7v10M18 7v10",
  briefcase: "M10 6V5a2 2 0 0 1 2-2h0a2 2 0 0 1 2 2v1M3 8h18v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8ZM3 13h18",
  target: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10ZM12 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z",
  customer: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM22 21v-2a4 4 0 0 0-3-3.87",
  supplier: "M3 7h13v10H3V7ZM16 10h3l2 3v4h-5v-7ZM7 20a2 2 0 1 0 0-4 2 2 0 0 0 0 4ZM18 20a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z",
  package: "M21 8l-9-5-9 5 9 5 9-5ZM3 8v8l9 5 9-5V8M12 13v8",
  shield: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z",
  lock: "M7 11V7a5 5 0 0 1 10 0v4M5 11h14v10H5V11Z",
  clipboard: "M9 4h6M9 4a3 3 0 0 0-3 3v14h12V7a3 3 0 0 0-3-3M9 4a3 3 0 0 1 6 0M9 12h6M9 16h6",
  settings: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06A1.65 1.65 0 0 0 15 19.4a1.65 1.65 0 0 0-1 .6l-.06.06a2 2 0 1 1-3.88 0L10 20a1.65 1.65 0 0 0-1-.6 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-.6-1l-.06-.06a2 2 0 1 1 0-3.88L4 10a1.65 1.65 0 0 0 .6-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6c.38 0 .74-.13 1-.36l.06-.06a2 2 0 1 1 3.88 0l.06.06c.26.23.62.36 1 .36.7 0 1.35-.26 1.82-.73l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06c-.47.47-.73 1.12-.73 1.82 0 .38.13.74.36 1l.06.06a2 2 0 1 1 0 3.88l-.06.06c-.23.26-.36.62-.36 1Z",
  database: "M4 6c0-1.66 3.58-3 8-3s8 1.34 8 3-3.58 3-8 3-8-1.34-8-3ZM4 6v6c0 1.66 3.58 3 8 3s8-1.34 8-3V6M4 12v6c0 1.66 3.58 3 8 3s8-1.34 8-3v-6",
  key: "M21 2l-2 2M15 8l4-4M7 14a5 5 0 1 1 3.54-8.54A5 5 0 0 1 7 14ZM7 14v3H4v3H1v3",
  activity: "M22 12h-4l-3 8L9 4l-3 8H2",
  api: "M8 9l-4 3 4 3M16 9l4 3-4 3M14 4l-4 16",
  docs: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6ZM14 2v6h6M8 13h8M8 17h8",
  webhook: "M7 8a5 5 0 0 1 8.66-3.42M17 8h4V4M17 16a5 5 0 0 1-8.66 3.42M7 16H3v4M12 8v8M8 12h8",
  websocket: "M4 8h4l2 8h4l2-8h4M4 16h4M16 16h4",
  plug: "M8 2v5M16 2v5M7 7h10v4a5 5 0 0 1-10 0V7ZM12 16v6",
  sync: "M21 12a9 9 0 0 1-15.5 6.3L3 16M3 16h6M3 16v6M3 12a9 9 0 0 1 15.5-6.3L21 8M21 8h-6M21 8V2",
  file: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6Z",
  info: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20ZM12 16v-4M12 8h.01",
  book: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20M4 19.5A2.5 2.5 0 0 0 6.5 22H20V4H6.5A2.5 2.5 0 0 0 4 6.5v13Z",
  chart: "M3 3v18h18M7 16v-5M12 16V8M17 16v-9",
  dashboard: "M4 4h6v7H4V4Zm10 0h6v4h-6V4Zm0 8h6v8h-6v-8ZM4 15h6v5H4v-5Z",
  idBadge: "M8 4h8M9 4V2h6v2M5 5h14v16H5V5Zm3 11a4 4 0 0 1 8 0M12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z",
  calendarRange: "M7 2v4M17 2v4M3 9h18M5 4h14a2 2 0 0 1 2 2v14H3V6a2 2 0 0 1 2-2Zm2 9h3M14 13h3M7 17h10",
  route: "M6 19a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm12-8a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM9 16h3a3 3 0 0 0 3-3V8",
  workflow: "M4 4h6v5H4V4Zm10 11h6v5h-6v-5ZM10 6h4a3 3 0 0 1 3 3v6M7 9v7a2 2 0 0 0 2 2h5",
  calendarStar: "M7 2v4M17 2v4M3 9h18M5 4h14a2 2 0 0 1 2 2v14H3V6a2 2 0 0 1 2-2Zm7 8 1 2 2 .3-1.5 1.5.4 2.2-1.9-1-1.9 1 .4-2.2-1.5-1.5 2-.3 1-2Z",
  umbrella: "M3 12a9 9 0 0 1 18 0H3Zm9 0v7a2 2 0 0 0 4 0",
  device: "M5 3h14v14H5V3Zm4 18h6M12 17v4M8 7h8M8 11h5",
  event: "M6 3v3M18 3v3M4 8h16M5 5h14a1 1 0 0 1 1 1v14H4V6a1 1 0 0 1 1-1Zm3 7h3v3H8v-3Z",
  messageCheck: "M21 15a4 4 0 0 1-4 4H8l-5 3v-7a4 4 0 0 1-1-3V7a4 4 0 0 1 4-4h11a4 4 0 0 1 4 4v8Zm-13-4 2 2 5-5",
  fingerprint: "M12 11a3 3 0 0 1 3 3c0 3-1 5-2 7M9 21c1-2 1-4 1-7a2 2 0 0 1 4 0M6 19c1-2 1-4 1-6a5 5 0 0 1 10 0c0 3-.5 6-2 9M4 15v-2a8 8 0 0 1 16 0v2M7 6a8 8 0 0 1 10 0",
  timeCard: "M4 4h16v16H4V4Zm4 4h5M8 12h8M8 16h4M16 7v3l2 1",
  clipboardCheck: "M9 4h6M9 4a3 3 0 0 0-3 3v14h12V7a3 3 0 0 0-3-3M8 14l2 2 5-5",
  fileImport: "M14 2H6a2 2 0 0 0-2 2v16h16V8l-6-6Zm0 0v6h6M12 11v7M9 14l3-3 3 3",
  calculator: "M5 2h14v20H5V2Zm3 4h8v4H8V6Zm0 8h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01M16 18h.01",
  hourBank: "M3 10h18M5 10v9M9 10v9M15 10v9M19 10v9M3 19h18M12 2l9 5H3l9-5Zm5 12a5 5 0 1 1-5-5",
  calendarCheck: "M7 2v4M17 2v4M3 9h18M5 4h14a2 2 0 0 1 2 2v14H3V6a2 2 0 0 1 2-2Zm5 9v4l3 2",
  layersCheck: "M12 2 3 7l9 5 9-5-9-5ZM3 12l9 5 9-5M3 17l9 5 9-5M15 13l2 2 4-4",
  reports: "M5 3h14v18H5V3Zm4 13v-3M12 16V9M15 16v-5M8 7h8",
  chartClock: "M3 3v18h18M7 16v-5M12 16V8M17 12a4 4 0 1 1-4 4M17 14v3l2 1",
  archive: "M4 4h16v4H4V4Zm2 4h12v12H6V8Zm4 4h4",
  fileExport: "M14 2H6a2 2 0 0 0-2 2v16h9M14 2v6h6M14 16h7M18 12l4 4-4 4",
  connector: "M8 2v5M16 2v5M7 7h10v4a5 5 0 0 1-10 0V7Zm5 9v3h4M17 17l3 2-3 2",
  queue: "M4 6h13M4 12h10M4 18h7M19 15v6M16 18h6",
  link: "M10 13a5 5 0 0 0 7.5.5l2-2a5 5 0 0 0-7-7l-1.1 1.1M14 11a5 5 0 0 0-7.5-.5l-2 2a5 5 0 0 0 7 7l1.1-1.1",
  logs: "M4 4h16v16H4V4Zm4 4h8M8 12h8M8 16h5",
  stethoscope: "M6 3v5a4 4 0 0 0 8 0V3M4 3h4M12 3h4M10 14v2a4 4 0 0 0 8 0v-1M20 13a2 2 0 1 1-4 0 2 2 0 0 1 4 0Z",
  technical: "M5 3h14v18H5V3Zm3 4h8M8 11h8M8 15h4M16 15h.01",
  alert: "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.36a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0ZM12 9v4M12 17h.01",
  chevronDown: "M6 9l6 6 6-6",
  logout: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9",
  plus: "M12 5v14M5 12h14",
  close: "M18 6 6 18M6 6l12 12",
};

const svgSize = computed(() => String(props.size));
const path = computed(() => paths[props.name || "circle"] || paths.circle);
</script>

<template>
  <svg
    class="app-icon"
    :width="svgSize"
    :height="svgSize"
    viewBox="0 0 24 24"
    fill="none"
    aria-hidden="true"
    focusable="false"
  >
    <path
      :d="path"
      stroke="currentColor"
      :stroke-width="strokeWidth"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>
