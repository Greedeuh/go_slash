<template>
  <div>
    <SearchBar v-model="search" @keydown="reset_index_if_letter" />
    <div class="list-group">
      <a
        href="#"
        v-for="(shortcut, i) in fuzzed_or_all"
        :key="i"
        :class="{ active: i == selected_index }"
        class="list-group-item d-flex justify-content-between align-items-start"
        @click="take_selected(i)"
      >
        <div class="ms-2 me-auto">
          <span class="fw-bold">
            {{ shortcut.shortcut }}
          </span>
          {{ shortcut.url }}
        </div>
      </a>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { useVueFuse } from "vue-fuse";

import SearchBar from "./Search.vue";

interface Window {
  shortcuts: Shortcut[];
}

interface Shortcut {
  shortcut: string;
  url: string;
}

const CONTROL_KEYS = ["ArrowUp", "ArrowDown", "Enter", "Tab"];
const SHORTCUTS = (window as unknown as Window).shortcuts;

let key_press: (e: KeyboardEvent) => void;

export default defineComponent({
  name: "Partial",
  components: { SearchBar },
  setup() {
    const { search, results, noResults } = useVueFuse(SHORTCUTS, {
      keys: [
        { name: "shortcut", weight: 2 },
        { name: "url", weight: 1 },
      ],
    });

    return {
      search,
      shortcuts_fuzzed: results,
      noResults,
    };
  },
  computed: {
    fuzzed_or_all(): Shortcut[] {
      return this.shortcuts_fuzzed.length
        ? (this.shortcuts_fuzzed as unknown as Shortcut[])
        : this.shortcuts;
    },
  },
  data() {
    return { selected_index: -1, shortcuts: SHORTCUTS };
  },
  created() {
    key_press = (e: KeyboardEvent) => {
      let key = e.key;
      if (CONTROL_KEYS.includes(key)) {
        e.preventDefault();

        if (key === "ArrowUp" && this.selected_index >= 0) {
          this.selected_index -= 1;
        }
        if (
          key === "ArrowDown" &&
          this.selected_index + 1 < this.fuzzed_or_all.length
        ) {
          this.selected_index += 1;
        }

        if (key === "Tab") {
          this.take_selected(this.selected_index < 0 ? 0 : this.selected_index);
        }
        if (key === "Enter") {
          this.go_selected(this.selected_index);
        }
      }
    };
    window.addEventListener("keydown", key_press);
  },
  unmounted() {
    window.removeEventListener("keydown", key_press);
  },
  methods: {
    reset_index_if_letter({ key }: KeyboardEvent) {
      if (key.length === 1) this.selected_index = -1;
    },
    take_selected(selected_index: number) {
      this.search = this.fuzzed_or_all[selected_index].shortcut;
      this.selected_index = -1;
    },
    go_selected(selected_index: number) {
      if (selected_index === -1) {
        window.location.href = "/" + this.search;
      } else {
        window.location.href =
          "/" + this.fuzzed_or_all[selected_index].shortcut;
      }
    },
  },
});
</script>

<style></style>
