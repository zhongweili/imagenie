import React from 'react';
import { useInView } from 'react-intersection-observer';

const ReasonCard = ({ icon, title, description, delay }) => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  return (
    <div 
      ref={ref}
      className={`bg-white dark:bg-gray-900 rounded-xl p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-opacity duration-1000 ease-out ${
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

const WhyImagenie = () => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const reasons = [
    {
      icon: 'fa-code',
      title: 'Modern Tech Stack',
      description: 'Developed with the powerful Tauri 2 and Vue 3, delivering an ultra-lightweight desktop app.',
      delay: 0.1,
    },
    {
      icon: 'fa-bolt',
      title: 'AI-Enhanced Speed',
      description: 'Built on ONNX Runtime, optimized for lightning-fast processing on your local machine.',
      delay: 0.2,
    },
    {
      icon: 'fa-paint-brush',
      title: 'Stunning UI',
      description: 'Designed for an intuitive experience, Imagenie brings powerful tools in a user-friendly interface.',
      delay: 0.3,
    },
  ];

  return (
    <section className="py-20 bg-gray-50 dark:bg-gray-800">
      <div className="container mx-auto px-6">
        <div 
          ref={ref}
          className={`text-center mb-16 transition-opacity duration-1000 ease-out ${
            inView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
          }`}
        >
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Why Choose Imagenie?</h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Built with cutting-edge technology for speed, privacy, and ease of use.
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {reasons.map((reason, index) => (
            <ReasonCard
              key={index}
              icon={reason.icon}
              title={reason.title}
              description={reason.description}
              delay={reason.delay}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export default WhyImagenie; 