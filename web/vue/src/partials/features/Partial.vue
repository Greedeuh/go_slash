<template>
  <div>
    <SwitchGroup
      v-for="(inside, name) in features"
      :key="name"
      :name="name"
      :inside="inside"
      @toggle="toggle"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import axios from "axios";

import SwitchGroup from "./SwitchGroup.vue";

interface Window {
  features: Features;
}

interface Features {
  login: { simple: boolean };
}

let win = window as unknown as Window;
const FEATURES = win.features;

export default defineComponent({
  name: "Partial",
  components: {
    SwitchGroup,
  },
  data() {
    return {
      features: FEATURES,
      disabled: false,
    };
  },
  methods: {
    toggle(e: {
      name: string;
      value: boolean;
      success_cb: () => void;
      rollback_cb: () => void;
    }) {
      const keys = e.name.split(".");
      const features: any = {};

      let last_key = features;
      keys.forEach(function (key, i) {
        if (i + 1 < keys.length) {
          last_key[key] = {};
          last_key = last_key[key];
        } else {
          last_key[key] = e.value;
        }
      });

      console.log(features);
      axios
        .patch("/go/features", features)
        .then((res) => {
          if (res.status !== 200) {
            e.rollback_cb();
          } else {
            e.success_cb();
          }
        })
        .catch((err) => {
          console.log(err);
          e.rollback_cb();
        });
    },
  },
});
</script>

<style></style>
