import { type CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.webmobile.app',
  appName: 'WebMobile',
  webDir: 'dist',
  bundledWebRuntime: false,
  ios: {
    loggingBehavior: 'none',
  },
  plugins: {
    Keyboard: {
      resize: 'none',
    },
  },
};

export default config;
