import{_ as e,y as a,x as n,W as t}from"./plugin-vue_export-helper.f07d1dea.js";const f='{"title":"Templating","description":"","frontmatter":{},"headers":[{"level":2,"title":"Background","slug":"background"},{"level":2,"title":"Syntax","slug":"syntax"},{"level":3,"title":"Configuring","slug":"configuring"},{"level":3,"title":"Applying","slug":"applying"}],"relativePath":"features/04-templating.md","lastUpdated":1653960932350}',s={},o=t(`<h1 id="templating-examples" tabindex="-1">Templating<sub>[<a href="/config/guide/07-templating.html"><strong>Examples</strong></a>]</sub> <a class="header-anchor" href="#templating-examples" aria-hidden="true">#</a></h1><h2 id="background" tabindex="-1">Background <a class="header-anchor" href="#background" aria-hidden="true">#</a></h2><p>As is always the case, there are quite a few applications that share a same set of properties. For example, we want to have uniform looks for Qt and GTK applications. Templating utility is developed under the <strong>DRY</strong> (<strong>D</strong>on&#39;t <strong>R</strong>epeat <strong>Y</strong>ourself) principle, it allows to manage these shared properties in one place: change once, apply everywhere.</p><h2 id="syntax" tabindex="-1">Syntax <a class="header-anchor" href="#syntax" aria-hidden="true">#</a></h2><h3 id="configuring" tabindex="-1">Configuring <a class="header-anchor" href="#configuring" aria-hidden="true">#</a></h3><p>To manage shared properties, add a section <code>[context]</code> to <code>dt-cli</code>&#39;s config file. For example, to set a property named <code>cursor-size</code> for the <code>gui</code> group to value <code>24</code>:</p><div class="language-toml"><pre><code><span class="token comment"># ~/.config/dt/cli.toml</span>
<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
<span class="token punctuation">[</span><span class="token table class-name">context</span><span class="token punctuation">]</span>
<span class="token key property">gui.cursor-size</span> <span class="token punctuation">=</span> <span class="token number">24</span>
<span class="token comment">## Or, as TOML allows it:</span>
<span class="token comment">#[context.gui]</span>
<span class="token comment">#cursor-size = 24</span>
<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
</code></pre></div><p>See the <a href="/config/guide/07-templating.html">configuration guide</a> for detailed usages.</p><h3 id="applying" tabindex="-1">Applying <a class="header-anchor" href="#applying" aria-hidden="true">#</a></h3><p><code>dt-cli</code> uses Rust&#39;s <a href="https://docs.rs/handlebars/latest/handlebars/" target="_blank" rel="noopener noreferrer">Handlebars crate</a> to render templates. Handlebars is tested and widely used, according to its descriptions:</p><div class="info custom-block"><p class="custom-block-title">INFO</p><p>Handlebars-rust is the template engine that renders the official Rust website <a href="http://rust-lang.org" target="_blank" rel="noopener noreferrer">rust-lang.org</a>.</p></div><p>For example, to apply a property named <code>cursor-size</code> to all source files under the <code>gui</code> group:</p><div class="language-ini"><pre><code>...
<span class="token key attr-name">gtk-cursor-theme-size</span><span class="token punctuation">=</span><span class="token value attr-value">{{{ gui.cursor-size }}}</span>
...
</code></pre></div><p>With <code>context.gui.cursor-size</code> being set to <code>24</code> (as in <a href="#configuring">previous section</a>), the above template (in a group with name <code>gui</code>) will be rendered as:</p><div class="language-ini"><pre><code><span class="token comment"># ~/.config/gtk-3.0/settings.ini</span>
...
<span class="token key attr-name">gtk-cursor-theme-size</span><span class="token punctuation">=</span><span class="token value attr-value">24</span>
...
</code></pre></div><div class="warning custom-block"><p class="custom-block-title">INFO</p><p>The time consumed while rendering can be quite noticeable if the template being rendered is huge. To skip rendering for a group, use the <a href="/config/key-references.html#renderable-1"><code>renderable = false</code></a> option in the config file.</p></div><p>The <a href="https://docs.rs/handlebars/latest/handlebars/" target="_blank" rel="noopener noreferrer">Handlebars crate</a> also allows syntaxes like looping and conditioning, the <a href="https://docs.rs/handlebars/latest/handlebars/#built-in-helpers" target="_blank" rel="noopener noreferrer">built-in helpers</a> are understood in <code>dt-cli</code>&#39;s templates. Please refer to the <a href="https://docs.rs/handlebars/latest/handlebars/" target="_blank" rel="noopener noreferrer">Handlebars crate</a>&#39;s page for syntax guides.</p>`,17),r=[o];function i(l,c,p,d,u,g){return n(),a("div",null,r)}var m=e(s,[["render",i]]);export{f as __pageData,m as default};
