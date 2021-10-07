module.exports = {
  lang: "en-US",
  title: "dt-cli",
  description: "Documentations for dt-cli <https://github.com/blurgyy/dt>",

  themeConfig: {
    repo: "blurgyy/dt",
    docsDir: "docs",
    nav: [
      {text: "dt-cli", link: "/"},
      {text: "Guide", link: "/config/guide/"},
      {text: "Key References", link: "/config/key-references"},
    ],
    sidebar: {
      "/config/": [
        {
          text: "Hands-on Guide",
          children: [
            {
              text: "Basics",
              link: "/config/guide/",
            },
            {
              text: "Defining Default Behaviours",
              link: "/config/guide/default-behaviours",
            },
            {
              text: "Groups",
              link: "/config/guide/groups",
            },
            {
              text: "Syncing Methods",
              link: "/config/guide/syncing-methods",
            },
            {
              text: "Host-specific Config",
              link: "/config/guide/host-specific",
            },
          ],
        },
        {
          text: "Key References",
          link: "/config/key-references",
        },
      ],
      "/": [
        {text: "Getting Started", link: "/"},
        {
          text: "Host-specific Syncing",
          link: "/host-specific",
        },
      ],
    }
  }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 06 2021, 13:04 [CST]
