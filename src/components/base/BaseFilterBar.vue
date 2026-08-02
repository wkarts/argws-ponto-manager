<script setup lang="ts">
import { ref, useId, watch } from "vue";

const props = withDefaults(defineProps<{
  title?: string;
  description?: string;
  collapsible?: boolean;
  defaultExpanded?: boolean;
  loading?: boolean;
  density?: "default" | "compact";
}>(), {
  title: "Filtros",
  description: "",
  collapsible: false,
  defaultExpanded: true,
  loading: false,
  density: "default",
});

const expanded = ref(props.defaultExpanded);
const contentId = useId();

watch(() => props.defaultExpanded, (value) => {
  expanded.value = value;
});
</script>

<template>
  <section
    class="card filter-bar base-filter-bar"
    :class="[`base-filter-bar--${density}`, { 'base-filter-bar--collapsed': !expanded }]"
    :aria-busy="loading"
  >
    <header class="base-filter-bar__header">
      <div class="base-filter-bar__heading">
        <h3>{{ title }}</h3>
        <p v-if="description">{{ description }}</p>
      </div>
      <button
        v-if="collapsible"
        class="secondary base-filter-bar__toggle"
        type="button"
        :aria-expanded="expanded"
        :aria-controls="contentId"
        @click="expanded = !expanded"
      >
        {{ expanded ? "Recolher filtros" : "Exibir filtros" }}
      </button>
    </header>

    <div :id="contentId" v-show="expanded" class="base-filter-bar__layout">
      <div class="filter-grid base-filter-bar__fields"><slot /></div>
      <div v-if="$slots.actions" class="actions filter-actions base-filter-bar__actions">
        <slot name="actions" />
      </div>
    </div>

    <div v-if="expanded && $slots.advanced" class="base-filter-bar__advanced">
      <slot name="advanced" />
    </div>

    <div v-if="$slots.summary" class="base-filter-bar__summary">
      <slot name="summary" />
    </div>
  </section>
</template>

<style scoped>
.base-filter-bar {
  container: filterbar / inline-size;
  display: grid;
  gap: 12px;
  padding: clamp(11px, 1vw, 14px);
}

.base-filter-bar__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.base-filter-bar__heading {
  min-width: 0;
}

.base-filter-bar__heading h3 {
  margin: 0;
  font-size: 14px;
  line-height: 1.25;
}

.base-filter-bar__heading p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.35;
}

.base-filter-bar__toggle {
  flex: 0 0 auto;
  min-height: 34px;
  white-space: nowrap;
}

.base-filter-bar__layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 12px;
  align-items: end;
}

.base-filter-bar__fields {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: 10px 12px;
  align-items: end;
  min-width: 0;
}

.base-filter-bar__fields :deep(.field) {
  grid-column: span 3;
  min-width: 0;
}

.base-filter-bar__fields :deep(.filter-field--wide),
.base-filter-bar__fields :deep(.filter-field--search) {
  grid-column: span 4;
}

.base-filter-bar__fields :deep(.filter-field--date),
.base-filter-bar__fields :deep(.filter-field--status),
.base-filter-bar__fields :deep(.filter-field--compact) {
  grid-column: span 2;
}

.base-filter-bar__fields :deep(.filter-field--full) {
  grid-column: 1 / -1;
}

.base-filter-bar__fields :deep(.filter-field--toggle) {
  grid-column: span 2;
  min-height: 36px;
  align-self: end;
  display: flex;
  align-items: center;
  padding-bottom: 1px;
}

.base-filter-bar__fields :deep(.field input:not([type="checkbox"])),
.base-filter-bar__fields :deep(.field select),
.base-filter-bar__fields :deep(.field button) {
  min-height: 36px;
}

.base-filter-bar__actions {
  justify-content: flex-end;
  align-items: center;
  min-height: 36px;
  margin: 0;
  white-space: nowrap;
}

.base-filter-bar__actions :deep(button),
.base-filter-bar__actions :deep(.link-button) {
  min-height: 36px;
}

.base-filter-bar__advanced {
  padding-top: 11px;
  border-top: 1px solid var(--border-color);
}

.base-filter-bar__advanced :deep(.field) {
  min-width: 0;
}

.base-filter-bar__summary {
  min-width: 0;
}

.base-filter-bar--compact {
  gap: 9px;
  padding: 10px 12px;
}

.base-filter-bar--collapsed {
  gap: 0;
}

@container filterbar (max-width: 980px) {
  .base-filter-bar__fields :deep(.field) {
    grid-column: span 4;
  }

  .base-filter-bar__fields :deep(.filter-field--wide),
  .base-filter-bar__fields :deep(.filter-field--search) {
    grid-column: span 6;
  }

  .base-filter-bar__fields :deep(.filter-field--date),
  .base-filter-bar__fields :deep(.filter-field--status),
  .base-filter-bar__fields :deep(.filter-field--compact),
  .base-filter-bar__fields :deep(.filter-field--toggle) {
    grid-column: span 3;
  }
}

@container filterbar (max-width: 760px) {
  .base-filter-bar__layout {
    grid-template-columns: 1fr;
  }

  .base-filter-bar__fields :deep(.field),
  .base-filter-bar__fields :deep(.filter-field--wide),
  .base-filter-bar__fields :deep(.filter-field--search) {
    grid-column: span 6;
  }

  .base-filter-bar__actions {
    justify-content: flex-end;
    white-space: normal;
  }
}

@container filterbar (max-width: 500px) {
  .base-filter-bar__header {
    align-items: stretch;
    flex-direction: column;
  }

  .base-filter-bar__toggle {
    width: 100%;
  }

  .base-filter-bar__fields :deep(.field),
  .base-filter-bar__fields :deep(.filter-field--wide),
  .base-filter-bar__fields :deep(.filter-field--search),
  .base-filter-bar__fields :deep(.filter-field--date),
  .base-filter-bar__fields :deep(.filter-field--status),
  .base-filter-bar__fields :deep(.filter-field--compact),
  .base-filter-bar__fields :deep(.filter-field--toggle) {
    grid-column: 1 / -1;
  }

  .base-filter-bar__actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .base-filter-bar__actions :deep(button),
  .base-filter-bar__actions :deep(.link-button) {
    width: 100%;
  }
}
</style>
