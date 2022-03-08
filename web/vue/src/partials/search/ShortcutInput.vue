<template>
  <form @submit.prevent="save">
    <div class="input-group mb-3 input-group-m">
      <span class="input-group-text">Bind</span>
      <input
        v-model="shortcut"
        :disabled="initial_shortcut"
        minlength="1"
        required
        type="text"
        name="shortcut"
        class="form-control"
        placeholder="shortcut"
      />
      <span class="input-group-text">to</span>
      <input
        v-model="url"
        required
        pattern="https?://(www\.)?[-a-zA-Z0-9()@:%_\+.~#?&//=]{1,256}"
        title="This field should be an URL starting with http(s)://"
        type="text"
        name="url"
        class="form-control"
        placeholder="https://my-favorite-tool"
        @keydown.enter.prevent=""
      />
      <button id="btn-add" class="btn btn-primary" type="submit">
        Save <i class="icon-save"></i>
      </button>
    </div>
  </form>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "ShortcutInput",
  props: {
    initial_shortcut: String,
    initial_url: String,
  },
  data() {
    return { shortcut: this.initial_shortcut, url: this.initial_url };
  },
  emits: ["save"],
  methods: {
    save() {
      let on_success;
      if (this.initial_shortcut) {
        on_success = () => {}; // eslint-disable-line
      } else {
        on_success = () => {
          this.shortcut = "";
          this.url = "";
        };
      }

      this.$emit("save", {
        shortcut: this.shortcut,
        url: this.url,
        on_success,
      });
    },
  },
});
</script>

<style>
input[name="shortcut"] {
  max-width: 300px;
}
</style>
