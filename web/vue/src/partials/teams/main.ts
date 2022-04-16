import { createApp } from "vue";
import Partial from "./Partial.vue";

createApp(Partial).mount("#vue-partial");

export interface Team {
  slug: string;
  title: string;
  is_private: boolean;
  is_accepted: boolean;
  user_link?: UserTeamLink;
}

export interface UserTeamLink {
  is_admin: boolean;
  is_accepted: boolean;
  rank: number;
}
