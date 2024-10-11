import './index.css'
import { AuthProvider } from '@/contexts/AuthContext'
import Login from '@/components/page/login'
import Dashboard from '@/components/page/dashboard'
import { useAuth } from '@/contexts/AuthContext'

function AppContent() {
  const { authenticated } = useAuth();

  return (
    <div className="h-screen w-screen">
      {authenticated ? <Dashboard /> : <Login />}
    </div>
  )
}

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  )
}

export default App
