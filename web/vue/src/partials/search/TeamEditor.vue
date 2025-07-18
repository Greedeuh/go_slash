<template>
  <div aria-label="Team editor">
    <form @submit.prevent="save" class="bg-light border rounded mb-4 p-4">
      <div class="input-group mb-2">
        <label class="input-group-text" for="title">Title</label>
        <input
          type="text"
          class="form-control"
          name="title"
          id="title"
          v-model="inner_team.title"
        />
      </div>
      <div class="form-check form-switch">
        <input
          class="form-check-input"
          type="checkbox"
          role="switch"
          name="is_private"
          id="is_private"
          v-model="inner_team.is_private"
        />
        <label class="form-check-label" for="is_private"
          >Private</label
        >
      </div>
      <div class="form-check form-switch">
        <input
          class="form-check-input"
          type="checkbox"
          role="switch"
          name="is_accepted"
          id="is_accepted"
          v-model="inner_team.is_accepted"
        />
        <label class="form-check-label" for="is_accepted"
          >Enable</label
        >
      </div>
      <input type="submit" class="btn btn-primary mt-2" value="Save" />
    </form>
    <UserList
      :user_links="team.user_links"
      @toggle="toggle"
      @kick="kick"
      @accept="accept"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import _ from "lodash";
import { Team, TeamCapability, UserTeamLink } from "../../models";
import UserList from "./UserList.vue";
import axios from "axios";

interface Data {
  inner_team: Team;
}

export default defineComponent({
  name: "TeamEditor",
  components: {
    UserList,
  },
  props: {
    team: { required: true, type: Object as PropType<Team> },
  },
  data(): Data {
    return { inner_team: _.clone(this.team) as Team };
  },
  emits: ["save", "kick", "accept"],
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
    toggle(
      user_link: UserTeamLink,
      { capability, value }: { capability: TeamCapability; value: boolean }
    ) {
      if (value) {
        axios
          .put(
            `/go/teams/${this.team.slug}/users/${user_link.user_mail}/capabilities/${capability}`
          )
          .then((res) => {
            if (res.status === 200) {
              user_link.capabilities = [...user_link.capabilities, capability];
            }
          });
      } else {
        axios
          .delete(
            `/go/teams/${this.team.slug}/users/${user_link.user_mail}/capabilities/${capability}`
          )
          .then((res) => {
            if (res.status === 200) {
              user_link.capabilities = user_link.capabilities.filter(
                (c) => c !== capability
              );
            }
          });
      }
    },
    kick(user_link: UserTeamLink) {
      this.$emit("kick", user_link);
    },
    accept(user_link: UserTeamLink) {
      this.$emit("accept", user_link);
    },
  },
});
</script>

<style></style>
