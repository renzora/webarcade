if(typeof document!=='undefined'){const s=document.createElement('style');s.setAttribute('data-plugin','demo');s.textContent=".absolute{position:absolute}.relative{position:relative}.mx-auto{margin-inline:auto}.h-full{height:100%}.overflow-x-auto{overflow-x:auto}.overflow-y-auto{overflow-y:auto}.opacity-0{opacity:0}.group-hover\\:opacity-100{&:is(:where(.group):hover *){@media (hover:hover){opacity:100%}}}";document.head.appendChild(s);}
var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });

// external-global:@/api/plugin
var m = window.WebArcadeAPI;
var createPlugin = m.createPlugin;
var usePluginAPI = m.usePluginAPI;
var viewportTypes = m.viewportTypes;
var api = m.api;
var BRIDGE_API = m.BRIDGE_API;
var WEBARCADE_WS = m.WEBARCADE_WS;

// ../node_modules/@tabler/icons-solidjs/dist/esm/defaultAttributes.js
var defaultAttributes = {
  outline: {
    xmlns: "http://www.w3.org/2000/svg",
    width: 24,
    height: 24,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    "stroke-width": 2,
    "stroke-linecap": "round",
    "stroke-linejoin": "round"
  },
  filled: {
    xmlns: "http://www.w3.org/2000/svg",
    width: 24,
    height: 24,
    viewBox: "0 0 24 24",
    fill: "currentColor",
    stroke: "none"
  }
};

// external-global:solid-js
var m2 = window.SolidJS;
var createSignal = m2.createSignal;
var createEffect = m2.createEffect;
var createMemo = m2.createMemo;
var createRoot = m2.createRoot;
var createContext = m2.createContext;
var useContext = m2.useContext;
var onMount = m2.onMount;
var onCleanup = m2.onCleanup;
var onError = m2.onError;
var untrack = m2.untrack;
var batch = m2.batch;
var on = m2.on;
var createDeferred = m2.createDeferred;
var createRenderEffect = m2.createRenderEffect;
var createComputed = m2.createComputed;
var createReaction = m2.createReaction;
var createSelector = m2.createSelector;
var observable = m2.observable;
var from = m2.from;
var mapArray = m2.mapArray;
var indexArray = m2.indexArray;
var Show = m2.Show;
var For = m2.For;
var Switch = m2.Switch;
var Match = m2.Match;
var Index = m2.Index;
var ErrorBoundary = m2.ErrorBoundary;
var Suspense = m2.Suspense;
var SuspenseList = m2.SuspenseList;
var children = m2.children;
var lazy = m2.lazy;
var createResource = m2.createResource;
var createUniqueId = m2.createUniqueId;
var splitProps = m2.splitProps;
var mergeProps = m2.mergeProps;
var getOwner = m2.getOwner;
var runWithOwner = m2.runWithOwner;
var DEV = m2.DEV;
var enableScheduling = m2.enableScheduling;
var enableExternalSource = m2.enableExternalSource;

// external-global:solid-js/web
var m3 = window.SolidJSWeb;
var render = m3.render;
var hydrate = m3.hydrate;
var renderToString = m3.renderToString;
var renderToStream = m3.renderToStream;
var isServer = m3.isServer;
var Portal = m3.Portal;
var Dynamic = m3.Dynamic;
var template = m3.template;
var insert = m3.insert;
var createComponent = m3.createComponent;
var memo = m3.memo;
var effect = m3.effect;
var className = m3.className;
var classList = m3.classList;
var style = m3.style;
var spread = m3.spread;
var assign = m3.assign;
var setAttribute = m3.setAttribute;
var setAttributeNS = m3.setAttributeNS;
var addEventListener = m3.addEventListener;
var delegateEvents = m3.delegateEvents;
var clearDelegatedEvents = m3.clearDelegatedEvents;
var setProperty = m3.setProperty;
var getNextElement = m3.getNextElement;
var getNextMatch = m3.getNextMatch;
var getNextMarker = m3.getNextMarker;
var runHydrationEvents = m3.runHydrationEvents;
var getHydrationKey = m3.getHydrationKey;
var Assets = m3.Assets;
var HydrationScript = m3.HydrationScript;
var NoHydration = m3.NoHydration;
var Hydration = m3.Hydration;
var ssr = m3.ssr;
var ssrClassList = m3.ssrClassList;
var ssrStyle = m3.ssrStyle;
var ssrSpread = m3.ssrSpread;
var ssrElement = m3.ssrElement;
var escape = m3.escape;
var resolveSSRNode = m3.resolveSSRNode;
var use = m3.use;
var dynamicProperty = m3.dynamicProperty;
var SVGElements = m3.SVGElements;
var setStyleProperty = m3.setStyleProperty;

// ../node_modules/solid-js/h/dist/h.js
var $ELEMENT = Symbol("hyper-element");
function createHyperScript(r) {
  function h2() {
    let args = [].slice.call(arguments), e, classes = [], multiExpression = false;
    while (Array.isArray(args[0])) args = args[0];
    if (args[0][$ELEMENT]) args.unshift(h2.Fragment);
    typeof args[0] === "string" && detectMultiExpression(args);
    const ret = /* @__PURE__ */ __name(() => {
      while (args.length) item(args.shift());
      if (e instanceof Element && classes.length) e.classList.add(...classes);
      return e;
    }, "ret");
    ret[$ELEMENT] = true;
    return ret;
    function item(l) {
      const type = typeof l;
      if (l == null) ;
      else if ("string" === type) {
        if (!e) parseClass(l);
        else e.appendChild(document.createTextNode(l));
      } else if ("number" === type || "boolean" === type || "bigint" === type || "symbol" === type || l instanceof Date || l instanceof RegExp) {
        e.appendChild(document.createTextNode(l.toString()));
      } else if (Array.isArray(l)) {
        for (let i = 0; i < l.length; i++) item(l[i]);
      } else if (l instanceof Element) {
        r.insert(e, l, multiExpression ? null : void 0);
      } else if ("object" === type) {
        let dynamic = false;
        const d = Object.getOwnPropertyDescriptors(l);
        for (const k in d) {
          if (k === "class" && classes.length !== 0) {
            const fixedClasses = classes.join(" "), value = typeof d["class"].value === "function" ? () => fixedClasses + " " + d["class"].value() : fixedClasses + " " + l["class"];
            Object.defineProperty(l, "class", {
              ...d[k],
              value
            });
            classes = [];
          }
          if (k !== "ref" && k.slice(0, 2) !== "on" && typeof d[k].value === "function") {
            r.dynamicProperty(l, k);
            dynamic = true;
          } else if (d[k].get) dynamic = true;
        }
        dynamic ? r.spread(e, l, e instanceof SVGElement, !!args.length) : r.assign(e, l, e instanceof SVGElement, !!args.length);
      } else if ("function" === type) {
        if (!e) {
          let props, next = args[0];
          if (next == null || typeof next === "object" && !Array.isArray(next) && !(next instanceof Element)) props = args.shift();
          props || (props = {});
          if (args.length) {
            props.children = args.length > 1 ? args : args[0];
          }
          const d = Object.getOwnPropertyDescriptors(props);
          for (const k in d) {
            if (Array.isArray(d[k].value)) {
              const list = d[k].value;
              props[k] = () => {
                for (let i = 0; i < list.length; i++) {
                  while (list[i][$ELEMENT]) list[i] = list[i]();
                }
                return list;
              };
              r.dynamicProperty(props, k);
            } else if (typeof d[k].value === "function" && !d[k].value.length) r.dynamicProperty(props, k);
          }
          e = r.createComponent(l, props);
          args = [];
        } else {
          while (l[$ELEMENT]) l = l();
          r.insert(e, l, multiExpression ? null : void 0);
        }
      }
    }
    __name(item, "item");
    function parseClass(string) {
      const m4 = string.split(/([\.#]?[^\s#.]+)/);
      if (/^\.|#/.test(m4[1])) e = document.createElement("div");
      for (let i = 0; i < m4.length; i++) {
        const v = m4[i], s = v.substring(1, v.length);
        if (!v) continue;
        if (!e) e = r.SVGElements.has(v) ? document.createElementNS("http://www.w3.org/2000/svg", v) : document.createElement(v);
        else if (v[0] === ".") classes.push(s);
        else if (v[0] === "#") e.setAttribute("id", s);
      }
    }
    __name(parseClass, "parseClass");
    function detectMultiExpression(list) {
      for (let i = 1; i < list.length; i++) {
        if (typeof list[i] === "function") {
          multiExpression = true;
          return;
        } else if (Array.isArray(list[i])) {
          detectMultiExpression(list[i]);
        }
      }
    }
    __name(detectMultiExpression, "detectMultiExpression");
  }
  __name(h2, "h");
  h2.Fragment = (props) => props.children;
  return h2;
}
__name(createHyperScript, "createHyperScript");
var h = createHyperScript({
  spread,
  assign,
  insert,
  createComponent,
  dynamicProperty,
  SVGElements
});

// ../node_modules/@tabler/icons-solidjs/dist/esm/createSolidComponent.js
var createSolidComponent = /* @__PURE__ */ __name((type, iconName, iconNamePascal, iconNode) => {
  const Component = /* @__PURE__ */ __name((props) => {
    const [localProps, rest] = splitProps(props, ["color", "size", "stroke", "title", "children", "class"]), attributes = defaultAttributes[type];
    const svgProps = {
      ...attributes,
      width: /* @__PURE__ */ __name(() => localProps.size != null ? localProps.size : attributes.width, "width"),
      height: /* @__PURE__ */ __name(() => localProps.size != null ? localProps.size : attributes.height, "height"),
      title: /* @__PURE__ */ __name(() => localProps.title != null ? localProps.title : void 0, "title"),
      ...type === "filled" ? {
        fill: /* @__PURE__ */ __name(() => localProps.color != null ? localProps.color : "currentColor", "fill")
      } : {
        stroke: /* @__PURE__ */ __name(() => localProps.color != null ? localProps.color : "currentColor", "stroke"),
        "stroke-width": /* @__PURE__ */ __name(() => localProps.stroke != null ? localProps.stroke : attributes["stroke-width"], "stroke-width")
      },
      class: /* @__PURE__ */ __name(() => `tabler-icon tabler-icon-${iconName} ${localProps.class != null ? localProps.class : ""}`, "class")
    };
    return h(
      "svg",
      [svgProps, rest],
      [
        localProps.title && h("title", {}, localProps.title),
        ...iconNode.map(([tag, attrs]) => h(tag, attrs)),
        localProps.children
      ]
    );
  }, "Component");
  Component.displayName = `${iconNamePascal}`;
  return Component;
}, "createSolidComponent");

// ../node_modules/@tabler/icons-solidjs/dist/esm/icons/IconBook.js
var IconBook = createSolidComponent("outline", "book", "Book", [["path", { "d": "M3 19a9 9 0 0 1 9 0a9 9 0 0 1 9 0" }], ["path", { "d": "M3 6a9 9 0 0 1 9 0a9 9 0 0 1 9 0" }], ["path", { "d": "M3 6l0 13" }], ["path", { "d": "M12 6l0 13" }], ["path", { "d": "M21 6l0 13" }]]);

// ../node_modules/@tabler/icons-solidjs/dist/esm/icons/IconCheck.js
var IconCheck = createSolidComponent("outline", "check", "Check", [["path", { "d": "M5 12l5 5l10 -10" }]]);

// ../node_modules/@tabler/icons-solidjs/dist/esm/icons/IconCopy.js
var IconCopy = createSolidComponent("outline", "copy", "Copy", [["path", { "d": "M7 7m0 2.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667z" }], ["path", { "d": "M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1" }]]);

// ../plugins/demo/viewport.jsx
var _tmpl$ = /* @__PURE__ */ template(`<div class="relative group my-4"><pre class="bg-base-300 rounded-lg p-4 overflow-x-auto text-sm font-mono text-base-content"><code></code></pre><button class="absolute top-2 right-2 btn btn-xs btn-ghost opacity-0 group-hover:opacity-100">`);
var _tmpl$2 = /* @__PURE__ */ template(`<div class="h-full overflow-y-auto bg-base-200"><div class="max-w-3xl mx-auto p-8"><h1 class="text-3xl font-bold mb-6">Quick Start</h1><p class="text-base-content/70 mb-6">Create a plugin in under a minute.</p><div class=space-y-6><div><h2 class="text-lg font-semibold mb-2">1. Create Plugin</h2></div><div><h2 class="text-lg font-semibold mb-2">2. Edit index.jsx</h2></div><div><h2 class="text-lg font-semibold mb-2">3. Build & Run`);
function Code(props) {
  const [copied, setCopied] = createSignal(false);
  const copy = /* @__PURE__ */ __name(() => {
    navigator.clipboard.writeText(props.children);
    setCopied(true);
    setTimeout(() => setCopied(false), 2e3);
  }, "copy");
  return (() => {
    var _el$ = _tmpl$(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$2.nextSibling;
    insert(_el$3, () => props.children.trim());
    _el$4.$$click = copy;
    insert(_el$4, (() => {
      var _c$ = memo(() => !!copied());
      return () => _c$() ? createComponent(IconCheck, {
        "class": "w-3 h-3"
      }) : createComponent(IconCopy, {
        "class": "w-3 h-3"
      });
    })());
    return _el$;
  })();
}
__name(Code, "Code");
function GuideViewport() {
  return (() => {
    var _el$5 = _tmpl$2(), _el$6 = _el$5.firstChild, _el$7 = _el$6.firstChild, _el$8 = _el$7.nextSibling, _el$9 = _el$8.nextSibling, _el$0 = _el$9.firstChild, _el$1 = _el$0.firstChild, _el$10 = _el$0.nextSibling, _el$11 = _el$10.firstChild, _el$12 = _el$10.nextSibling, _el$13 = _el$12.firstChild;
    insert(_el$0, createComponent(Code, {
      lang: "bash",
      children: `bun run plugin:new my-plugin`
    }), null);
    insert(_el$10, createComponent(Code, {
      lang: "jsx",
      children: `import { createPlugin } from '@/api/plugin';

export default createPlugin({
  id: 'my-plugin',
  name: 'My Plugin',
  version: '1.0.0',

  async onStart(api) {
    api.viewport('main', {
      label: 'My Plugin',
      component: () => <div class="p-4">Hello World</div>
    });
    api.open('main');
  }
});`
    }), null);
    insert(_el$12, createComponent(Code, {
      lang: "bash",
      children: `bun run plugin:build my-plugin`
    }), null);
    return _el$5;
  })();
}
__name(GuideViewport, "GuideViewport");
delegateEvents(["click"]);

// ../plugins/demo/index.jsx
var index_default = createPlugin({
  id: "demo",
  name: "Plugin Guide",
  version: "1.0.0",
  description: "Plugin development documentation",
  author: "WebArcade Team",
  async onStart(api2) {
    api2.viewport("guide", {
      label: "Quick Start",
      icon: IconBook,
      component: GuideViewport,
      onActivate: /* @__PURE__ */ __name((api3) => {
        api3.showTabs(true);
      }, "onActivate")
    });
    await api2.setWindowSize(800, 700);
    await api2.centerWindow();
    api2.open("guide");
  }
});
export {
  index_default as default
};
/*! Bundled license information:

@tabler/icons-solidjs/dist/esm/defaultAttributes.js:
@tabler/icons-solidjs/dist/esm/createSolidComponent.js:
@tabler/icons-solidjs/dist/esm/icons/IconBook.js:
@tabler/icons-solidjs/dist/esm/icons/IconCheck.js:
@tabler/icons-solidjs/dist/esm/icons/IconCopy.js:
@tabler/icons-solidjs/dist/esm/tabler-icons-solidjs.js:
  (**
   * @license @tabler/icons-solidjs v3.34.1 - MIT
   *
   * This source code is licensed under the MIT license.
   * See the LICENSE file in the root directory of this source tree.
   *)
*/
