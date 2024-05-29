import bun from 'bun';

try {
  console.info('Starting the build process...');
  const outputNode = await bun.build({
    entrypoints: ['src/index.ts'],
    outdir: 'dist',
    format: 'esm',
    minify: true,
    sourcemap: 'external',
    target: 'node',
    // target: 'esnext', // Adjust target based on your project needs
  });
  console.info('Node Build completed successfully!', outputNode);
  const outputBrowser = await bun.build({
    entrypoints: ['src/browser.ts'],
    outdir: 'dist',
    format: 'esm',
    minify: true,
    sourcemap: 'external',
    target: 'browser',
    // target: 'esnext', // Adjust target based on your project needs
  });
  console.info('Browser Build completed successfully!', outputBrowser);
} catch (error) {
  console.error('Build failed:', error);
}
