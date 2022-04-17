<template>
  <div role="list" class="list-group" :aria-label="ariaLabel">
    <draggable :list="teams" @change="move" item-key="slug" group="people">
      <template #item="{ element }">
        <TeamRow
          :team="element"
          @leave="leave"
          :administer="administer"
          @delete_team="delete_team"
          @accept="accept"
        />
      </template>
    </draggable>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import draggable from "vuedraggable";
import { Team, UserTeamLink, sort_by_rank } from "./main";
import TeamRow from "./TeamRow.vue";
import _ from "lodash";

const clean_teams_rank = (acc: Team[], team: Team) => {
  const last_rank = (acc[acc.length - 1].user_link as UserTeamLink).rank;
  team = _.cloneDeep(team);
  const link = team.user_link as UserTeamLink;
  if (link.rank !== last_rank + 1) {
    link.rank = last_rank + 1;
    link.rank_modified = true;
  }
  return [...acc, team];
};

export default defineComponent({
  name: "UserTeamList",
  components: { draggable, TeamRow },
  props: {
    teams: { required: true, type: Object as PropType<Team[]> },
    ariaLabel: String,
    administer: Boolean,
  },
  emits: ["leave", "change_ranks", "delete_team", "accept"],
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

      const choose_next_rank = (team: Team) => {
        const link = team.user_link as UserTeamLink;

        if (link.rank === oldIndex) {
          // the team moved

          return {
            [team.slug]: newIndex,
          };
        } else if (link.rank >= lowerIndex && link.rank <= upperIndex) {
          // team between drag & drop

          return {
            [team.slug]: link.rank + moveOperation,
          };
        } else if (link.rank_modified) {
          // team that clean_teams_rank modified because rank were not perfect at begening
          return { [team.slug]: link.rank };
        }
      };

      let new_ranks = _.clone(this.teams)
        .sort(sort_by_rank)
        .reduce(clean_teams_rank, [
          {
            user_link: {
              rank: -1,
            },
          },
        ] as Team[])
        .map(choose_next_rank)
        .filter((team) => team)
        .reduce((acc, value) => {
          return { ...acc, ...value };
        });
      this.$emit("change_ranks", new_ranks);
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

[role="listitem"] {
  cursor: grab;
}
</style>
