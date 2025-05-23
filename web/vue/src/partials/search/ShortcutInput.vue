<template>
  <form @submit.prevent="save">
    <div class="input-group mb-3 input-group-m">
      <span class="input-group-text">Bind</span>
      <input
        v-model="shortcut"
        minlength="1"
        required
        type="text"
        name="shortcut"
        class="form-control"
        placeholder="shortcut"
        @focus="global_focus(true)"
        @blur="global_focus(false)"
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
        @focus="global_focus(true)"
        @blur="global_focus(false)"
      />
      <span v-if="admin_teams" class="input-group-text">for</span>
      <select
        v-if="admin_teams"
        v-model="team"
        class="form-select"
        name="team"
        @focus="global_focus(true)"
        @blur="global_focus(false)"
      >
        <option v-for="team in admin_teams" :key="team.slug" :value="team.slug">
          {{ team.slug === "" ? "Global team" : team.slug }}
        </option>
      </select>
      <button
        aria-label="Add shortcut"
        class="btn btn-primary"
        type="submit"
        @focus="global_focus(true)"
        @blur="global_focus(false)"
      >
        Save <i class="icon-save"></i>
      </button>
    </div>
  </form>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import { Shortcut } from "./Partial.vue";

export default defineComponent({
  name: "ShortcutInput",
  props: {
    initial_shortcut: Object as PropType<Shortcut>,
    initial_url: String,
    admin_teams: Object,
  },
  data() {
    return {
      shortcut: this.initial_shortcut?.shortcut,
      url: this.initial_shortcut?.url,
      team: this.initial_shortcut?.team_slug,
    };
  },
  emits: ["save"],
  methods: {
    save() {
      let on_success = () => {
        this.shortcut = "";
        this.url = "";
        this.team = "";
      };

      this.$emit("save", {
        shortcut: this.shortcut,
        url: this.url,
        team_slug: this.team,
        on_success,
      });
    },
    // lazy way to stop the global listning of partial on enter, tab ... while on our current form
    global_focus(yes_or_no: boolean) {
      (window as any).focus_flag = yes_or_no; // eslint-disable-line
    },
  },
});
</script>

<style>
input[name="shortcut"] {
  max-width: 300px;
}
</style>
