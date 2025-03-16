import React, { useState, useEffect } from 'react';
import { useInView } from 'react-intersection-observer';

const Header = () => {
  const { ref: titleRef, inView: titleInView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const { ref: imageRef, inView: imageInView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const [useWebComponent, setUseWebComponent] = useState(false);

  useEffect(() => {
    // Check if the custom element is defined
    const isWebComponentDefined = !!customElements.get('img-comparison-slider');
    console.log('Header - img-comparison-slider defined:', isWebComponentDefined);
    setUseWebComponent(isWebComponentDefined);
    
    // If not defined, try to import it
    if (!isWebComponentDefined) {
      import('img-comparison-slider')
        .then(() => {
          console.log('Header - img-comparison-slider imported successfully');
          setUseWebComponent(true);
        })
        .catch(error => {
          console.error('Header - Failed to import img-comparison-slider:', error);
        });
    }
  }, []);

  return (
    <header className="relative overflow-hidden bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-800 dark:to-gray-900 min-h-screen flex items-center">
      <div className="absolute inset-0 bg-grid-pattern opacity-10"></div>
      <div className="container mx-auto px-6 py-16 md:py-24 relative z-10">
        <div className="flex flex-col md:flex-row items-center">
          <div 
            ref={titleRef}
            className={`md:w-1/2 transition-opacity duration-1000 ease-out ${
              titleInView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
            }`}
            style={{ transitionDelay: '0.1s' }}
          >
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6 bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-indigo-600 dark:from-blue-400 dark:to-indigo-400">
              Your Personal Genie for Stunning Images
            </h1>
            <p className="text-lg md:text-xl mb-8 text-gray-700 dark:text-gray-300 max-w-xl">
              Imagenie is a sleek, privacy-first desktop application that brings advanced AI image processing to your fingertips. From stunning upscaling to precise background removal.
            </p>
            <div className="flex flex-col sm:flex-row gap-4">
              <a href="https://github.com/zhongweili/imagenie/releases" className="btn-primary inline-flex items-center justify-center px-6 py-3 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white font-medium transition-all">
                <i className="fas fa-download mr-2"></i> Download Now
              </a>
              <a href="#features" className="inline-flex items-center justify-center px-6 py-3 rounded-lg bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 hover:border-indigo-500 dark:hover:border-indigo-500 text-gray-700 dark:text-gray-300 font-medium transition-all">
                <i className="fas fa-info-circle mr-2"></i> Learn More
              </a>
            </div>
          </div>
          <div 
            ref={imageRef}
            className={`md:w-1/2 mt-12 md:mt-0 transition-opacity duration-1000 ease-out ${
              imageInView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
            }`}
            style={{ transitionDelay: '0.3s' }}
          >
            <div className="relative mx-auto max-w-lg">
              <div className="absolute -inset-1 bg-gradient-to-r from-indigo-500 to-purple-500 rounded-lg blur opacity-30 dark:opacity-40 animate-pulse"></div>
              <div className="relative rounded-lg shadow-xl overflow-hidden">
                {useWebComponent ? (
                  <img-comparison-slider hover="hover">
                    <img slot="before" src="/images/case1.png" alt="Before Processing" className="w-full" />
                    <img slot="after" src="/images/case1_restored.png" alt="After Processing" className="w-full" />
                    <div slot="handle" className="handle-icon">
                      <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <circle cx="12" cy="12" r="10" fill="white"></circle>
                        <path d="M8 12L16 12" stroke="#333"></path>
                        <path d="M12 8L12 16" stroke="#333"></path>
                      </svg>
                    </div>
                  </img-comparison-slider>
                ) : (
                  <div className="comparison-slider">
                    <img src="/images/case1_restored.png" alt="After Processing" className="w-full" />
                    <div className="before">
                      <img src="/images/case1.png" alt="Before Processing" className="w-full" />
                    </div>
                    <div className="slider-line"></div>
                    <div className="slider-handle">
                      <i className="fas fa-arrows-alt-h"></i>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="absolute bottom-10 left-1/2 transform -translate-x-1/2 animate-bounce">
        <a href="#features" className="text-gray-500 dark:text-gray-400">
          <i className="fas fa-chevron-down text-2xl"></i>
        </a>
      </div>
    </header>
  );
};

export default Header; 