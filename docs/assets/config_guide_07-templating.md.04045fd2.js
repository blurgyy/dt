import{_ as e,y as t,x as s,W as n}from"./plugin-vue_export-helper.f07d1dea.js";const k='{"title":"Templating","description":"","frontmatter":{},"headers":[{"level":2,"title":"Setting Values","slug":"setting-values"},{"level":2,"title":"Writing Templates","slug":"writing-templates"},{"level":2,"title":"Skipping Rendering","slug":"skipping-rendering"},{"level":2,"title":"Advanced Syntaxes","slug":"advanced-syntaxes"}],"relativePath":"config/guide/07-templating.md","lastUpdated":1692638873737}',a={},o=n(`<h1 id="templating" tabindex="-1">Templating <a class="header-anchor" href="#templating" aria-hidden="true">#</a></h1><p><code>dt-cli</code> allows its source files to be templates, the templates are rendered with values defined in <code>dt-cli</code>&#39;s config file. Here is a simple example that parameterizes several GUI-related properties, and render a template to its destination with <code>dt-cli</code>.</p><div class="info custom-block"><p class="custom-block-title">NOTE</p><p>Only templating a single file shows little benefit. This is just a toy example that demonstrates the basic usage of templating. In real-world uses, the <code>sources</code> array can include more template files, so that templating can actually ease config file management.</p></div><h2 id="setting-values" tabindex="-1">Setting Values <a class="header-anchor" href="#setting-values" aria-hidden="true">#</a></h2><p>In <code>dt-cli</code>&#39;s config file, add another section with name <code>[context]</code>. Here is where the values are set. We will define the following values:</p><div class="language-toml"><div class="highlight-lines"><br><br><br><br><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><div class="highlighted">\xA0</div><br><br><div class="highlighted">\xA0</div><br><br><br><br><br><br></div><pre><code><span class="token comment"># Contents of ~/.config/dt/cli.toml</span>
<span class="token punctuation">[</span><span class="token table class-name">global</span><span class="token punctuation">]</span>
<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>

<span class="token punctuation">[</span><span class="token table class-name">context</span><span class="token punctuation">]</span>
<span class="token key property">gui.font-size</span> <span class="token punctuation">=</span> <span class="token number">15</span>
<span class="token key property">gui.cursor-size</span> <span class="token punctuation">=</span> <span class="token number">24</span>

<span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;gui&quot;</span>
<span class="token key property">base</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/gui&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span>
  <span class="token string">&quot;gtk-3.0/settings.ini&quot;</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config&quot;</span>
</code></pre></div><p>In this config example, we have two values under the <code>context.gui</code> section.</p><h2 id="writing-templates" tabindex="-1">Writing Templates <a class="header-anchor" href="#writing-templates" aria-hidden="true">#</a></h2><p>Templates are understood by <code>dt-cli</code> with the <a href="https://handlebarsjs.com/guide/" target="_blank" rel="noopener noreferrer">Handlebars</a> syntax. We can template the gtk settings file in the <code>gui</code> group (se above) as:</p><div class="language-ini"><pre><code><span class="token comment"># Contents of ~/dt/gui/gtk-3.0/settings.ini</span>
<span class="token section"><span class="token punctuation">[</span><span class="token section-name selector">Settings</span><span class="token punctuation">]</span></span>
<span class="token key attr-name">gtk-cursor-theme-size</span><span class="token punctuation">=</span><span class="token value attr-value">{{{ gui.cursor-size }}}</span>
<span class="token key attr-name">gtk-font-name</span><span class="token punctuation">=</span><span class="token value attr-value">system-ui {{{ gui.font-size }}}</span>
</code></pre></div><p>After this, running <code>dt-cli</code> and <code>~/.config/gtk-3.0/settings.ini</code> will have our templated values:</p><div class="language-ini"><pre><code><span class="token comment"># Contents of ~/.config/gtk-3.0/settings.ini</span>
<span class="token section"><span class="token punctuation">[</span><span class="token section-name selector">Settings</span><span class="token punctuation">]</span></span>
<span class="token key attr-name">gtk-cursor-theme-size</span><span class="token punctuation">=</span><span class="token value attr-value">24</span>
<span class="token key attr-name">gtk-font-name</span><span class="token punctuation">=</span><span class="token value attr-value">system-ui 15</span>
</code></pre></div><h2 id="skipping-rendering" tabindex="-1">Skipping Rendering <a class="header-anchor" href="#skipping-rendering" aria-hidden="true">#</a></h2><p>By default, <code>dt-cli</code> treats all source files as templates to be rendered. Sometimes we want to skip rendering, for example when a source file is huge, or when a source file contains strings that conflicts with the Handlebars syntax, or whatever. To skip rendering for a group, use the <a href="/config/key-references.html#renderable-1"><code>renderable = false</code></a> option:</p><div class="language-toml"><div class="highlight-lines"><br><br><br><br><br><br><br><br><br><br><br><br><br><br><br><div class="highlighted">\xA0</div><br></div><pre><code><span class="token comment"># Contents of ~/.config/dt/cli.toml</span>
<span class="token punctuation">[</span><span class="token table class-name">global</span><span class="token punctuation">]</span>
<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>

<span class="token punctuation">[</span><span class="token table class-name">context</span><span class="token punctuation">]</span>
<span class="token key property">gui.font-size</span> <span class="token punctuation">=</span> <span class="token number">15</span>
<span class="token key property">gui.cursor-size</span> <span class="token punctuation">=</span> <span class="token number">24</span>

<span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;gui&quot;</span>
<span class="token key property">base</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/dt/gui&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span>
  <span class="token string">&quot;gtk-3.0/settings.ini&quot;</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config&quot;</span>
<span class="token key property">renderable</span> <span class="token punctuation">=</span> <span class="token boolean">false</span>
</code></pre></div><p>After another run of <code>dt-cli</code>, <code>~/.config/gtk-3.0/settings.ini</code> will have contents identical to the original (unrendered) template:</p><div class="language-ini"><pre><code><span class="token comment"># Contents of ~/.config/gtk-3.0/settings.ini</span>
<span class="token section"><span class="token punctuation">[</span><span class="token section-name selector">Settings</span><span class="token punctuation">]</span></span>
<span class="token key attr-name">gtk-cursor-theme-size</span><span class="token punctuation">=</span><span class="token value attr-value">{{{ gui.cursor-size }}}</span>
<span class="token key attr-name">gtk-font-name</span><span class="token punctuation">=</span><span class="token value attr-value">system-ui {{{ gui.font-size }}}</span>
</code></pre></div><p>Finally, template rendering can be disabled globally by adding the <a href="/config/key-references.html#renderable-1"><code>renderable = false</code></a> line to the <code>[global]</code> section:</p><div class="language-toml"><div class="highlight-lines"><br><br><div class="highlighted">\xA0</div><br></div><pre><code><span class="token punctuation">[</span><span class="token table class-name">global</span><span class="token punctuation">]</span>
<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
<span class="token key property">renderable</span> <span class="token punctuation">=</span> <span class="token boolean">false</span>
</code></pre></div><h2 id="advanced-syntaxes" tabindex="-1">Advanced Syntaxes <a class="header-anchor" href="#advanced-syntaxes" aria-hidden="true">#</a></h2><p>The <a href="https://docs.rs/handlebars/latest/handlebars/#built-in-helpers" target="_blank" rel="noopener noreferrer">built-in helpers</a> of the <a href="https://docs.rs/handlebars/latest/handlebars/" target="_blank" rel="noopener noreferrer">Handlebars crate</a> are understood in dt-cli&#39;s templates. Please refer to the <a href="https://docs.rs/handlebars/latest/handlebars/" target="_blank" rel="noopener noreferrer">Handlebars crate</a>&#39;s page for guides on those basic control flow syntaxes like looping and conditioning.</p><p>In addition, <code>dt-cli</code> provides some <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/index.html" target="_blank" rel="noopener noreferrer">more helpers</a> to further boost the power of templating. The following table lists their names and descriptions, <strong>click on their names in the table to see their respective usages in detail</strong>.</p><table><thead><tr><th style="text-align:center;">helper</th><th style="text-align:left;">description</th></tr></thead><tbody><tr><td style="text-align:center;"><a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.get_mine.html" target="_blank" rel="noopener noreferrer"><code>get_mine</code></a></td><td style="text-align:left;">Retrieves the value for current host from a map, returns a default value when current host is not recorded in the map</td></tr><tr><td style="text-align:center;"><a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_host.html" target="_blank" rel="noopener noreferrer"><code>if_host</code></a></td><td style="text-align:left;">Tests if current machine\u2019s hostname matches a set of given string(s)</td></tr><tr><td style="text-align:center;"><a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_os.html" target="_blank" rel="noopener noreferrer"><code>if_os</code></a></td><td style="text-align:left;">Conditions on values parsed from target machine\u2019s <code>/etc/os-release</code> file</td></tr><tr><td style="text-align:center;"><a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_uid.html" target="_blank" rel="noopener noreferrer"><code>if_uid</code></a></td><td style="text-align:left;">Tests if current user\u2019s effective uid matches a set of given integer(s)</td></tr><tr><td style="text-align:center;"><a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_user.html" target="_blank" rel="noopener noreferrer"><code>if_user</code></a></td><td style="text-align:left;">Tests if current user\u2019s username matches a set of given string(s)</td></tr><tr><td style="text-align:center;"><a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_host.html" target="_blank" rel="noopener noreferrer"><code>unless_host</code></a>, <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_os.html" target="_blank" rel="noopener noreferrer"><code>unless_os</code></a>, <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_uid.html" target="_blank" rel="noopener noreferrer"><code>unless_uid</code></a>, <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_user.html" target="_blank" rel="noopener noreferrer"><code>unless_user</code></a></td><td style="text-align:left;">Negated versions of <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_host.html" target="_blank" rel="noopener noreferrer"><code>if_host</code></a>, <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_os.html" target="_blank" rel="noopener noreferrer"><code>if_os</code></a>, <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_uid.html" target="_blank" rel="noopener noreferrer"><code>if_uid</code></a>, <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_user.html" target="_blank" rel="noopener noreferrer"><code>if_user</code></a></td></tr></tbody></table><div class="info custom-block"><p class="custom-block-title">NOTE</p><p>Above table might get out-dated, check out <a href="https://docs.rs/dt-core/latest/dt_core/registry/helpers/index.html" target="_blank" rel="noopener noreferrer">https://docs.rs/dt-core/latest/dt_core/registry/helpers/index.html</a> for a list of supported helpers <sub>(in addition to those already supported by the <a href="https://docs.rs/handlebars/latest/handlebars/" target="_blank" rel="noopener noreferrer">Handlebars crate</a>)</sub> and their usages.</p></div>`,24),r=[o];function l(p,c,i,d,u,h){return s(),t("div",null,r)}var f=e(a,[["render",l]]);export{k as __pageData,f as default};
