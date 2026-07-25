import { useEffect, useRef, useState } from "react";
import type { CSSProperties } from "react";
import styles from "./Board.module.css";

/** Serialized fillable blank used by Rust board parsing and the UI. */
export const BLANK = "_";
/** Serialized permanent hole used by Rust board parsing and the UI. */
export const HOLE = "!";

type TileProps = {
  boardType: "Create" | "Play";
  letter: string;
  idx: number;
  isHardSet: boolean;
  isHole: boolean;
  updateSelectedTile: (idx: number) => void;
  isSelected: boolean;
};

type BoardProps = {
  /** Create mode permits hole toggles; play mode protects starting clues. */
  boardType: "Create" | "Play";
  /**
   * Create-mode clue-selection state. Before locking, `hardSet` means normal
   * active tiles; after locking, it means letters visible to players.
   */
  filteringLetters: boolean;
  width: number;
  height: number;
  boardLetters: string;
  hardSet: boolean[];
  /**
   * Only needed while creating. During play, hard-set letters are immutable.
   */
  setHardSet?: React.Dispatch<React.SetStateAction<boolean[]>>;
  setBoardLetters: React.Dispatch<React.SetStateAction<string>>;
  selectedTile?: number;
  setSelectedTile?: React.Dispatch<React.SetStateAction<number>>;
  /** Called by play mode when the user changes a letter, used to count plays. */
  onUserLetterPlaced?: () => void;
};

function Tile({
  boardType,
  letter,
  idx,
  isHardSet,
  isHole,
  updateSelectedTile,
  isSelected,
}: TileProps) {
  return (
    <div
      className={
        `${styles.tile} ` +
        `${isHardSet ? "" : styles.notHardSet} ` +
        `${isSelected ? styles.selectedTile : ""} ` +
        `${isHole ? (boardType === "Create" ? styles.holeTileCreate : styles.holeTilePlay) : ""}`
      }
      onClick={() => {
        updateSelectedTile(idx);
      }}
    >
      <span className={styles.tileLetter}>
        {letter === BLANK || letter === HOLE ? " " : letter}
      </span>
    </div>
  );
}

/** Interactive board for puzzle creation and play.
 *
 * The board is controlled by a row-major `boardLetters` string. It supports
 * click selection, Tab/arrow navigation, letter entry, Backspace clearing,
 * and Create-mode spacebar hole toggling.
 */
export function Board({
  boardType,
  filteringLetters,
  width,
  height,
  boardLetters,
  hardSet,
  setBoardLetters,
  setHardSet,
  selectedTile: controlledSelectedTile,
  setSelectedTile: controlledSetSelectedTile,
  onUserLetterPlaced,
}: BoardProps) {
  const [internalSelectedTile, setInternalSelectedTile] = useState(-1);
  const mobileInputRef = useRef<HTMLInputElement>(null);
  const boardLettersRef = useRef(boardLetters);
  const selectedTile = controlledSelectedTile ?? internalSelectedTile;
  const setSelectedTile = controlledSetSelectedTile ?? setInternalSelectedTile;

  function isSelectableTile(idx: number) {
    return (
      idx >= 0 &&
      idx < boardLetters.length &&
      (boardType === "Create" || boardLetters[idx] !== HOLE)
    );
  }

  function getNextTabTile(idx: number) {
    for (let offset = 1; offset <= boardLetters.length; offset++) {
      const nextIdx = (idx + offset) % boardLetters.length;

      if (isSelectableTile(nextIdx)) {
        return nextIdx;
      }
    }

    return idx;
  }

  function getArrowTile(idx: number, key: string) {
    const row = Math.floor(idx / width);
    const col = idx % width;

    switch (key) {
      case "ArrowRight":
        return col === width - 1 ? idx : idx + 1;
      case "ArrowLeft":
        return col === 0 ? idx : idx - 1;
      case "ArrowDown":
        return row === height - 1 ? idx : idx + width;
      case "ArrowUp":
        return row === 0 ? idx : idx - width;
      default:
        return idx;
    }
  }

  function applyBoardKey(key: string) {
    const idx = selectedTile;

    if (idx === -1) {
      return false;
    }

    // Board navigation
    if (key === "Tab") {
      setSelectedTile(getNextTabTile(idx));
      return true;
    }

    if (
      key === "ArrowRight" ||
      key === "ArrowLeft" ||
      key === "ArrowDown" ||
      key === "ArrowUp"
    ) {
      const nextIdx = getArrowTile(idx, key);

      if (isSelectableTile(nextIdx)) {
        setSelectedTile(nextIdx);
      }
      return true;
    }

    let newChar = boardLetters[idx];

    // Change letter
    if (/^[a-zA-Z]$/.test(key)) {
      // No changing letters when filtering
      // No changing a hard set letter when playing
      if (!filteringLetters && !(boardType === "Play" && hardSet[idx])) {
        newChar = key.toLowerCase();

        if (boardType === "Play" && boardLetters[idx] !== newChar) {
          onUserLetterPlaced?.();
        }
      }
    }

    // Remove letter
    else if (key === "Backspace") {
      // Toggle showing when filtering
      // Hard set hole/empty doesn't make sense; holes are by nature hard set already
      if (filteringLetters && newChar !== BLANK && newChar !== HOLE) {
        setHardSet?.(hardSet.with(idx, !hardSet[idx]));
      }

      // Remove letter when not filtering
      // If playing, no removing a hard set letter
      else if (!(boardType === "Play" && hardSet[idx])) newChar = BLANK;
    }

    // Toggle hole when creating
    else if (key === " " && boardType === "Create") {
      // Whether filtering or not, we can put in a hole
      if (newChar === BLANK) newChar = HOLE;
      else if (newChar === HOLE) newChar = BLANK;
      else return true;
    } else {
      return false;
    }

    setBoardLetters([...boardLetters].with(idx, newChar).join(""));
    return true;
  }

  function focusMobileInput() {
    mobileInputRef.current?.focus({ preventScroll: true });
  }

  useEffect(() => {
    boardLettersRef.current = boardLetters;
  });

  useEffect(() => {
    const selectedLetter = boardLettersRef.current[selectedTile];

    if (
      selectedLetter !== undefined &&
      (boardType === "Create" || selectedLetter !== HOLE)
    ) {
      focusMobileInput();
    }
  }, [selectedTile, boardType]);

  // Keyboard handling is centralized at window level so selected tiles behave
  // like a focused grid without rendering individual text inputs.
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target instanceof HTMLSelectElement
      ) {
        return;
      }

      if (applyBoardKey(e.key)) {
        e.preventDefault();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });

  if (width <= 0 || height <= 0 || boardLetters.length === 0) {
    return null;
  }

  const boardDimension = Math.max(width, height);
  const boardStyle = {
    gridTemplateColumns: `repeat(${width}, 1fr)`,
    "--board-width": width,
    "--board-height": height,
    "--board-dimension": boardDimension,
  } as CSSProperties &
    Record<"--board-width" | "--board-height" | "--board-dimension", number>;

  return (
    <div className={styles.boardFrame}>
      <input
        ref={mobileInputRef}
        className={styles.mobileInput}
        type="text"
        inputMode="text"
        autoCapitalize="characters"
        autoComplete="off"
        autoCorrect="off"
        spellCheck={false}
        aria-label="Selected tile input"
        onInput={(event) => {
          const input = event.currentTarget;
          const letter = input.value.slice(-1);

          if (letter !== "") {
            applyBoardKey(letter);
          }

          input.value = "";
        }}
        onKeyDown={(event) => {
          if (applyBoardKey(event.key)) {
            event.preventDefault();
          }
        }}
      />
      <div className={styles.board} style={boardStyle}>
        {[...boardLetters].map((letter, i) => (
          <Tile
            boardType={boardType}
            key={i}
            letter={letter.toUpperCase()}
            idx={i}
            isHardSet={hardSet[i]}
            isHole={letter === HOLE}
            updateSelectedTile={(idx: number) => {
              if (boardType === "Create" || letter !== HOLE) {
                const nextSelectedTile = selectedTile === idx ? -1 : idx;

                setSelectedTile(nextSelectedTile);
                if (nextSelectedTile !== -1) {
                  focusMobileInput();
                }
              }
            }}
            isSelected={selectedTile === i}
          />
        ))}
      </div>
    </div>
  );
}
