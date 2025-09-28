import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Tank",
  description: "Table Abstraction and Navigation Kit",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    search: {
      provider: 'local',
      options: {
        detailedView: true,
      },
    },

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Docs', link: '/docs' },
      { text: 'API', link: 'https://docs.rs/tank/' },
    ],

    sidebar: [
      {
        text: 'Docs',
        items: [
          { text: 'Introduction', link: '/docs' },
          { text: 'Getting started', link: '/getting-started' },
          { text: 'Entity definition', link: '/entity-definition' },
          { text: 'Data retrieval', link: '/data-retrieval' },
        ],
      },

    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/barsdeveloper/tank' },
    ]
  },
})
