import { useState, useEffect, useRef } from "react";

import { Board } from "@components/Board";
import { PlayWordList } from "@components/WordList";
import type { Words } from "@components/WordList";
import { Wrapper } from "@components/Wrapper";
import { useParams } from "react-router-dom";

type Cell = {
  x: number;
  y: number;
};

function cellKey(cell: Cell): string {
  return `${cell.x},${cell.y}`;
}

function generateStartingLetters(
  width: number,
  height: number,
  holes: Cell[],
  startingLetters: Map<string, string>,
): string {
  const holeKeys = new Set(holes.map(cellKey));

  return Array.from({ length: width * height }, (_, i) => {
    const key = cellKey({ x: i % width, y: Math.floor(i / width) });
    if (holeKeys.has(key)) return "#";
    if (startingLetters.has(key)) return startingLetters.get(key)!;
    return "_";
  }).join("");
}

export default function PlayPage() {
  const { puzzleId } = useParams();

  const puzzleFetched = useRef(false);

  const [boardLetters, setBoardLetters] = useState("");
  const [hardSet, setHardSet] = useState<boolean[]>([]);
  const [w, setWidth] = useState(0);
  const [h, setHeight] = useState(0);
  const [words, setWords] = useState<Words>({
    found: [],
    missing: [],
    extra: [],
  });

  useEffect(() => {
    fetch(`/api/puzzle/${puzzleId}`)
      .then((res) => res.json())
      .then((puzzle) => {
        console.log(puzzle);
        setWidth(puzzle.width);
        setHeight(puzzle.height);

        const startingLettersMap = new Map<string, string>(
          (puzzle.starting_letters as [[number, number], string][]).map(
            ([cell, char]) => [cellKey({ x: cell[0], y: cell[1] }), char],
          ),
        );

        const initialLetters = generateStartingLetters(
          puzzle.width,
          puzzle.height,
          puzzle.holes,
          startingLettersMap,
        );

        // intialize board w/ letters
        // any initial letters means they are hard set
        setBoardLetters(initialLetters);
        setHardSet([...initialLetters].map((letter) => letter !== "_"));

        puzzleFetched.current = true;
      });
  }, []);

  useEffect(() => {
    // console.log("New board letters: '" + fromCreateBoardLetters(boardLetters, true) + "'");
    if (!puzzleFetched.current) {
      return;
    }

    fetch(`/api/check-puzzle/${puzzleId}/letters/${boardLetters}`)
      .then((res) => res.json())
      .then((data) => {
        // console.log(data)
        setWords(data);
      });
  }, [boardLetters]);

  return (
    <main>
      <Wrapper>
        <Board
          boardType="Play"
          filteringLetters={false}
          width={w}
          height={h}
          boardLetters={boardLetters}
          hardSet={hardSet}
          setBoardLetters={setBoardLetters}
        />
        <PlayWordList words={words} />
      </Wrapper>
    </main>
  );
}
