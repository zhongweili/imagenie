import React from 'react';
import Header from '../components/Header';
import Footer from '../components/Footer';

const TermsOfService = () => {
  return (
    <div className="bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-200">
      <Header />
      <div className="container mx-auto px-6 py-12">
        <h1 className="text-3xl font-bold mb-8">Terms of Service</h1>
        
        <div className="space-y-6">
          <section>
            <h2 className="text-xl font-semibold mb-3">1. Acceptance of Terms</h2>
            <p>By accessing or using Imagenie, you agree to be bound by these Terms of Service. If you do not agree to these terms, please do not use our application.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">2. Open Source License</h2>
            <p>Imagenie is an open-source desktop application released under the MIT License. This means you are free to:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Use the software for any purpose</li>
              <li>Study how the software works and modify it</li>
              <li>Distribute the software</li>
              <li>Improve the software and release your improvements to the public</li>
            </ul>
            <p className="mt-2">The full text of the MIT License can be found in our GitHub repository.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">3. Description of Service</h2>
            <p>Imagenie is a desktop application that provides AI-powered image generation and editing capabilities. The application runs locally on your device. We reserve the right to modify, update, or discontinue any aspect of the application at any time.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">4. User Content</h2>
            <p>You retain full ownership of any content you create using our application. Since Imagenie runs locally on your device:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>We do not claim any rights to images you generate</li>
              <li>We do not collect or store your content on our servers</li>
              <li>You are solely responsible for backing up your content</li>
            </ul>
            <p className="mt-2">You agree not to use our application to create content that:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Infringes on intellectual property rights</li>
              <li>Contains illegal, harmful, or offensive material</li>
              <li>Violates the privacy or publicity rights of others</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">5. Generated Images</h2>
            <p>Images generated using our application:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Are owned by you</li>
              <li>May be used for personal or commercial purposes</li>
              <li>Should not be used to create illegal, harmful, or offensive content</li>
              <li>May not be represented as created by a human artist without proper disclosure</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">6. Prohibited Uses</h2>
            <p>You agree not to use our application to:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Generate content that violates any applicable law or regulation</li>
              <li>Create deepfakes or misleading content without proper disclosure</li>
              <li>Generate content that infringes on third-party rights</li>
              <li>Create content that promotes hatred, violence, or discrimination</li>
              <li>Attempt to reverse engineer or extract our AI models beyond what is permitted by the MIT License</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">7. Intellectual Property</h2>
            <p>While Imagenie is open-source under the MIT License, certain elements may be subject to additional licenses:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>Third-party libraries and components used in the application are subject to their respective licenses</li>
              <li>The Imagenie name and logo are trademarks and may not be used without permission in contexts outside of the normal use or distribution of the software</li>
              <li>AI models included or used by the application may have separate licensing terms</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">8. Limitation of Liability</h2>
            <p>THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">9. Modifications to the Application</h2>
            <p>As an open-source project, Imagenie may be modified and distributed by third parties. We are not responsible for modified versions of the application not officially released by us. If you modify the application:</p>
            <ul className="list-disc pl-6 mt-2 space-y-1">
              <li>You must comply with the terms of the MIT License</li>
              <li>You should clearly indicate that your version is modified</li>
              <li>You may not use our trademarks in a way that suggests endorsement of your modified version</li>
            </ul>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">10. Changes to Terms</h2>
            <p>We may modify these Terms of Service at any time. We will notify you of any changes by posting the new Terms on our website. Your continued use of the application after any such changes constitutes your acceptance of the new Terms.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">11. Governing Law</h2>
            <p>These Terms shall be governed by and construed in accordance with the laws of the jurisdiction in which the primary maintainers of the project reside, without regard to its conflict of law provisions.</p>
          </section>
          
          <section>
            <h2 className="text-xl font-semibold mb-3">12. Contact Us</h2>
            <p>If you have any questions about these Terms, please contact us at:</p>
            <p className="mt-2">Email: support@imagenie.us</p>
          </section>
          
          <p className="mt-8 text-sm">Last Updated: {new Date().toLocaleDateString()}</p>
        </div>
      </div>
      <Footer />
    </div>
  );
};

export default TermsOfService; 