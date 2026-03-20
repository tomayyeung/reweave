import { useEffect, useState } from "react";
import styles from "./Board.module.css";

type TileProps = {
  letter: string;
  idx: number;
  updateSelectedTile: (idx: number) => void;
  isSelected: boolean;
};

type BoardProps = {
  board: BoardData;
  boardLetters: string;
  setBoardLetters: React.Dispatch<React.SetStateAction<string>>;
};

interface BoardData {
  width: number;
  height: number;
  letters: string;
}

function Tile({ letter, idx, updateSelectedTile, isSelected }: TileProps) {
  return (
    <div
      className={`${styles.tile} ${isSelected ? styles.selectedTile : ""}`}
      onClick={() => {
        updateSelectedTile(idx);
      }}
    >
      <span className={styles.tileLetter}>{letter}</span>
    </div>
  );
}

export function Board({ board, boardLetters, setBoardLetters }: BoardProps) {
  const [selectedTile, setSelectedTile] = useState(-1);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (selectedTile === -1) {
        return;
      }

      let newChar;
      if (/^[a-zA-Z]$/.test(e.key)) {
        newChar = e.key;
      } else if (e.key === "Backspace") {
        newChar = " ";
      } else {
        return;
      }

      setBoardLetters(
        boardLetters.substring(0, selectedTile) +
          newChar +
          boardLetters.substring(selectedTile + 1),
      );
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectedTile]);

  return (
    <div
      className={styles.board}
      style={{
        gridTemplateColumns: `repeat(${board.height}, 1fr)`,
      }}
    >
      {[...board.letters].map((letter, i) => (
        <Tile
          key={i}
          letter={letter.toUpperCase()}
          idx={i}
          updateSelectedTile={(idx: number) => {
            setSelectedTile(selectedTile === idx ? -1 : idx);
          }}
          isSelected={selectedTile === i}
        />
      ))}
    </div>
  );
}
