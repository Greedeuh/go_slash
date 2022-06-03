<template>
  <div>
    <div class="d-flex mb-4">
      <div class="alert alert-primary flex-fill m-0" role="alert">
        Drag and drop to prioritize team shortcuts in case of duplicates
      </div>
      <div class="align-self-center">
        <button
          v-if="
            capabilities.includes('TeamsWrite') ||
            capabilities.includes('TeamsWriteWithValidation')
          "
          @click="start_create"
          class="btn btn-lg btn-primary ms-2"
          aria-label="Start creating team"
          data-bs-toggle="modal"
          data-bs-target="#create_modal"
        >
          Create
          <i class="icon-plus ms-1"></i>
        </button>
        <button
          v-if="
            capabilities.includes('TeamsWrite') ||
            user_teams_capabilities.includes('TeamsWrite')
          "
          @click="set_administer"
          class="btn btn-lg ms-2"
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
    <CreateTeamModal
      :start_create_count="start_create_count"
      @created="team_created"
      :capabilities="capabilities"
    />
  </div>
</template>

<script lang="ts">
import axios from "axios";
import { defineComponent } from "vue";
import {
  Team,
  UserTeamLink,
  sort_by_rank,
  TeamCapability,
  Capability,
  ALL_TEAM_CAPABILITIES,
} from "../../models";
import TeamList from "./TeamList.vue";
import UserTeamList from "./UserTeamList.vue";
import _ from "lodash";
import CreateTeamModal from "./CreateTeamModal.vue";

interface Window {
  teams: Team[];
  capabilities: Capability[];
}

let win = window as unknown as Window;
const TEAMS = win.teams;

interface Data {
  teams: Team[];
  capabilities: Capability[];
  administer: boolean;
  start_create_count: number;
}

const CAPABILITIES = win.capabilities;

export default defineComponent({
  name: "Partial",
  components: { UserTeamList, TeamList, CreateTeamModal },
  data(): Data {
    return {
      teams: TEAMS,
      capabilities: CAPABILITIES,
      administer: false,
      start_create_count: 0,
    };
  },
  computed: {
    user_teams(): Team[] {
      return this.teams.filter((team) => team.user_link).sort(sort_by_rank);
    },
    other_teams(): Team[] {
      return this.teams.filter((team) => !team.user_link);
    },
    next_rank(): number {
      return this.user_teams.reduce((rank, team) => {
        return Math.max(rank ?? 0, team.user_link?.rank ?? 0);
      }, 0);
    },
    user_teams_capabilities(): TeamCapability[] {
      return this.user_teams.flatMap(
        (team) => team.user_link?.capabilities ?? []
      );
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
              is_accepted: !team.is_private,
              rank: this.next_rank,
              // TODO: real capacities
              capabilities: [],
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
            if (team) {
              team.is_accepted = true;
            }
          }
        })
        .catch(console.error);
    },
    team_created(team: Team) {
      this.teams = [
        ...this.teams,
        {
          ...team,
          user_link: {
            rank: this.next_rank,
            is_accepted: true,
            capabilities: ALL_TEAM_CAPABILITIES,
          },
        },
      ];
    },
    start_create() {
      this.start_create_count = this.start_create_count + 1;
    },
  },
});
</script>

<style scoped></style>
