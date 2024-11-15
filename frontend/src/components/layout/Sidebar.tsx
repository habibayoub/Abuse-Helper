import { ReactNode } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { Button } from "@/components/ui/button"
import {
    AlertTriangle,
    BarChart2,
    Home,
    Mail,
    Settings,
    Ticket,
    Users,
    LogOut,
    User,
} from "lucide-react"
import { useAuth } from '@/contexts/AuthContext';

interface NavItemProps {
    href: string;
    icon: ReactNode;
    children: ReactNode;
    isActive?: boolean;
}

const NavItem = ({ href, icon, children, isActive }: NavItemProps) => {
    const navigate = useNavigate();

    return (
        <a
            href="#"
            className={`block py-2 px-4 ${isActive
                ? 'bg-blue-50 border-r-4 border-[#0067a4] text-[#0067a4]'
                : 'text-gray-600 hover:bg-gray-50'
                }`}
            onClick={(e) => {
                e.preventDefault();
                navigate(href);
            }}
        >
            <span className="flex items-center">
                {icon}
                {children}
            </span>
        </a>
    );
};

interface SidebarProps {
    isOpen: boolean;
}

export default function Sidebar({ isOpen }: SidebarProps) {
    const { userInfo, logout } = useAuth();
    const location = useLocation();

    const isCurrentPath = (path: string) => location.pathname === path;

    return (
        <aside className={`bg-white w-64 fixed h-full z-30 border-r border-gray-200 shadow-sm transition-transform duration-300 ease-in-out ${isOpen ? 'translate-x-0' : '-translate-x-full'} lg:translate-x-0 flex flex-col`}>
            <div className="p-5 flex flex-col items-center border-b border-gray-200">
                <img src="/bell-logo.svg" alt="Bell Logo" className="w-16 h-16" />
                <h2 className="text-lg font-extrabold tracking-wider text-[#0067a4]">
                    ABUSE HELPER
                </h2>
            </div>

            <div className="py-4 px-5 border-b border-gray-200">
                <div className="flex items-center space-x-3">
                    <User className="h-6 w-6 text-[#0067a4]" />
                    <div>
                        <p className="font-semibold">{userInfo?.name || 'User'}</p>
                        <p className="text-xs text-gray-400">{userInfo?.email || 'Email'}</p>
                        <p className="text-xs text-gray-400">
                            Role: {userInfo?.role || 'No role assigned'}
                        </p>
                    </div>
                </div>
            </div>

            <nav className="mt-2 flex-grow text-sm">
                <NavItem
                    href="/dashboard"
                    icon={<Home className="mr-3" size={18} />}
                    isActive={isCurrentPath('/dashboard')}
                >
                    Dashboard
                </NavItem>
                <NavItem
                    href="/reports"
                    icon={<AlertTriangle className="mr-2" size={18} />}
                    isActive={isCurrentPath('/reports')}
                >
                    Threat Reports
                </NavItem>
                <NavItem
                    href="/tickets"
                    icon={<Ticket className="mr-2" size={18} />}
                    isActive={isCurrentPath('/tickets')}
                >
                    Tickets
                </NavItem>
                <NavItem
                    href="/ip-addresses"
                    icon={<Users className="mr-2" size={18} />}
                    isActive={isCurrentPath('/ip-addresses')}
                >
                    IP Addresses
                </NavItem>
                <NavItem
                    href="/emails"
                    icon={<Mail className="mr-2" size={18} />}
                    isActive={isCurrentPath('/emails')}
                >
                    Emails
                </NavItem>
                <NavItem
                    href="/analytics"
                    icon={<BarChart2 className="mr-2" size={18} />}
                    isActive={isCurrentPath('/analytics')}
                >
                    Analytics
                </NavItem>
                <NavItem
                    href="/settings"
                    icon={<Settings className="mr-2" size={18} />}
                    isActive={isCurrentPath('/settings')}
                >
                    Settings
                </NavItem>
            </nav>

            <div className="px-4 py-2 border-t border-gray-200">
                <Button variant="outline" className="w-full flex items-center justify-center" onClick={logout}>
                    <LogOut className="mr-2 h-4 w-4" />
                    Log Out
                </Button>
            </div>

            <div className="p-4 text-center text-xs text-gray-500">
                © {new Date().getFullYear()} Abuse Helper
            </div>
        </aside>
    );
}
