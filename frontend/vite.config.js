import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		strictPort: false,
		host: true,
		fs: {
			strict: false,
			allow: ['..']
		},
		proxy: {
			'/api': {
				target: 'http://127.0.0.1:3000',
				changeOrigin: true
			},
			'/ws': {
				target: 'ws://127.0.0.1:3000',
				ws: true
			}
		}
	},
	build: {
		target: 'esnext',
		outDir: 'build',
		assetsDir: '_app',
		rollupOptions: {
			input: {
				main: './index.html'
			},
			output: {
				manualChunks: (id) => {
					if (id.includes('node_modules')) {
						// Regrouper les dépendances lourdes
						if (id.includes('libsodium')) return 'libsodium';
						if (id.includes('chart.js')) return 'chart';
						if (id.includes('simple-peer')) return 'peer';
						return 'vendor';
					}
				}
			}
		}
	},
	optimizeDeps: {
		include: [
			'uuid',
			'libsodium-wrappers',
			'simple-peer',
			'chart.js'
		]
	},
	resolve: {
		alias: {
			'$lib': './src/lib',
			'$components': './src/components',
			'$routes': './src/routes',
			'$assets': './static'
		}
	}
});
