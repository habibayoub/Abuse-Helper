import Dashboard from "./pages/Dashboard";
import "./App.css";
import React from "react";
import Navbar from "./components/Navbar";

function App() {
  return (
    <>
      <Navbar />
      <div className="App">
        <header className="App-header">
          <Dashboard />
        </header>
      </div>
    </>
  );
}

export default App;
