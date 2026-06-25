import { useState } from "react";

import { Word } from "./Word";
import styles from "./WordList.module.css";

const NO_DEFINITION_TITLE = "No Definitions Found";

export type DefinitionState =
  | { status: "loading" }
  | {
      status: "loaded";
      meanings: DefinitionMeaning[];
      pronunciation?: DefinitionPronunciation;
      sourceUrls: string[];
    }
  | { status: "not-found" }
  | { status: "error" };

export type DefinitionMeaning = {
  partOfSpeech: string;
  definitions: string[];
};

export type DefinitionPronunciation = {
  text?: string;
  audio?: string;
};

type DictionaryEntry = {
  title?: string;
  sourceUrls?: string[];
  phonetics?: {
    text?: string;
    audio?: string;
  }[];
  meanings?: {
    partOfSpeech?: string;
    definitions?: {
      definition?: string;
    }[];
  }[];
};

export type DictionaryCache = Record<string, DefinitionState>;

function getDefinitionMeanings(data: DictionaryEntry[]): DefinitionMeaning[] {
  return data.flatMap((entry) =>
    (entry.meanings ?? []).flatMap((meaning) => {
      const definitions = (meaning.definitions ?? [])
        .map(({ definition }) => definition)
        .filter((definition): definition is string => definition !== undefined);

      if (definitions.length === 0) return [];

      return [
        {
          partOfSpeech: meaning.partOfSpeech ?? "Meaning",
          definitions,
        },
      ];
    }),
  );
}

function getPronunciation(
  data: DictionaryEntry[],
): DefinitionPronunciation | undefined {
  const phonetics = data.flatMap((entry) => entry.phonetics ?? []);
  const normalized = phonetics
    .map(({ text, audio }) => ({
      text,
      audio: audio && audio.trim() !== "" ? audio : undefined,
    }))
    .filter(({ text, audio }) => text !== undefined || audio !== undefined);

  return (
    normalized.find(({ text, audio }) => text !== undefined && audio !== undefined) ??
    normalized[0]
  );
}

function getSourceUrls(data: DictionaryEntry[]): string[] {
  const sourceUrls = data
    .flatMap((entry) => entry.sourceUrls ?? [])
    .filter((sourceUrl) => sourceUrl.trim() !== "");

  return Array.from(new Set(sourceUrls));
}

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

type WordEntry = { word: string; kind: "found" | "missing" | "extra" };

function mergeGroups(
  found: [number, string[]][],
  missing: [number, string[]][],
  extra: [number, string[]][],
): [number, WordEntry[]][] {
  const merged: Record<number, WordEntry[]> = {};

  const add = (groups: [number, string[]][], kind: WordEntry["kind"]) => {
    for (const [length, words] of groups) {
      (merged[length] ??= []).push(...words.map((word) => ({ word, kind })));
    }
  };

  add(found, "found");
  add(missing, "missing");
  add(extra, "extra");

  return Object.entries(merged)
    .map(([key, entries]): [number, WordEntry[]] => [
      Number(key),
      entries.sort((a, b) => a.word.localeCompare(b.word)),
    ])
    .sort(([a], [b]) => a - b);
}

export type Words = {
  found?: string[];
  missing?: string[];
  extra?: string[];
  /** used for create */
  all?: string[];
};

type WordListContentProps = {
  words: Words;
  selectedWord: string | null;
  definitions: DictionaryCache;
  onSelectWord: (word: string) => void;
  onCloseDefinition: () => void;
};

function PlayWordList({
  words,
  selectedWord,
  definitions,
  onSelectWord,
  onCloseDefinition,
}: WordListContentProps) {
  const sortedFoundWords = groupAndSort(words.found!);
  const sortedMissingWords = groupAndSort(words.missing!);
  const sortedExtraWords = groupAndSort(words.extra!);

  const grouped = mergeGroups(
    sortedFoundWords,
    sortedMissingWords,
    sortedExtraWords,
  );

  return (
    <div className={styles.wordList}>
      {grouped.map(([length, entries]) => {
        const byKind = entries.reduce(
          (acc, entry) => {
            (acc[entry.kind] ??= []).push(entry);
            return acc;
          },
          {} as Record<WordEntry["kind"], WordEntry[]>,
        );

        return (
          <li key={length}>
            <p className={styles.lengthLabel}>{length} letters: </p>
            {(["found", "missing", "extra"] as const).map((kind) =>
              byKind[kind]?.map(({ word }) => (
                <Word
                  key={word}
                  word={word}
                  className={styles[kind]}
                  selectedWord={selectedWord}
                  definitions={definitions}
                  onSelect={onSelectWord}
                  onClose={onCloseDefinition}
                />
              )),
            )}
          </li>
        );
      })}
    </div>
  );
}

function CreateWordList({
  words,
  selectedWord,
  definitions,
  onSelectWord,
  onCloseDefinition,
}: WordListContentProps) {
  const sortedWords = groupAndSort(words.all!);

  return (
    <div className={styles.wordList}>
      {sortedWords.map(([length, words], idx) => {
        return (
          <div key={idx}>
            <p className={styles.lengthLabel}>{length} letters</p>
            <p>
              {words.map((word) => (
                <Word
                  key={word}
                  word={word}
                  selectedWord={selectedWord}
                  definitions={definitions}
                  onSelect={onSelectWord}
                  onClose={onCloseDefinition}
                />
              ))}
            </p>
          </div>
        );
      })}
    </div>
  );
}

export function WordList({
  listType,
  words,
}: {
  listType: "Create" | "Play";
  words: Words;
}) {
  const [definitions, setDefinitions] = useState<DictionaryCache>({});
  const [selectedWord, setSelectedWord] = useState<string | null>(null);

  async function selectWord(word: string) {
    const normalizedWord = word.toLowerCase();
    const cachedDefinition = definitions[normalizedWord];

    setSelectedWord(normalizedWord);

    if (cachedDefinition && cachedDefinition.status !== "error") return;

    setDefinitions((currentDefinitions) => ({
      ...currentDefinitions,
      [normalizedWord]: { status: "loading" },
    }));

    try {
      const response = await fetch(
        `https://api.dictionaryapi.dev/api/v2/entries/en/${encodeURIComponent(normalizedWord)}`,
      );
      const data = (await response.json()) as
        | DictionaryEntry[]
        | DictionaryEntry;

      if (!Array.isArray(data) && data.title === NO_DEFINITION_TITLE) {
        setDefinitions((currentDefinitions) => ({
          ...currentDefinitions,
          [normalizedWord]: { status: "not-found" },
        }));
        return;
      }

      if (!response.ok || !Array.isArray(data)) {
        setDefinitions((currentDefinitions) => ({
          ...currentDefinitions,
          [normalizedWord]: { status: "error" },
        }));
        return;
      }

      const definitionMeanings = getDefinitionMeanings(data);
      const pronunciation = getPronunciation(data);
      const sourceUrls = getSourceUrls(data);

      setDefinitions((currentDefinitions) => ({
        ...currentDefinitions,
        [normalizedWord]:
          definitionMeanings.length > 0
            ? {
                status: "loaded",
                meanings: definitionMeanings,
                pronunciation,
                sourceUrls,
              }
            : { status: "not-found" },
      }));
    } catch {
      setDefinitions((currentDefinitions) => ({
        ...currentDefinitions,
        [normalizedWord]: { status: "error" },
      }));
    }
  }

  const wordListProps = {
    words,
    selectedWord,
    definitions,
    onSelectWord: selectWord,
    onCloseDefinition: () => setSelectedWord(null),
  };

  if (listType === "Create") {
    return <CreateWordList {...wordListProps} />;
  } else {
    return <PlayWordList {...wordListProps} />;
  }
}
