<template>
  <div class="input-group mb-3 input-group-lg">
    <span class="input-group-text">GO /</span>
    <input
      :value="modelValue"
      @input="update_value($event.target.value)"
      type="search"
      placeholder="type then Up/Down then Tab/Enter"
      @keydown="keydown"
      ref="search_bar"
      class="form-control"
    />
    <span class="input-group-text"
      ><button
        @click="emit_administer"
        id="btn-administer"
        :class="{ 'btn-light': !administer, 'btn-secondary': administer }"
        class="btn"
      >
        <i class="icon-wrench"></i></button
    ></span>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "SearchBar",
  props: {
    modelValue: String,
    administer: Boolean,
  },
  emits: ["update:modelValue", "keydown", "administrate", "on-administer"],
  methods: {
    update_value(value: string) {
      this.$emit("update:modelValue", value);
    },
    keydown(e: KeyboardEvent) {
      this.$emit("keydown", e);
    },
    emit_administer() {
      this.$emit("on-administer");
    },
  },
  mounted() {
    (this.$refs as any).search_bar.focus(); // eslint-disable-line
  },
});
</script>

<style></style>
