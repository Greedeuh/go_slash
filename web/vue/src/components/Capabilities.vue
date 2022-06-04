<template>
  <div>
    <strong>Capabilities :</strong>
    <div
      v-for="capability in capabilities"
      :key="capability"
      class="form-check form-switch"
    >
      <input
        class="form-check-input"
        :name="capability"
        type="checkbox"
        role="switch"
        :checked="user_capabilities.includes(capability)"
        @click="toggle(capability, !user_capabilities.includes(capability))"
      />
      <label class="form-check-label">
        {{ capability }}
      </label>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { ALL_CAPABILITIES, ALL_TEAM_CAPABILITIES } from "../models";

export default defineComponent({
  name: "Capabilities",
  props: {
    user_capabilities: Array,
    type: String,
  },
  computed: {
    capabilities() {
      if (this.type === "team") {
        return ALL_TEAM_CAPABILITIES;
      } else {
        return ALL_CAPABILITIES;
      }
    },
  },
  emits: ["toggle"],
  methods: {
    toggle(capability: string, value: boolean) {
      this.$emit("toggle", { capability, value });
    },
  },
});
</script>

<style scoped></style>
