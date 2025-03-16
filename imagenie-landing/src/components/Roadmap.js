import React from 'react';
import { useInView } from 'react-intersection-observer';

const RoadmapItem = ({ title, items, delay }) => {
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
      <h3 className="text-xl font-semibold mb-4">{title}</h3>
      <ul className="space-y-3">
        {items.map((item, index) => (
          <li key={index} className="flex items-start">
            <span className={`flex-shrink-0 w-6 h-6 ${item.completed ? 'bg-green-100 dark:bg-green-900' : 'bg-gray-100 dark:bg-gray-700'} rounded-full flex items-center justify-center mr-3 mt-0.5`}>
              <i className={`fas ${item.completed ? 'fa-check text-green-600 dark:text-green-400' : 'fa-clock text-gray-500 dark:text-gray-400'} text-sm`}></i>
            </span>
            <span className="text-gray-700 dark:text-gray-300">{item.text}</span>
          </li>
        ))}
      </ul>
    </div>
  );
};

const Roadmap = () => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const completedItems = [
    { text: 'Image Upscaling', completed: true },
    { text: 'Face Restoration', completed: true },
    { text: 'Background Removal', completed: true },
    { text: 'Support Windows, macOS', completed: true },
  ];

  const upcomingItems = [
    { text: 'Batch Processing', completed: false },
    { text: 'Support for Additional AI Models', completed: false },
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
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Roadmap</h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            We're constantly improving Imagenie with new features and capabilities.
          </p>
        </div>
        
        <div className="max-w-3xl mx-auto">
          <div className="space-y-6">
            <RoadmapItem title="Completed" items={completedItems} delay={0.1} />
            <RoadmapItem title="Coming Soon" items={upcomingItems} delay={0.3} />
          </div>
        </div>
      </div>
    </section>
  );
};

export default Roadmap; 