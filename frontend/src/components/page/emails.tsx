import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { format } from "date-fns"
import { Eye, Menu, Mail, Send } from "lucide-react"
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
import api from '@/lib/axios'  // Import the axios instance

// Define the Email interface based on the backend model
interface Email {
    id: string
    sender: string
    recipients: string[]
    subject: string
    body: string
    received_at: string
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

export default function EmailsPage() {
    const [emails, setEmails] = useState<Email[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [sidebarOpen, setSidebarOpen] = useState(false)
    const [sendingEmail, setSendingEmail] = useState(false);
    const [selectedEmail, setSelectedEmail] = useState<Email | null>(null);
    const [refreshing, setRefreshing] = useState(false);

    useEffect(() => {
        fetchEmails();
        // Set up polling every minute
        // const interval = setInterval(fetchEmails, 60000);
        // return () => clearInterval(interval);
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

    const { register, handleSubmit, reset, formState: { errors } } = useForm<EmailForm>({
        defaultValues: {
            to: '',
            subject: '',
            body: ''
        }
    });

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
        } catch (err) {
            console.error('Error sending email:', err);
            setError('Failed to send email');
        } finally {
            setSendingEmail(false);
        }
    };

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
                                            Showing {emails.length} received emails
                                        </p>
                                    </div>
                                    <div className="flex gap-2">
                                        <Dialog>
                                            <DialogTrigger asChild>
                                                <Button variant="default">
                                                    <Send className="mr-2 h-4 w-4" /> Send Email
                                                </Button>
                                            </DialogTrigger>
                                            <DialogContent className="sm:max-w-[525px]">
                                                <DialogHeader>
                                                    <DialogTitle>Send Email</DialogTitle>
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
                                                                pattern: {
                                                                    value: /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i,
                                                                    message: "Invalid email address"
                                                                }
                                                            })}
                                                        />
                                                        {errors.to && <p className="text-sm text-red-500">{errors.to.message}</p>}
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
                                                        <Button type="button" variant="outline" onClick={() => reset()}>
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
                                    <div className="overflow-x-auto">
                                        <table className="w-full">
                                            <thead>
                                                <tr className="text-xs font-semibold tracking-wide text-left text-gray-500 uppercase border-b bg-gray-50">
                                                    <th className="px-4 py-3">From</th>
                                                    <th className="px-4 py-3">To</th>
                                                    <th className="px-4 py-3">Subject</th>
                                                    <th className="px-4 py-3">Preview</th>
                                                    <th className="px-4 py-3">Received</th>
                                                    <th className="px-4 py-3">Action</th>
                                                </tr>
                                            </thead>
                                            <tbody className="bg-white divide-y">
                                                {emails.map((email) => (
                                                    <tr key={email.id} className="text-gray-700">
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
                                                            <Dialog onOpenChange={(open) => !open && setSelectedEmail(null)}>
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
                                                                                <h3 className="font-semibold">Received</h3>
                                                                                <p>{format(new Date(selectedEmail.received_at), 'PPpp')}</p>
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

                                        {emails.length === 0 && (
                                            <div className="text-center py-4 text-gray-500">
                                                No emails found
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
