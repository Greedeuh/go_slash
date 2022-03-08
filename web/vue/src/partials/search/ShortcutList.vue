<template>
  <div role="list" class="list-group">
    <a
      v-for="(shortcut, i) in shortcuts"
      :href="administer ? shortcut.shortcut + '?no_redirect' : shortcut.url"
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
        <span v-if="shortcut.new" class="badge bg-success">NEW</span>
      </div>
      <div
        v-if="administer"
        class="btn-group"
        role="group"
        aria-label="Basic mixed styles example"
      >
        <button
          id="btn-delete"
          @click.prevent="delete_shortcut(shortcut.shortcut)"
          class="btn btn-danger"
        >
          <i class="icon-trash"></i>
        </button>
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
    administer: Boolean,
  },
  emits: ["click_shortcut_index", "delete_shortcut"],
  methods: {
    click_shortcut_index(index: number) {
      this.$emit("click_shortcut_index", index);
    },
    delete_shortcut(shortcut: string) {
      this.$emit("delete_shortcut", shortcut);
    },
  },
});
</script>

<style>
.list-group {
  justify-content: space-between;
}
</style>
