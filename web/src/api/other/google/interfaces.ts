export interface GoogleFont {
  family: string;
  variants: [
    '100',
    '100italic',
    '300',
    '300italic',
    'regular',
    'italic',
    '500',
    '500italic',
    '700',
    '700italic',
    '900',
    '900italic'
  ];
  subsets: ['cyrillic', 'cyrillic-ext', 'greek', 'greek-ext', 'latin', 'latin-ext', 'vietnamese'];
  version: 'v30';
  lastModified: string;
  files: Record<string, string>;
  category: string;
  kind: 'webfonts#webfont';
  menu: string; //'http://fonts.gstatic.com/s/roboto/v30/KFOmCnqEu92Fr1Mu5GxK.woff2';
}
