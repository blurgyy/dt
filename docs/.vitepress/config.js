module.exports = {
  lang: "en-US",
  title: "dt-cli",
  description: "Documentations for dt-cli <https://github.com/blurgyy/dt>",

  themeConfig: {
    repo: "blurgyy/dt",
    docsDir: "docs",
    nav: [
      {
        text: "dt-cli",
        link: "/",
        activeMatch: "^/$|^/host-specific$|^/contributing$|^/installation$",
      },
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
              link: "/config/guide/01-default-behaviours",
            },
            {
              text: "Groups",
              link: "/config/guide/02-groups",
            },
            {
              text: "Syncing Methods",
              link: "/config/guide/03-syncing-methods",
            },
            {
              text: "Host-specific Config",
              link: "/config/guide/04-host-specific",
            },
            {
              text: "Error Handling",
              link: "/config/guide/05-error-handling",
            },
          ],
        },
        {
          text: "Key References",
          link: "/config/key-references",
        },
      ],
      "/": [
        {
          text: "👀 Overview",
          link: "/",
        },
        {
          text: "🚀 Installation",
          link: "/installation"
        },
        {
          text: "💠 Features",
          link: "/features/",
          children: [
            {
              text: "Host-specific Syncing",
              link: "/features/host-specific",
            },
            {
              text: "Priority Resolving",
              link: "/features/scope",
            },
          ]
        },
        {
          text: "📨 Contributing",
          link: "/contributing",
        },
      ],
    }
  }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 06 2021, 13:04 [CST]
