import React from 'react';
import { Link } from 'react-router-dom';

const Footer = () => {
  return (
    <footer className="bg-gray-900 text-gray-400 py-12">
      <div className="container mx-auto px-6">
        <div className="flex flex-col md:flex-row justify-between items-center">
          <div className="mb-6 md:mb-0">
            <h2 className="text-2xl font-bold text-white mb-2">Imagenie</h2>
            <p className="text-sm">Your Personal Genie for Stunning Images</p>
          </div>
          <div className="flex space-x-6">
            <a href="https://github.com/zhongweili/imagenie" className="text-gray-400 hover:text-white transition-colors">
              <i className="fab fa-github text-xl"></i>
            </a>
          </div>
        </div>
        <div className="border-t border-gray-800 mt-8 pt-8 flex flex-col md:flex-row justify-between items-center">
          <p className="text-sm">© {new Date().getFullYear()} Imagenie. All rights reserved.</p>
          <div className="mt-4 md:mt-0">
            <Link to="/privacy-policy" className="text-sm text-gray-500 hover:text-gray-300 transition-colors mr-4">Privacy Policy</Link>
            <Link to="/terms-of-service" className="text-sm text-gray-500 hover:text-gray-300 transition-colors">Terms of Service</Link>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer; 