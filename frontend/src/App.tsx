import { Routes, Route } from "react-router-dom";
import CreatePage from "./pages/create/Create";
import PlayPage from "./pages/play/Play";

function App() {
  return (
    <Routes>
      <Route path="/create" element={<CreatePage />} />
      <Route path="/play/:puzzleId" element={<PlayPage />} />
    </Routes>
  )
}

export default App;
