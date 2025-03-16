import React, { useEffect, useState } from 'react';
import { useInView } from 'react-intersection-observer';

const ShowcaseItem = ({ title, beforeImage, afterImage, description, delay, useWebComponent }) => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  return (
    <div 
      ref={ref}
      className={`transition-opacity duration-1000 ease-out ${
        inView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
      }`}
      style={{ transitionDelay: `${delay}s` }}
    >
      <h3 className="text-2xl font-semibold mb-4">{title}</h3>
      <div className="rounded-lg overflow-hidden shadow-xl">
        {useWebComponent ? (
          <img-comparison-slider>
            <img slot="before" src={beforeImage} alt={`Before ${title}`} className="w-full" />
            <img slot="after" src={afterImage} alt={`After ${title}`} className={`w-full ${title === 'Background Removal' ? 'bg-white' : ''}`} />
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
            <img src={afterImage} alt={`After ${title}`} className={`w-full ${title === 'Background Removal' ? 'bg-white' : ''}`} />
            <div className="before">
              <img src={beforeImage} alt={`Before ${title}`} className="w-full" />
            </div>
            <div className="slider-line"></div>
            <div className="slider-handle">
              <i className="fas fa-arrows-alt-h"></i>
            </div>
          </div>
        )}
      </div>
      <p className="mt-4 text-gray-600 dark:text-gray-400">
        {description}
      </p>
    </div>
  );
};

const Showcase = () => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });
  
  const [useWebComponent, setUseWebComponent] = useState(false);

  useEffect(() => {
    // Check if the custom element is defined
    const isWebComponentDefined = !!customElements.get('img-comparison-slider');
    console.log('img-comparison-slider defined:', isWebComponentDefined);
    setUseWebComponent(isWebComponentDefined);
    
    // If not defined, try to import it
    if (!isWebComponentDefined) {
      import('img-comparison-slider')
        .then(() => {
          console.log('img-comparison-slider imported successfully');
          setUseWebComponent(true);
        })
        .catch(error => {
          console.error('Failed to import img-comparison-slider:', error);
        });
    }
  }, []);

  const showcaseItems = [
    {
      title: 'Face Restoration',
      beforeImage: '/images/case1.png',
      afterImage: '/images/case1_restored.png',
      description: 'Enhance facial features with AI-driven precision for perfect portraits.',
      delay: 0.1,
    },
    {
      title: 'Image Upscaling',
      beforeImage: '/images/case2.png',
      afterImage: '/images/case2_upscaled.png',
      description: 'Transform low-resolution images into crisp, detailed visuals with our advanced upscaling technology.',
      delay: 0.3,
    },
    {
      title: 'Background Removal',
      beforeImage: '/images/case3.png',
      afterImage: '/images/case3_removed.png',
      description: 'Effortlessly remove backgrounds with pixel-perfect precision for clean, professional results.',
      delay: 0.5,
    }
  ];

  return (
    <section className="py-20 bg-white dark:bg-gray-900">
      <div className="container mx-auto px-6">
        <div 
          ref={ref}
          className={`text-center mb-16 transition-opacity duration-1000 ease-out ${
            inView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
          }`}
        >
          <h2 className="text-3xl md:text-4xl font-bold mb-4">See Imagenie in Action</h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Experience the power of AI image processing with these real-world examples. Slide to compare before and after results.
          </p>
        </div>
        
        <div className="grid grid-cols-1 gap-16">
          {showcaseItems.map((item, index) => (
            <ShowcaseItem
              key={index}
              title={item.title}
              beforeImage={item.beforeImage}
              afterImage={item.afterImage}
              description={item.description}
              delay={item.delay}
              useWebComponent={useWebComponent}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export default Showcase; 