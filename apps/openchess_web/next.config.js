// Import your environment configuration
await import("./src/env.js");

/** @type {import("next").NextConfig} */
const coreConfig = {
  typescript: {
    ignoreBuildErrors: true,
  },

  webpack: (config, { isServer }) => {
    // Enable .wasm file handling
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      layers: true,
    };

    // Add support for loading .wasm files in the browser environment
    if (!isServer) {
      config.module.rules.push({
        test: /\.wasm$/,
        type: "webassembly/async",
      });
    }

    return config;
  },
};

export default coreConfig;
