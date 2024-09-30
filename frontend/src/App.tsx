// import { useState, useEffect } from 'react'
import './index.css'
import Dashboard from '@/components/page/dashboard'

function App() {
  // const [message, setMessage] = useState<string>('')

  // useEffect(() => {
  //   fetch('/api/status')
  //     .then(response => response.text())
  //     .then(data => setMessage(data))
  //     .catch(error => console.error('Error:', error))
  // }, [])

  return (
    <Dashboard />
  )
}

export default App
