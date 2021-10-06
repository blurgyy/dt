module.exports = {
  lang: "en-US",
  title: "dt-cli",
  description: "Documentations for dt-cli <https://github.com/blurgyy/dt>",

  themeConfig: {
    repo: "blurgyy/dt",
    docsDir: "docs",
    nav: [
      {text: "dt-cli", link: "/"},
      {text: "Config", link: "/config/guide/"},
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
              text: "Defining default behaviours",
              link: "/config/guide/default-behaviours",
            },
            {
              text: "Groups",
              link: "/config/guide/groups",
            },
            {
              text: "Syncing methods",
              link: "/config/guide/syncing-methods"
            },
          ],
        },
        {
          text: "Key References",
          link: "/config/key-references",
        },
      ],
      "/": [
        {text: "Getting started", link: "/"},
        {
          text: "Host-specific Syncing",
          children: [
            {
              text: "Introduction",
              link: "/host-specific/",
            },
            {
              text: "Hostname Suffix",
              link: "/host-specific/suffix"
            },
          ],
        },
      ],
    }
  }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 06 2021, 13:04 [CST]
