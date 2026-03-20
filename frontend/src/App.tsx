import { useState, useEffect } from "react";

import { Board } from "./components/Board";
import { WordList } from "./components/WordList";
import { Wrapper } from "./components/Wrapper";

function App() {
  // const [msg, setMsg] = useState("tmp");

  // useEffect(() => {
  //   fetch("/api/hello")
  //     .then((res) => res.json())
  //     .then((data) => setMsg(data.text));
  // }, []);
  const w = 3;
  const h = 3;

  const [boardLetters, setBoardLetters] = useState("_________"); // empty 9 spaces
  useEffect(() => {
    console.log("New board letters: '" + boardLetters + "'");
    const updateWords = async () => {
      fetch(`/api/find?width=${w}&height=${h}&letters=${boardLetters}`)
        .then((res) => res.json())
        .then((data) => console.log(data));
    };

    updateWords();
  }, [boardLetters]);

  return (
    <>
      {/* <p>Message: {msg}</p> */}
      <Wrapper>
        <Board
          board={{
            width: 3,
            height: 3,
            letters: boardLetters,
          }}
          boardLetters={boardLetters}
          setBoardLetters={setBoardLetters}
        />
        <WordList />
      </Wrapper>
    </>
  );
}

export default App;
