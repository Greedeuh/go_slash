<template>
  <a
    :href="`/go/teams/${team.slug}`"
    role="listitem"
    class="list-group-item-action list-group-item d-flex justify-content-between align-items-start"
  >
    <div class="ms-2 me-auto content">
      <i v-if="team.user_link" class="icon-reorder me-2"></i>
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
      <button
        v-if="administer && !team.is_accepted"
        @click.prevent="accept(team.slug)"
        type="button"
        class="btn btn-success"
        aria-label="Accept team"
      >
        Accept
        <i class="icon-check-sign ms-1"></i>
      </button>
      <button
        v-if="
          administer &&
          (capabilities.includes('TeamsWrite') ||
            team_capabilities.includes('TeamsWrite'))
        "
        @click.prevent="delete_team(team.slug)"
        type="button"
        class="btn btn-danger"
        aria-label="Delete team"
      >
        <i class="icon-trash"></i>
      </button>
    </div>
  </a>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import { Team, TeamCapability } from "../../models";

export default defineComponent({
  name: "TeamRow",
  props: {
    team: { required: true, type: Object as PropType<Team> },
    administer: Boolean,
    capabilities: Array,
  },
  emits: ["join", "leave", "delete_team", "accept"],
  computed: {
    team_capabilities(): TeamCapability[] {
      if (this.team.user_link?.is_accepted) {
        return this.team.user_link.capabilities ?? [];
      } else {
        return [];
      }
    },
  },
  methods: {
    join(slug: string) {
      this.$emit("join", slug);
    },
    leave(slug: string) {
      this.$emit("leave", slug);
    },
    delete_team(slug: string) {
      this.$emit("delete_team", slug);
    },
    accept(slug: string) {
      this.$emit("accept", slug);
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

.list-group {
  margin-bottom: 16px;
}
</style>
