import type { Config } from 'tailwindcss';
import colors from 'tailwindcss/colors';

// @ts-ignore
delete colors['lightBlue'];
// @ts-ignore
delete colors['warmGray'];
// @ts-ignore
delete colors['trueGray'];
// @ts-ignore
delete colors['coolGray'];
// @ts-ignore
delete colors['blueGray'];

const busterColors = {
  primary: '#7C3AED',
  background: {
    base: '#FCFCFC',
    DEFAULT: '#f8f8f8'
  }
};
const busterColorsDark = {
  //
};

const config = {
  darkMode: 'class',
  content: ['./src/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    transparent: 'transparent',
    current: 'currentColor',
    fontSize: {
      xxs: '10px',
      xs: '11px',
      sm: '12px',
      base: '13px',
      md: '14px',
      lg: '18px',
      xl: '18px',
      '2xl': '20px',
      '3xl': '24px',
      '4xl': '30px'
    },
    extend: {
      colors: {
        // light mode
        buster: busterColors,
        // dark mode
        'dark-buster': busterColorsDark
      },
      boxShadow: {
        // light
        'buster-input': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
        'buster-card': '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
        'buster-dropdown': '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
        // dark
        'dark-buster-input': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
        'dark-buster-card': '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
        'dark-buster-dropdown': '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
        //slider
        'buster-slider': 'white 3px 0px 5px 2px'
      },
      fontSize: {
        'buster-label': ['0.75rem', { lineHeight: '1rem' }],
        'buster-default': ['0.875rem', { lineHeight: '1.25rem' }],
        'buster-title': ['1.125rem', { lineHeight: '1.75rem' }],
        'buster-metric': ['1.875rem', { lineHeight: '2.25rem' }]
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))'
      }
    },
    screens: {
      xs: '480px',
      sm: '577px',
      md: '768px',
      lg: '992px',
      xl: '1400px',
      xxl: '1600px',
      '2xl': '1600px'
    }
  },
  plugins: [],
  safelist: [
    {
      pattern:
        /^(bg-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950))$/,
      variants: ['hover', 'ui-selected']
    },
    {
      pattern:
        /^(text-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950))$/,
      variants: ['hover', 'ui-selected']
    },
    {
      pattern:
        /^(border-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950))$/,
      variants: ['hover', 'ui-selected']
    },
    {
      pattern:
        /^(ring-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950))$/
    },
    {
      pattern:
        /^(stroke-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950))$/
    },
    {
      pattern:
        /^(fill-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950))$/
    },
    // custom colors charts
    ...['[#32a852]', '[#fcba03]'].flatMap((customColor) => [
      `bg-${customColor}`,
      `border-${customColor}`,
      `hover:bg-${customColor}`,
      `hover:border-${customColor}`,
      `hover:text-${customColor}`,
      `fill-${customColor}`,
      `ring-${customColor}`,
      `stroke-${customColor}`,
      `text-${customColor}`,
      `ui-selected:bg-${customColor}]`,
      `ui-selected:border-${customColor}]`,
      `ui-selected:text-${customColor}`
    ])
  ]
};

export default config;
export { config, colors as tailwindColors, busterColors, busterColorsDark };
