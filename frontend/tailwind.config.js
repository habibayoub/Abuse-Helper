const { nextui } = require( "@nextui-org/theme" );

/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    "./node_modules/@nextui-org/theme/dist/**/*.{js,ts,jsx,tsx}",
    './src/**/*.{ts,tsx}',
  ],
  prefix: "",
  theme: {

  },
  plugins: [require( "tailwindcss-animate" ), nextui( { addCommonColors: true } )],
}

