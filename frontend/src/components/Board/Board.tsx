import { useEffect, useState } from "react";
import styles from "./Board.module.css";

type TileProps = {
  letter: string;
  idx: number;
  isHardSet: boolean;
  isHole: boolean;
  updateSelectedTile: (idx: number) => void;
  isSelected: boolean;
};

type BoardProps = {
  boardType: "Create" | "Play";
  /**
   * create: done making word list, now removing letters from puzzle
   */
  filteringLetters: boolean;
  width: number;
  height: number;
  boardLetters: string;
  hardSet: boolean[];
  /**
   * only needed for playing. when playing, hard set is constant
   */
  setHardSet?: React.Dispatch<React.SetStateAction<boolean[]>>;
  setBoardLetters: React.Dispatch<React.SetStateAction<string>>;
};

function Tile({
  letter,
  idx,
  isHardSet,
  isHole,
  updateSelectedTile,
  isSelected,
}: TileProps) {
  return (
    <div
      className={`${styles.tile} ${isHardSet ? "" : styles.notHardSet} ${isSelected ? styles.selectedTile : ""} ${isHole ? styles.holeTile : ""}`}
      onClick={() => {
        updateSelectedTile(idx);
      }}
    >
      <span className={styles.tileLetter}>{letter === "_" ? " " : letter}</span>
    </div>
  );
}

export function Board({
  boardType,
  filteringLetters,
  width,
  height,
  boardLetters,
  hardSet,
  setBoardLetters,
  setHardSet,
}: BoardProps) {
  const [selectedTile, setSelectedTile] = useState(-1);
  const [holes, setHoles] = useState(new Array(width * height).fill(false));

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      let idx = selectedTile;

      if (idx === -1) {
        return;
      }

      let newChar = boardLetters[idx];

      // Change letter
      if (/^[a-zA-Z]$/.test(e.key)) {
        // No changing letters when filtering
        // No changing a hard set letter when playing
        if (!filteringLetters && !(boardType === "Play" && hardSet[idx]))
          newChar = e.key;
      }

      // Remove letter
      else if (e.key === "Backspace") {
        // Toggle showing when filtering
        // Hard set hole/empty doesn't make sense; holes are by nature hard set already
        if (filteringLetters && newChar !== "_" && newChar !== "#") {
          setHardSet?.(hardSet.with(idx, !hardSet[idx]));
        }

        // Remove letter when not filtering
        // If playing, no removing a hard set letter
        else if (!(boardType === "Play" && hardSet[idx])) newChar = "_";
      }

      // Toggle hole when creating
      else if (e.key === "Space" && boardType === "Create") {
        // Whether filtering or not, we can put in a hole
        if (newChar === "_") newChar = "#";
        else if (newChar === "#") newChar = "_";

        setHoles(holes.with(idx, newChar === "#"));
      }

      setBoardLetters([...boardLetters].with(idx, newChar).join(""));
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectedTile, filteringLetters, boardLetters, hardSet]);

  return (
    <div
      className={styles.board}
      style={{
        gridTemplateColumns: `repeat(${height}, 1fr)`,
      }}
    >
      {[...boardLetters].map((letter, i) => (
        <Tile
          key={i}
          letter={letter.toUpperCase()}
          idx={i}
          isHardSet={hardSet[i]}
          isHole={holes[i]}
          updateSelectedTile={(idx: number) => {
            if (letter !== "#")
              setSelectedTile(selectedTile === idx ? -1 : idx);
          }}
          isSelected={selectedTile === i}
        />
      ))}
    </div>
  );
}
