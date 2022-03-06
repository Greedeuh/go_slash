<template>
  <div>
    <SearchBar
      v-model="search"
      @keydown="reset_index_if_letter"
      :administer="administer"
      @on-administer="set_administer"
    />
    <ShortcutList
      :shortcuts="fuzzed_or_all"
      :selected_index="selected_index"
      :administer="administer"
      @delete_shortcut="delete_shortcut"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import Fuse from "fuse.js";
import axios from "axios";

import SearchBar from "./Search.vue";
import ShortcutList from "./ShortcutList.vue";

interface Window {
  shortcuts: Shortcut[];
}

export interface Shortcut {
  shortcut: string;
  url: string;
}

function setup_fuse(shortcuts: Shortcut[]) {
  return new Fuse(shortcuts, {
    keys: [
      { name: "shortcut", weight: 2 },
      { name: "url", weight: 1 },
    ],
  });
}

const CONTROL_KEYS = ["ArrowUp", "ArrowDown", "Enter", "Tab"];
const SHORTCUTS = (window as unknown as Window).shortcuts;

let key_press: (e: KeyboardEvent) => void;

export default defineComponent({
  name: "Partial",
  components: { SearchBar, ShortcutList },
  data() {
    return {
      selected_index: -1,
      shortcuts: SHORTCUTS,
      administer: false,
      fuse: setup_fuse(SHORTCUTS),
      search: "",
    };
  },
  computed: {
    fuzzed_or_all(): Shortcut[] {
      let shortcuts_fuzzed = this.fuse
        .search(this.search)
        .map((res) => res.item);
      return shortcuts_fuzzed.length ? shortcuts_fuzzed : this.shortcuts;
    },
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
    set_administer() {
      this.administer = !this.administer;
    },
    delete_shortcut(shortcut: string) {
      axios.delete("/" + shortcut).then((res) => {
        if (res.status === 200) {
          this.shortcuts = this.shortcuts.filter(
            (s) => s.shortcut !== shortcut
          );
          this.fuse.setCollection(this.shortcuts);
        }
      });
    },
  },
});
</script>

<style></style>
