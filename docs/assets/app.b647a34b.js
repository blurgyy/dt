import{i as O,c as et,e as tt,a as nt,b as st,d as ot,f as Se,h as Ee,g as rt,j as at,k as it,l as Ce,m as ct,s as lt,r as ut,n as h,o as Ae,p as dt,q as G,t as ft,w as pt,u as k,v as B,_ as y,x as d,y as p,z as l,A as b,B as ee,C as A,D as f,E as Pe,F as C,G as Ne,H as Re,I as Te,J as j,K as q,L as M,M as W,N as te,O as g,P as $,Q as H,R as ht,S as Be,T as X,U as _t,V as P}from"./plugin-vue_export-helper.f07d1dea.js";const mt="modulepreload",ce={},vt="/",Ie=function(t,n){return!n||n.length===0?t():Promise.all(n.map(s=>{if(s=`${vt}${s}`,s in ce)return;ce[s]=!0;const o=s.endsWith(".css"),a=o?'[rel="stylesheet"]':"";if(document.querySelector(`link[href="${s}"]${a}`))return;const r=document.createElement("link");if(r.rel=o?"stylesheet":mt,o||(r.as="script",r.crossOrigin=""),r.href=s,document.head.appendChild(r),o)return new Promise((i,c)=>{r.addEventListener("load",i),r.addEventListener("error",()=>c(new Error(`Unable to preload CSS for ${s}`)))})})).then(()=>t())},gt="http://www.w3.org/2000/svg",T=typeof document!="undefined"?document:null,le=T&&T.createElement("template"),bt={insert:(e,t,n)=>{t.insertBefore(e,n||null)},remove:e=>{const t=e.parentNode;t&&t.removeChild(e)},createElement:(e,t,n,s)=>{const o=t?T.createElementNS(gt,e):T.createElement(e,n?{is:n}:void 0);return e==="select"&&s&&s.multiple!=null&&o.setAttribute("multiple",s.multiple),o},createText:e=>T.createTextNode(e),createComment:e=>T.createComment(e),setText:(e,t)=>{e.nodeValue=t},setElementText:(e,t)=>{e.textContent=t},parentNode:e=>e.parentNode,nextSibling:e=>e.nextSibling,querySelector:e=>T.querySelector(e),setScopeId(e,t){e.setAttribute(t,"")},cloneNode(e){const t=e.cloneNode(!0);return"_value"in e&&(t._value=e._value),t},insertStaticContent(e,t,n,s,o,a){const r=n?n.previousSibling:t.lastChild;if(o&&(o===a||o.nextSibling))for(;t.insertBefore(o.cloneNode(!0),n),!(o===a||!(o=o.nextSibling)););else{le.innerHTML=s?`<svg>${e}</svg>`:e;const i=le.content;if(s){const c=i.firstChild;for(;c.firstChild;)i.appendChild(c.firstChild);i.removeChild(c)}t.insertBefore(i,n)}return[r?r.nextSibling:t.firstChild,n?n.previousSibling:t.lastChild]}};function kt(e,t,n){const s=e._vtc;s&&(t=(t?[t,...s]:[...s]).join(" ")),t==null?e.removeAttribute("class"):n?e.setAttribute("class",t):e.className=t}function wt(e,t,n){const s=e.style,o=O(n);if(n&&!o){for(const a in n)Y(s,a,n[a]);if(t&&!O(t))for(const a in t)n[a]==null&&Y(s,a,"")}else{const a=s.display;o?t!==n&&(s.cssText=n):t&&e.removeAttribute("style"),"_vod"in e&&(s.display=a)}}const ue=/\s*!important$/;function Y(e,t,n){if(Se(n))n.forEach(s=>Y(e,t,s));else if(n==null&&(n=""),t.startsWith("--"))e.setProperty(t,n);else{const s=$t(e,t);ue.test(n)?e.setProperty(Ee(s),n.replace(ue,""),"important"):e[s]=n}}const de=["Webkit","Moz","ms"],K={};function $t(e,t){const n=K[t];if(n)return n;let s=rt(t);if(s!=="filter"&&s in e)return K[t]=s;s=at(s);for(let o=0;o<de.length;o++){const a=de[o]+s;if(a in e)return K[t]=a}return t}const fe="http://www.w3.org/1999/xlink";function yt(e,t,n,s,o){if(s&&t.startsWith("xlink:"))n==null?e.removeAttributeNS(fe,t.slice(6,t.length)):e.setAttributeNS(fe,t,n);else{const a=it(t);n==null||a&&!Ce(n)?e.removeAttribute(t):e.setAttribute(t,a?"":n)}}function xt(e,t,n,s,o,a,r){if(t==="innerHTML"||t==="textContent"){s&&r(s,o,a),e[t]=n==null?"":n;return}if(t==="value"&&e.tagName!=="PROGRESS"&&!e.tagName.includes("-")){e._value=n;const c=n==null?"":n;(e.value!==c||e.tagName==="OPTION")&&(e.value=c),n==null&&e.removeAttribute(t);return}let i=!1;if(n===""||n==null){const c=typeof e[t];c==="boolean"?n=Ce(n):n==null&&c==="string"?(n="",i=!0):c==="number"&&(n=0,i=!0)}try{e[t]=n}catch{}i&&e.removeAttribute(t)}const[He,Lt]=(()=>{let e=Date.now,t=!1;if(typeof window!="undefined"){Date.now()>document.createEvent("Event").timeStamp&&(e=performance.now.bind(performance));const n=navigator.userAgent.match(/firefox\/(\d+)/i);t=!!(n&&Number(n[1])<=53)}return[e,t]})();let Q=0;const St=Promise.resolve(),Et=()=>{Q=0},Ct=()=>Q||(St.then(Et),Q=He());function At(e,t,n,s){e.addEventListener(t,n,s)}function Pt(e,t,n,s){e.removeEventListener(t,n,s)}function Nt(e,t,n,s,o=null){const a=e._vei||(e._vei={}),r=a[t];if(s&&r)r.value=s;else{const[i,c]=Rt(t);if(s){const u=a[t]=Tt(s,o);At(e,i,u,c)}else r&&(Pt(e,i,r,c),a[t]=void 0)}}const pe=/(?:Once|Passive|Capture)$/;function Rt(e){let t;if(pe.test(e)){t={};let n;for(;n=e.match(pe);)e=e.slice(0,e.length-n[0].length),t[n[0].toLowerCase()]=!0}return[Ee(e.slice(2)),t]}function Tt(e,t){const n=s=>{const o=s.timeStamp||He();(Lt||o>=n.attached-1)&&ct(Bt(s,n.value),t,5,[s])};return n.value=e,n.attached=Ct(),n}function Bt(e,t){if(Se(t)){const n=e.stopImmediatePropagation;return e.stopImmediatePropagation=()=>{n.call(e),e._stopped=!0},t.map(s=>o=>!o._stopped&&s&&s(o))}else return t}const he=/^on[a-z]/,It=(e,t,n,s,o=!1,a,r,i,c)=>{t==="class"?kt(e,s,o):t==="style"?wt(e,n,s):nt(t)?st(t)||Nt(e,t,n,s,r):(t[0]==="."?(t=t.slice(1),!0):t[0]==="^"?(t=t.slice(1),!1):Ht(e,t,s,o))?xt(e,t,s,a,r,i,c):(t==="true-value"?e._trueValue=s:t==="false-value"&&(e._falseValue=s),yt(e,t,s,o))};function Ht(e,t,n,s){return s?!!(t==="innerHTML"||t==="textContent"||t in e&&he.test(t)&&ot(n)):t==="spellcheck"||t==="draggable"||t==="translate"||t==="form"||t==="list"&&e.tagName==="INPUT"||t==="type"&&e.tagName==="TEXTAREA"||he.test(t)&&O(n)?!1:t in e}const Dt=tt({patchProp:It},bt);let J,_e=!1;function Ot(){return J=_e?J:et(Dt),_e=!0,J}const Mt=(...e)=>{const t=Ot().createApp(...e),{mount:n}=t;return t.mount=s=>{const o=Ut(s);if(o)return n(o,!0,o instanceof SVGElement)},t};function Ut(e){return O(e)?document.querySelector(e):e}var Ft='{"lang":"en-US","title":"dt-cli","description":"Documentations for dt-cli <https://github.com/blurgyy/dt>","base":"/","head":[["link",{"rel":"stylesheet","href":"https://fonts.googleapis.com/css2?family=Roboto+Mono:wght@400;700&family=Rubik:wght@300..900&display=swap"}],["meta",{"property":"twitter:card","content":"summary_large_image"}],["meta",{"property":"twitter:image","content":"/home-everywhere.png"}],["meta",{"property":"og:image","content":"/home-everywhere.png"}],["meta",{"property":"og:image:width","content":"1200"}],["meta",{"property":"og:image:height","content":"794"}]],"themeConfig":{"repo":"blurgyy/dt","docsDir":"docs","nav":[{"text":"Overview","link":"/","activeMatch":"^/$|^/host-specific$|^/installation$|^/features|^/contributing$"},{"text":"Guide","link":"/config/guide/"},{"text":"Key References","link":"/config/key-references"}],"sidebar":{"/config/":[{"text":"Hands-on Guide","children":[{"text":"Basics","link":"/config/guide/"},{"text":"Defining Default Behaviours","link":"/config/guide/01-default-behaviours"},{"text":"Groups","link":"/config/guide/02-groups"},{"text":"Syncing Methods","link":"/config/guide/03-syncing-methods"},{"text":"Host-specific Config","link":"/config/guide/04-host-specific"},{"text":"Scopes","link":"/config/guide/05-priority"},{"text":"Filename Manipulating","link":"/config/guide/06-filename-manipulating"},{"text":"Templating","link":"/config/guide/07-templating"},{"text":"Error Handling","link":"/config/guide/99-error-handling"}]},{"text":"Key References","link":"/config/key-references"}],"/":[{"text":"\u{1F440} Overview","link":"/"},{"text":"\u{1F4A0} Features","link":"/features/","children":[{"text":"Host-specific Syncing","link":"/features/01-host-specific"},{"text":"Priority Resolving","link":"/features/02-scope"},{"text":"Filename Manipulating","link":"/features/03-filename-manipulating"},{"text":"Templating","link":"/features/04-templating"}]},{"text":"\u{1F680} Installation","link":"/installation"},{"text":"\u{1F4E8} Contributing","link":"/contributing"}]}},"locales":{},"langs":{}}';const De=/^https?:/i,S=typeof window!="undefined";function jt(e,t){t.sort((n,s)=>{const o=s.split("/").length-n.split("/").length;return o!==0?o:s.length-n.length});for(const n of t)if(e.startsWith(n))return n}function me(e,t){const n=jt(t,Object.keys(e));return n?e[n]:void 0}function qt(e){const{locales:t}=e.themeConfig||{},n=e.locales;return t&&n?Object.keys(t).reduce((s,o)=>(s[o]={label:t[o].label,lang:n[o].lang},s),{}):{}}function Wt(e,t){t=zt(e,t);const n=me(e.locales||{},t),s=me(e.themeConfig.locales||{},t);return Object.assign({},e,n,{themeConfig:Object.assign({},e.themeConfig,s,{locales:{}}),lang:(n||e).lang,locales:{},langs:qt(e)})}function zt(e,t){if(!S)return t;const n=e.base,s=n.endsWith("/")?n.slice(0,-1):n;return t.slice(s.length)}const Oe=Symbol(),ne=lt(Gt(Ft));function Gt(e){return ut(JSON.parse(e))}function Kt(e){const t=h(()=>Wt(ne.value,e.path));return{site:t,theme:h(()=>t.value.themeConfig),page:h(()=>e.data),frontmatter:h(()=>e.data.frontmatter),lang:h(()=>t.value.lang),localePath:h(()=>{const{langs:n,lang:s}=t.value,o=Object.keys(n).find(a=>n[a].lang===s);return I(o||"/")}),title:h(()=>e.data.title?e.data.title+" | "+t.value.title:t.value.title),description:h(()=>e.data.description||t.value.description)}}function E(){const e=Ae(Oe);if(!e)throw new Error("vitepress data not properly injected in app");return e}function Jt(e,t){return`${e}${t}`.replace(/\/+/g,"/")}function I(e){return De.test(e)?e:Jt(ne.value.base,e)}function Me(e){let t=e.replace(/\.html$/,"");if(t=decodeURIComponent(t),t.endsWith("/")&&(t+="index"),S){const n="/";t=t.slice(n.length).replace(/\//g,"_")+".md";const s=__VP_HASH_MAP__[t.toLowerCase()];t=`${n}assets/${t}.${s}.js`}else t=`./${t.slice(1).replace(/\//g,"_")}.md.js`;return t}const Ue=Symbol(),ve="http://a.com",Vt=()=>({path:"/",component:null,data:null});function Xt(e,t){const n=dt(Vt());function s(r=S?location.href:"/"){const i=new URL(r,ve);return!i.pathname.endsWith("/")&&!i.pathname.endsWith(".html")&&(i.pathname+=".html",r=i.pathname+i.search+i.hash),S&&(history.replaceState({scrollPosition:window.scrollY},document.title),history.pushState(null,"",r)),a(r)}let o=null;async function a(r,i=0){const c=new URL(r,ve),u=o=c.pathname;try{let _=e(u);if("then"in _&&typeof _.then=="function"&&(_=await _),o===u){o=null;const{default:m,__pageData:x}=_;if(!m)throw new Error(`Invalid route component: ${m}`);n.path=u,n.component=G(m),n.data=G(JSON.parse(x)),S&&ft(()=>{if(c.hash&&!i){let w=null;try{w=document.querySelector(decodeURIComponent(c.hash))}catch(L){console.warn(L)}if(w){ge(w,c.hash);return}}window.scrollTo(0,i)})}}catch(_){_.message.match(/fetch/)||console.error(_),o===u&&(o=null,n.path=u,n.component=t?G(t):null)}}return S&&(window.addEventListener("click",r=>{const i=r.target.closest("a");if(i){const{href:c,protocol:u,hostname:_,pathname:m,hash:x,target:w}=i,L=window.location,R=m.match(/\.\w+$/);!r.ctrlKey&&!r.shiftKey&&!r.altKey&&!r.metaKey&&w!=="_blank"&&u===L.protocol&&_===L.hostname&&!(R&&R[0]!==".html")&&(r.preventDefault(),m===L.pathname?x&&x!==L.hash&&(history.pushState(null,"",x),window.dispatchEvent(new Event("hashchange")),ge(i,x,i.classList.contains("header-anchor"))):s(c))}},{capture:!0}),window.addEventListener("popstate",r=>{a(location.href,r.state&&r.state.scrollPosition||0)}),window.addEventListener("hashchange",r=>{r.preventDefault()})),{route:n,go:s}}function Yt(){const e=Ae(Ue);if(!e)throw new Error("useRouter() is called without provider.");return e}function N(){return Yt().route}function ge(e,t,n=!1){let s=null;try{s=e.classList.contains(".header-anchor")?e:document.querySelector(decodeURIComponent(t))}catch(o){console.warn(o)}if(s){const o=s.offsetTop;!n||Math.abs(o-window.scrollY)>window.innerHeight?window.scrollTo(0,o):window.scrollTo({left:0,top:o,behavior:"smooth"})}}function Qt(e,t){let n=[],s=!0;const o=a=>{if(s){s=!1;return}const r=[],i=Math.min(n.length,a.length);for(let c=0;c<i;c++){let u=n[c];const[_,m,x=""]=a[c];if(u.tagName.toLocaleLowerCase()===_){for(const w in m)u.getAttribute(w)!==m[w]&&u.setAttribute(w,m[w]);for(let w=0;w<u.attributes.length;w++){const L=u.attributes[w].name;L in m||u.removeAttribute(L)}u.innerHTML!==x&&(u.innerHTML=x)}else document.head.removeChild(u),u=be(a[c]),document.head.append(u);r.push(u)}n.slice(i).forEach(c=>document.head.removeChild(c)),a.slice(i).forEach(c=>{const u=be(c);document.head.appendChild(u),r.push(u)}),n=r};pt(()=>{const a=e.data,r=t.value,i=a&&a.title,c=a&&a.description,u=a&&a.frontmatter.head;document.title=(i?i+" | ":"")+r.title,document.querySelector("meta[name=description]").setAttribute("content",c||r.description),o([...u?en(u):[]])})}function be([e,t,n]){const s=document.createElement(e);for(const o in t)s.setAttribute(o,t[o]);return n&&(s.innerHTML=n),s}function Zt(e){return e[0]==="meta"&&e[1]&&e[1].name==="description"}function en(e){return e.filter(t=>!Zt(t))}const tn=k({name:"VitePressContent",setup(){const e=N();return()=>B("div",{style:{position:"relative"}},[e.component?B(e.component):null])}});const nn=/#.*$/,sn=/(index)?\.(md|html)$/,U=/\/$/,on=/^[a-z]+:/i;function se(e){return Array.isArray(e)}function oe(e){return on.test(e)}function rn(e,t){if(t===void 0)return!1;const n=ke(`/${e.data.relativePath}`),s=ke(t);return n===s}function ke(e){return decodeURI(e).replace(nn,"").replace(sn,"")}function an(e,t){const n=e.endsWith("/"),s=t.startsWith("/");return n&&s?e.slice(0,-1)+t:!n&&!s?`${e}/${t}`:e+t}function Z(e){return/^\//.test(e)?e:`/${e}`}function Fe(e){return e.replace(/(index)?(\.(md|html))?$/,"")||"/"}function cn(e){return e===!1||e==="auto"||se(e)}function ln(e){return e.children!==void 0}function un(e){return se(e)?e.length===0:!e}function re(e,t){if(cn(e))return e;t=Z(t);for(const n in e)if(t.startsWith(Z(n)))return e[n];return"auto"}function je(e){return e.reduce((t,n)=>(n.link&&t.push({text:n.text,link:Fe(n.link)}),ln(n)&&(t=[...t,...je(n.children)]),t),[])}const dn=["href","aria-label"],fn=["src"],pn=k({name:"NavBarTitle",setup(e){const{site:t,theme:n,localePath:s}=E();return(o,a)=>(d(),p("a",{class:"nav-bar-title",href:l(s),"aria-label":`${l(t).title}, back to home`},[l(n).logo?(d(),p("img",{key:0,class:"logo",src:l(I)(l(n).logo),alt:"Logo"},null,8,fn)):b("",!0),ee(" "+A(l(t).title),1)],8,dn))}});var hn=y(pn,[["__scopeId","data-v-442480ce"]]);function _n(){const{site:e,localePath:t,theme:n}=E();return h(()=>{const s=e.value.langs,o=Object.keys(s);if(o.length<2)return null;const r=N().path.replace(t.value,""),i=o.map(u=>({text:s[u].label,link:`${u}${r}`}));return{text:n.value.selectText||"Languages",items:i}})}const mn=["GitHub","GitLab","Bitbucket"].map(e=>[e,new RegExp(e,"i")]);function vn(){const{site:e}=E();return h(()=>{const t=e.value.themeConfig,n=t.docsRepo||t.repo;if(!n)return null;const s=gn(n);return{text:bn(s,t.repoLabel),link:s}})}function gn(e){return De.test(e)?e:`https://github.com/${e}`}function bn(e,t){if(t)return t;const n=e.match(/^https?:\/\/[^/]+/);if(!n)return"Source";const s=mn.find(([o,a])=>a.test(n[0]));return s&&s[0]?s[0]:"Source"}function qe(e){const t=N(),n=oe(e.value.link);return{props:h(()=>{const o=we(`/${t.data.relativePath}`);let a=!1;if(e.value.activeMatch)a=new RegExp(e.value.activeMatch).test(o);else{const r=we(e.value.link);a=r==="/"?r===o:o.startsWith(r)}return{class:{active:a,isExternal:n},href:n?e.value.link:I(e.value.link),target:e.value.target||(n?"_blank":null),rel:e.value.rel||(n?"noopener noreferrer":null),"aria-label":e.value.ariaLabel}}),isExternal:n}}function we(e){return e.replace(/#.*$/,"").replace(/\?.*$/,"").replace(/\.(html|md)$/,"").replace(/\/index$/,"/")}const kn={},wn={class:"icon outbound",xmlns:"http://www.w3.org/2000/svg","aria-hidden":"true",x:"0px",y:"0px",viewBox:"0 0 100 100",width:"15",height:"15"},$n=f("path",{fill:"currentColor",d:"M18.8,85.1h56l0,0c2.2,0,4-1.8,4-4v-32h-8v28h-48v-48h28v-8h-32l0,0c-2.2,0-4,1.8-4,4v56C14.8,83.3,16.6,85.1,18.8,85.1z"},null,-1),yn=f("polygon",{fill:"currentColor",points:"45.7,48.7 51.3,54.3 77.2,28.5 77.2,37.2 85.2,37.2 85.2,14.9 62.8,14.9 62.8,22.9 71.5,22.9"},null,-1),xn=[$n,yn];function Ln(e,t){return d(),p("svg",wn,xn)}var ae=y(kn,[["render",Ln]]);const Sn={class:"nav-link"},En=k({name:"NavLink",props:{item:null},setup(e){const n=Pe(e),{props:s,isExternal:o}=qe(n.item);return(a,r)=>(d(),p("div",Sn,[f("a",Ne({class:"item"},l(s)),[ee(A(e.item.text)+" ",1),l(o)?(d(),C(ae,{key:0})):b("",!0)],16)]))}});var $e=y(En,[["__scopeId","data-v-3ca3777c"]]);const Cn=e=>(Re("data-v-09b45080"),e=e(),Te(),e),An={class:"nav-dropdown-link-item"},Pn=Cn(()=>f("span",{class:"arrow"},null,-1)),Nn={class:"text"},Rn={class:"icon"},Tn=k({name:"NavDropdownLinkItem",props:{item:null},setup(e){const n=Pe(e),{props:s,isExternal:o}=qe(n.item);return(a,r)=>(d(),p("div",An,[f("a",Ne({class:"item"},l(s)),[Pn,f("span",Nn,A(e.item.text),1),f("span",Rn,[l(o)?(d(),C(ae,{key:0})):b("",!0)])],16)]))}});var Bn=y(Tn,[["__scopeId","data-v-09b45080"]]);const In=["aria-label"],Hn={class:"button-text"},Dn={class:"dialog"},On=k({name:"NavDropdownLink",props:{item:null},setup(e){const t=N(),n=j(!1);q(()=>t.path,()=>{n.value=!1});function s(){n.value=!n.value}return(o,a)=>(d(),p("div",{class:M(["nav-dropdown-link",{open:n.value}])},[f("button",{class:"button","aria-label":e.item.ariaLabel,onClick:s},[f("span",Hn,A(e.item.text),1),f("span",{class:M(["button-arrow",n.value?"down":"right"])},null,2)],8,In),f("ul",Dn,[(d(!0),p(W,null,te(e.item.items,r=>(d(),p("li",{key:r.text,class:"dialog-item"},[g(Bn,{item:r},null,8,["item"])]))),128))])],2))}});var ye=y(On,[["__scopeId","data-v-00eb7447"]]);const Mn={key:0,class:"nav-links"},Un={key:1,class:"item"},Fn={key:2,class:"item"},jn=k({name:"NavLinks",setup(e){const{theme:t}=E(),n=_n(),s=vn(),o=h(()=>t.value.nav||s.value||n.value);return(a,r)=>l(o)?(d(),p("nav",Mn,[l(t).nav?(d(!0),p(W,{key:0},te(l(t).nav,i=>(d(),p("div",{key:i.text,class:"item"},[i.items?(d(),C(ye,{key:0,item:i},null,8,["item"])):(d(),C($e,{key:1,item:i},null,8,["item"]))]))),128)):b("",!0),l(n)?(d(),p("div",Un,[g(ye,{item:l(n)},null,8,["item"])])):b("",!0),l(s)?(d(),p("div",Fn,[g($e,{item:l(s)},null,8,["item"])])):b("",!0)])):b("",!0)}});var We=y(jn,[["__scopeId","data-v-081535f9"]]);const qn={emits:["toggle"]},Wn=f("svg",{class:"icon",xmlns:"http://www.w3.org/2000/svg","aria-hidden":"true",role:"img",viewBox:"0 0 448 512"},[f("path",{fill:"currentColor",d:"M436 124H12c-6.627 0-12-5.373-12-12V80c0-6.627 5.373-12 12-12h424c6.627 0 12 5.373 12 12v32c0 6.627-5.373 12-12 12zm0 160H12c-6.627 0-12-5.373-12-12v-32c0-6.627 5.373-12 12-12h424c6.627 0 12 5.373 12 12v32c0 6.627-5.373 12-12 12zm0 160H12c-6.627 0-12-5.373-12-12v-32c0-6.627 5.373-12 12-12h424c6.627 0 12 5.373 12 12v32c0 6.627-5.373 12-12 12z",class:""})],-1),zn=[Wn];function Gn(e,t,n,s,o,a){return d(),p("div",{class:"sidebar-button",onClick:t[0]||(t[0]=r=>e.$emit("toggle"))},zn)}var Kn=y(qn,[["render",Gn]]);const Jn=e=>(Re("data-v-38994b46"),e=e(),Te(),e),Vn={class:"nav-bar"},Xn=Jn(()=>f("div",{class:"flex-grow"},null,-1)),Yn={class:"nav"},Qn=k({name:"NavBar",emits:["toggle"],setup(e){return(t,n)=>(d(),p("header",Vn,[g(Kn,{onToggle:n[0]||(n[0]=s=>t.$emit("toggle"))}),g(hn),Xn,f("div",Yn,[g(We)]),$(t.$slots,"search",{},void 0,!0)]))}});var Zn=y(Qn,[["__scopeId","data-v-38994b46"]]);function es(){let e=null,t=null;const n=rs(s,300);function s(){const r=ts(),i=ns(r);for(let c=0;c<i.length;c++){const u=i[c],_=i[c+1],[m,x]=os(c,u,_);if(m){history.replaceState(null,document.title,x||" "),o(x);return}}}function o(r){if(a(t),a(e),t=document.querySelector(`.sidebar a[href="${r}"]`),!t)return;t.classList.add("active");const i=t.closest(".sidebar-links > ul > li");i&&i!==t.parentElement?(e=i.querySelector("a"),e&&e.classList.add("active")):e=null}function a(r){r&&r.classList.remove("active")}H(()=>{s(),window.addEventListener("scroll",n)}),ht(()=>{o(decodeURIComponent(location.hash))}),Be(()=>{window.removeEventListener("scroll",n)})}function ts(){return[].slice.call(document.querySelectorAll(".sidebar a.sidebar-link-item"))}function ns(e){return[].slice.call(document.querySelectorAll(".header-anchor")).filter(t=>e.some(n=>n.hash===t.hash))}function ss(){return document.querySelector(".nav-bar").offsetHeight}function xe(e){const t=ss();return e.parentElement.offsetTop-t-15}function os(e,t,n){const s=window.scrollY;return e===0&&s===0?[!0,null]:s<xe(t)?[!1,null]:!n||s<xe(n)?[!0,decodeURIComponent(t.hash)]:[!1,null]}function rs(e,t){let n,s=!1;return()=>{n&&clearTimeout(n),s?n=setTimeout(e,t):(e(),s=!0,setTimeout(()=>{s=!1},t))}}function as(){const e=N(),{site:t}=E();return es(),h(()=>{const n=e.data.headers,s=e.data.frontmatter.sidebar,o=e.data.frontmatter.sidebarDepth;if(s===!1)return[];if(s==="auto")return Le(n,o);const a=re(t.value.themeConfig.sidebar,e.data.relativePath);return a===!1?[]:a==="auto"?Le(n,o):a})}function Le(e,t){const n=[];if(e===void 0)return[];let s;return e.forEach(({level:o,title:a,slug:r})=>{if(o-1>t)return;const i={text:a,link:`#${r}`};o===2?(s=i,n.push(i)):s&&(s.children||(s.children=[])).push(i)}),n}const ze=e=>{const t=N(),{site:n,frontmatter:s}=E(),o=e.depth||1,a=s.value.sidebarDepth||1/0,r=t.data.headers,i=e.item.text,c=is(n.value.base,e.item.link),u=e.item.children,_=rn(t,e.item.link),m=o<a?Ge(_,u,r,o+1):null;return B("li",{class:"sidebar-link"},[B(c?"a":"p",{class:{"sidebar-link-item":!0,active:_},href:c},i),m])};function is(e,t){return t===void 0||t.startsWith("#")?t:an(e,t)}function Ge(e,t,n,s=1){return t&&t.length>0?B("ul",{class:"sidebar-links"},t.map(o=>B(ze,{item:o,depth:s}))):e&&n?Ge(!1,cs(n),void 0,s):null}function cs(e){return Ke(ls(e))}function ls(e){e=e.map(n=>Object.assign({},n));let t;return e.forEach(n=>{n.level===2?t=n:t&&(t.children||(t.children=[])).push(n)}),e.filter(n=>n.level===2)}function Ke(e){return e.map(t=>({text:t.title,link:`#${t.slug}`,children:t.children?Ke(t.children):void 0}))}const us={key:0,class:"sidebar-links"},ds=k({name:"SideBarLinks",setup(e){const t=as();return(n,s)=>l(t).length>0?(d(),p("ul",us,[(d(!0),p(W,null,te(l(t),o=>(d(),C(l(ze),{item:o},null,8,["item"]))),256))])):b("",!0)}});const fs=k({name:"SideBar",props:{open:{type:Boolean}},setup(e){return(t,n)=>(d(),p("aside",{class:M(["sidebar",{open:e.open}])},[g(We,{class:"nav"}),$(t.$slots,"sidebar-top",{},void 0,!0),g(ds),$(t.$slots,"sidebar-bottom",{},void 0,!0)],2))}});var ps=y(fs,[["__scopeId","data-v-20a0ea58"]]);const hs=/bitbucket.org/;function _s(){const{page:e,theme:t,frontmatter:n}=E(),s=h(()=>{const{repo:a,docsDir:r="",docsBranch:i="master",docsRepo:c=a,editLinks:u}=t.value,_=n.value.editLink!=null?n.value.editLink:u,{relativePath:m}=e.value;return!_||!m||!a?null:ms(a,c,r,i,m)}),o=h(()=>t.value.editLinkText||"Edit this page");return{url:s,text:o}}function ms(e,t,n,s,o){return hs.test(e)?gs(e,t,n,s,o):vs(e,t,n,s,o)}function vs(e,t,n,s,o){return(oe(t)?t:`https://github.com/${t}`).replace(U,"")+`/edit/${s}/`+(n?n.replace(U,"")+"/":"")+o}function gs(e,t,n,s,o){return(oe(t)?t:e).replace(U,"")+`/src/${s}/`+(n?n.replace(U,"")+"/":"")+o+`?mode=edit&spa=0&at=${s}&fileviewer=file-view-default`}const bs={class:"edit-link"},ks=["href"],ws=k({name:"EditLink",setup(e){const{url:t,text:n}=_s();return(s,o)=>(d(),p("div",bs,[l(t)?(d(),p("a",{key:0,class:"link",href:l(t),target:"_blank",rel:"noopener noreferrer"},[ee(A(l(n))+" ",1),g(ae,{class:"icon"})],8,ks)):b("",!0)]))}});var $s=y(ws,[["__scopeId","data-v-6166d546"]]);const ys={key:0,class:"last-updated"},xs={class:"prefix"},Ls={class:"datetime"},Ss=k({name:"LastUpdated",setup(e){const{theme:t,page:n}=E(),s=h(()=>{const r=t.value.lastUpdated;return r!==void 0&&r!==!1}),o=h(()=>{const r=t.value.lastUpdated;return r===!0?"Last Updated":r}),a=j("");return H(()=>{a.value=new Date(n.value.lastUpdated).toLocaleString("en-US")}),(r,i)=>l(s)?(d(),p("p",ys,[f("span",xs,A(l(o))+":",1),f("span",Ls,A(a.value),1)])):b("",!0)}});var Es=y(Ss,[["__scopeId","data-v-5e9d8ba2"]]);const Cs={class:"page-footer"},As={class:"edit"},Ps={class:"updated"},Ns=k({name:"PageFooter",setup(e){return(t,n)=>(d(),p("footer",Cs,[f("div",As,[g($s)]),f("div",Ps,[g(Es)])]))}});var Rs=y(Ns,[["__scopeId","data-v-0673e1a5"]]);function Ts(){const{page:e,theme:t}=E(),n=h(()=>Fe(Z(e.value.relativePath))),s=h(()=>{const c=re(t.value.sidebar,n.value);return se(c)?je(c):[]}),o=h(()=>s.value.findIndex(c=>c.link===n.value)),a=h(()=>{if(t.value.nextLinks!==!1&&o.value>-1&&o.value<s.value.length-1)return s.value[o.value+1]}),r=h(()=>{if(t.value.prevLinks!==!1&&o.value>0)return s.value[o.value-1]}),i=h(()=>!!a.value||!!r.value);return{next:a,prev:r,hasLinks:i}}const Bs={},Is={xmlns:"http://www.w3.org/2000/svg",viewBox:"0 0 24 24"},Hs=f("path",{d:"M19,11H7.4l5.3-5.3c0.4-0.4,0.4-1,0-1.4s-1-0.4-1.4,0l-7,7c-0.1,0.1-0.2,0.2-0.2,0.3c-0.1,0.2-0.1,0.5,0,0.8c0.1,0.1,0.1,0.2,0.2,0.3l7,7c0.2,0.2,0.5,0.3,0.7,0.3s0.5-0.1,0.7-0.3c0.4-0.4,0.4-1,0-1.4L7.4,13H19c0.6,0,1-0.4,1-1S19.6,11,19,11z"},null,-1),Ds=[Hs];function Os(e,t){return d(),p("svg",Is,Ds)}var Ms=y(Bs,[["render",Os]]);const Us={},Fs={xmlns:"http://www.w3.org/2000/svg",viewBox:"0 0 24 24"},js=f("path",{d:"M19.9,12.4c0.1-0.2,0.1-0.5,0-0.8c-0.1-0.1-0.1-0.2-0.2-0.3l-7-7c-0.4-0.4-1-0.4-1.4,0s-0.4,1,0,1.4l5.3,5.3H5c-0.6,0-1,0.4-1,1s0.4,1,1,1h11.6l-5.3,5.3c-0.4,0.4-0.4,1,0,1.4c0.2,0.2,0.5,0.3,0.7,0.3s0.5-0.1,0.7-0.3l7-7C19.8,12.6,19.9,12.5,19.9,12.4z"},null,-1),qs=[js];function Ws(e,t){return d(),p("svg",Fs,qs)}var zs=y(Us,[["render",Ws]]);const Gs={key:0,class:"next-and-prev-link"},Ks={class:"container"},Js={class:"prev"},Vs=["href"],Xs={class:"text"},Ys={class:"next"},Qs=["href"],Zs={class:"text"},eo=k({name:"NextAndPrevLinks",setup(e){const{hasLinks:t,prev:n,next:s}=Ts();return(o,a)=>l(t)?(d(),p("div",Gs,[f("div",Ks,[f("div",Js,[l(n)?(d(),p("a",{key:0,class:"link",href:l(I)(l(n).link)},[g(Ms,{class:"icon icon-prev"}),f("span",Xs,A(l(n).text),1)],8,Vs)):b("",!0)]),f("div",Ys,[l(s)?(d(),p("a",{key:0,class:"link",href:l(I)(l(s).link)},[f("span",Zs,A(l(s).text),1),g(zs,{class:"icon icon-next"})],8,Qs)):b("",!0)])])])):b("",!0)}});var to=y(eo,[["__scopeId","data-v-09248457"]]);const no={class:"page"},so={class:"container"},oo=k({name:"Page",setup(e){return(t,n)=>{const s=X("Content");return d(),p("main",no,[f("div",so,[$(t.$slots,"top",{},void 0,!0),g(s,{class:"content"}),g(Rs),g(to),$(t.$slots,"bottom",{},void 0,!0)])])}}});var ro=y(oo,[["__scopeId","data-v-19335ccc"]]);const ao={key:0,id:"ads-container"},io=k({name:"Layout",setup(e){const t=_t(()=>Ie(()=>import("./Home.93fd653c.js"),["assets/Home.93fd653c.js","assets/plugin-vue_export-helper.f07d1dea.js"])),n=()=>null,s=n,o=n,a=n,r=N(),{site:i,page:c,theme:u,frontmatter:_}=E(),m=h(()=>!!_.value.customLayout),x=h(()=>!!_.value.home),w=h(()=>Object.keys(i.value.langs).length>1),L=h(()=>{const v=u.value;return _.value.navbar===!1||v.navbar===!1?!1:i.value.title||v.logo||v.repo||v.nav}),R=j(!1),Ve=h(()=>_.value.home||_.value.sidebar===!1?!1:!un(re(u.value.sidebar,r.data.relativePath))),z=v=>{R.value=typeof v=="boolean"?v:!R.value},Xe=z.bind(null,!1);q(r,Xe);const Ye=h(()=>[{"no-navbar":!L.value,"sidebar-open":R.value,"no-sidebar":!Ve.value}]);return(v,ie)=>{const Qe=X("Content"),Ze=X("Debug");return d(),p(W,null,[f("div",{class:M(["theme",l(Ye)])},[l(L)?(d(),C(Zn,{key:0,onToggle:z},{search:P(()=>[$(v.$slots,"navbar-search",{},()=>[l(u).algolia?(d(),C(l(a),{key:0,options:l(u).algolia,multilang:l(w)},null,8,["options","multilang"])):b("",!0)])]),_:3})):b("",!0),g(ps,{open:R.value},{"sidebar-top":P(()=>[$(v.$slots,"sidebar-top")]),"sidebar-bottom":P(()=>[$(v.$slots,"sidebar-bottom")]),_:3},8,["open"]),f("div",{class:"sidebar-mask",onClick:ie[0]||(ie[0]=yo=>z(!1))}),l(m)?(d(),C(Qe,{key:1})):l(x)?$(v.$slots,"home",{key:2},()=>[g(l(t),null,{hero:P(()=>[$(v.$slots,"home-hero")]),features:P(()=>[$(v.$slots,"home-features")]),footer:P(()=>[$(v.$slots,"home-footer")]),_:3})]):(d(),C(ro,{key:3},{top:P(()=>[$(v.$slots,"page-top-ads",{},()=>[l(u).carbonAds&&l(u).carbonAds.carbon?(d(),p("div",ao,[(d(),C(l(s),{key:"carbon"+l(c).relativePath,code:l(u).carbonAds.carbon,placement:l(u).carbonAds.placement},null,8,["code","placement"]))])):b("",!0)]),$(v.$slots,"page-top")]),bottom:P(()=>[$(v.$slots,"page-bottom"),$(v.$slots,"page-bottom-ads",{},()=>[l(u).carbonAds&&l(u).carbonAds.custom?(d(),C(l(o),{key:"custom"+l(c).relativePath,code:l(u).carbonAds.custom,placement:l(u).carbonAds.placement},null,8,["code","placement"])):b("",!0)])]),_:3}))],2),g(Ze)],64)}}}),co={class:"theme"},lo=f("h1",null,"404",-1),uo=["href"],fo=k({name:"NotFound",setup(e){const{site:t}=E(),n=["There's nothing here.","How did we get here?","That's a Four-Oh-Four.","Looks like we've got some broken links."];function s(){return n[Math.floor(Math.random()*n.length)]}return(o,a)=>(d(),p("div",co,[lo,f("blockquote",null,A(s()),1),f("a",{href:l(t).base,"aria-label":"go to home"},"Take me home.",8,uo)]))}}),F={Layout:io,NotFound:fo};const V=new Set,Je=()=>document.createElement("link"),po=e=>{const t=Je();t.rel="prefetch",t.href=e,document.head.appendChild(t)},ho=e=>{const t=new XMLHttpRequest;t.open("GET",e,t.withCredentials=!0),t.send()};let D;const _o=S&&(D=Je())&&D.relList&&D.relList.supports&&D.relList.supports("prefetch")?po:ho;function mo(){if(!S||!window.IntersectionObserver)return;let e;if((e=navigator.connection)&&(e.saveData||/2g/.test(e.effectiveType)))return;const t=window.requestIdleCallback||setTimeout;let n=null;const s=()=>{n&&n.disconnect(),n=new IntersectionObserver(a=>{a.forEach(r=>{if(r.isIntersecting){const i=r.target;n.unobserve(i);const{pathname:c}=i;if(!V.has(c)){V.add(c);const u=Me(c);_o(u)}}})}),t(()=>{document.querySelectorAll("#app a").forEach(a=>{const{target:r,hostname:i,pathname:c}=a,u=c.match(/\.\w+$/);u&&u[0]!==".html"||r!=="_blank"&&i===location.hostname&&(c!==location.pathname?n.observe(a):V.add(c))})})};H(s);const o=N();q(()=>o.path,s),Be(()=>{n&&n.disconnect()})}const vo=k({setup(e,{slots:t}){const n=j(!1);return H(()=>{n.value=!0}),()=>n.value&&t.default?t.default():null}}),go=F.NotFound||(()=>"404 Not Found"),bo={name:"VitePressApp",setup(){const{site:e}=E();return H(()=>{q(()=>e.value.lang,t=>{document.documentElement.lang=t},{immediate:!0})}),mo(),()=>B(F.Layout)}};function ko(){const e=$o(),t=wo();t.provide(Ue,e);const n=Kt(e.route);return t.provide(Oe,n),S&&Qt(e.route,n.site),t.component("Content",tn),t.component("ClientOnly",vo),t.component("Debug",()=>null),Object.defineProperty(t.config.globalProperties,"$frontmatter",{get(){return n.frontmatter.value}}),F.enhanceApp&&F.enhanceApp({app:t,router:e,siteData:ne}),{app:t,router:e}}function wo(){return Mt(bo)}function $o(){let e=S,t;return Xt(n=>{let s=Me(n);return e&&(t=s),(e||t===s)&&(s=s.replace(/\.js$/,".lean.js")),S?(e=!1,Ie(()=>import(s),[])):require(s)},go)}if(S){const{app:e,router:t}=ko();t.go().then(()=>{e.mount("#app")})}export{$e as N,ko as createApp,E as u,I as w};
