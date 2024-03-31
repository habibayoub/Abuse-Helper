import React, { useEffect, useState } from "react";

const Dashboard = () => {
    const [status, setStatus] = useState("");

    useEffect(() => {
        fetch("/api/status")
            .then((res) => res.json())
            .then((res) => {
                setStatus(JSON.stringify(res));
                console.log(res);
            })
            .catch(console.error);
    }, [setStatus]);

    return (
        <>{status}


        </>
    );
}

export default Dashboard;