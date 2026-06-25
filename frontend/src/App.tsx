import { lazy, Suspense } from "react";
import { Routes, Route } from "react-router-dom";

const CreatePage = lazy(() => import("./pages/create/Create"));
const PlayPage = lazy(() => import("./pages/play/Play"));

function App() {
  return (
    <Suspense fallback={<p>Loading...</p>}>
      <Routes>
        <Route path="/create" element={<CreatePage />} />
        <Route path="/play/:puzzleId" element={<PlayPage />} />
      </Routes>
    </Suspense>
  )
}

export default App;
