import React, { useState } from 'react';

function CustomerForm() {
    const [email, setEmail] = useState( '' );
    const [customerData, setCustomerData] = useState( null );

    const handleSubmit = async ( event ) => {
        event.preventDefault();

        try {
            const response = await fetch( '/api/private/find_customer', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify( { email } ),
            } );

            if ( response.ok ) {
                const customerData = await response.json();
                setCustomerData( customerData )
                console.log( 'customer data:', customerData );
            } else {
                console.error( 'failed to fetch customer' );
            }
        } catch ( error ) {
            console.error( 'error submitting form', error );
        }
    };

    return (
        <div>
            <form onSubmit={handleSubmit}>
                <label htmlFor="email">Email:</label>
                <input
                    type="email"
                    id="email"
                    value={email}
                    onChange={( e ) => setEmail( e.target.value )}
                    required
                />
                <button type="submit">Find Customer</button>
            </form>
            {customerData && <p>Found: {JSON.stringify( customerData, null, 2 )}</p>}
        </div>
    );
}

export default CustomerForm;
