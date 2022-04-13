<template>
  <div role="list" class="list-group" :aria-label="ariaLabel">
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

export default defineComponent({
  name: "TeamList",
  props: {
    teams: Array,
    ariaLabel: String,
  },
  emits: ["join", "leave"],
  methods: {
    join(slug: string) {
      this.$emit("join", slug);
    },
    leave(slug: string) {
      this.$emit("leave", slug);
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
