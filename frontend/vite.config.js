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
	// Résoudre le problème avec libsodium-wrappers (module natif)
	optimizeDeps: {
		include: ['libsodium-wrappers']
	},
	ssr: {
		noExternal: ['libsodium-wrappers', 'libsodium']
	},
	build: {
		// Ignorer les avertissements a11y pour le build
		rollupOptions: {
			onwarn(warning, warn) {
				const ignoredCodes = [
					'a11y_click_events_have_key_events',
					'a11y_no_noninteractive_element_to_interactive_role'
				];
				
				if (ignoredCodes.some(code => warning.message.includes(code))) {
					return;
				}
				
				warn(warning);
			}
		}
	}
});
