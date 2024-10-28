import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { format } from "date-fns"
import { Eye, Menu, TicketsIcon } from "lucide-react"
import Sidebar from '@/components/layout/Sidebar'
import api from '@/lib/axios'  // Import the axios instance

// Define the Ticket interface based on the backend model
interface Ticket {
    id: string
    ticket_type: 'Malware' | 'Phishing' | 'Scam' | 'Spam' | 'Other'
    status: 'Open' | 'InProgress' | 'Closed' | 'Resolved'
    ip_address: string | null
    email_id: string
    subject: string
    description: string
    created_at: string
    updated_at: string
}

// Status badge colors mapping
const statusColors: Record<string, { text: string; bg: string }> = {
    Open: { text: 'text-yellow-700', bg: 'bg-yellow-100' },
    InProgress: { text: 'text-blue-700', bg: 'bg-blue-100' },
    Closed: { text: 'text-gray-700', bg: 'bg-gray-100' },
    Resolved: { text: 'text-green-700', bg: 'bg-green-100' },
}

export default function TicketsPage() {
    const [tickets, setTickets] = useState<Ticket[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [sidebarOpen, setSidebarOpen] = useState(false)

    useEffect(() => {
        fetchTickets()
    }, [])

    const fetchTickets = async () => {
        try {
            const response = await api.get('/ticket/tickets')
            setTickets(response.data)
        } catch (err) {
            console.error('Error fetching tickets:', err)
            setError('Failed to fetch tickets')
        } finally {
            setLoading(false)
        }
    }

    const toggleSidebar = () => {
        setSidebarOpen(!sidebarOpen)
    }

    return (
        <div className="flex h-full w-full bg-gray-100">
            {/* Sidebar */}
            <Sidebar isOpen={sidebarOpen} />

            {/* Main Content */}
            <div className="flex-1 flex flex-col overflow-hidden lg:ml-64">
                {/* Header */}
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

                {/* Main Content */}
                <main className="flex-1 overflow-x-hidden overflow-y-auto bg-gray-100">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <Card>
                            <CardHeader>
                                <div className="flex justify-between items-center">
                                    <div>
                                        <CardTitle className="text-xl font-bold">All Tickets</CardTitle>
                                        <p className="text-sm text-muted-foreground">
                                            Showing {tickets.length} tickets
                                        </p>
                                    </div>
                                    <Button 
                                        variant="outline" 
                                        size="sm"
                                        onClick={fetchTickets}
                                    >
                                        <TicketsIcon className="mr-2 h-4 w-4" /> Refresh
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
                                                    <th className="px-4 py-3">Ticket ID</th>
                                                    <th className="px-4 py-3">IP Address</th>
                                                    <th className="px-4 py-3">Type</th>
                                                    <th className="px-4 py-3">Date</th>
                                                    <th className="px-4 py-3">Status</th>
                                                    <th className="px-4 py-3">Action</th>
                                                </tr>
                                            </thead>
                                            <tbody className="bg-white divide-y">
                                                {tickets.map((ticket) => (
                                                    <tr key={ticket.id} className="text-gray-700">
                                                        <td className="px-4 py-3">#{ticket.id.slice(0, 8)}</td>
                                                        <td className="px-4 py-3">{ticket.ip_address || 'N/A'}</td>
                                                        <td className="px-4 py-3">{ticket.ticket_type}</td>
                                                        <td className="px-4 py-3">
                                                            {format(new Date(ticket.created_at), 'yyyy-MM-dd')}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            <span className={`px-2 py-1 font-semibold text-xs leading-tight ${statusColors[ticket.status].text} ${statusColors[ticket.status].bg} rounded-full`}>
                                                                {ticket.status}
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            <Button variant="outline" size="sm">
                                                                <Eye className="h-4 w-4" />
                                                            </Button>
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
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    )
}
