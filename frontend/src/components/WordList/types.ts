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

export type DictionaryCache = Record<string, DefinitionState>;

export type CreateWords = {
  kind: "create";
  all: string[];
};

export type PlayWords = {
  kind: "play";
  found: string[];
  missing: string[];
  extra: string[];
};

export type Words = CreateWords | PlayWords;
