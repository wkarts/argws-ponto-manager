<script setup lang="ts">
import { dismissSplash, splashState } from "../services/splash";
</script>

<template>
  <div class="splash-stack" aria-live="polite" aria-atomic="true">
    <div
      v-for="item in splashState.messages"
      :key="item.id"
      class="splash-item"
      :class="`splash-${item.tone}`"
      role="status"
    >
      <span>{{ item.text }}</span>
      <button type="button" class="splash-close" @click="dismissSplash(item.id)">×</button>
    </div>
  </div>
</template>

<style scoped>
.splash-stack {
  position: fixed;
  top: 14px;
  right: 16px;
  z-index: 1000;
  display: grid;
  gap: 8px;
  width: min(420px, calc(100vw - 20px));
  pointer-events: none;
}

.splash-item {
  pointer-events: auto;
  border-radius: 10px;
  border: 1px solid transparent;
  padding: 10px 12px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  box-shadow: 0 8px 18px rgba(15, 23, 42, 0.18);
}

.splash-success {
  background: #ecfdf5;
  border-color: #86efac;
  color: #14532d;
}

.splash-error {
  background: #fef2f2;
  border-color: #fca5a5;
  color: #7f1d1d;
}

.splash-info {
  background: #eff6ff;
  border-color: #93c5fd;
  color: #1e3a8a;
}

.splash-close {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
  color: inherit;
}
</style>
