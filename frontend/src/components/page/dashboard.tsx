import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
    AlertTriangle,
    Menu,
    TicketPlus,
    TicketsIcon,
    Users,
    Mail,
    Eye
} from "lucide-react"
import { Bar, Pie } from "react-chartjs-2"
import { Chart as ChartJS, CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend, ArcElement } from "chart.js"
import Sidebar from "@/components/layout/Sidebar"

// Register ChartJS components
ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend, ArcElement)

// Define the primary color used throughout the dashboard
const PANTONE_301 = "#0067a4"

export default function Dashboard() {
    // State for controlling sidebar visibility on mobile
    const [sidebarOpen, setSidebarOpen] = useState(false)
    // State for controlling active tab in the Report Statistics section
    const [activeTab, setActiveTab] = useState("year")

    /**
     * Data for the Report Statistics bar chart
     */
    const reportStatisticsData = {
        labels: ["2014", "2015", "2016", "2017", "2018", "2019", "2020", "2021", "2022", "2023"],
        datasets: [
            {
                label: "IPs Discovered",
                data: [300, 450, 400, 480, 520, 600, 450, 500, 550, 480],
                backgroundColor: PANTONE_301,
            },
            {
                label: "New Reports",
                data: [200, 300, 250, 350, 400, 450, 350, 400, 450, 400],
                backgroundColor: `${PANTONE_301}80`, // 50% opacity
            }
        ]
    }

    /**
     * Data for the Popular Cities pie chart
     */
    const popularCitiesData = {
        labels: ["Toronto, ON", "Montreal, QC", "Vancouver, BC", "Calgary, AB", "Ottawa, ON", "Other"],
        datasets: [
            {
                data: [120, 90, 70, 60, 50, 110],
                backgroundColor: [
                    PANTONE_301,
                    `${PANTONE_301}CC`, // 80% opacity
                    `${PANTONE_301}99`, // 60% opacity
                    `${PANTONE_301}66`, // 40% opacity
                    `${PANTONE_301}33`, // 20% opacity
                    `${PANTONE_301}1A`, // 10% opacity
                ],
            }
        ]
    }

    /**
     * Toggle sidebar visibility
     */
    const toggleSidebar = () => {
        setSidebarOpen(!sidebarOpen)
    }

    return (
        <div className="flex h-full w-full bg-gray-100">
            {/* Sidebar */}
            <Sidebar isOpen={sidebarOpen} />

            {/* Main Content */}
            <div className="flex-1 flex flex-col overflow-hidden lg:ml-64">
                {/* Header with blurry background */}
                <header className="bg-white bg-opacity-70 backdrop-filter backdrop-blur-md shadow-sm sticky top-0 z-10">
                    <div className="flex items-center justify-between p-4">
                        <div className="flex items-center">
                            <Button variant="outline" size="icon" onClick={toggleSidebar} className="lg:hidden">
                                <Menu className="h-6 w-6" />
                            </Button>
                            <h1 className="text-xl font-semibold ml-4">Dashboard</h1>
                        </div>
                    </div>
                </header>

                {/* Dashboard Content */}
                <main className="flex-1 overflow-x-hidden overflow-y-auto bg-gray-100">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6 mb-8">
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Total Reports</CardTitle>
                                    <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">4,567</div>
                                    <p className="text-xs text-green-500">+10.5% from last week</p>
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">IP Addresses Discovered</CardTitle>
                                    <Users className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">3,890</div>
                                    <p className="text-xs text-green-500">+21.2% from last week</p>
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Emails Received</CardTitle>
                                    <Mail className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">1,234</div>
                                    <p className="text-xs text-red-500">-10.2% from last week</p>
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Tickets Created</CardTitle>
                                    <TicketPlus className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">1,156</div>
                                    <p className="text-xs text-green-500">+8.5% from last week</p>
                                </CardContent>
                            </Card>
                        </div>

                        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
                            <Card className="col-span-2">
                                <CardHeader>
                                    <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center space-y-2 sm:space-y-0">
                                        <div>
                                            <CardTitle className="text-xl font-bold">Report Statistics</CardTitle>
                                            <p className="text-sm text-muted-foreground">Total reports processed this year</p>
                                            <p className="text-2xl font-bold mt-2">89,456 <span className="text-sm" style={{ color: PANTONE_301 }}>+3.24%</span></p>
                                        </div>
                                        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full sm:w-auto">
                                            <TabsList className="grid w-full grid-cols-3 sm:w-auto">
                                                <TabsTrigger value="day">Day</TabsTrigger>
                                                <TabsTrigger value="month">Month</TabsTrigger>
                                                <TabsTrigger value="year">Year</TabsTrigger>
                                            </TabsList>
                                        </Tabs>
                                    </div>
                                </CardHeader>
                                <CardContent>
                                    <Bar data={reportStatisticsData} options={{ responsive: true, maintainAspectRatio: false }} height={300} />
                                </CardContent>
                            </Card>
                            <Card className="flex flex-col">
                                <CardHeader>
                                    <div className="flex justify-between items-center">
                                        <CardTitle className="text-xl font-bold">Popular Cities</CardTitle>
                                    </div>
                                </CardHeader>
                                <CardContent className="flex-grow">
                                    <div className="h-full">
                                        <Pie data={popularCitiesData} options={{
                                            responsive: true,
                                            maintainAspectRatio: false,
                                            plugins: {
                                                legend: {
                                                    position: 'bottom',
                                                    labels: {
                                                        boxWidth: 12,
                                                    }
                                                },
                                                tooltip: {
                                                    callbacks: {
                                                        label: (context) => {
                                                            const label = context.label || '';
                                                            const value = context.parsed || 0;
                                                            const total = context.dataset.data.reduce((acc, data) => acc + data, 0);
                                                            const percentage = ((value / total) * 100).toFixed(1);
                                                            return `${label}: ${percentage}%`;
                                                        }
                                                    }
                                                }
                                            },
                                        }} />
                                    </div>
                                </CardContent>
                            </Card>
                        </div>

                        <Card>
                            <CardHeader>
                                <div className="flex justify-between items-center">
                                    <CardTitle className="text-xl font-bold">Recent Tickets</CardTitle>
                                    <Button variant="outline" size="sm">
                                        <TicketsIcon className="mr-2 h-4 w-4" /> Tickets
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent>
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
                                            <tr className="text-gray-700">
                                                <td className="px-4 py-3">#1234</td>
                                                <td className="px-4 py-3">192.168.1.1</td>
                                                <td className="px-4 py-3">Phishing</td>
                                                <td className="px-4 py-3">2023-06-20</td>
                                                <td className="px-4 py-3">
                                                    <span className="px-2 py-1 font-semibold text-xs leading-tight text-green-700 bg-green-100 rounded-full">Resolved</span>
                                                </td>
                                                <td className="px-4 py-3">
                                                    <Button variant="outline" size="sm"><Eye className="h-4 w-4" /></Button>
                                                </td>
                                            </tr>
                                            <tr className="text-gray-700">
                                                <td className="px-4 py-3">#1235</td>
                                                <td className="px-4 py-3">10.0.0.1</td>
                                                <td className="px-4 py-3">Malware</td>
                                                <td className="px-4 py-3">2023-06-19</td>
                                                <td className="px-4 py-3">
                                                    <span className="px-2 py-1 font-semibold text-xs leading-tight text-yellow-700 bg-yellow-100 rounded-full">In Progress</span>
                                                </td>
                                                <td className="px-4 py-3">
                                                    <Button variant="outline" size="sm"><Eye className="h-4 w-4" /></Button>
                                                </td>
                                            </tr>
                                            <tr className="text-gray-700">
                                                <td className="px-4 py-3">#1236</td>
                                                <td className="px-4 py-3">172.16.0.1</td>
                                                <td className="px-4 py-3">Spam</td>
                                                <td className="px-4 py-3">2023-06-18</td>
                                                <td className="px-4 py-3">
                                                    <span className="px-2 py-1 font-semibold text-xs leading-tight text-red-700 bg-red-100 rounded-full">Urgent</span>
                                                </td>
                                                <td className="px-4 py-3">
                                                    <Button variant="outline" size="sm"><Eye className="h-4 w-4" /></Button>
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </div>
    )
}
