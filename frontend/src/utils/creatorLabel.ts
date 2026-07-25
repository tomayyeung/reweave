export const officialName = "Reverse Search Team";

export type Creator = {
  username: string;
  displayName: string | null;
  official: boolean;
};

export function creatorLabel(creator: Creator) {
  if (creator.official) {
    return officialName;
  }

  return creator.displayName ?? creator.username;
}
