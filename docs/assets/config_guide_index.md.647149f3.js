import{_ as t,c as a,o as s,a as e}from"./app.b8a0027b.js";const h='{"title":"Basics","description":"","frontmatter":{"title":"Basics"},"relativePath":"config/guide/index.md","lastUpdated":1636082912857}',n={},o=e(`<h1 id="basics" tabindex="-1">Basics <a class="header-anchor" href="#basics" aria-hidden="true">#</a></h1><p>Configurations are composed with <strong>groups</strong>. A <code>local</code> group is added to the configuration file by adding a <code>[[local]]</code> section.</p><p>Assuming your configuration files for <code>Neovim</code> reside in <code>~/dt/nvim</code>, and all match the globbing pattern <code>*init.vim</code>, a minimal working example can then be configured as:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;Neovim&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/nvim&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span><span class="token string">&quot;*init.vim&quot;</span><span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/nvim&quot;</span>
</code></pre></div><p>This content causes <code>dt-cli</code> to perform the following steps:</p><ol><li>Create a &quot;staging&quot; directory at <code>~/.cache/dt/staging</code> (which is the default staging location);</li><li>Create the group&#39;s staging directory at <code>~/.cache/dt/staging/Neovim</code>;</li><li>Find all items (recursively if an item is a directory) that matches glob <code>~/dt/nvim/*init.vim</code> and store them back in the <code>sources</code> array;</li><li>For each item in the <code>sources</code> array, first copy it to the group&#39;s staging directory (<code>~/.cache/dt/staging/Neovim</code>), then symlink it to the target directory (<code>~/.config/nvim</code>), abort if a target file already exists.</li></ol><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>Details of above steps are explained in the <a href="./syncing-methods.html">Syncing Methods</a> section.</p></div><div class="warning custom-block"><p class="custom-block-title">WARNING</p><p>Aborting on existing target files is probably not what you want. Read on for a better solution!</p></div>`,8),i=[o];function c(p,r,l,d,u,g){return s(),a("div",null,i)}var k=t(n,[["render",c]]);export{h as __pageData,k as default};