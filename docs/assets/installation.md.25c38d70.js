import{_ as a,c as e,o as t,a as l}from"./app.8da8cbf3.js";const g='{"title":"Install","description":"","frontmatter":{"title":"Install"},"headers":[{"level":2,"title":"AUR","slug":"aur"},{"level":2,"title":"Alternative Ways","slug":"alternative-ways"}],"relativePath":"installation.md","lastUpdated":1634454237283}',n={},s=l(`<h1 id="install" tabindex="-1">Install <a class="header-anchor" href="#install" aria-hidden="true">#</a></h1><h2 id="aur" tabindex="-1">AUR <a class="header-anchor" href="#aur" aria-hidden="true">#</a></h2><p><code>dt-cli</code> is in the <a href="https://aur.archlinux.org/packages/dt-cli/" target="_blank" rel="noopener noreferrer">AUR</a>, you can install it with your favorite package manager:</p><div class="language-shell"><pre><code>$ paru -S dt-cli
</code></pre></div><h2 id="alternative-ways" tabindex="-1">Alternative Ways <a class="header-anchor" href="#alternative-ways" aria-hidden="true">#</a></h2><p>Alternatively, you can:</p><ul><li><p>Download latest <a href="https://github.com/blurgyy/dt/releases/latest" target="_blank" rel="noopener noreferrer">release</a> from GitHub</p></li><li><p>Install from <a href="https://crates.io/crates/dt-cli/" target="_blank" rel="noopener noreferrer">crates.io</a>:</p><div class="language-shell"><pre><code>$ cargo <span class="token function">install</span> dt-cli
</code></pre></div></li><li><p>Build from source:</p><div class="language-shell"><pre><code>$ <span class="token function">git</span> clone github.com:blurgyy/dt.git
$ <span class="token builtin class-name">cd</span> dt
$ cargo <span class="token builtin class-name">test</span> --release
$ cargo <span class="token function">install</span> --path<span class="token operator">=</span>dt-cli
</code></pre></div></li></ul>`,7),r=[s];function i(o,c,d,p,h,u){return t(),e("div",null,r)}var f=a(n,[["render",i]]);export{g as __pageData,f as default};