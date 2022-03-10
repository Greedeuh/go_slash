<template>
  <ul class="list-group" role="list">
    <li
      v-for="feature in features"
      :key="feature.name"
      class="list-group-item"
      role="listitem"
    >
      <div class="form-check form-switch">
        <input
          class="form-check-input"
          type="checkbox"
          role="switch"
          v-model="feature.active"
          @change="(e) => toggle(feature)"
          :disabled="disabled"
        />
        <label class="form-check-label" for="flexSwitchCheckDefault">{{
          feature.name
        }}</label>
      </div>
    </li>
  </ul>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import axios from "axios";

interface Window {
  features: Feature[];
}

interface Feature {
  name: string;
  active: boolean;
}

let win = window as unknown as Window;
const FEATURES = win.features;

export default defineComponent({
  name: "Partial",
  data() {
    return {
      features: FEATURES,
      disabled: false,
    };
  },
  methods: {
    toggle(feature: Feature) {
      this.disabled = true;

      const rollback_state = () => {
        const f = this.features.find((f) => f.name === feature.name);
        if (f) f.active = !feature.active;
      };

      axios
        .put("/go/features", feature)
        .then((res) => {
          if (res.status !== 200) {
            rollback_state();
          }
          this.disabled = false;
        })
        .catch((e) => {
          console.log(e);
          rollback_state();
          this.disabled = false;
        });
    },
  },
});
</script>

<style>
.list-group {
  justify-content: space-between;
}
</style>
