// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://jdbohrman-tech.github.io/roselite',
	integrations: [
		starlight({
			title: 'Roselite',
			description: 'P2P Static Site Hosting via Veilid DHT. Deploy static content with zero censorship and zero single points of failure.',
			logo: {
				src: './src/assets/logo.svg',
			},
			customCss: [
				'@fontsource/geist-sans/400.css',
				'@fontsource/geist-sans/600.css',
				'@fontsource/geist-mono/400.css',
				'./src/styles/custom.css',
			],
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/jdbohrman-tech/roselite',
				},
			],
			editLink: {
				baseUrl: 'https://github.com/jdbohrman-tech/roselite/edit/main/docs/',
			},
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quick Start', slug: 'getting-started/quick-start' },
					],
				},
				{
					label: 'Architecture',
					items: [
						{ label: 'Overview', slug: 'architecture' },
						{ label: 'DHT Gateway', slug: 'api/gateway-api' },
						{ label: '.veilidpkg Format', slug: 'reference/veilidpkg-format' },
					],
				},
				{
					label: 'CLI Reference',
					items: [
						{ label: 'Commands', slug: 'cli/commands' },
					],
				},
				{
					label: 'Help',
					items: [
						{ label: 'FAQ', slug: 'help/faq' },
					],
				},
				{
					label: 'Community',
					items: [
						{ label: 'Contributing', slug: 'community/contributing' },
					],
				},
			],
		}),
	],
});
