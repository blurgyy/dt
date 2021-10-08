import{_ as n,c as a,o as s,a as t}from"./app.0e8b29b6.js";const f='{"title":"Defining Default Behaviours","description":"","frontmatter":{},"relativePath":"config/guide/default-behaviours.md","lastUpdated":1633681331582}',e={},o=t(`__VP_STATIC_START__<h1 id="defining-default-behaviours" tabindex="-1">Defining Default Behaviours <a class="header-anchor" href="#defining-default-behaviours" aria-hidden="true">#</a></h1><p>Note that when syncing our configuration files for <code>Neovim</code> in the <a href="/config/guide/">basic config</a>, <code>dt-cli</code> <em>aborts</em> on existing target files. When populating items to another machine, it&#39;s better to directly overwrite (assuming you know what you are doing) the target file, so the <a href="/config/guide/">basic config</a> is suboptimal. What we could do is to additionally <strong>define the default overwriting behaviours</strong> with a <code>[global]</code> section in the configuration:</p><div class="language-toml"><div class="highlight-lines"><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><br><br><br><br><br><br></div><pre><code><span class="token punctuation">[</span><span class="token table class-name">global</span><span class="token punctuation">]</span>
<span class="token key property">allow_overwrite</span> <span class="token punctuation">=</span> <span class="token boolean">true</span>


<span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;Neovim&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/nvim&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span><span class="token string">&quot;*init.vim&quot;</span><span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/nvim&quot;</span>
</code></pre></div><p>This time, with the added <code>allow_overwrite = true</code>, existence of target file no longer aborts the syncing process.</p>__VP_STATIC_END__`,4),i=[o];function c(p,r,l,u,d,h){return s(),a("div",null,i)}var k=n(e,[["render",c]]);export{f as __pageData,k as default};
