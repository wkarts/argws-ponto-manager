import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./styles.css";
import { useSessionStore } from "./stores/session";

async function bootstrap() {
  const app = createApp(App);
  const pinia = createPinia();
  app.use(pinia);

  const session = useSessionStore(pinia);
  await session.restore();

  app.use(router);
  await router.isReady();
  app.mount("#app");
}

bootstrap();
