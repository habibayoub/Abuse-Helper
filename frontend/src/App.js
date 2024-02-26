import React, { useEffect, useState } from "react";
import "./App.css";

function App() {
  const [message, setMessage] = useState();
  const [customersList, setCustomersList] = useState();
  useEffect( () => {
    fetch( "/api/customers" )
      .then( ( res ) => res.json() )
      .then( ( res ) => { setMessage( `Found ${res.length} customers` ); setCustomersList( JSON.stringify( res ) ) } )
      .catch( console.error );
  }, [setMessage, setCustomersList] );
  return (
    <div className="App">
      <header className="App-header">
        <p>{message || "Loading..."}</p>
        <p>{customersList || "Fetching..."}</p>
      </header>
    </div>
  );
}

export default App;
