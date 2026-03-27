import styles from "./WordList.module.css";

function groupAndSort(words: string[]): [number, string[]][] {
  const groups: Record<number, string[]> = {};

  for (const word of words) {
    const len = word.length;
    (groups[len] ??= []).push(word);
  }

  return Object.entries(groups)
    .map(([key, value]): [number, string[]] => [Number(key), value.sort()])
    .sort(([a], [b]) => a - b);
}

export type Words = {
  found: string[],
  missing: string[],
  extra: string[]
}

export function WordList({ words }: { words: Words }) {
  const sortedFoundWords = groupAndSort(words.found);
  const sortedMissingWords = groupAndSort(words.missing);
  const sortedExtraWords = groupAndSort(words.extra);

  return (
    <div className={styles.wordList}>
      {/* {sortedWords.map(([length, words], idx) => {
        // console.log(length, words);
        return <div key={idx}>
          <p className={styles.lengthLabel}>{length} letters</p>
          <p>{words.join(" ")}</p>
        </div>;
      })} */}
    </div>
  );
}
