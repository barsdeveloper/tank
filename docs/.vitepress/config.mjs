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
      { text: 'Docs', link: '/1-introduction' },
      { text: 'API', link: 'https://docs.rs/tank/' },
    ],

    sidebar: [
      {
        text: 'Docs',
        items: [
          { text: 'Introduction', link: '/1-introduction' },
          { text: 'Getting started', link: '/2-getting-started' },
          { text: 'Connection', link: '/3-connection' },
          { text: 'Entity definition', link: '/4-entity-definition' },
          { text: 'Entity operations', link: '/5-entity-operations' },
          { text: 'Types', link: '/6-types' },
        ],
      },

    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/barsdeveloper/tank' },
    ]
  },
  markdown: {
    config(md) {
    },
  },
})
