export type Capability =
  | "Features"
  | "TeamsRead"
  | "TeamsWrite"
  | "TeamsWriteWithValidation"
  | "UsersAdmin"
  | "UsersTeamsRead"
  | "UsersTeamsWrite";

export type TeamCapability = "ShortcutsWrite" | "TeamsWrite";

export interface Team {
  slug: string;
  title: string;
  is_private: boolean;
  is_accepted: boolean;
  user_link?: UserTeamLink;
}

export interface UserTeamLink {
  capabilities: TeamCapability[];
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
  capabilities: Capability[];
}

export const ALL_CAPABILITIES = [
  "Features",
  "TeamsRead",
  "TeamsWrite",
  "TeamsWriteWithValidation",
  "UsersAdmin",
  "UsersTeamsRead",
  "UsersTeamsWrite",
].sort() as Capability[];

export const ALL_TEAM_CAPABILITIES = [
  "ShortcutsWrite",
  "TeamsWrite",
].sort() as TeamCapability[];
