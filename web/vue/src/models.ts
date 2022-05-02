export type Capabilities = "ShortcutWrite";

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
  rank_modified?: boolean;
}

export function sort_by_rank(a: Team, b: Team): number {
  return (
    (a.user_link as UserTeamLink).rank - (b.user_link as UserTeamLink).rank
  );
}

export interface User {
  mail: string;
  capabilities: Capabilities[];
}
