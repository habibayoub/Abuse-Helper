import React, { useState } from 'react';

function CustomerForm() {
    const [email, setEmail] = useState( '' );
    const [ip, setIp] = useState( '' );
    const [id, setId] = useState();
    const [customerData, setCustomerData] = useState( null );

    const handleEmailSubmit = async ( event ) => {
        event.preventDefault();

        try {
            const response = await fetch( '/api/private/find_customer', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify( { email} ),
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
    const handleIpSubmit = async ( event ) => {
        event.preventDefault();

        try {
            const response = await fetch( '/api/private/find_customer', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify( { ip } ),
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
    const handleIdSubmit = async ( event ) => {
        event.preventDefault();

        try {
            const response = await fetch( '/api/private/find_customer', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify( { id: parseInt(id, 10) } ),
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
            <form onSubmit={handleEmailSubmit}>
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
            <form onSubmit={handleIpSubmit}>
                <label htmlFor="ip">IP Address:</label>
                <input
                    type="ip"
                    id="ip"
                    value={ip}
                    onChange={( e ) => setIp( e.target.value )}
                    required
                />
                <button type="submit">Find Customer</button>
            </form>
            <form onSubmit={handleIdSubmit}>
                <label htmlFor="id">ID:</label>
                <input
                    type="id"
                    id="id"
                    value={id}
                    onChange={( e ) => setId( e.target.value )}
                    required
                />
                <button type="submit">Find Customer</button>
            </form>
            {customerData && <p>Found: {JSON.stringify( customerData, null, 2 )}</p>}
        </div>
    );
}

export default CustomerForm;
