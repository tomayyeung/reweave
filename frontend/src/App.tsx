import { useState, useEffect } from "react";

import { Board } from "./components/Board";
import { WordList } from "./components/WordList";
import { Wrapper } from "./components/Wrapper";

function App() {
  const w = 4;
  const h = 4;

  const [boardLetters, setBoardLetters] = useState("_".repeat(w * h)); // empty spaces
  const [words, setWords] = useState([]);

  useEffect(() => {
    // console.log("New board letters: '" + boardLetters + "'");
    const updateWords = async () => {
      fetch(`/api/find?width=${w}&height=${h}&letters=${boardLetters}`)
        .then((res) => res.json())
        .then((data) => {
          // console.log(data)
          setWords(data);
        });
    };

    updateWords();
  }, [boardLetters]);

  return (
    <main>
      <Wrapper>
        <Board
          boardType="Create"
          board={{
            width: w,
            height: h,
            letters: boardLetters,
          }}
          boardLetters={boardLetters}
          setBoardLetters={setBoardLetters}
        />
        <WordList words={words} />
      </Wrapper>

      <form
        // className={styles.form}
        action={async (formData) => {
          const res = await fetch("/api/create_puzzle", {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              puzzle_id: formData.get("puzzle_name"),
              width: w,
              height: h,
              letters: boardLetters,
              words: words,
            }),
          });

          console.log(res);
          // const data = await res.json();
          // console.log(data);
        }}
        autoComplete="off"
      >
        <label htmlFor="puzzle_name">Puzzle name</label>
        <input name="puzzle_name" />
        <button type="submit">Submit puzzle</button>
      </form>
    </main>
  );
}

export default App;
