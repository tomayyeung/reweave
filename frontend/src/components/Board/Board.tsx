import styles from "./Board.module.css";

export interface BoardData {
  width: number;
  height: number;
  letters: string;
}

function Tile({ letter }: { letter: string }) {
  return (
    <div className={styles.tile}>
      <span className={styles.tileLetter}>{letter}</span>
    </div>
  );
}

export function Board({ board }: { board: BoardData }) {
  return (
    <div
      className={styles.board}
      style={{
        gridTemplateColumns: `repeat(${board.height}, 1fr)`,
      }}
    >
      {[...board.letters].map((letter, i) => (
        <Tile key={i} letter={letter.toUpperCase()} />
      ))}
    </div>
  );
}
