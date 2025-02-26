// filename: apps/openchess_web/next.config.js
// Import your environment configuration
// await import("./src/env.js");

/** @type {import("next").NextConfig} */
const nextConfig = {
  typescript: {
    ignoreBuildErrors: true,
  },
  webpack: (config, { isServer }) => {
    // Enable .wasm file handling on the server side
    config.resolve = {
      ...config.resolve, // Preserve existing resolve configurations
      symlinks: false,
    };
    // config.infrastructureLogging = { debug: /PackFileCache/ };
    config.cache = false;
    if (isServer) {
      config.experiments = {
        ...config.experiments,
        asyncWebAssembly: true,
      };
    }
    // Remove client-side .wasm handling if not needed
    return config;
  },
  plugins: [
  ],
};

export default nextConfig;
