import axios from 'axios';
import { GoogleFont } from './interfaces';

export const listAllGoogleFontsFromGoogle = async () => {
  const baseURL = `https://webfonts.googleapis.com/v1/webfonts`;
  const GOOGLE_FONTS_API = `${baseURL}?capability=WOFF2&sort=popularity&key=${process.env.NEXT_PUBLIC_WEB_FONT_API_KEY}`;
  const response = await axios.get(GOOGLE_FONTS_API);
  return response.data.items as GoogleFont[];
};
