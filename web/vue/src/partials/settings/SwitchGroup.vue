<template>
  <div>
    <div v-if="is_group">
      <h1>{{ name }}</h1>
      <SwitchGroup
        v-for="(inside2, name2) in inside"
        :key="name2"
        :name="name2"
        :inside="inside2"
        @toggle="
          (e) =>
            toggle(name + '.' + e.name, e.value, e.success_cb, e.rollback_cb)
        "
      />
    </div>
    <div v-else class="form-check form-switch" role="article">
      <input
        class="form-check-input"
        type="checkbox"
        role="switch"
        v-model="current_value"
        :disabled="disabled"
        @change="(e) => toggle(name)"
      />
      <label class="form-check-label" for="flexSwitchCheckDefault"
        >{{ name }}
        <div
          v-if="disabled"
          class="spinner-border spinner-border-sm"
          role="status"
        >
          <span class="visually-hidden">Loading...</span>
        </div></label
      >
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "SwitchGroup",
  props: ["name", "inside"],
  emits: ["toggle"],
  data() {
    return { current_value: this.inside, disabled: false };
  },
  computed: {
    is_group() {
      const obj = "" + this.inside;
      const is_bool = obj === "true" || obj === "false";
      return !is_bool;
    },
  },
  methods: {
    toggle(
      name: string,
      value: boolean | undefined,
      success_cb: () => void | undefined,
      rollback_cb: () => void | undefined
    ) {
      if (
        rollback_cb === undefined &&
        value === undefined &&
        success_cb === undefined
      ) {
        this.disabled = true;
        value = this.current_value;
        rollback_cb = () => {
          this.current_value = !this.current_value;
          this.disabled = false;
        };
        success_cb = () => {
          this.disabled = false;
        };
      }
      this.$emit("toggle", { name, value, rollback_cb, success_cb });
    },
  },
});
</script>

<style></style>
