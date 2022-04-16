<template>
  <div role="list" class="list-group" :aria-label="ariaLabel">
    <TeamRow v-for="team in teams" :key="team.slug" :team="team" @join="join" />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import TeamRow from "./TeamRow.vue";

export default defineComponent({
  name: "TeamList",
  components: {
    TeamRow,
  },
  props: {
    teams: Array,
    ariaLabel: String,
  },
  emits: ["join", "leave"],
  methods: {
    join(slug: string) {
      this.$emit("join", slug);
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
