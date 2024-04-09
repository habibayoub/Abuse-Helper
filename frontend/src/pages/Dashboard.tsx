import React, { useEffect, useState } from "react";

interface Customer {
    id: number;
    email: string;
    ip: string;
}

interface NCTNS {
    uuid: string;
    source_time: string;
    time: string;
    ip: string;
    reverse_dns: string;
    domain_name: string;
    asn: string;
    as_name: string;
    category: string;
    type_: string;
    malware_family: string;
    vulnerability: string;
    tag: string;
    source_name: string;
    comment: string;
    description: string;
    description_url: string;
    destination_ip: string;
    destination_port: number;
    port: number;
    protocol: string;
    transport_protocol: string;
    http_request: string;
    user_agent: string;
    username: string;
    url: string;
    destination_domain_name: string;
    status: string;
    observation_time: string;
    source_feed: string;
}


interface CustomerTableRow extends Customer {
    key: string;
}

interface NCTNSTableRow extends NCTNS {
    key: string;
}

interface TableColumnData {
    key: string;
    label: string;
}


const Dashboard: React.FC = () => {
    // State to store the status of the API
    const [status, setStatus] = useState<string>("");
    // State to store the customer data
    const [customerRows, setCustomerRows] = useState<CustomerTableRow[]>([]);
    const [customerCols, setCustomerCols] = useState<TableColumnData[]>([]);
    const [nctnsRows, setNCTNSRows] = useState<NCTNSTableRow[]>([]);
    const [nctnsCols, setNCTNSCols] = useState<TableColumnData[]>([]);

    // Fetch the status of the API
    useEffect(() => {
        fetch("/api/status")
            .then((res) => res.json())
            .then((res) => {
                // Set the status to the response
                setStatus(JSON.stringify(res));
                console.log(res);
            })
            .catch(console.error);
    }, []);

    // Fetch the customer data
    useEffect(() => {
        fetch("/api/customer/list")
            .then((res) => res.json())
            .then((data: Customer[]) => {

                // Map the data to include a key
                const updatedRows: CustomerTableRow[] = data.map((item, index) => ({
                    ...item,
                    key: `${index}`,
                }));

                // Set the customer data
                setCustomerRows(updatedRows);

                // Define the columns for the table
                const columns: TableColumnData[] = [
                    {
                        key: "id",
                        label: "ID",
                    },
                    {
                        key: "email",
                        label: "Email",
                    },
                    {
                        key: "ip",
                        label: "IP",
                    },
                ];

                // Set the columns
                setCustomerCols(columns);
            })
            .catch(console.error);
    }, [setCustomerCols, setCustomerRows]);

    useEffect(() => {
        fetch("/api/nctns/list")
            .then((res) => res.json())
            .then((data: NCTNS[]) => {

                // Map the data to include a key
                const updatedRows: NCTNSTableRow[] = data.map((item, index) => ({
                    ...item,
                    key: `${index}`,
                }));

                // Set the NCTNS data
                setNCTNSRows(updatedRows);

                // Define the columns for the table
                const columns: TableColumnData[] = [
                    { key: 'uuid', label: 'UUID' },
                    { key: 'source_time', label: 'Source Time' },
                    { key: 'time', label: 'Time' },
                    { key: 'ip', label: 'IP Address' },
                    { key: 'reverse_dns', label: 'Reverse DNS' },
                    { key: 'domain_name', label: 'Domain Name' },
                    { key: 'asn', label: 'ASN' },
                    { key: 'as_name', label: 'AS Name' },
                    { key: 'category', label: 'Category' },
                    { key: 'type_', label: 'Type' },
                    { key: 'malware_family', label: 'Malware Family' },
                    { key: 'vulnerability', label: 'Vulnerability' },
                    { key: 'tag', label: 'Tag' },
                    { key: 'source_name', label: 'Source Name' },
                    { key: 'comment', label: 'Comment' },
                    { key: 'description', label: 'Description' },
                    { key: 'description_url', label: 'Description URL' },
                    { key: 'destination_ip', label: 'Destination IP' },
                    { key: 'destination_port', label: 'Destination Port' },
                    { key: 'port', label: 'Port' },
                    { key: 'protocol', label: 'Protocol' },
                    { key: 'transport_protocol', label: 'Transport Protocol' },
                    { key: 'http_request', label: 'HTTP Request' },
                    { key: 'user_agent', label: 'User Agent' },
                    { key: 'username', label: 'Username' },
                    { key: 'url', label: 'URL' },
                    { key: 'destination_domain_name', label: 'Destination Domain Name' },
                    { key: 'status', label: 'Status' },
                    { key: 'observation_time', label: 'Observation Time' },
                    { key: 'source_feed', label: 'Source Feed' },
                ];

                // Set the columns
                setNCTNSCols(columns);
            })
            .catch(console.error);
    }, [setNCTNSCols, setNCTNSRows]);

    return (
        <>
            <div className=" min-h-[100dvh] flex flex-col gap-1 pt-4">
                {status}

                <p className="font-semibold">Customers</p>

                <p className="font-semibold">NCTNS</p>

            </div >
        </>
    );
}

export default Dashboard;