import{_ as s,c as n,o as a,a as t}from"./app.0e8b29b6.js";const h='{"title":"Groups","description":"","frontmatter":{},"headers":[{"level":2,"title":"Overriding Default Behaviours","slug":"overriding-default-behaviours"}],"relativePath":"config/guide/groups.md","lastUpdated":1633681331582}',e={},o=t(`<h1 id="groups" tabindex="-1">Groups <a class="header-anchor" href="#groups" aria-hidden="true">#</a></h1><p>Syncing only one group is boring. We can also add more groups.</p><p>Assuming your configuration files for <code>VSCode</code> lies at <code>~/dt/VSCode/User/settings.json</code>, the group for syncing this file can be:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;VSCode&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/VSCode&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span><span class="token string">&quot;User/settings.json&quot;</span><span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/Code - OSS/&quot;</span>
</code></pre></div><p>Appending this <code>VSCode</code> group to our previous config file, we obtain:</p><div class="language-toml"><div class="highlight-lines"><br><br><br><br><br><br><br><br><br><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><br></div><pre><code><span class="token punctuation">[</span><span class="token table class-name">global</span><span class="token punctuation">]</span>
<span class="token key property">allow_overwrite</span> <span class="token punctuation">=</span> <span class="token boolean">true</span>


<span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;Neovim&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/nvim&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span><span class="token string">&quot;*init.vim&quot;</span><span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/nvim&quot;</span>
<span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;VSCode&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/VSCode&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span><span class="token string">&quot;User/settings.json&quot;</span><span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/Code - OSS/&quot;</span>
</code></pre></div><p>After syncing with this config, the <strong>target item</strong> <code>~/.config/Code - OSS/User/settins.json</code> will be a symlink of the <strong>staged item</strong><code>~/.cache/dt/staging/VSCode/User/settings.json</code>, where the <strong>staged item</strong> mirrors the content of the <strong>source item</strong> <code>~/dt/VSCode/User/settings.json</code>.</p><h2 id="overriding-default-behaviours" tabindex="-1">Overriding Default Behaviours <a class="header-anchor" href="#overriding-default-behaviours" aria-hidden="true">#</a></h2><p>But what if we exceptionally want the <code>VSCode</code> group to <strong>not</strong> overwrite the target file if it already exists? No worries, here is the recipe of overriding a default behaviour for the <code>VSCode</code> group:</p><div class="language-toml"><div class="highlight-lines"><br><br><br><br><br><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><br></div><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;VSCode&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/VSCode&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span><span class="token string">&quot;User/settings.json&quot;</span><span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/Code - OSS/&quot;</span>

<span class="token key property">allow_overwrite</span> <span class="token punctuation">=</span> <span class="token boolean">false</span>
</code></pre></div><p>A group&#39;s behaviour will precedent the default behaviour if explicitly specified. Listing all overridable configs and their default values here:</p><ul><li><code>allow_overwrite</code>: <code>false</code></li><li><code>hostname_sep</code>: <code>&quot;@@&quot;</code></li><li><code>method</code>: <code>&quot;Symlink&quot;</code></li></ul><p>References to those keys can be found at <a href="/config/key-references.html">Key References</a>.</p><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>So far we have not explained why does <code>dt-cli</code> sync files in such a (somewhat complex) manner. You might ask:</p><blockquote><p>So what is <em>staging</em>, in particular?</p></blockquote><p>Read on for a comprehensive explanation for this decision!</p></div>`,14),p=[o];function c(i,r,l,u,d,k){return a(),n("div",null,p)}var v=s(e,[["render",c]]);export{h as __pageData,v as default};
