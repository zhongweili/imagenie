# Imagenie Landing Page

A modern, responsive landing page for the Imagenie desktop application built with React and TailwindCSS.

## Features

- Modern, clean design inspired by Linear's aesthetic
- Fully responsive layout for mobile, tablet, and desktop
- Dark/light mode toggle that respects system preferences
- Smooth animations and transitions
- Built with React and TailwindCSS

## Getting Started

### Prerequisites

- Node.js (v14 or later)
- npm or yarn

### Installation

1. Clone the repository
```bash
git clone https://github.com/yourusername/imagenie-landing.git
cd imagenie-landing
```

2. Install dependencies
```bash
npm install
# or
yarn install
```

3. Start the development server
```bash
npm start
# or
yarn start
```

4. Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

## Building for Production

```bash
npm run build
# or
yarn build
```

This will create an optimized production build in the `build` folder.

## Project Structure

```
imagenie-landing/
├── public/
│   ├── index.html
│   └── manifest.json
├── src/
│   ├── components/
│   │   ├── Download.js
│   │   ├── Features.js
│   │   ├── Footer.js
│   │   ├── Header.js
│   │   ├── Roadmap.js
│   │   ├── Showcase.js
│   │   ├── ThemeToggle.js
│   │   └── WhyImagenie.js
│   ├── App.js
│   ├── index.js
│   ├── index.css
│   └── reportWebVitals.js
├── package.json
├── tailwind.config.js
└── postcss.config.js
```

## Technologies Used

- [React](https://reactjs.org/)
- [TailwindCSS](https://tailwindcss.com/)
- [React Intersection Observer](https://github.com/thebuilder/react-intersection-observer)
- [Font Awesome](https://fontawesome.com/)

## License

This project is licensed under the MIT License.

## Acknowledgments

- [Imagenie](https://github.com/zhongweili/imagenie) - The desktop application this landing page is for
- [Linear](https://linear.app/) - Design inspiration 