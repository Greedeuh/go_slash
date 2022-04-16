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
    </div>
  </a>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "TeamRow",
  props: {
    team: Object,
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
