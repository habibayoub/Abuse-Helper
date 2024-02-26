import React, { useEffect, useState } from "react";
import "./App.css";
import CustomerForm from "./components/Search";

function App() {
  const [customerCount, setCustomerCount] = useState();
  const [customersList, setCustomersList] = useState();
  const [show, setShow] = useState( false );

  const toggleListVisibility = () => {
    setShow( !show );
  };

  useEffect( () => {
    fetch( "/api/customers" )
      .then( ( res ) => res.json() )
      .then( ( res ) => { setCustomerCount( `Found ${res.length} customers` ); setCustomersList( JSON.stringify( res ) ) } )
      .catch( console.error );
  }, [setCustomerCount, setCustomersList] );

  return (
    <div className="App">
      <header className="App-header">
        <p>{customerCount || "Loading..."}</p>
        <button onClick={toggleListVisibility}>
          {show ? 'Hide' : 'Show'} List
        </button>
        <p>{show && customersList}</p>
        <CustomerForm />
      </header>
    </div>
  );
}

export default App;
