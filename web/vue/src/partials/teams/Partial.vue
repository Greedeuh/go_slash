<template>
  <div>
    <TeamList aria-label="User teams" :teams="user_teams" @leave="leave" />
    <TeamList aria-label="Other teams" :teams="other_teams" @join="join" />
  </div>
</template>

<script lang="ts">
import axios from "axios";
import { defineComponent } from "vue";
import { Team, UserTeamLink } from "./main";
import TeamList from "./TeamList.vue";

interface Window {
  teams: Team[];
}

let win = window as unknown as Window;
const TEAMS = win.teams;

interface Data {
  teams: Team[];
}

export default defineComponent({
  name: "Partial",
  components: { TeamList },
  data(): Data {
    return {
      teams: TEAMS,
    };
  },
  computed: {
    user_teams(): Team[] {
      return this.teams
        .filter((team) => team.user_link)

        .sort(
          (a, b) =>
            (a.user_link as UserTeamLink).rank -
            (b.user_link as UserTeamLink).rank
        );
    },
    other_teams(): Team[] {
      return this.teams.filter((team) => !team.user_link);
    },
  },
  methods: {
    join(slug: string) {
      axios
        .post("/go/user/teams/" + slug, { rank: 0 })
        .then((res) => {
          let team = this.teams.find((t) => t.slug === slug);
          if (res.status === 201 && team) {
            team.user_link = {
              is_admin: false,
              is_accepted: !team.is_private,
              rank: 0,
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

<style scoped></style>
