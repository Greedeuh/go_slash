<template>
  <form @submit.prevent="enter">
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
      <span class="input-group-text">
        <div class="btn-group">
          <button id="btn-enter" class="btn btn-primary" type="submit">
            <i class="icon-search"></i></button
          ><button
            @click="emit_administer"
            id="btn-administer"
            :class="{ 'btn-light': !administer, 'btn-secondary': administer }"
            class="btn"
            type="button"
          >
            <i class="icon-wrench"></i>
          </button>
        </div>
      </span>
    </div>
  </form>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "SearchBar",
  props: {
    modelValue: String,
    administer: Boolean,
  },
  emits: [
    "update:modelValue",
    "keydown",
    "administrate",
    "on-administer",
    "enter",
  ],
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
    enter() {
      this.$emit("enter");
    },
  },
  mounted() {
    (this.$refs as any).search_bar.focus(); // eslint-disable-line
  },
});
</script>

<style></style>
