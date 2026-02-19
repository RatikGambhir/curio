import { BrowserRouter as Router, Route, Routes } from "react-router-dom";

import "./App.css";
import Chat from "./pages/Chat";
import Home from "./pages/Home";
import Landing from "./pages/Landing";
import Login from "./pages/Login";
import ProfileSettings from "./pages/ProfileSettings";
import Register from "./pages/Register";

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/" element={<Landing />} />
        <Route path="/chat" element={<Chat />} />
        <Route path="/home" element={<Home />} />
        <Route path="/profile" element={<ProfileSettings />} />
        <Route path="/register" element={<Register />} />
      </Routes>
    </Router>
  );
}

export default App;
