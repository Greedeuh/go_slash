<template>
  <div role="list" class="list-group">
    <a
      v-for="team in teams"
      :href="`/go/teams/${team.slug}`"
      :key="team.slug"
      role="listitem"
      class="list-group-item d-flex justify-content-between align-items-start"
      >{{ team.title }}
      <div>
        <i v-if="team.is_private" class="icon-lock"></i
        ><i v-if="!team.is_accepted" class="icon-check-empty ms-2"></i
        ><i v-if="team.is_accepted" class="icon-check ms-2"></i></div
    ></a>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

interface Window {
  teams: string;
}

let win = window as unknown as Window;
const TEAMS = win.teams;

export default defineComponent({
  name: "Partial",
  data() {
    return {
      teams: TEAMS,
    };
  },
});
</script>

<style>
i:before {
  width: 10px;
}
</style>
