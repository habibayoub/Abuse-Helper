import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { format } from "date-fns"
import { Menu, Eye, RefreshCw, Mail, Search } from "lucide-react"
import Sidebar from '@/components/layout/Sidebar';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
    DialogFooter,
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
import { Label } from "@/components/ui/label"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Checkbox } from "@/components/ui/checkbox"

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

interface CreateTicketRequest {
    subject: string;
    description: string;
    ticket_type: string;
    email_ids: string[];
}

interface CreateTicketResponse {
    failed_emails: [string, string][];
}

interface SearchFilters {
    status?: TicketStatus;
    ticket_type?: TicketType;
    has_emails?: boolean;
}

interface SearchOptions {
    query: string;
    filters?: SearchFilters;
    from?: number;
    size?: number;
}

interface SearchResponse {
    hits: Ticket[];
    total: number;
}

enum TicketType {
    Malware = "Malware",
    Phishing = "Phishing",
    Scam = "Scam",
    Spam = "Spam",
    DDoS = "DDoS",
    Botnet = "Botnet",
    DataBreach = "DataBreach",
    IdentityTheft = "IdentityTheft",
    Ransomware = "Ransomware",
    CyberStalking = "CyberStalking",
    IntellectualPropertyTheft = "IntellectualPropertyTheft",
    Harassment = "Harassment",
    UnauthorizedAccess = "UnauthorizedAccess",
    CopyrightViolation = "CopyrightViolation",
    BruteForce = "BruteForce",
    C2 = "C2",
    Other = "Other"
}

enum TicketStatus {
    Open = "Open",
    InProgress = "InProgress",
    Closed = "Closed",
    Resolved = "Resolved"
}

export default function TicketsPage() {
    const [tickets, setTickets] = useState<Ticket[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [sidebarOpen, setSidebarOpen] = useState(false)
    const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null)
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const [searchQuery, setSearchQuery] = useState("")
    const [searching, setSearching] = useState(false)
    const [searchFilters, setSearchFilters] = useState<SearchFilters>({})

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

    const validateForm = (data: CreateTicketRequest): string | null => {
        if (!data.subject.trim()) return "Subject is required"
        if (!data.description.trim()) return "Description is required"
        if (!data.ticket_type) return "Ticket type is required"
        if (data.email_ids.some(id => !id.match(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i))) {
            return "Invalid email ID format"
        }
        return null
    }

    const handleCreateTicket = async (formData: FormData): Promise<CreateTicketResponse> => {
        try {
            const emailIds = formData.get('email_ids')?.toString().trim()
            const data: CreateTicketRequest = {
                subject: formData.get('subject')?.toString() || '',
                description: formData.get('description')?.toString() || '',
                ticket_type: formData.get('ticket_type')?.toString() || 'Other',
                email_ids: emailIds ? emailIds.split(',').map(id => id.trim()) : [],
            }

            const validationError = validateForm(data)
            if (validationError) {
                setError(validationError)
                throw new Error(validationError)
            }

            const response = await api.post<CreateTicketResponse>('/tickets/create', data)
            console.log('Created ticket:', response.data)

            if (response.data.failed_emails.length > 0) {
                const failedMessages = response.data.failed_emails
                    .map(([id, error]) => `Email ${id}: ${error}`)
                    .join('\n')
                setError(`Some emails could not be linked:\n${failedMessages}`)
            } else {
                setError(null)
            }

            fetchTickets()
            return response.data
        } catch (err) {
            console.error('Error creating ticket:', err)
            if (!error) { // Only set error if not already set by validation
                setError(err instanceof Error ? err.message : 'Failed to create ticket')
            }
            throw err
        }
    }

    const updateTicketStatus = async (ticketId: string, newStatus: string) => {
        try {
            const response = await api.put(`/tickets/${ticketId}/status`, newStatus)
            console.log('Updated ticket status:', response.data)
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

    const handleSearch = async (query: string) => {
        if (!query.trim() && !Object.keys(searchFilters).length) {
            fetchTickets()
            return
        }

        setSearching(true)
        try {
            const searchOptions: SearchOptions = {
                query: query.trim(),
                filters: searchFilters,
                size: 50
            }

            console.log("Search options:", searchOptions)

            const response = await api.get<SearchResponse>('/tickets/search', {
                params: searchOptions
            })

            console.log("Search response:", response.data)

            if (response.data && response.data.hits) {
                setTickets(response.data.hits)
                setError(null)
            } else {
                console.log("No tickets found in search results")
                setTickets([])
            }
        } catch (err) {
            console.error('Error searching tickets:', err)
            setError('Failed to search tickets')
            fetchTickets()
        } finally {
            setSearching(false)
        }
    }

    useEffect(() => {
        const timeoutId = setTimeout(() => {
            handleSearch(searchQuery)
        }, 500)

        return () => clearTimeout(timeoutId)
    }, [searchQuery, searchFilters])

    const SearchFiltersComponent = () => (
        <div className="flex gap-2 items-center mt-2">
            <Select
                value={searchFilters.status}
                onValueChange={(value: TicketStatus) =>
                    setSearchFilters(prev => ({
                        ...prev,
                        status: value
                    }))
                }
            >
                <SelectTrigger className="w-[150px]">
                    <SelectValue placeholder="Status" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="Open">Open</SelectItem>
                    <SelectItem value="InProgress">In Progress</SelectItem>
                    <SelectItem value="Closed">Closed</SelectItem>
                    <SelectItem value="Resolved">Resolved</SelectItem>
                </SelectContent>
            </Select>

            <Select
                value={searchFilters.ticket_type}
                onValueChange={(value) =>
                    setSearchFilters(prev => ({
                        ...prev,
                        ticket_type: value as TicketType
                    }))
                }
            >
                <SelectTrigger className="w-[150px]">
                    <SelectValue placeholder="Type" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value={TicketType.Malware}>{TicketType.Malware}</SelectItem>
                    <SelectItem value={TicketType.Phishing}>{TicketType.Phishing}</SelectItem>
                    <SelectItem value={TicketType.Scam}>{TicketType.Scam}</SelectItem>
                    <SelectItem value={TicketType.Spam}>{TicketType.Spam}</SelectItem>
                    <SelectItem value={TicketType.DDoS}>{TicketType.DDoS}</SelectItem>
                    <SelectItem value={TicketType.Botnet}>{TicketType.Botnet}</SelectItem>
                    <SelectItem value={TicketType.DataBreach}>{TicketType.DataBreach}</SelectItem>
                    <SelectItem value={TicketType.IdentityTheft}>{TicketType.IdentityTheft}</SelectItem>
                    <SelectItem value={TicketType.Ransomware}>{TicketType.Ransomware}</SelectItem>
                    <SelectItem value={TicketType.CyberStalking}>{TicketType.CyberStalking}</SelectItem>
                    <SelectItem value={TicketType.IntellectualPropertyTheft}>{TicketType.IntellectualPropertyTheft}</SelectItem>
                    <SelectItem value={TicketType.Harassment}>{TicketType.Harassment}</SelectItem>
                    <SelectItem value={TicketType.UnauthorizedAccess}>{TicketType.UnauthorizedAccess}</SelectItem>
                    <SelectItem value={TicketType.CopyrightViolation}>{TicketType.CopyrightViolation}</SelectItem>
                    <SelectItem value={TicketType.BruteForce}>{TicketType.BruteForce}</SelectItem>
                    <SelectItem value={TicketType.C2}>{TicketType.C2}</SelectItem>
                    <SelectItem value={TicketType.Other}>{TicketType.Other}</SelectItem>
                </SelectContent>
            </Select>

            <div className="flex items-center space-x-2">
                <Checkbox
                    id="hasEmails"
                    checked={searchFilters.has_emails}
                    onCheckedChange={(checked) =>
                        setSearchFilters(prev => ({
                            ...prev,
                            has_emails: checked as boolean
                        }))
                    }
                />
                <label
                    htmlFor="hasEmails"
                    className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                    Has Emails
                </label>
            </div>
        </div>
    )

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
                        <div className="flex gap-2 items-center">
                            <div className="relative w-64">
                                <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                                <Input
                                    placeholder="Search tickets..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    className={`pl-8 ${searching ? 'opacity-50' : ''}`}
                                    disabled={searching}
                                />
                                {searching && (
                                    <div className="absolute right-2 top-2">
                                        <RefreshCw className="h-4 w-4 animate-spin" />
                                    </div>
                                )}
                            </div>
                            <SearchFiltersComponent />
                            <Button onClick={() => setShowCreateDialog(true)}>
                                Create Ticket
                            </Button>
                            <Button variant="outline" size="sm" onClick={fetchTickets}>
                                <RefreshCw className="mr-2 h-4 w-4" /> Refresh
                            </Button>
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
                                                                            <div>
                                                                                <h3 className="font-semibold">Status</h3>
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
                            </CardContent>
                        </Card>

                        {/* Create Ticket Dialog */}
                        <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                            <DialogContent>
                                <DialogHeader>
                                    <DialogTitle>Create New Ticket</DialogTitle>
                                </DialogHeader>
                                <form onSubmit={async (e) => {
                                    e.preventDefault()
                                    try {
                                        await handleCreateTicket(new FormData(e.currentTarget))
                                        if (!error) { // Only close if no error
                                            setShowCreateDialog(false)
                                        }
                                    } catch (err) {
                                        // Error is already handled in handleCreateTicket
                                    }
                                }}>
                                    <div className="space-y-4">
                                        <div>
                                            <Label htmlFor="subject">Subject</Label>
                                            <Input id="subject" name="subject" required />
                                        </div>
                                        <div>
                                            <Label htmlFor="description">Description</Label>
                                            <Textarea id="description" name="description" required />
                                        </div>
                                        <div>
                                            <Label htmlFor="ticket_type">Type</Label>
                                            <Select name="ticket_type" defaultValue="Other">
                                                <SelectTrigger>
                                                    <SelectValue placeholder="Select type" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    <SelectItem value="Malware">Malware</SelectItem>
                                                    <SelectItem value="Phishing">Phishing</SelectItem>
                                                    <SelectItem value="Scam">Scam</SelectItem>
                                                    <SelectItem value="Spam">Spam</SelectItem>
                                                    <SelectItem value="DDoS">DDoS</SelectItem>
                                                    <SelectItem value="Botnet">Botnet</SelectItem>
                                                    <SelectItem value="DataBreach">Data Breach</SelectItem>
                                                    <SelectItem value="IdentityTheft">Identity Theft</SelectItem>
                                                    <SelectItem value="Ransomware">Ransomware</SelectItem>
                                                    <SelectItem value="CyberStalking">Cyber Stalking</SelectItem>
                                                    <SelectItem value="IntellectualPropertyTheft">IP Theft</SelectItem>
                                                    <SelectItem value="Harassment">Harassment</SelectItem>
                                                    <SelectItem value="UnauthorizedAccess">Unauthorized Access</SelectItem>
                                                    <SelectItem value="CopyrightViolation">Copyright Violation</SelectItem>
                                                    <SelectItem value="BruteForce">Brute Force</SelectItem>
                                                    <SelectItem value="C2">Command & Control</SelectItem>
                                                    <SelectItem value="Other">Other</SelectItem>
                                                </SelectContent>
                                            </Select>
                                        </div>
                                        <div>
                                            <Label htmlFor="email_ids">Email IDs (Optional)</Label>
                                            <Input
                                                id="email_ids"
                                                name="email_ids"
                                                placeholder="Comma-separated UUIDs"
                                            />
                                            <p className="text-sm text-gray-500 mt-1">
                                                Leave empty to create a standalone ticket
                                            </p>
                                        </div>
                                    </div>
                                    <DialogFooter className="mt-4">
                                        <Button type="submit">Create</Button>
                                    </DialogFooter>
                                </form>
                            </DialogContent>
                        </Dialog>
                    </div>
                </main>
            </div>
        </div>
    )
}
