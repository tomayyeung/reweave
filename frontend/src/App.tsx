import { useState, useEffect } from "react";

import { Board } from "./components/Board";
import { WordList } from "./components/WordList";
import { Wrapper } from "./components/Wrapper";

function App() {
  const w = 4;
  const h = 4;

  const [boardLetters, setBoardLetters] = useState("_".repeat(w*h)); // empty spaces
  const [words, setWords] = useState([]);

  useEffect(() => {
    // console.log("New board letters: '" + boardLetters + "'");
    const updateWords = async () => {
      fetch(`/api/find?width=${w}&height=${h}&letters=${boardLetters}`)
        .then((res) => res.json())
        .then((data) => {
          // console.log(data)
          setWords(data)
        });
    };

    updateWords();
  }, [boardLetters]);

  return (
    <>
      <Wrapper>
        <Board
          board={{
            width: w,
            height: h,
            letters: boardLetters,
          }}
          boardLetters={boardLetters}
          setBoardLetters={setBoardLetters}
        />
        <WordList words={words}/>
      </Wrapper>
    </>
  );
}

export default App;
