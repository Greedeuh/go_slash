<template>
  <div class="input-group mb-3 input-group-lg">
    <span class="input-group-text">GO /</span>
    <input
      :value="modelValue"
      @input="update_value($event.target.value)"
      type="text"
      placeholder="type then Tab or Enter"
      @keydown="keydown"
      ref="search_bar"
      class="form-control"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "SearchBar",
  props: {
    modelValue: String,
  },
  emits: ["update:modelValue", "keydown"],
  methods: {
    update_value(value: string) {
      this.$emit("update:modelValue", value);
    },
    keydown(e: KeyboardEvent) {
      this.$emit("keydown", e);
    },
  },
  mounted() {
    (this.$refs as any).search_bar.focus(); // eslint-disable-line
  },
});
</script>

<style></style>
