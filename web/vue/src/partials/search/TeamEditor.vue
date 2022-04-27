<template>
  <form @submit.prevent="save" class="bg-light border rounded mb-4 p-4">
    <div class="input-group mb-2">
      <span class="input-group-text" id="basic-addon1">Title</span>
      <input
        type="text"
        class="form-control"
        name="title"
        v-model="inner_team.title"
      />
    </div>
    <div class="form-check form-switch">
      <input
        class="form-check-input"
        type="checkbox"
        role="switch"
        name="is_private"
        v-model="inner_team.is_private"
      />
      <label class="form-check-label" for="flexSwitchCheckDefault"
        >Private</label
      >
    </div>
    <div class="form-check form-switch">
      <input
        class="form-check-input"
        type="checkbox"
        role="switch"
        name="is_accepted"
        v-model="inner_team.is_accepted"
      />
      <label class="form-check-label" for="flexSwitchCheckDefault"
        >Enable</label
      >
    </div>
    <input type="submit" class="btn btn-primary mt-2" value="Save" />
  </form>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import _ from "lodash";
import { Team } from "../../models";

interface Data {
  inner_team: Team;
}

export default defineComponent({
  name: "TeamEditor",
  props: {
    team: { required: true, type: Object as PropType<Team> },
  },
  data(): Data {
    return { inner_team: _.clone(this.team) as Team };
  },
  emits: ["save"],
  methods: {
    save() {
      // eslint-disable-next-line
      const team: any = {};
      if (this.team.title !== this.inner_team.title) {
        team.title = this.inner_team.title;
      }
      if (this.team.is_private !== this.inner_team.is_private) {
        team.is_private = this.inner_team.is_private;
      }
      if (this.team.is_accepted !== this.inner_team.is_accepted) {
        team.is_accepted = this.inner_team.is_accepted;
      }

      if (Object.keys(team).length !== 0) {
        this.$emit("save", { slug: this.team.slug, team });
      }
    },
  },
});
</script>

<style></style>
