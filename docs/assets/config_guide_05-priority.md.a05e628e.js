import{_ as n,c as s,o as a,a as t}from"./app.1279ae71.js";const h='{"title":"Scopes","description":"","frontmatter":{},"headers":[{"level":2,"title":"Examples","slug":"examples"},{"level":3,"title":"Dropin","slug":"dropin"},{"level":3,"title":"App","slug":"app"},{"level":3,"title":"General","slug":"general"}],"relativePath":"config/guide/05-priority.md","lastUpdated":1639152630926}',e={},o=t(`<h1 id="scopes" tabindex="-1">Scopes <a class="header-anchor" href="#scopes" aria-hidden="true">#</a></h1><p>A group&#39;s <a href="/config/key-references.html#scope"><code>scope</code></a> decides the priority of its items. When multiple groups contain a same item, only the group with the highest priority will do sync that specific item. This machanism minimizes filesystem I/O operations, which makes <code>dt-cli</code> to appear faster, and achieves finer control over what to sync with <code>dt-cli</code> without having to picking out each application&#39;s config files from your dotfile library.</p><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>This feature is meant to be used with <code>dt-cli</code>&#39;s <a href="/#usage">command-line argument</a>, see the <a href="/features/scope.html">Background</a> subsection of this feature&#39;s introduction for more details.</p></div><h2 id="examples" tabindex="-1">Examples <a class="header-anchor" href="#examples" aria-hidden="true">#</a></h2><h3 id="dropin" tabindex="-1"><code>Dropin</code> <a class="header-anchor" href="#dropin" aria-hidden="true">#</a></h3><p>On <a href="https://archlinux.org" target="_blank" rel="noopener noreferrer">Arch Linux</a>, package <a href="https://archlinux.org/packages/extra/x86_64/fontconfig/" target="_blank" rel="noopener noreferrer"><code>fontconfig</code></a> provides a file <code>/usr/share/fontconfig/conf.avail/10-sub-pixel-rgb.conf</code>, which <a href="http://www.lagom.nl/lcd-test/subpixel.php" target="_blank" rel="noopener noreferrer">works for most monitors</a>. A drop-in group can be defined as:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">scope</span> <span class="token punctuation">=</span> <span class="token string">&quot;Dropin&quot;</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;fontconfig-system&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;/usr/share/fontconfig/conf.avail/&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span>
  <span class="token comment"># Pixel Alignment.  Test monitor&#39;s subpixel layout at</span>
  <span class="token comment"># &lt;http://www.lagom.nl/lcd-test/subpixel.php&gt;, reference:</span>
  <span class="token comment"># &lt;https://wiki.archlinux.org/title/Font_configuration#Pixel_alignment&gt;</span>
  <span class="token string">&quot;10-sub-pixel-rgb.conf&quot;</span><span class="token punctuation">,</span>
  <span class="token comment"># Enable lcdfilter.  Reference:</span>
  <span class="token comment"># &lt;https://forum.endeavouros.com/t/faq-bad-font-rendering-in-firefox-and-other-programs/13430/3&gt;</span>
  <span class="token string">&quot;11-lcdfilter-default.conf&quot;</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config/fontconfig/conf.d&quot;</span>
</code></pre></div><h3 id="app" tabindex="-1"><code>App</code> <a class="header-anchor" href="#app" aria-hidden="true">#</a></h3><p>For example, a group of GUI applications under the <a href="https://wayland.freedesktop.org" target="_blank" rel="noopener noreferrer">wayland protocol</a> could be defined as:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">scope</span> <span class="token punctuation">=</span> <span class="token string">&quot;General&quot;</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;gui&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;/path/to/your/dotfiles/library/root&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span>
  <span class="token string">&quot;.gtkrc-2.0&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.local/share/icons&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.local/share/fcitx5&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.config/sway&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.config/swaylock&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.config/waybar&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.config/dunst&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;.config/gtk-*.0&quot;</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~&quot;</span>
</code></pre></div><h3 id="general" tabindex="-1"><code>General</code> <a class="header-anchor" href="#general" aria-hidden="true">#</a></h3><p>This scope is mostly used in the fallback groups, for example:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">scope</span> <span class="token punctuation">=</span> <span class="token string">&quot;General&quot;</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;xdg_config_home&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;/path/to/your/dotfiles/library/root/.config&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span>
  <span class="token string">&quot;*&quot;</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~/.config&quot;</span>
<span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">scope</span> <span class="token punctuation">=</span> <span class="token string">&quot;General&quot;</span>
<span class="token key property">name</span> <span class="token punctuation">=</span> <span class="token string">&quot;misc&quot;</span>
<span class="token key property">basedir</span> <span class="token punctuation">=</span> <span class="token string">&quot;/path/to/your/dotfiles/library/root&quot;</span>
<span class="token key property">sources</span> <span class="token punctuation">=</span> <span class="token punctuation">[</span>
  <span class="token string">&quot;.[!.]*&quot;</span><span class="token punctuation">,</span>
  <span class="token string">&quot;..?*&quot;</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
<span class="token key property">target</span> <span class="token punctuation">=</span> <span class="token string">&quot;~&quot;</span>
</code></pre></div>`,13),p=[o];function c(r,l,i,u,k,d){return a(),s("div",null,p)}var f=n(e,[["render",c]]);export{h as __pageData,f as default};
