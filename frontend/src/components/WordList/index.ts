import type { Words } from "./WordList";

export type { Words } from "./WordList";
export { WordList } from "./WordList";

export function wordsAsStringArr(words: Words) {
  return [...words.found ?? [], ...words.missing ?? [], ...words.extra ?? [], ...words.all ?? []];
}

export function allWordsFound(words: Words) {
  return words.missing!.length === 0 && words.extra!.length === 0;
}