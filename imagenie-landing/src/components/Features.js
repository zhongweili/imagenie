import React from 'react';
import { useInView } from 'react-intersection-observer';

const FeatureCard = ({ icon, title, description, delay }) => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  return (
    <div 
      ref={ref}
      className={`feature-card bg-white dark:bg-gray-900 rounded-xl p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-opacity duration-1000 ease-out ${
        inView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
      }`}
      style={{ transitionDelay: `${delay}s` }}
    >
      <div className="w-12 h-12 bg-indigo-100 dark:bg-indigo-900 rounded-full flex items-center justify-center mb-4">
        <i className={`fas ${icon} text-indigo-600 dark:text-indigo-400 text-xl`}></i>
      </div>
      <h3 className="text-xl font-semibold mb-2">{title}</h3>
      <p className="text-gray-600 dark:text-gray-400">
        {description}
      </p>
    </div>
  );
};

const Features = () => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const features = [
    {
      icon: 'fa-expand',
      title: 'Intelligent Image Upscaling',
      description: 'Restore details and clarity in low-resolution images with AI-powered upscaling technology.',
      delay: 0.1,
    },
    {
      icon: 'fa-user',
      title: 'Face Restoration',
      description: 'Fix and enhance facial features with AI-driven precision for perfect portraits.',
      delay: 0.2,
    },
    {
      icon: 'fa-cut',
      title: 'Background Removal',
      description: 'Automatically isolate subjects with pixel-perfect accuracy for clean compositions.',
      delay: 0.3,
    },
    {
      icon: 'fa-sliders-h',
      title: 'Real-Time Comparison',
      description: 'Swipe through before & after sliders to see the magic unfold in real-time.',
      delay: 0.4,
    },
    {
      icon: 'fa-globe',
      title: 'Fully Open Source',
      description: 'MIT Licensed, free and open source.',
      delay: 0.5,
    },
    {
      icon: 'fa-lock',
      title: '100% Local Processing',
      description: 'No cloud, no risks—complete control over your image privacy and security.',
      delay: 0.6,
    },
  ];

  return (
    <section id="features" className="py-20 bg-gray-50 dark:bg-gray-800">
      <div className="container mx-auto px-6">
        <div 
          ref={ref}
          className={`text-center mb-16 transition-opacity duration-1000 ease-out ${
            inView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
          }`}
        >
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Features at a Glance</h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Imagenie brings powerful AI tools to your desktop with complete privacy and control.
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <FeatureCard
              key={index}
              icon={feature.icon}
              title={feature.title}
              description={feature.description}
              delay={feature.delay}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features; 