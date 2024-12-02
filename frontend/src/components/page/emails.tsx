import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { format } from "date-fns"
import { Eye, Menu, Mail, Send, Ticket, Search, RefreshCw } from "lucide-react"
import Sidebar from '@/components/layout/Sidebar';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { useForm } from "react-hook-form"
import api from '@/lib/axios'
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Badge } from "@/components/ui/badge"
import { Link } from "react-router-dom"
import { Checkbox } from "@/components/ui/checkbox"

// Define the Email interface based on the backend model
interface Email {
    id: string
    sender: string
    recipients: string[]
    subject: string
    body: string
    received_at: string
    is_sent: boolean
    ticket_ids: string[]
}

interface EmailForm {
    to: string
    subject: string
    body: string
}

interface SendEmailRequest {
    recipient: {
        email: string;
    };
    subject: string;
    body: string;
}

interface SearchFilters {
    has_tickets?: boolean;
    is_sent?: boolean;
}

interface SearchOptions {
    query: string;
    filters?: SearchFilters;
    from?: number;
    size?: number;
}

interface SearchResponse {
    hits: Email[];
    total: number;
}

export default function EmailsPage() {
    const [emails, setEmails] = useState<Email[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [sidebarOpen, setSidebarOpen] = useState(false)
    const [sendingEmail, setSendingEmail] = useState(false);
    const [selectedEmail, setSelectedEmail] = useState<Email | null>(null);
    const [refreshing, setRefreshing] = useState(false);
    const [isReply, setIsReply] = useState(false);
    const [searchQuery, setSearchQuery] = useState("")
    const [searching, setSearching] = useState(false)
    const [searchFilters, setSearchFilters] = useState<SearchFilters>({})

    useEffect(() => {
        fetchEmails();
    }, []);

    const fetchEmails = async () => {
        setRefreshing(true);
        try {
            const response = await api.get('/email/list');
            setEmails(response.data);
            setError(null);
        } catch (err) {
            console.error('Error fetching emails:', err);
            setError('Failed to fetch emails');
        } finally {
            setRefreshing(false);
            setLoading(false);
        }
    };

    const toggleSidebar = () => {
        setSidebarOpen(!sidebarOpen)
    }

    // Function to truncate text with ellipsis
    const truncateText = (text: string, maxLength: number) => {
        return text.length > maxLength ? `${text.substring(0, maxLength)}...` : text
    }

    const { register, handleSubmit, reset, setValue } = useForm<EmailForm>({
        defaultValues: {
            to: '',
            subject: '',
            body: ''
        }
    });

    // Function to handle reply
    const handleReply = (email: Email) => {
        setValue('to', email.sender);
        setValue('subject', `Re: ${email.subject}`);
        setValue('body', `\n\n> ${email.body.split('\n').join('\n> ')}`);
        setIsReply(true);
    };

    // Reset form when dialog closes
    const handleDialogClose = () => {
        reset();
        setIsReply(false);
    };

    const onSubmit = async (data: EmailForm) => {
        setSendingEmail(true);
        try {
            const emailRequest: SendEmailRequest = {
                recipient: {
                    email: data.to,
                },
                subject: data.subject,
                body: data.body
            };

            await api.post('/email/send', emailRequest);
            await fetchEmails();
            reset();
            setError(null);
            setIsReply(false);
        } catch (err) {
            console.error('Error sending email:', err);
            setError('Failed to send email');
        } finally {
            setSendingEmail(false);
        }
    };

    const sentEmails = emails.filter(email => email.is_sent)
    const receivedEmails = emails.filter(email => !email.is_sent)

    const EmailTable = ({ emails }: { emails: Email[] }) => (
        <div className="overflow-x-auto">
            <table className="w-full">
                <thead>
                    <tr className="text-xs font-semibold tracking-wide text-left text-gray-500 uppercase border-b bg-gray-50">
                        <th className="px-4 py-3">ID</th>
                        <th className="px-4 py-3">From</th>
                        <th className="px-4 py-3">To</th>
                        <th className="px-4 py-3">Subject</th>
                        <th className="px-4 py-3">Preview</th>
                        <th className="px-4 py-3">Date</th>
                        <th className="px-4 py-3">Tickets</th>
                        <th className="px-4 py-3">Actions</th>
                    </tr>
                </thead>
                <tbody className="bg-white divide-y">
                    {emails && emails.map((email) => (
                        <tr key={email.id} className="text-gray-700">
                            <td className="px-4 py-3">
                                <code className="bg-gray-100 px-2 py-1 rounded text-sm">
                                    {email.id.slice(0, 8)}
                                </code>
                            </td>
                            <td className="px-4 py-3">
                                {truncateText(email.sender, 30)}
                            </td>
                            <td className="px-4 py-3">
                                {truncateText(email.recipients.join(", "), 30)}
                            </td>
                            <td className="px-4 py-3">
                                {truncateText(email.subject, 40)}
                            </td>
                            <td className="px-4 py-3">
                                {truncateText(email.body, 50)}
                            </td>
                            <td className="px-4 py-3">
                                {format(new Date(email.received_at), 'MMM d, yyyy HH:mm')}
                            </td>
                            <td className="px-4 py-3">
                                <div className="flex gap-2 flex-wrap">
                                    {email.ticket_ids && email.ticket_ids.length > 0 ? (
                                        email.ticket_ids.map((ticketId) => (
                                            <Link key={ticketId} to={`/tickets/${ticketId}`}>
                                                <Badge variant="secondary" className="cursor-pointer hover:bg-secondary/80">
                                                    <Ticket className="w-3 h-3 mr-1" />
                                                    {ticketId.slice(0, 8)}
                                                </Badge>
                                            </Link>
                                        ))
                                    ) : (
                                        <Badge variant="outline">No tickets</Badge>
                                    )}
                                </div>
                            </td>
                            <td className="px-4 py-3">
                                <div className="flex gap-2">
                                    <Dialog>
                                        <DialogTrigger asChild>
                                            <Button
                                                variant="outline"
                                                size="sm"
                                                onClick={() => setSelectedEmail(email)}
                                            >
                                                <Eye className="h-4 w-4" />
                                            </Button>
                                        </DialogTrigger>
                                        <DialogContent className="sm:max-w-[725px]">
                                            <DialogHeader>
                                                <DialogTitle>Email Details</DialogTitle>
                                            </DialogHeader>
                                            {selectedEmail && (
                                                <div className="space-y-4">
                                                    <div>
                                                        <h3 className="font-semibold">From</h3>
                                                        <p>{selectedEmail.sender}</p>
                                                    </div>
                                                    <div>
                                                        <h3 className="font-semibold">To</h3>
                                                        <p>{selectedEmail.recipients.join(", ")}</p>
                                                    </div>
                                                    <div>
                                                        <h3 className="font-semibold">Subject</h3>
                                                        <p>{selectedEmail.subject}</p>
                                                    </div>
                                                    <div>
                                                        <h3 className="font-semibold">Content</h3>
                                                        <p className="whitespace-pre-wrap">{selectedEmail.body}</p>
                                                    </div>
                                                    <div>
                                                        <h3 className="font-semibold">Date</h3>
                                                        <p>{format(new Date(selectedEmail.received_at), 'PPpp')}</p>
                                                    </div>
                                                    <div className="flex justify-end">
                                                        <Button
                                                            variant="default"
                                                            onClick={() => {
                                                                handleReply(selectedEmail);
                                                            }}
                                                        >
                                                            <Mail className="mr-2 h-4 w-4" /> Reply
                                                        </Button>
                                                    </div>
                                                </div>
                                            )}
                                        </DialogContent>
                                    </Dialog>
                                    {!email.is_sent && (
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={() => {
                                                handleReply(email);
                                            }}
                                        >
                                            <Mail className="h-4 w-4" />
                                        </Button>
                                    )}
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {(!emails || emails.length === 0) && (
                <div className="text-center py-4 text-gray-500">
                    No emails found
                </div>
            )}
        </div>
    )

    const handleSearch = async (query: string, isInSentTab: boolean) => {
        if (!query.trim() && !searchFilters.has_tickets) {
            fetchEmails()
            return
        }

        setSearching(true)
        try {
            const searchOptions: SearchOptions = {
                query: query.trim(),
                filters: {
                    ...searchFilters,
                    is_sent: isInSentTab
                },
                size: 50
            }

            console.log("Search options:", searchOptions)

            const response = await api.get<SearchResponse>('/email/search', {
                params: searchOptions
            })

            console.log("Search response:", response.data)

            if (response.data && response.data.hits) {
                setEmails(response.data.hits)
                setError(null)
            } else {
                console.log("No emails found in search results")
                setEmails([])
            }
        } catch (err) {
            console.error('Error searching emails:', err)
            setError('Failed to search emails')
        } finally {
            setSearching(false)
        }
    }

    useEffect(() => {
        const timeoutId = setTimeout(() => {
            const activeTab = document.querySelector('[data-state="active"]')?.getAttribute('value')
            const isInSentTab = activeTab === 'sent'
            handleSearch(searchQuery, isInSentTab)
        }, 500)

        return () => clearTimeout(timeoutId)
    }, [searchQuery, searchFilters])

    const SearchFiltersComponent = () => (
        <div className="flex gap-2 items-center mt-2">
            <div className="flex items-center space-x-2">
                <Checkbox
                    id="hasTickets"
                    checked={searchFilters.has_tickets}
                    onCheckedChange={(checked) =>
                        setSearchFilters(prev => ({
                            ...prev,
                            has_tickets: checked as boolean
                        }))
                    }
                />
                <label
                    htmlFor="hasTickets"
                    className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                    Has Tickets
                </label>
            </div>
        </div>
    )

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
                            <h1 className="text-xl font-semibold ml-4">Emails</h1>
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
                                        <CardTitle className="text-xl font-bold">Email Inbox</CardTitle>
                                        <p className="text-sm text-muted-foreground">
                                            {searching ? "Searching..." : `Showing ${emails.length} emails`}
                                        </p>
                                    </div>
                                    <div className="flex gap-2 items-center">
                                        <div className="relative w-64">
                                            <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                                            <Input
                                                placeholder="Search emails..."
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
                                        <Dialog open={isReply} onOpenChange={(open) => !open && handleDialogClose()}>
                                            <DialogTrigger asChild>
                                                <Button variant="default">
                                                    <Send className="mr-2 h-4 w-4" /> Send Email
                                                </Button>
                                            </DialogTrigger>
                                            <DialogContent className="sm:max-w-[525px]">
                                                <DialogHeader>
                                                    <DialogTitle>{isReply ? 'Reply to Email' : 'Send Email'}</DialogTitle>
                                                </DialogHeader>
                                                <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                                                    <div className="space-y-2">
                                                        <label htmlFor="to" className="text-sm font-medium">
                                                            To
                                                        </label>
                                                        <Input
                                                            id="to"
                                                            placeholder="recipient@example.com"
                                                            {...register("to", {
                                                                required: "Email is required",
                                                            })}
                                                        />
                                                    </div>
                                                    <div className="space-y-2">
                                                        <label htmlFor="subject" className="text-sm font-medium">
                                                            Subject
                                                        </label>
                                                        <Input
                                                            id="subject"
                                                            placeholder="Email subject"
                                                            {...register("subject", { required: true })}
                                                        />
                                                    </div>
                                                    <div className="space-y-2">
                                                        <label htmlFor="body" className="text-sm font-medium">
                                                            Message
                                                        </label>
                                                        <Textarea
                                                            id="body"
                                                            placeholder="Type your message here"
                                                            rows={5}
                                                            {...register("body", { required: true })}
                                                        />
                                                    </div>
                                                    <div className="flex justify-end gap-2">
                                                        <Button type="button" variant="outline" onClick={handleDialogClose}>
                                                            Cancel
                                                        </Button>
                                                        <Button type="submit" disabled={sendingEmail}>
                                                            {sendingEmail ? (
                                                                "Sending..."
                                                            ) : (
                                                                <>
                                                                    <Send className="mr-2 h-4 w-4" /> Send
                                                                </>
                                                            )}
                                                        </Button>
                                                    </div>
                                                </form>
                                            </DialogContent>
                                        </Dialog>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={fetchEmails}
                                            disabled={refreshing}
                                        >
                                            {refreshing ? (
                                                "Refreshing..."
                                            ) : (
                                                <>
                                                    <Mail className="mr-2 h-4 w-4" /> Refresh
                                                </>
                                            )}
                                        </Button>
                                    </div>
                                </div>
                            </CardHeader>
                            <CardContent>
                                {loading ? (
                                    <div className="text-center py-4">Loading emails...</div>
                                ) : error ? (
                                    <div className="text-center text-red-500 py-4">{error}</div>
                                ) : (
                                    <Tabs defaultValue="received" className="w-full">
                                        <TabsList className="mb-4">
                                            <TabsTrigger value="received">
                                                Received ({receivedEmails.length})
                                            </TabsTrigger>
                                            <TabsTrigger value="sent">
                                                Sent ({sentEmails.length})
                                            </TabsTrigger>
                                        </TabsList>
                                        <TabsContent value="received">
                                            <EmailTable emails={receivedEmails} />
                                        </TabsContent>
                                        <TabsContent value="sent">
                                            <EmailTable emails={sentEmails} />
                                        </TabsContent>
                                    </Tabs>
                                )}
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    )
}
