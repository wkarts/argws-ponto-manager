<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { entityConfigs, type EntityField } from "../config/entities";
import { comboList, deleteEntity, listEntity, saveEntity, type ComboOption } from "../services/crud";
import { booleanLabel } from "../services/format";

const props = defineProps<{
  entityKey: string;
}>();

const config = computed(() => entityConfigs[props.entityKey]);
const rows = ref<Record<string, unknown>[]>([]);
const search = ref("");
const saving = ref(false);
const loading = ref(false);
const error = ref("");
const form = reactive<Record<string, unknown>>({ id: undefined });
const optionsMap = ref<Record<string, ComboOption[]>>({});

function defaultFieldValue(field: EntityField): unknown {
  if (field.type === "checkbox") return true;
  return "";
}

function resetForm() {
  Object.keys(form).forEach((key) => delete form[key]);
  form.id = undefined;
  for (const field of config.value.fields) {
    form[field.key] = defaultFieldValue(field);
  }
}

async function loadOptions() {
  const relationFields = config.value.fields.filter((field) => field.type === "select" && field.relationEntity);
  const entries = await Promise.all(
    relationFields.map(async (field) => ({
      key: field.key,
      items: await comboList(field.relationEntity as string)
    }))
  );

  const next: Record<string, ComboOption[]> = {};
  for (const entry of entries) {
    next[entry.key] = entry.items;
  }
  optionsMap.value = next;
}

function getOptionLabel(fieldKey: string, value: unknown): string {
  const options = optionsMap.value[fieldKey] || [];
  const matched = options.find((item) => String(item.id) === String(value));
  return matched?.label || String(value ?? "");
}

function displayValue(column: string, row: Record<string, unknown>): string {
  const value = row[column];
  const field = config.value.fields.find((item) => item.key === column);

  if (field?.type === "checkbox") {
    return booleanLabel(value);
  }

  if (field?.type === "select") {
    return getOptionLabel(field.key, value);
  }

  return String(value ?? "");
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listEntity(config.value.key, search.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar registros.";
  } finally {
    loading.value = false;
  }
}

function editRow(row: Record<string, unknown>) {
  Object.keys(form).forEach((key) => delete form[key]);
  for (const field of config.value.fields) {
    form[field.key] = row[field.key] ?? defaultFieldValue(field);
  }
  form.id = row.id;
}

async function persist() {
  saving.value = true;
  error.value = "";
  try {
    await saveEntity(config.value.key, { ...form });
    await load();
    resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar registro.";
  } finally {
    saving.value = false;
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja remover este registro?")) return;
  try {
    await deleteEntity(config.value.key, id);
    await load();
    if (Number(form.id) === id) {
      resetForm();
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover registro.";
  }
}

onMounted(async () => {
  resetForm();
  await loadOptions();
  await load();
});

watch(
  () => props.entityKey,
  async () => {
    resetForm();
    await loadOptions();
    await load();
  }
);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">{{ config.title }}</h2>
        <div class="muted">CRUD local da entidade {{ config.key }}</div>
      </div>

      <div class="actions">
        <input v-model="search" placeholder="Pesquisar..." @keyup.enter="load" />
        <button class="secondary" @click="load">Buscar</button>
        <button class="secondary" @click="resetForm">Novo</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card">
        <h3 style="margin-top: 0;">Cadastro</h3>
        <form class="grid" @submit.prevent="persist">
          <div v-for="field in config.fields" :key="field.key" class="field">
            <label :for="field.key">
              {{ field.label }}
              <span v-if="field.required" class="required">*</span>
            </label>

            <textarea
              v-if="field.type === 'textarea'"
              :id="field.key"
              v-model="form[field.key]"
              rows="3"
              :placeholder="field.placeholder"
            />

            <input
              v-else-if="field.type === 'checkbox'"
              :id="field.key"
              v-model="form[field.key]"
              type="checkbox"
              class="checkbox-input"
            />

            <select
              v-else-if="field.type === 'select'"
              :id="field.key"
              v-model="form[field.key]"
            >
              <option value="">Selecione</option>
              <option v-for="item in optionsMap[field.key] || []" :key="item.id" :value="item.id">
                {{ item.label }}
              </option>
            </select>

            <input
              v-else
              :id="field.key"
              v-model="form[field.key]"
              :placeholder="field.placeholder"
              :type="field.type === 'number'
                ? 'number'
                : field.type === 'date'
                  ? 'date'
                  : field.type === 'time'
                    ? 'time'
                    : field.type === 'email'
                      ? 'email'
                      : field.type === 'tel'
                        ? 'tel'
                        : field.type === 'password'
                          ? 'password'
                          : 'text'"
            />
          </div>

          <div class="actions">
            <button class="primary" type="submit" :disabled="saving">
              {{ saving ? 'Salvando...' : 'Salvar' }}
            </button>
            <button class="secondary" type="button" @click="resetForm">Limpar</button>
          </div>
        </form>
      </div>

      <div class="card">
        <h3 style="margin-top: 0;">Listagem</h3>
        <div v-if="loading" class="muted">Carregando...</div>

        <div class="table-wrap" v-else>
          <table>
            <thead>
              <tr>
                <th v-for="column in config.columns" :key="column">{{ column }}</th>
                <th>ações</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rows" :key="String(row.id)">
                <td v-for="column in config.columns" :key="column">
                  {{ displayValue(column, row) }}
                </td>
                <td>
                  <div class="actions compact-actions">
                    <button class="secondary" @click="editRow(row)">Editar</button>
                    <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                  </div>
                </td>
              </tr>
              <tr v-if="rows.length === 0">
                <td :colspan="config.columns.length + 1" class="muted">Nenhum registro encontrado.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
