import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
    AlertTriangle,
    BarChart2,
    Bell,
    Download,
    Eye,
    FileText,
    Home,
    Mail,
    Menu,
    Search,
    Settings,
    Ticket,
    Users,
    X
} from "lucide-react"
import { Bar } from "react-chartjs-2"
import { Chart as ChartJS, CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend } from "chart.js"

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const PANTONE_301 = "#003F87"

export default function Dashboard() {
    const [sidebarOpen, setSidebarOpen] = useState(false)
    const [activeTab, setActiveTab] = useState("year")

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

    const topProvincesData = {
        labels: ["Ontario", "Quebec", "British Columbia", "Alberta", "Manitoba"],
        datasets: [
            {
                label: "Reports",
                data: [120, 90, 70, 60, 50],
                backgroundColor: [
                    PANTONE_301,
                    `${PANTONE_301}CC`, // 80% opacity
                    `${PANTONE_301}99`, // 60% opacity
                    `${PANTONE_301}66`, // 40% opacity
                    `${PANTONE_301}33`, // 20% opacity
                ],
            }
        ]
    }

    const toggleSidebar = () => {
        setSidebarOpen(!sidebarOpen)
    }

    return (
        <div className="flex h-full w-full bg-gray-100">
            {/* Sidebar */}
            <aside className={`bg-white w-64 fixed h-full z-30 border-r border-gray-200 shadow-sm transition-transform duration-300 ease-in-out ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'} lg:translate-x-0`}>
                <div className="p-4 flex justify-between items-center border-b border-gray-200">
                    <h2 className="text-2xl font-bold" style={{ color: PANTONE_301 }}>AbuseHelper</h2>
                    <Button variant="ghost" size="icon" onClick={toggleSidebar} className="lg:hidden">
                        <X className="h-6 w-6" />
                    </Button>
                </div>
                <nav className="mt-4">
                    <a href="#" className="block py-2 px-4 bg-blue-50 border-r-4" style={{ color: PANTONE_301, borderColor: PANTONE_301 }}>
                        <span className="flex items-center">
                            <Home className="mr-2" size={20} />
                            Dashboard
                        </span>
                    </a>
                    <a href="#" className="block py-2 px-4 text-gray-600 hover:bg-gray-50">
                        <span className="flex items-center">
                            <AlertTriangle className="mr-2" size={20} />
                            Abuse Reports
                        </span>
                    </a>
                    <a href="#" className="block py-2 px-4 text-gray-600 hover:bg-gray-50">
                        <span className="flex items-center">
                            <Ticket className="mr-2" size={20} />
                            Tickets
                        </span>
                    </a>
                    <a href="#" className="block py-2 px-4 text-gray-600 hover:bg-gray-50">
                        <span className="flex items-center">
                            <Users className="mr-2" size={20} />
                            Users
                        </span>
                    </a>
                    <a href="#" className="block py-2 px-4 text-gray-600 hover:bg-gray-50">
                        <span className="flex items-center">
                            <BarChart2 className="mr-2" size={20} />
                            Analytics
                        </span>
                    </a>
                    <a href="#" className="block py-2 px-4 text-gray-600 hover:bg-gray-50">
                        <span className="flex items-center">
                            <Settings className="mr-2" size={20} />
                            Settings
                        </span>
                    </a>
                </nav>
            </aside>

            {/* Main Content */}
            <div className="flex-1 flex flex-col overflow-hidden lg:ml-64">
                {/* Header */}
                <header className="bg-white shadow-sm">
                    <div className="flex items-center justify-between p-4">
                        <div className="flex items-center">
                            <Button variant="ghost" size="icon" onClick={toggleSidebar} className="lg:hidden">
                                <Menu className="h-6 w-6" />
                            </Button>
                            <h1 className="text-xl font-semibold ml-4">Overview</h1>
                        </div>
                        <div className="flex items-center space-x-4">
                            <div className="relative hidden md:block">
                                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" size={20} />
                                <Input type="text" placeholder="Search" className="pl-10 pr-4 py-2 rounded-full w-64" />
                            </div>
                            <Button variant="ghost" size="icon">
                                <Bell className="h-6 w-6" />
                            </Button>
                            <Button variant="ghost" size="icon">
                                <img src="/placeholder.svg?height=32&width=32" width={32} height={32} className="rounded-full" alt="User avatar" />
                            </Button>
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
                                    <CardTitle className="text-sm font-medium">IPs Discovered</CardTitle>
                                    <Ticket className="h-4 w-4 text-muted-foreground" />
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
                                    <CardTitle className="text-sm font-medium">Emails Read</CardTitle>
                                    <FileText className="h-4 w-4 text-muted-foreground" />
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
                            <Card>
                                <CardHeader>
                                    <div className="flex justify-between items-center">
                                        <CardTitle className="text-xl font-bold">Top Provinces</CardTitle>
                                        <Button variant="ghost" size="sm">Overview</Button>
                                    </div>
                                </CardHeader>
                                <CardContent>
                                    <Bar data={topProvincesData} options={{
                                        indexAxis: 'y',
                                        responsive: true,
                                        maintainAspectRatio: false,
                                        plugins: {
                                            legend: {
                                                display: false,
                                            },
                                        },
                                    }} height={200} />
                                </CardContent>
                            </Card>
                        </div>

                        <Card>
                            <CardHeader>
                                <div className="flex justify-between items-center">
                                    <CardTitle className="text-xl font-bold">Recent Reports</CardTitle>
                                    <Button variant="outline" size="sm">
                                        <Download className="mr-2 h-4 w-4" /> Report
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent>
                                <div className="overflow-x-auto">
                                    <table className="w-full">
                                        <thead>
                                            <tr className="text-xs font-semibold tracking-wide text-left text-gray-500 uppercase border-b bg-gray-50">
                                                <th className="px-4 py-3">Report ID</th>
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
                                                    <Button variant="ghost" size="sm"><Eye className="h-4 w-4" /></Button>
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
                                                    <Button variant="ghost" size="sm"><Eye className="h-4 w-4" /></Button>
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
                                                    <Button variant="ghost" size="sm"><Eye className="h-4 w-4" /></Button>
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