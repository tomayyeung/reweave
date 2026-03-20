import { useState, useEffect } from "react";

import { Board } from "./components/Board";
import { WordList } from "./components/WordList";
import { Wrapper } from "./components/Wrapper";

function App() {
  const [msg, setMsg] = useState("tmp");

  useEffect(() => {
    fetch("/api/hello")
      .then((res) => res.json())
      .then((data) => setMsg(data.text));
  }, []);


  const [boardLetters, setBoardLetters] = useState("         "); // empty 9 spaces
  useEffect(() => {
    console.log("New board letters:", boardLetters);
  }, [boardLetters]);

  return (
    <>
      <p>Message: {msg}</p>
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
