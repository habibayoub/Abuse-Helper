import './index.css'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import Dashboard from '@/components/page/dashboard'
import EmailsPage from '@/components/page/emails'
import Login from '@/components/page/login'
import { AuthProvider, useAuth } from '@/contexts/AuthContext'

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
                        <Route path="*" element={<Navigate to="/dashboard" />} />
                    </Routes>
                </div>
            </Router>
        </AuthProvider>
    )
}

export default App
