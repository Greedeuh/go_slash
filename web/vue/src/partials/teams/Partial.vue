<template>
  <div>
    <div class="d-flex mb-4">
      <div class="alert alert-primary flex-fill m-0" role="alert">
        Drag and drop to prioritize team shortcuts in case of duplicates
      </div>
      <div v-if="right === 'write'" class="align-self-center ms-2">
        <button
          @click="set_administer"
          class="btn btn-lg"
          :class="{ 'btn-light': !administer, 'btn-secondary': administer }"
          aria-label="Administrate"
        >
          <i class="icon-wrench"></i>
        </button>
      </div>
    </div>
    <UserTeamList
      aria-label="User teams"
      :teams="user_teams"
      @leave="leave"
      @change_ranks="change_ranks"
      :administer="administer"
      @delete_team="delete_team"
      @accept="accept"
    />
    <TeamList
      aria-label="Other teams"
      :teams="other_teams"
      @join="join"
      :administer="administer"
      @delete_team="delete_team"
      @accept="accept"
    />
  </div>
</template>

<script lang="ts">
import axios from "axios";
import { defineComponent } from "vue";
import { Team, UserTeamLink, sort_by_rank } from "./main";
import TeamList from "./TeamList.vue";
import UserTeamList from "./UserTeamList.vue";
import _ from "lodash";

interface Window {
  teams: Team[];
  right: string;
}

let win = window as unknown as Window;
const TEAMS = win.teams;

interface Data {
  teams: Team[];
  right: string;
  administer: boolean;
}

const RIGHT = win.right;

export default defineComponent({
  name: "Partial",
  components: { UserTeamList, TeamList },
  data(): Data {
    return {
      teams: TEAMS,
      right: RIGHT,
      administer: false,
    };
  },
  computed: {
    user_teams(): Team[] {
      return this.teams.filter((team) => team.user_link).sort(sort_by_rank);
    },
    other_teams(): Team[] {
      return this.teams.filter((team) => !team.user_link);
    },
    next_rank() {
      const that = this as { user_teams: Team[] };
      const length = that.user_teams.length;
      return length === 0
        ? 0
        : (that.user_teams[length - 1].user_link as UserTeamLink).rank + 1;
    },
  },
  methods: {
    join(slug: string) {
      axios
        .post("/go/user/teams/" + slug, { rank: this.next_rank })
        .then((res) => {
          let team = this.teams.find((t) => t.slug === slug);
          if (res.status === 201 && team) {
            team.user_link = {
              is_admin: false,
              is_accepted: !team.is_private,
              rank: this.next_rank,
            };
          }
        })
        .catch(console.error);
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
        .catch(console.error);
    },
    // eslint-disable-next-line
    change_ranks(new_ranks: any) {
      const old_teams = this.teams;
      this.teams = this.teams.map((team) => {
        const new_rank = new_ranks[team.slug];
        if (new_rank !== undefined) {
          team = _.cloneDeep(team);
          (team.user_link as UserTeamLink).rank = new_rank;
        }
        return team;
      });
      axios
        .put("/go/user/teams/ranks", new_ranks)
        .then((res) => {
          if (res.status !== 200) {
            this.teams = old_teams;
          }
        })
        .catch(console.error);
    },
    set_administer() {
      this.administer = !this.administer;
    },
    delete_team(slug: string) {
      axios
        .delete("/go/teams/" + slug)
        .then((res) => {
          if (res.status === 200) {
            this.teams = this.teams.filter((team) => team.slug !== slug);
          }
        })
        .catch(console.error);
    },
    accept(slug: string) {
      axios
        .patch("/go/teams/" + slug, { is_accepted: true })
        .then((res) => {
          if (res.status === 200) {
            const team = this.teams.find((team) => team.slug == slug);
            team.is_accepted = true;
          }
        })
        .catch(console.error);
    },
  },
});
</script>

<style scoped></style>
