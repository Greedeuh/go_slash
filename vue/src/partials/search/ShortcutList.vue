<template>
  <div role="list" class="list-group">
    <a
      v-for="(shortcut, i) in shortcuts"
      :href="shortcut.url"
      :key="i"
      role="listitem"
      :class="{ active: i == selected_index }"
      class="list-group-item d-flex justify-content-between align-items-start"
      @click="click_shortcut_index(i)"
    >
      <div class="ms-2 me-auto">
        <span class="fw-bold">
          {{ shortcut.shortcut }}
        </span>
        {{ shortcut.url }}
      </div>
    </a>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "ShortcutList",
  props: {
    shortcuts: Array,
    selected_index: Number,
  },
  emits: ["click_shortcut_index"],
  methods: {
    click_shortcut_index(index: number) {
      this.$emit("click_shortcut_index", index);
    },
  },
});
</script>

<style></style>
