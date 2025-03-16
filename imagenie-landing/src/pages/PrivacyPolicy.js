import React from 'react';
import Header from '../components/Header';
import Footer from '../components/Footer';

const PrivacyPolicy = () => {
  return (
    <div className="bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-200">
      <Header />
      <div className="container mx-auto px-6 py-12">
        <h1 className="text-3xl font-bold mb-8">Privacy Policy</h1>
        
        <div className="space-y-6">
          <section>
            <h2 className="text-xl font-semibold mb-3">1. Introduction</h2>
            <p>Welcome to Imagenie. We respect your privacy and are committed to protecting your personal data. This Privacy Policy explains how we collect, use, disclose, and safeguard your information when you use our open-source desktop application for image generation.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">2. Open Source Nature</h2>
            <p>Imagenie is an open-source desktop application released under the MIT License. This means:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>You can freely use, modify, and distribute the application</li>
              <li>You can create your own versions based on our source code</li>
              <li>The application runs locally on your device</li>
              <li>We do not collect data from your local usage unless explicitly stated</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">3. Information We Collect</h2>
            <p>Since Imagenie is a desktop application that runs locally, we collect minimal information:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Anonymous usage statistics if you opt-in to sharing analytics</li>
              <li>Information you explicitly choose to share with us (such as bug reports or feedback)</li>
              <li>Basic device information for crash reports (if you enable this feature)</li>
            </ul>
            <p className="mt-2">We do not collect:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>The images you generate locally</li>
              <li>Your prompts or inputs used for image generation</li>
              <li>Personal files on your device</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">4. How We Use Your Information</h2>
            <p>If you choose to share information with us, we use it to:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Improve the application based on usage patterns and feedback</li>
              <li>Fix bugs and technical issues</li>
              <li>Develop new features</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">5. Third-Party Services</h2>
            <p>The application may include links to third-party websites or services. We are not responsible for the privacy practices of these third parties. We encourage you to read the privacy policies of any third-party services you interact with.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">6. Your Rights</h2>
            <p>As an open-source application user, you have complete control over your data:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>You can use the application without sharing any data with us</li>
              <li>You can opt out of any analytics or crash reporting features</li>
              <li>You can modify the source code to remove any data collection functionality</li>
              <li>You can request deletion of any data you've explicitly shared with us</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">7. Children's Privacy</h2>
            <p>Our application is not intended for children under 13 years of age. We do not knowingly collect personal information from children under 13.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">8. Changes to This Privacy Policy</h2>
            <p>We may update our Privacy Policy from time to time. We will notify you of any changes by posting the new Privacy Policy on our website and updating the "Last Updated" date.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">9. Contact Us</h2>
            <p>If you have any questions about this Privacy Policy, please contact us at:</p>
            <p className="mt-2">Email: support@imagenie.us</p>
          </section>
          
          <p className="mt-8 text-sm">Last Updated: {new Date().toLocaleDateString()}</p>
        </div>
      </div>
      <Footer />
    </div>
  );
};

export default PrivacyPolicy; 