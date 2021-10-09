import{_ as e,c as i,o,a as t}from"./app.fbfb8d02.js";const f='{"title":"Error Handling","description":"","frontmatter":{},"headers":[{"level":2,"title":"Config Validating","slug":"config-validating"},{"level":2,"title":"Sources Expanding","slug":"sources-expanding"},{"level":2,"title":"Syncing","slug":"syncing"}],"relativePath":"config/guide/error-handling.md","lastUpdated":1633751360210}',s={},a=t('<h1 id="error-handling" tabindex="-1">Error Handling <a class="header-anchor" href="#error-handling" aria-hidden="true">#</a></h1><p>For an application that works with your daily configuration files, it&#39;s crucial that it handles error in an expectable manner.</p><p><code>dt-cli</code> looks for possible errors in 3 stages throughout its running course, this section describes the checking in details.</p><h2 id="config-validating" tabindex="-1">Config Validating <a class="header-anchor" href="#config-validating" aria-hidden="true">#</a></h2><p>Firstly, after a config file has been successfully loaded into memory, <code>dt-cli</code> validates each field of the config object. Specifically, the following cases are considered invalid:</p><ul><li>Any group that has empty <code>name</code>/<code>basedir</code>/<code>target</code></li><li>Any group name that <ul><li>is also another group&#39;s name (duplicated group name)</li><li>contains <code>/</code> (used for creating subdirectory under <code>staging</code> directory)</li></ul></li><li>Any group that has the same <code>basedir</code> and <code>target</code></li><li>Any group whose <code>basedir</code> contains any occurrences (at least 1) of <code>hostname_sep</code></li><li>Any source item that: <ul><li>begins with <code>../</code> (references the parent of base directory)</li><li>begins with <code>~</code> or <code>/</code> (path is absolute)</li><li>is <code>.*</code> (bad globbing pattern, it will expand to parent directory)</li><li>ends with <code>/.*</code> (bad globbing pattern)</li></ul></li></ul><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>Checking operations in this step does not touch the filesystem, only matchs string patterns. This is for spotting obvious errors as fast as possible.</p></div><h2 id="sources-expanding" tabindex="-1">Sources Expanding <a class="header-anchor" href="#sources-expanding" aria-hidden="true">#</a></h2><p>If the above validating step passed successfully, <code>dt-cli</code> begins to iterate through every group, recursively expand all sources according their file hierarchy, the <code>basedir</code>s are also checked expanded to <a href="/host-specific.html">host-specific</a> ones wherever possible. The following cases are considered invalid:</p><ul><li>The group&#39;s <code>basedir</code> is non-existent</li><li>The group&#39;s <code>basedir</code> exists but is not a directory</li><li>The group&#39;s <code>target</code> exists and is not a directory</li><li>The group&#39;s <code>target</code> is non-existent but cannot be created</li><li>When any group uses the <code>Symlink</code> <a href="/config/guide/syncing-methods.html">syncing method</a>: <ul><li><code>staging</code> exists but iis not a directory</li><li><code>staging</code> is non-existent but cannot be created</li></ul></li></ul><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>Checking operations does not create or modify anything, only query the filesystem to check existences and permissions.</p></div><h2 id="syncing" tabindex="-1">Syncing <a class="header-anchor" href="#syncing" aria-hidden="true">#</a></h2><p>Finally, if no error can be found, <code>dt-cli</code> carefully (and efficiently) syncs the <strong>expanded</strong> source items to the target directory. During this process, according to the values of <a href="/config/key-references.html#allow-overwrite-1"><code>allow_overwrite</code></a>, different logging levels will be set when encountered with existing target items. Any other cases (e.g. a directory changes its permission to readonly) unhandled by the above 2 steps will cause <code>dt-cli</code> to panic.</p><div class="tip custom-block"><p class="custom-block-title">TIP</p><p>If you think there&#39;s anything missing here, your contribution is welcome! Start by following the <a href="/contributing.html">contributing guide</a>.</p></div>',14),n=[a];function c(r,d,l,h,g,u){return o(),i("div",null,n)}var y=e(s,[["render",c]]);export{f as __pageData,y as default};
