import { Routes, Route } from "react-router-dom";
import CreatePage from "./pages/create/Create";

function App() {
  return (
    <Routes>
      <Route path="/create" element={<CreatePage />} />
    </Routes>
  )
}

export default App;
