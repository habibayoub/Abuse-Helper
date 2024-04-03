import Dashboard from "./pages/Dashboard";
import "./App.css";
import React from "react";
import Navbar from "./components/Navbar/Navbar";
import { createTheme, MantineProvider } from '@mantine/core';


function App() {
  return (
    <MantineProvider forceColorScheme="dark" >
      <Navbar />
      <div className="App">
        <header className="App-header">
          <Dashboard />
        </header>
      </div>
    </MantineProvider>
  );
}

export default App;
