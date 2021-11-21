import{_ as e,c as t,o as s,a as o}from"./app.b78d5d90.js";const g='{"title":"Syncing Methods","description":"","frontmatter":{},"headers":[{"level":2,"title":"Overview","slug":"overview"},{"level":3,"title":"Copy","slug":"copy"},{"level":3,"title":"Symlink","slug":"symlink"},{"level":2,"title":"Default Method","slug":"default-method"},{"level":2,"title":"Overriding","slug":"overriding"}],"relativePath":"config/guide/03-syncing-methods.md","lastUpdated":1637466052699}',a={},n=o(`__VP_STATIC_START__<h1 id="syncing-methods" tabindex="-1">Syncing Methods <a class="header-anchor" href="#syncing-methods" aria-hidden="true">#</a></h1><p>Until the last section, no comments has been given on the <strong>stage</strong> -&gt; <strong>symlink</strong> steps. This section explains all the details a user wants to know about this process.</p><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>If you are interested in all the details of the process, I refer you to the implementation of <code>dt_core::syncing::sync_core</code><a href="https://github.com/blurgyy/dt/blob/main/dt-core/src/syncing.rs" target="_blank" rel="noopener noreferrer">here</a>.</p></div><h2 id="overview" tabindex="-1">Overview <a class="header-anchor" href="#overview" aria-hidden="true">#</a></h2><p>There are 2 available syncing methods: <code>Copy</code> and <code>Symlink</code>, where <code>Symlink</code> is the chosen default.</p><h3 id="copy" tabindex="-1"><code>Copy</code> <a class="header-anchor" href="#copy" aria-hidden="true">#</a></h3><p>Directly copies source items defined in <code>sources</code> arrays to target.</p><h3 id="symlink" tabindex="-1"><code>Symlink</code> <a class="header-anchor" href="#symlink" aria-hidden="true">#</a></h3><p>First copies source items defined in <code>sources</code> arrays (this is called <em>staging</em>) to <strong>current group&#39;s</strong> staging directory (see <a href="/config/key-references.html#staging"><code>global.staging</code></a> and <a href="/config/key-references.html#name"><code>name</code></a>), then symlinks the staged items to target.</p><h2 id="default-method" tabindex="-1">Default Method <a class="header-anchor" href="#default-method" aria-hidden="true">#</a></h2><p><code>dt-cli</code> chooses <code>Symlink</code> as the default behaviour. The added <em>staging</em> step:</p><ul><li>Makes it possible to organize sources according to their group <a href="/config/key-references.html#name"><code>name</code></a>s, which <code>Copy</code> does not.<div class="tip custom-block"><p class="custom-block-title">TIP</p><p>This means it allows human-readable directory structures, because groups are organized by your given <a href="/config/key-references.html#name"><code>name</code></a>s. You can also create a git repository at the staging root directory if you want,</p></div></li><li>Makes it possible to control permission of organized items from system-level <code>sources</code> which you shouldn&#39;t directly modify.</li><li>When the target and source are the same (by accident), <code>Copy</code> does not guarantee integrity of the source item, while <code>Symlink</code> preserves the file content in the staging directory.</li><li>Make all further symlinks point at most to the staged items.<div class="tip custom-block"><p class="custom-block-title">TIP</p><p>This particularly helpful when you manage user-scope systemd services with symlinks. According to <a href="https://man.archlinux.org/man/systemctl.1" target="_blank" rel="noopener noreferrer"><code>systemctl(1)</code></a>:</p><blockquote><p>Disables one or more units. This removes all symlinks to the unit files backing the specified units from the unit configuration directory, and hence undoes any changes made by enable or link. Note that this removes all symlinks to matching unit files, including manually created symlinks, and not just those actually created by enable or link.</p></blockquote><p>That said, when disabling services (with <code>systemctl --user disable</code>), <code>systemctl</code> removes all symlinks (<strong>including user-created ones!</strong>).</p><p>With this added <em>staging</em> process, your source files will be well protected.</p></div></li><li>Protects original items if you want to make experimental changes.</li></ul><h2 id="overriding" tabindex="-1">Overriding <a class="header-anchor" href="#overriding" aria-hidden="true">#</a></h2><p>You can always override the default syncing method to <code>Copy</code> conveniently by adding <code>method = &quot;Copy&quot;</code> to the <code>[global]</code> section:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token table class-name">global</span><span class="token punctuation">]</span>
<span class="token key property">method</span> <span class="token punctuation">=</span> <span class="token string">&quot;Copy&quot;</span>
</code></pre></div><p>Or specify the syncing method for a given group similarly:</p><div class="language-toml"><pre><code><span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token table class-name">local</span><span class="token punctuation">]</span><span class="token punctuation">]</span>
<span class="token key property">method</span> <span class="token punctuation">=</span> <span class="token string">&quot;Copy&quot;</span>
</code></pre></div>__VP_STATIC_END__`,17),i=[n];function c(r,l,d,h,p,u){return s(),t("div",null,i)}var y=e(a,[["render",c]]);export{g as __pageData,y as default};
