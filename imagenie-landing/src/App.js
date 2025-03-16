import React, { useEffect, useState } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Header from './components/Header';
import Features from './components/Features';
import Showcase from './components/Showcase';
import WhyImagenie from './components/WhyImagenie';
import Download from './components/Download';
import Roadmap from './components/Roadmap';
import Footer from './components/Footer';
import ThemeToggle from './components/ThemeToggle';
import PrivacyPolicy from './pages/PrivacyPolicy';
import TermsOfService from './pages/TermsOfService';

// Import the web component
import 'img-comparison-slider';

function HomePage({ theme, toggleTheme }) {
  return (
    <div className="bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-200">
      <ThemeToggle theme={theme} toggleTheme={toggleTheme} />
      <Header />
      <Features />
      <Showcase />
      <WhyImagenie />
      <Download />
      <Roadmap />
      <Footer />
    </div>
  );
}

function App() {
  const [theme, setTheme] = useState(
    localStorage.theme === 'dark' || 
    (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches) 
      ? 'dark' 
      : 'light'
  );

  useEffect(() => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
      localStorage.theme = 'dark';
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.theme = 'light';
    }
  }, [theme]);

  const toggleTheme = () => {
    setTheme(theme === 'dark' ? 'light' : 'dark');
  };

  return (
    <Router>
      <Routes>
        <Route path="/" element={<HomePage theme={theme} toggleTheme={toggleTheme} />} />
        <Route path="/privacy-policy" element={<PrivacyPolicy />} />
        <Route path="/terms-of-service" element={<TermsOfService />} />
      </Routes>
    </Router>
  );
}

export default App; 