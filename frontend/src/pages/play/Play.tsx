import { useState, useEffect } from "react";

import { Board, BLANK, HOLE } from "@/components/Board";
import { WordList, allWordsFound } from "@components/WordList";
import type { Words } from "@components/WordList";
import { Wrapper } from "@components/Wrapper";
import { useParams } from "react-router-dom";
import { API_URL } from "@/config";

import { check, load_puzzle as loadPuzzle } from "@wasm/frontend";
import { Popup } from "@/components/Popup";
import styles from "./Play.module.css";

type PendingReveal = "solution" | "random" | "selected";

export default function PlayPage() {
  const { puzzleId } = useParams();

  const [puzzleFetched, setPuzzleFetched] = useState<boolean | undefined>(
    undefined,
  );

  const [boardLetters, setBoardLetters] = useState("");
  const [hardSet, setHardSet] = useState<boolean[]>([]);

  const [puzzleName, setPuzzleName] = useState("");
  const [w, setWidth] = useState(0);
  const [h, setHeight] = useState(0);

  const [answer, setAnswer] = useState("");
  const [pendingReveal, setPendingReveal] = useState<PendingReveal | undefined>();
  const [gaveUp, setGaveUp] = useState(false);
  const [usedHint, setUsedHint] = useState(false);
  const [selectedTile, setSelectedTile] = useState(-1);

  const words: Words = puzzleFetched
    ? check(boardLetters)
    : { found: [], missing: [], extra: [] };
  const complete = puzzleFetched && allWordsFound(words);
  const showRevealActions = puzzleFetched === true && !complete && !gaveUp;

  useEffect(() => {
    const route = `${API_URL}/api/puzzle/${puzzleId}`;
    fetch(route)
      .then((res) => res.json())
      .then((puzzle) => {
        console.log(puzzle);

        try {
          // load puzzle for wasm
          loadPuzzle(puzzle);

          // then load puzzle for rendering
          setPuzzleName(puzzle.name);
          setWidth(puzzle.width);
          setHeight(puzzle.height);

          const initialLetters = puzzle.letters;

          console.log(initialLetters);

          // intialize board w/ letters
          // any initial letters means they are hard set
          setBoardLetters(initialLetters);
          setHardSet([...initialLetters].map((letter) => letter !== BLANK));

          // initialize answer
          setAnswer(puzzle.answer);
          setSelectedTile(-1);
          setUsedHint(false);
          setGaveUp(false);

          setPuzzleFetched(true);
        } catch {
          const error: string = puzzle.error;
          if (error.startsWith("invalid puzzle id")) {
            setPuzzleFetched(false);
          }
        }
      });
  }, [puzzleId]);

  function revealTile(idx: number) {
    const answerTile = answer[idx];

    if (idx < 0 || idx >= answer.length || answerTile === undefined) {
      return;
    }

    const revealedTile = answerTile === BLANK ? HOLE : answerTile;
    setBoardLetters([...boardLetters].with(idx, revealedTile).join(""));
    setHardSet(hardSet.with(idx, true));
    setUsedHint(true);
  }

  function revealRandomTile() {
    const eligibleTiles = [...answer]
      .map((letter, idx) => ({ letter, idx }))
      .filter(
        ({ letter, idx }) => letter !== BLANK && letter !== HOLE && !hardSet[idx],
      );

    if (eligibleTiles.length === 0) {
      return;
    }

    const { idx } = eligibleTiles[Math.floor(Math.random() * eligibleTiles.length)];
    revealTile(idx);
  }

  function getRevealPopupText(reveal: PendingReveal) {
    switch (reveal) {
      case "solution":
        return "Reveal the full solution?";
      case "random":
        return "Reveal a random tile?";
      case "selected":
        return "Reveal the selected tile?";
    }
  }

  function getRevealConfirmText(reveal: PendingReveal) {
    switch (reveal) {
      case "solution":
        return "Reveal solution";
      case "random":
        return "Reveal random tile";
      case "selected":
        return "Reveal selected tile";
    }
  }

  function confirmReveal() {
    switch (pendingReveal) {
      case "solution":
        setBoardLetters(answer);
        setGaveUp(true);
        break;
      case "random":
        revealRandomTile();
        break;
      case "selected":
        revealTile(selectedTile);
        break;
    }

    setPendingReveal(undefined);
  }

  function getMain(fetchedStatus: boolean | undefined) {
    switch (fetchedStatus) {
      case undefined:
        return <p>Loading puzzle...</p>;
      case false:
        return <p>Puzzle not found</p>;
      default:
        return (
          <Board
            boardType="Play"
            filteringLetters={false}
            width={w}
            height={h}
            boardLetters={boardLetters}
            hardSet={hardSet}
            setBoardLetters={setBoardLetters}
            selectedTile={selectedTile}
            setSelectedTile={setSelectedTile}
          />
        );
    }
  }

  return (
    <main>
      <Wrapper>
        <div className={styles.boardPanel}>
          <div className={styles.header}>
            <h3>Puzzle: {puzzleName}</h3>
            {showRevealActions ? (
              <div className={styles.revealActions}>
                <button
                  type="button"
                  className={styles.revealButton}
                  onClick={() => setPendingReveal("solution")}
                >
                  Reveal solution
                </button>
                <button
                  type="button"
                  className={styles.revealButton}
                  onClick={() => setPendingReveal("random")}
                >
                  Reveal random tile
                </button>
                <button
                  type="button"
                  className={styles.revealButton}
                  disabled={selectedTile === -1}
                  onClick={() => setPendingReveal("selected")}
                >
                  Reveal selected tile
                </button>
              </div>
            ) : (
              <></>
            )}
            <h4 hidden={!complete || gaveUp || usedHint}>Completed!</h4>
            <h4 hidden={!complete || gaveUp || !usedHint}>Completed with hints!</h4>
            <h4 className={styles.revealedStatus} hidden={!gaveUp}>
              Solution revealed.
            </h4>
          </div>
          <div className={styles.boardSlot}>{getMain(puzzleFetched)}</div>
        </div>
        <WordList listType="Play" words={words} />
      </Wrapper>

      {complete && !gaveUp ? (
        <Popup text="Congratulations! Puzzle completed." />
      ) : (
        <></>
      )}

      {pendingReveal !== undefined && showRevealActions ? (
        <Popup
          text={getRevealPopupText(pendingReveal)}
          confirmText={getRevealConfirmText(pendingReveal)}
          cancelText="Cancel"
          onConfirm={confirmReveal}
          onCancel={() => setPendingReveal(undefined)}
        />
      ) : (
        <></>
      )}
    </main>
  );
}
