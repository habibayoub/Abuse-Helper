import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { format } from "date-fns"
import { Menu, Eye, RefreshCw, Mail } from "lucide-react"
import Sidebar from '@/components/layout/Sidebar';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog"
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Link } from "react-router-dom"
import api from '@/lib/axios'

interface Ticket {
    id: string;
    ticket_type: string;
    status: string;
    ip_address: string | null;
    subject: string;
    description: string;
    confidence_score: number | null;
    identified_threats: string[] | null;
    extracted_indicators: string[] | null;
    analysis_summary: string | null;
    created_at: string;
    updated_at: string;
    email_ids: string[];
}

export default function TicketsPage() {
    const [tickets, setTickets] = useState<Ticket[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [sidebarOpen, setSidebarOpen] = useState(false)
    const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null)

    useEffect(() => {
        fetchTickets()
        // Set up polling every minute
        const interval = setInterval(fetchTickets, 60000)
        return () => clearInterval(interval)
    }, [])

    const fetchTickets = async () => {
        try {
            const response = await api.get('/tickets/list')
            console.log('Fetched tickets:', response.data)
            setTickets(response.data)
            setError(null)
        } catch (err) {
            console.error('Error fetching tickets:', err)
            setError('Failed to fetch tickets')
        } finally {
            setLoading(false)
        }
    }

    const updateTicketStatus = async (ticketId: string, newStatus: string) => {
        try {
            await api.put(`/tickets/${ticketId}/status`, { status: newStatus })
            fetchTickets() // Refresh tickets after update
        } catch (err) {
            console.error('Error updating ticket status:', err)
            setError('Failed to update ticket status')
        }
    }

    const toggleSidebar = () => {
        setSidebarOpen(!sidebarOpen)
    }

    const truncateText = (text: string, maxLength: number) => {
        return text.length > maxLength ? `${text.substring(0, maxLength)}...` : text
    }

    const getStatusColor = (status: string) => {
        switch (status.toLowerCase()) {
            case 'open':
                return 'bg-red-100 text-red-800'
            case 'inprogress':
                return 'bg-yellow-100 text-yellow-800'
            case 'closed':
                return 'bg-gray-100 text-gray-800'
            case 'resolved':
                return 'bg-green-100 text-green-800'
            default:
                return 'bg-gray-100 text-gray-800'
        }
    }

    return (
        <div className="flex h-full w-full bg-gray-100">
            <Sidebar isOpen={sidebarOpen} />
            <div className="flex-1 flex flex-col overflow-hidden lg:ml-64">
                <header className="bg-white bg-opacity-70 backdrop-filter backdrop-blur-md shadow-sm sticky top-0 z-10">
                    <div className="flex items-center justify-between p-4">
                        <div className="flex items-center">
                            <Button variant="outline" size="icon" onClick={toggleSidebar} className="lg:hidden">
                                <Menu className="h-6 w-6" />
                            </Button>
                            <h1 className="text-xl font-semibold ml-4">Tickets</h1>
                        </div>
                    </div>
                </header>

                <main className="flex-1 overflow-x-hidden overflow-y-auto bg-gray-100">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <Card>
                            <CardHeader>
                                <div className="flex justify-between items-center">
                                    <div>
                                        <CardTitle className="text-xl font-bold">Ticket List</CardTitle>
                                        <p className="text-sm text-muted-foreground">
                                            Showing {tickets.length} tickets
                                        </p>
                                    </div>
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={fetchTickets}
                                    >
                                        <RefreshCw className="mr-2 h-4 w-4" /> Refresh
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent>
                                {loading ? (
                                    <div className="text-center py-4">Loading tickets...</div>
                                ) : error ? (
                                    <div className="text-center text-red-500 py-4">{error}</div>
                                ) : (
                                    <div className="overflow-x-auto">
                                        <table className="w-full">
                                            <thead>
                                                <tr className="text-xs font-semibold tracking-wide text-left text-gray-500 uppercase border-b bg-gray-50">
                                                    <th className="px-4 py-3">ID</th>
                                                    <th className="px-4 py-3">Type</th>
                                                    <th className="px-4 py-3">Status</th>
                                                    <th className="px-4 py-3">Subject</th>
                                                    <th className="px-4 py-3">IP Address</th>
                                                    <th className="px-4 py-3">Confidence</th>
                                                    <th className="px-4 py-3">Created</th>
                                                    <th className="px-4 py-3">Linked Emails</th>
                                                    <th className="px-4 py-3">Action</th>
                                                </tr>
                                            </thead>
                                            <tbody className="bg-white divide-y">
                                                {tickets.map((ticket) => (
                                                    <tr key={ticket.id} className="text-gray-700">
                                                        <td className="px-4 py-3">
                                                            <code className="bg-gray-100 px-2 py-1 rounded text-sm">
                                                                {ticket.id.slice(0, 8)}
                                                            </code>
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            {ticket.ticket_type}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(ticket.status)}`}>
                                                                {ticket.status}
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            {truncateText(ticket.subject, 40)}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            {ticket.ip_address || 'N/A'}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            {ticket.confidence_score ?
                                                                `${(ticket.confidence_score * 100).toFixed(1)}%` :
                                                                'N/A'}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            {format(new Date(ticket.created_at), 'MMM d, yyyy HH:mm')}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            <div className="flex gap-2 flex-wrap">
                                                                {ticket.email_ids && ticket.email_ids.length > 0 ? (
                                                                    ticket.email_ids.map((emailId) => (
                                                                        <Link key={emailId} to={`/emails/${emailId}`}>
                                                                            <Badge variant="secondary" className="cursor-pointer hover:bg-secondary/80">
                                                                                <Mail className="w-3 h-3 mr-1" />
                                                                                {emailId.slice(0, 8)}
                                                                            </Badge>
                                                                        </Link>
                                                                    ))
                                                                ) : (
                                                                    <Badge variant="outline">No emails</Badge>
                                                                )}
                                                            </div>
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            <Dialog>
                                                                <DialogTrigger asChild>
                                                                    <Button
                                                                        variant="outline"
                                                                        size="sm"
                                                                        onClick={() => setSelectedTicket(ticket)}
                                                                    >
                                                                        <Eye className="h-4 w-4" />
                                                                    </Button>
                                                                </DialogTrigger>
                                                                <DialogContent className="sm:max-w-[725px]">
                                                                    <DialogHeader>
                                                                        <DialogTitle>Ticket Details</DialogTitle>
                                                                    </DialogHeader>
                                                                    {selectedTicket && (
                                                                        <div className="space-y-4">
                                                                            <div>
                                                                                <h3 className="font-semibold">Subject</h3>
                                                                                <p>{selectedTicket.subject}</p>
                                                                            </div>
                                                                            <div>
                                                                                <h3 className="font-semibold">Description</h3>
                                                                                <p className="whitespace-pre-wrap">{selectedTicket.description}</p>
                                                                            </div>
                                                                            <div className="grid grid-cols-2 gap-4">
                                                                                <div>
                                                                                    <h3 className="font-semibold">Identified Threats</h3>
                                                                                    <ul className="list-disc list-inside">
                                                                                        {selectedTicket.identified_threats?.map((threat, index) => (
                                                                                            <li key={index}>{threat}</li>
                                                                                        )) || 'None'}
                                                                                    </ul>
                                                                                </div>
                                                                                <div>
                                                                                    <h3 className="font-semibold">Extracted Indicators</h3>
                                                                                    <ul className="list-disc list-inside">
                                                                                        {selectedTicket.extracted_indicators?.map((indicator, index) => (
                                                                                            <li key={index}>{indicator}</li>
                                                                                        )) || 'None'}
                                                                                    </ul>
                                                                                </div>
                                                                            </div>
                                                                            {selectedTicket.analysis_summary && (
                                                                                <div>
                                                                                    <h3 className="font-semibold">Analysis Summary</h3>
                                                                                    <p>{selectedTicket.analysis_summary}</p>
                                                                                </div>
                                                                            )}
                                                                        </div>
                                                                    )}
                                                                </DialogContent>
                                                            </Dialog>
                                                        </td>
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>

                                        {tickets.length === 0 && (
                                            <div className="text-center py-4 text-gray-500">
                                                No tickets found
                                            </div>
                                        )}
                                    </div>
                                )}
                                {selectedTicket && (
                                    <div className="mt-4">
                                        <Select
                                            onValueChange={(value) => updateTicketStatus(selectedTicket.id, value)}
                                            defaultValue={selectedTicket.status}
                                        >
                                            <SelectTrigger className="w-[180px]">
                                                <SelectValue placeholder="Update Status" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="Open">Open</SelectItem>
                                                <SelectItem value="InProgress">In Progress</SelectItem>
                                                <SelectItem value="Closed">Closed</SelectItem>
                                                <SelectItem value="Resolved">Resolved</SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    )
}
