<template>
  <div role="list" class="list-group" :aria-label="ariaLabel">
    <draggable :list="teams" @change="move" item-key="slug" group="people">
      <template #item="{ element }">
        <TeamRow :team="element" @leave="leave" />
      </template>
    </draggable>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import draggable from "vuedraggable";
import { Team, UserTeamLink } from "./main";
import TeamRow from "./TeamRow.vue";

export default defineComponent({
  name: "UserTeamList",
  components: { draggable, TeamRow },
  props: {
    teams: { required: true, type: Object as PropType<Team[]> },
    ariaLabel: String,
  },
  emits: ["leave", "change_ranks"],
  methods: {
    leave(slug: string) {
      this.$emit("leave", slug);
    },
    move(event: {
      moved: {
        oldIndex: number;
        newIndex: number;
      };
    }) {
      let { oldIndex, newIndex } = event.moved;
      const upperIndex = oldIndex > newIndex ? oldIndex : newIndex;
      const lowerIndex = oldIndex < newIndex ? oldIndex : newIndex;
      const moveOperation = oldIndex > newIndex ? 1 : -1;

      let new_ranks = this.teams
        .map((team) => {
          const link = team.user_link as UserTeamLink;

          if (link.rank === oldIndex) {
            return {
              [team.slug]: newIndex,
            };
          } else if (link.rank >= lowerIndex && link.rank <= upperIndex) {
            return {
              [team.slug]: link.rank + moveOperation,
            };
          }
        })
        .filter((team) => team)
        .reduce((acc, value) => {
          return { ...acc, ...value };
        });
      this.$emit("change_ranks", new_ranks);
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

[role="listitem"] {
  cursor: grab;
}
</style>
