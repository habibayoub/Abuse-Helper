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

// Define the Email interface based on the backend model
interface Email {
    id: string
    from: string
    to: string[]
    subject: string
    body: string
    received_at: string
}

const DUMMY_EMAILS: Email[] = [
    {
        id: "1",
        from: "john.doe@example.com",
        to: ["alice.smith@company.com"],
        subject: "Weekly Project Update - Q1 Goals",
        body: "Hi Alice, I wanted to share the latest updates on our Q1 project goals. We've made significant progress on...",
        received_at: "2024-03-15T10:30:00Z"
    },
    {
        id: "2",
        from: "marketing@newsletter.com",
        to: ["subscribers@company.com"],
        subject: "🚀 March Newsletter: Latest Product Updates",
        body: "Discover what's new this month! We're excited to announce several new features...",
        received_at: "2024-03-14T15:45:00Z"
    },
    {
        id: "3",
        from: "support@saasplatform.com",
        to: ["dev.team@company.com", "ops.team@company.com"],
        subject: "Important: Security Update Required",
        body: "Dear Customer, We've released a critical security patch that requires your immediate attention...",
        received_at: "2024-03-14T08:15:00Z"
    },
    {
        id: "4",
        from: "hr@company.com",
        to: ["all-staff@company.com"],
        subject: "Reminder: Team Building Event Next Week",
        body: "Hello everyone! This is a friendly reminder about our upcoming team building event next Thursday...",
        received_at: "2024-03-13T16:20:00Z"
    },
    {
        id: "5",
        from: "david.wilson@partner.org",
        to: ["projects@company.com"],
        subject: "Partnership Proposal - Q2 2024",
        body: "Dear Project Team, Following our discussion last week, I'm pleased to submit our formal partnership proposal...",
        received_at: "2024-03-13T11:05:00Z"
    }
];


interface EmailForm {
    to: string
    subject: string
    body: string
}

export default function EmailsPage() {
    const [emails, setEmails] = useState<Email[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [sidebarOpen, setSidebarOpen] = useState(false)

    useEffect(() => {
        fetchEmails()
    }, [])

    const fetchEmails = async () => {
        try {
            const response = await fetch('/api/email/list')
            if (!response.ok) {
                throw new Error('Failed to fetch emails')
            }
            const data = await response.json()
            setEmails(data)
        } catch (err) {
            console.log('Using dummy data due to fetch error:', err)
            setEmails(DUMMY_EMAILS)
            setError(null)
        } finally {
            setLoading(false)
        }
    }

    const toggleSidebar = () => {
        setSidebarOpen(!sidebarOpen)
    }

    // Function to truncate text with ellipsis
    const truncateText = (text: string, maxLength: number) => {
        return text.length > maxLength ? `${text.substring(0, maxLength)}...` : text
    }

    const { register, handleSubmit, reset } = useForm<EmailForm>()

    const onSubmit = async (data: EmailForm) => {
        try {
            const response = await fetch('/api/email/send', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    recipient: data.to,
                    subject: data.subject,
                    body: data.body,
                }),
            })

            if (!response.ok) {
                throw new Error('Failed to send email')
            }

            // Refresh emails list and reset form
            fetchEmails()
            reset()
        } catch (err) {
            console.error('Error sending email:', err)
            setError('Failed to send email')
        }
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
                                                            {...register("to", { required: true })}
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
                                                        <Button type="button" variant="outline" onClick={() => reset()}>
                                                            Cancel
                                                        </Button>
                                                        <Button type="submit">
                                                            <Send className="mr-2 h-4 w-4" /> Send
                                                        </Button>
                                                    </div>
                                                </form>
                                            </DialogContent>
                                        </Dialog>
                                        <Button 
                                            variant="outline" 
                                            size="sm"
                                            onClick={fetchEmails}
                                        >
                                            <Mail className="mr-2 h-4 w-4" /> Refresh
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
                                                            {truncateText(email.from, 30)}
                                                        </td>
                                                        <td className="px-4 py-3">
                                                            {truncateText(email.to.join(", "), 30)}
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
                                                            <Button 
                                                                variant="outline" 
                                                                size="sm"
                                                                onClick={() => {
                                                                    // TODO: Implement view email details
                                                                    console.log('View email:', email.id)
                                                                }}
                                                            >
                                                                <Eye className="h-4 w-4" />
                                                            </Button>
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
