module.exports = {
  lang: "en-US",
  title: "dt-cli",
  description: "Documentations for dt-cli <https://github.com/blurgyy/dt>",

  themeConfig: {
    repo: "blurgyy/dt-cli-docs",
    docsDir: "docs",
    nav: [
      {text: "dt-cli", link: "/"},
      {text: "Config", link: "/config/"},
    ],
    sidebar: {
      "/config/": [
        {
          text: "Hands-on Guide",
          children: [
            {
              text: "Basics",
              link: "/config/",
            },
            {
              text: "Defining default behaviours",
              link: "/config/default-behaviours",
            },
            {
              text: "Groups",
              link: "/config/groups",
            },
            {
              text: "Syncing methods",
              link: "/config/syncing-methods"
            },
          ],
        },
        {
          text: "Key References",
          link: "/config/key-references"
        },
      ],
      "/": [{text: "Getting started", link: "/"}],
    }
  }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 06 2021, 13:04 [CST]
