import React from 'react';

const ThemeToggle = ({ theme, toggleTheme }) => {
  return (
    <div className="fixed top-4 right-4 z-50">
      <button 
        onClick={toggleTheme}
        className="p-2 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 focus:outline-none"
      >
        <i className={`fas ${theme === 'dark' ? 'fa-sun' : 'fa-moon'}`}></i>
      </button>
    </div>
  );
};

export default ThemeToggle; 