import React from 'react';
import { useInView } from 'react-intersection-observer';

const Download = () => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  return (
    <section className="py-20 bg-gradient-to-br from-indigo-600 to-purple-700 text-white">
      <div className="container mx-auto px-6">
        <div 
          ref={ref}
          className={`max-w-3xl mx-auto text-center transition-opacity duration-1000 ease-out ${
            inView ? 'opacity-100 transform-none' : 'opacity-0 translate-y-4'
          }`}
        >
          <h2 className="text-3xl md:text-4xl font-bold mb-6">Ready to Transform Your Images?</h2>
          <p className="text-xl mb-8 text-indigo-100">
            Download Imagenie now and experience the power of AI image processing on your desktop.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4 flex-wrap">
            <a 
              href="https://github.com/zhongweili/imagenie/releases/download/app-v0.2.2/imagenie_0.2.2_aarch64.dmg" 
              className="btn-primary inline-flex items-center justify-center px-8 py-4 rounded-lg bg-white text-indigo-700 hover:bg-indigo-100 font-medium transition-all"
            >
              <i className="fab fa-apple mr-2 text-xl"></i> Download for Mac (Silicon)
            </a>
            <a 
              href="https://github.com/zhongweili/imagenie/releases/download/app-v0.2.2/imagenie_0.2.2_x64.dmg" 
              className="btn-primary inline-flex items-center justify-center px-8 py-4 rounded-lg bg-white text-indigo-700 hover:bg-indigo-100 font-medium transition-all"
            >
              <i className="fab fa-apple mr-2 text-xl"></i> Download for Mac (Intel)
            </a>
            <a 
              href="https://github.com/zhongweili/imagenie/releases/download/app-v0.2.2/imagenie_0.2.2_x64_en-US.msi" 
              className="btn-primary inline-flex items-center justify-center px-8 py-4 rounded-lg bg-indigo-800 hover:bg-indigo-900 text-white font-medium transition-all"
            >
              <i className="fab fa-windows mr-2 text-xl"></i> Download for Windows
            </a>
          </div>
          <p className="mt-6 text-indigo-200 text-sm">
            Currently available for macOS (Intel/Silicon) and Windows. Linux coming soon.
          </p>
        </div>
      </div>
    </section>
  );
};

export default Download; 