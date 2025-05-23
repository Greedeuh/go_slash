<template>
  <div>
    <div v-if="shortcut" role="alert" class="alert alert-warning">
      Shortcut "{{ shortcut.shortcut }}" does not exist yet.
    </div>
    <TeamEditor
      v-if="
        team &&
        (capabilities.includes('TeamsWrite') ||
          team.user_links.some(
            (u) => u.user_mail === mail && u.capabilities.includes('TeamsWrite')
          ))
      "
      :team="team"
      @save="save_team"
      @kick="kick"
      @accept="accept"
    />
    <SearchBar
      v-model="search"
      @keydown="reset_index_if_letter"
      :administer="administer"
      @on-administer="set_administer"
      @enter="go_selected(-1)"
      :editor="shortcut_write"
    />
    <ShortcutInput
      v-if="(administer || shortcut) && shortcut_write"
      @save="add"
      :initial_shortcut="shortcut"
      :admin_teams="admin_teams"
    />
    <ShortcutList
      :shortcuts="fuzzed_or_all"
      :selected_index="selected_index"
      :administer="administer"
      :admin_teams="admin_teams"
      @delete_shortcut="delete_shortcut"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import Fuse from "fuse.js";
import axios from "axios";

import SearchBar from "./Search.vue";
import ShortcutList from "./ShortcutList.vue";
import ShortcutInput from "./ShortcutInput.vue";
import TeamEditor from "./TeamEditor.vue";
import { User, Capability, UserTeamLink, Team } from "../../models";

interface Window {
  context: WindowContext;
}

interface WindowContext {
  shortcut?: Shortcut;
  shortcuts: Shortcut[];
  user?: User;
  // TODO rename or change => it's team with capability shortcut write
  teams?: Team[];
  team?: Team;
}

export interface Shortcut {
  shortcut: string;
  url: string;
  team_slug: string;
  new: boolean;
}

function setup_fuse(shortcuts: Shortcut[]) {
  return new Fuse(shortcuts, {
    keys: [
      { name: "shortcut", weight: 2 },
      { name: "url", weight: 1 },
    ],
  });
}

let win = window as unknown as Window;
const CONTROL_KEYS = ["ArrowUp", "ArrowDown", "Enter", "Tab", "Escape"];
const SHORTCUTS = win.context.shortcuts;
const SHORTCUT = win.context.shortcut;
const CAPABILITIES = win.context.user?.capabilities;
const MAIL = win.context.user?.mail;
const ADMIN_TEAMS = win.context.teams;
const TEAM = win.context.team;

let key_press: (e: KeyboardEvent) => void;

interface Data {
  selected_index: -1;
  shortcuts: Shortcut[];
  administer: boolean;
  fuse: Fuse<Shortcut>;
  search: string;
  shortcut?: Shortcut;
  capabilities: Capability[];
  admin_teams?: Team[];
  team?: Team;
  mail?: string;
}

export default defineComponent({
  name: "Partial",
  components: { SearchBar, ShortcutList, ShortcutInput, TeamEditor },
  data(): Data {
    return {
      selected_index: -1,
      shortcuts: SHORTCUTS,
      administer: false,
      fuse: setup_fuse(SHORTCUTS),
      search: SHORTCUT ? SHORTCUT.shortcut : "",
      shortcut: SHORTCUT,
      capabilities: CAPABILITIES ?? [],
      admin_teams: ADMIN_TEAMS,
      team: TEAM,
      mail: MAIL,
    };
  },
  computed: {
    fuzzed_or_all(): Shortcut[] {
      let shortcuts_fuzzed = this.fuse
        .search(this.search)
        .map((res) => res.item);
      return shortcuts_fuzzed.length ? shortcuts_fuzzed : this.shortcuts;
    },
    shortcut_write(): boolean {
      return this.admin_teams !== undefined && this.admin_teams.length > 0;
    },
  },
  created() {
    key_press = (e: KeyboardEvent) => {
      let key = e.key;

      if (
        CONTROL_KEYS.includes(key) &&
        !(window as any).focus_flag // eslint-disable-line
      ) {
        e.preventDefault();

        if (key === "Escape") this.administer = false;

        if (key === "ArrowUp" && this.selected_index >= 0)
          this.selected_index -= 1;

        if (
          key === "ArrowDown" &&
          this.selected_index + 1 < this.fuzzed_or_all.length
        ) {
          this.selected_index += 1;
        }

        if (key === "Tab")
          this.take_selected(this.selected_index < 0 ? 0 : this.selected_index);
        if (key === "Enter") this.go_selected(this.selected_index);
      }
    };
    window.addEventListener("keydown", key_press);
  },
  unmounted() {
    window.removeEventListener("keydown", key_press);
  },
  methods: {
    reset_index_if_letter({ key }: KeyboardEvent) {
      if (key.length === 1) this.selected_index = -1;
    },
    take_selected(selected_index: number) {
      this.search = this.fuzzed_or_all[selected_index].shortcut;
      this.selected_index = -1;
    },
    go_selected(selected_index: number) {
      if (selected_index === -1) {
        window.location.href = "/" + encodeURI(this.search);
      } else {
        window.location.href =
          "/" + this.fuzzed_or_all[selected_index].shortcut;
      }
    },
    set_administer() {
      this.administer = !this.administer;
    },
    delete_shortcut({
      shortcut,
      team_slug,
    }: {
      shortcut: string;
      team_slug: string;
    }) {
      axios.delete(`/${shortcut}?team=${team_slug}`).then((res) => {
        if (res.status === 200) {
          this.shortcuts = this.shortcuts.filter(
            (s) => s.shortcut !== shortcut || s.team_slug !== team_slug
          );
          this.fuse.setCollection(this.shortcuts);
        }
      });
    },
    add({
      shortcut,
      url,
      team_slug,
      on_success,
    }: {
      shortcut: string;
      url: string;
      team_slug: string;
      on_success: () => void;
    }) {
      axios.put(`/${shortcut}?team=${team_slug}`, { url }).then((res) => {
        if (res.status === 200) {
          const shortcuts = this.shortcuts.filter(
            (s) => s.shortcut !== shortcut || s.team_slug !== team_slug
          );
          shortcuts.unshift({ shortcut, url, team_slug: team_slug, new: true });
          this.shortcuts = shortcuts;
          this.fuse.setCollection(this.shortcuts);
          on_success();
        }
      });
    },
    save_team({ slug, team }: { slug: string; team: Team }) {
      axios.patch(`/go/teams/${slug}`, team).then((res) => {
        if (res.status === 200) {
          this.team = { ...this.team, ...team };
        }
      });
    },

    kick(user_link: UserTeamLink) {
      const team = this.team as Team;
      axios
        .delete(`/go/teams/${team.slug}/users/${user_link.user_mail}`)
        .then((res) => {
          if (res.status === 200) {
            // FIX not the good way to do this
            team.user_links = team.user_links?.filter((u) => u !== user_link);
          }
        });
    },
    accept(user_link: UserTeamLink) {
      const team = this.team as Team;
      axios
        .put(
          `/go/teams/${team.slug}/users/${user_link.user_mail}/is_accepted/true`
        )
        .then((res) => {
          if (res.status === 200) {
            // FIX not the good way to do this
            team.user_links = team.user_links?.filter((u) => u !== user_link);
          }
        });
    },
  },
});
</script>

<style></style>
