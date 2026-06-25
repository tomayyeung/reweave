import type { PlayWords, Words } from "./types";

export type { CreateWords, PlayWords, Words } from "./types";
export { WordList } from "./WordList";

export function wordsAsStringArr(words: Words) {
  if (words.kind === "create") {
    return words.all;
  }

  return [...words.found, ...words.missing, ...words.extra];
}

export function allWordsFound(words: PlayWords) {
  return words.missing.length === 0 && words.extra.length === 0;
}
