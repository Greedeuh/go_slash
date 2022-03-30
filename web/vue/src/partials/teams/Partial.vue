<template>
  <div role="list" class="list-group">
    <a
      v-for="team in teams"
      :href="`/go/teams/${team.slug}`"
      :key="team.slug"
      role="listitem"
      class="list-group-item-action list-group-item d-flex justify-content-between align-items-start"
    >
      <div class="ms-2 me-auto content">
        <span class="fw-bold">
          {{ team.title }}
        </span>
      </div>

      <div>
        <i v-if="team.is_private" class="icon-lock"></i
        ><i v-if="!team.is_accepted" class="icon-check-empty ms-2"></i
        ><i v-if="team.is_accepted" class="icon-check ms-2"></i>
        <button
          v-if="!team.user_link"
          @click.prevent="join(team.slug)"
          type="button"
          class="btn btn-primary"
        >
          Join
        </button>
        <button
          v-if="team.user_link && !team.user_link.is_accepted"
          type="button"
          class="btn btn-secondary"
          disabled
        >
          Waiting
        </button>
        <button
          v-if="team.user_link"
          @click.prevent="leave(team.slug)"
          type="button"
          class="btn btn-danger"
        >
          Leave
        </button>
      </div>
    </a>
  </div>
</template>

<script lang="ts">
import axios from "axios";
import { defineComponent } from "vue";

interface Window {
  teams: Team[];
}

let win = window as unknown as Window;
const TEAMS = win.teams;

interface Team {
  slug: string;
  title: string;
  is_private: boolean;
  is_accepted: boolean;
  user_link: {
    is_admin: boolean;
    is_accepted: boolean;
  };
}

export default defineComponent({
  name: "Partial",
  data() {
    return {
      teams: TEAMS,
    };
  },
  methods: {
    join(slug: string) {
      axios
        .post("/go/user/teams/" + slug)
        .then((res) => {
          let team = this.teams.find((t) => t.slug === slug);
          if (res.status === 201 && team) {
            team.user_link = {
              is_admin: false,
              is_accepted: !team.is_private,
            };
          }
        })
        .catch(console.log);
    },
    leave(slug: string) {
      axios
        .delete("/go/user/teams/" + slug)
        .then((res) => {
          let team = this.teams.find((t) => t.slug === slug);
          if (res.status === 200 && team) {
            team.user_link = undefined;
          }
        })
        .catch(console.log);
    },
  },
});
</script>

<style scoped>
span {
  line-height: 38px;
}

button {
  margin-left: 0.5em;
}
</style>
