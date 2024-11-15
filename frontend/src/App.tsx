import './index.css'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import Dashboard from '@/components/page/dashboard'
import EmailsPage from '@/components/page/emails'
import Login from '@/components/page/login'
import { AuthProvider, useAuth } from '@/contexts/AuthContext'
import TicketsPage from '@/components/page/tickets'
import { useEffect } from 'react';
import api from '@/lib/axios';  // Import our api instance instead of axios

// Update to use our api instance
function AxiosInterceptor({ children }: { children: React.ReactNode }) {
    const { accessToken } = useAuth();

    useEffect(() => {
        // Add a request interceptor to our api instance
        const interceptor = api.interceptors.request.use(
            (config) => {
                if (accessToken) {
                    config.headers.Authorization = `Bearer ${accessToken}`;
                }
                return config;
            },
            (error) => {
                return Promise.reject(error);
            }
        );

        // Clean up the interceptor when component unmounts
        return () => {
            api.interceptors.request.eject(interceptor);
        };
    }, [accessToken]);

    return <>{children}</>;
}

function ProtectedRoute({ children }: { children: React.ReactNode }) {
    const { authenticated } = useAuth();

    if (authenticated && window.location.pathname === '/') {
        return <Navigate to="/dashboard" />;
    }

    return authenticated ? <>{children}</> : <Navigate to="/login" />;
}

// Update the Login route to redirect if already authenticated
function LoginRoute() {
    const { authenticated } = useAuth();
    return authenticated ? <Navigate to="/dashboard" /> : <Login />;
}

function App() {
    return (
        <AuthProvider>
            <Router>
                <AxiosInterceptor>
                    <div className="h-screen w-screen">
                        <Routes>
                            <Route path="/login" element={<LoginRoute />} />
                            <Route path="/" element={<Navigate to="/dashboard" />} />
                            <Route
                                path="/dashboard"
                                element={
                                    <ProtectedRoute>
                                        <Dashboard />
                                    </ProtectedRoute>
                                }
                            />
                            <Route
                                path="/emails"
                                element={
                                    <ProtectedRoute>
                                        <EmailsPage />
                                    </ProtectedRoute>
                                }
                            />
                            <Route
                                path="/tickets"
                                element={
                                    <ProtectedRoute>
                                        <TicketsPage />
                                    </ProtectedRoute>
                                }
                            />
                            <Route path="*" element={<Navigate to="/dashboard" />} />
                        </Routes>
                    </div>
                </AxiosInterceptor>
            </Router>
        </AuthProvider>
    )
}

export default App
