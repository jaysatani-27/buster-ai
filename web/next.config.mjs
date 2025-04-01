import MillionLint from '@million/lint';
/** @type {import('next').NextConfig} */
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import dotenv from 'dotenv';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load env vars from parent directory in development
const envConfig = process.env.NODE_ENV === 'development' 
  ? dotenv.config({ path: '../.env' }).parsed 
  : {};

const nextConfig = {
  reactStrictMode: false,
  sassOptions: {
    includePaths: [path.join(__dirname, 'styles')]
  }
};

export default MillionLint.next({
  enabled: false,
  rsc: true
})(nextConfig);

//export default nextConfig;
