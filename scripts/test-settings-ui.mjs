// Dependency-free browser regression suite for the actual WebView settings document.
// Run: node scripts/test-settings-ui.mjs
// Open the printed localhost address. The mock bridge never writes application settings.
import http from 'node:http';
import { readFile } from 'node:fs/promises';

const settingsUrl = new URL('../ui/settings.html', import.meta.url);
const languageTags = ['en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it'];
const catalogs = Object.fromEntries(await Promise.all(languageTags.map(async language =>
  [language, JSON.parse(await readFile(new URL('../locales/' + language + '.json', import.meta.url), 'utf8'))])));
const port = Number(process.env.PECOFENCE_UI_TEST_PORT || 43190);
const fixture = {
  type: 'state',
  locale: 'zh-CN', translations: {},
  settings: {
    language: 'zh-CN',
    autostart: false, hideRealIcons: false, keepIconsHidden: false, hideInboxWhenIconsShown: false, offerNewFence: true, featureTips: true, autoCheckUpdates: true, perDesktopFences: false, quickHide: { enabled: true },
    showDesktop: 'keepVisible', theme: 'dark', themeStyle: 'fluent', labelColor: 'auto', titleAlign: 'left', tabBar: 'top', backdrop: 'acrylic', iconSize: 48,
    icons: { tintRgb: null, tintStrength: 0.6, chameleon: false, shortcutArrows: true },
    rollUp: { hoverPeek: true, clickToExpand: false, pushNeighbors: true, titleOnHover: false, hideInactiveScrollbar: false, switchTabOnHover: false, hideCountAndChevron: false },
    snapping: { enabled: true, gapPx: 8, sizeToCells: false, guideLines: true }, peek: { enabled: true, dim: true, hotkey: 'ctrlAltSpace', doubleCtrl: false },
    tools: { enabled: false, clockSeconds: false, dialogJump: false, everything: false },
  },
  rules: {
    keepUpdated: true, defaultTarget: 'inbox', unmatched: 'defaultTarget',
    list: [
      { id: 'rule-a', name: 'Documents', enabled: true, target: { fence: 'fence-a' }, allOf: [{ cond: 'ext', value: ['txt'] }], priorityClass: 'type' },
      { id: 'rule-b', name: 'Images', enabled: true, target: { fence: 'fence-b' }, allOf: [{ cond: 'ext', value: ['png'] }], priorityClass: 'type' },
    ],
  },
  fences: [
    { id: 'fence-a', title: 'Documents', kind: 'virtual', host: null, virtualDesktops: ['{B}'], iconSize: 48, spacing: 'normal', spacingPx: 0, labelLines: 2, layout: 'icons', iconAlign: 'left', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', titleOnHover: 'default', openFoldersInside: false, portal: null },
    { id: 'fence-b', title: 'Images', kind: 'virtual', host: 'Documents', iconSize: 96, spacing: 'loose', spacingPx: 16, labelLines: 1, layout: 'list', autoHeight: true, locked: false, excludeFromQuickHide: false, opacity: 'transparent', tint: null, tabColor: 'E74856', titleColor: 'theme', titleSize: 'normal', titleOnHover: 'default', portal: null },
    { id: 'inbox', title: 'Desktop', kind: 'inbox', host: null, iconSize: 48, spacing: 'normal', spacingPx: 0, labelLines: 3, layout: 'icons', autoHeight: false, locked: true, excludeFromQuickHide: true, opacity: 'solid', tint: '0078D4', tintTitleOnly: true, titleColor: 'tint', titleSize: 'large', titleOnHover: 'hover', hideWhenEmpty: true, desktopLayout: true, openFoldersInside: true, expandUpward: true, portal: null },
    { id: 'portal', title: 'Portal', kind: 'portal', host: null, iconSize: 32, spacing: 'compact', spacingPx: -12, labelLines: 0, layout: 'icons', iconAlign: 'center', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'clear', tint: '123456', titleColor: 'ABCDEF', titleSize: 'small', titleOnHover: 'always', portal: { navigate: true, titleIcon: false } },
  ],
  tintPalette: [{ name: '红', hex: 'E74856' }, { name: '蓝', hex: '0078D4' }, { name: '灰', hex: '7A7574' }],
  snapshots: [{ id: 'snapshot-a', name: 'Before changes', date: '2026-09-09', fenceCount: 4 }],
  backups: [{ name: '2026-09-09', path: 'C:\\PecoFence\\backups\\2026-09-09.json' }],
  monitors: [{ id: 'one', label: 'Display 1' }, { id: 'two', label: 'Display 2' }],
  virtualDesktops: [{ id: '{A}', name: 'Work', current: true }, { id: '{B}', name: 'Games', current: false }],
  desktopIconsHidden: false,
  version: 'test', configPath: 'C:\\PecoFence\\config.json', memoryMb: 28, itemCount: 12,
  themeMode: 'dark', accent: '#60CDFF',
  customThemes: ['Frosted Glass', 'Warm'], customThemeError: null, styleInUse: 'fluent',
  peekHotkey: {
    chosen: { mods: 3, vk: 32, keys: ['Ctrl', 'Alt', '空格'], label: 'Ctrl+Alt+空格' },
    active: { mods: 3, vk: 32, keys: ['Ctrl', 'Alt', '空格'], label: 'Ctrl+Alt+空格' },
    presets: [
      { mods: 3, vk: 32, keys: ['Ctrl', 'Alt', '空格'], label: 'Ctrl+Alt+空格' },
      { mods: 9, vk: 32, keys: ['Win', 'Alt', '空格'], label: 'Win+Alt+空格' },
      { mods: 6, vk: 32, keys: ['Ctrl', 'Shift', '空格'], label: 'Ctrl+Shift+空格' },
    ],
  },
  windowsDesktopHotkey: { chosen: null, registered: false },
  everything: { state: 'missing' },
};

function bridge() {
  const state = JSON.parse(document.getElementById('fixture').textContent);
  const catalogs = JSON.parse(document.getElementById('fixture-catalogs').textContent);
  window.testMessages = [];
  let receive;
  window.testSummary = (itemCount) => receive({ data: { type: 'workspaceSummary', fenceCount: state.fences.length, itemCount } });
  window.testRefresh = () => receive({ data: structuredClone(state) });
  window.testUpdate = (update) => receive({ data: { type: 'update', update } });
  window.testState = state;
  // The real page is a data: URL, not a secure context: secure-only APIs are missing there.
  delete Crypto.prototype.randomUUID;
  window.chrome = {
    webview: {
      addEventListener(_type, handler) { receive = handler; },
      postMessage(message) {
        window.testMessages.push(structuredClone(message));
        if (message.type === 'patchSettings') {
          state.settings = structuredClone(message.settings);
          state.locale = state.settings.language === 'system' ? 'en' : state.settings.language;
          state.translations = catalogs[state.locale] || {};
          state.desktopIconsHidden = state.settings.hideRealIcons;
          state.themeMode = message.settings.theme === 'light' ? 'light' : 'dark';
          // The native host also switches to the corresponding system accent shade.
          state.accent = state.themeMode === 'light' ? '#005FB8' : '#60CDFF';
          // A custom theme's base decides the style in use; a file not listed here does not load.
          const bases = { 'Frosted Glass': 'liquidGlass', Warm: 'fluent' }, custom = state.settings.customTheme;
          state.customThemeError = custom && !bases[custom] ? 'unknown key "blurr"' : null;
          state.styleInUse = (custom && bases[custom]) || state.settings.themeStyle || 'fluent';
        }
        if (message.name === 'repairIcons' || message.name === 'hideDesktopIcons') {
          state.settings.hideRealIcons = message.name === 'hideDesktopIcons';
          state.desktopIconsHidden = state.settings.hideRealIcons;
        }
        if (message.type === 'setRules') {
          // Mirror the host: a list the core cannot read (idle days beyond its u32, or any list
          // while testRefuseRules is set) is not saved; the stored rules go back with an error.
          const readable = !window.testRefuseRules && message.rules.list.every(r => r.allOf.every(c => c.cond !== 'idleDays' || c.value.min <= 4294967295));
          if (readable) state.rules = structuredClone(message.rules);
          else setTimeout(() => { window.testRefresh(); receive({ data: { type: 'toast', text: '没能保存这次改动，规则已恢复原样', error: true } }); }, 0);
        }
        // The host's pinyin of the asked characters (a few, enough for the tests).
        if (message.name === 'pinyin') {
          const table = { 隐: 'yin', 藏: 'cang', 自: 'zi', 动: 'dong', 滚: 'gun', 条: 'tiao', 开: 'kai', 机: 'ji', 启: 'qi', 标: 'biao', 题: 'ti', 栏: 'lan' };
          window.testPinyinAsked = message.chars;
          const map = Object.fromEntries([...message.chars].filter(c => table[c]).map(c => [c, table[c]]));
          setTimeout(() => receive({ data: { type: 'pinyin', map } }), 0);
        }
        if (message.name === 'setPeekHotkey') {
          const names = [[8, 'Win'], [2, 'Ctrl'], [1, 'Alt'], [4, 'Shift']].filter(([bit]) => message.mods & bit).map(([, n]) => n);
          const keys = [...names, message.vk === 32 ? '空格' : String.fromCharCode(message.vk)];
          const combo = { mods: message.mods, vk: message.vk, keys, label: keys.join('+') };
          // Win+Shift+Space (and testTaken, as "mods:vk") stand for a combination Windows holds.
          const taken = (message.mods === 12 && message.vk === 32) || window.testTaken === `${message.mods}:${message.vk}`;
          const text = !(message.mods & 11) ? '快捷键需要包含 Ctrl、Alt 或 Win'
            : taken ? combo.label + ' 已被 Windows 或其他程序占用，请换一个' : null;
          if (!text) {
            state.peekHotkey.chosen = state.peekHotkey.active = combo;
            state.settings.peek.customHotkey = message.mods === 3 && message.vk === 32 ? null : combo.label;
            state.settings.peek.doubleCtrl = false;
            setTimeout(window.testRefresh, 0);
          }
          setTimeout(() => receive({ data: { type: 'peekHotkeyResult', ok: !text, text } }), 0);
        }
        if (message.name === 'setWindowsDesktopHotkey') {
          // No key = 清除. Peek's Ctrl+Alt+Space is refused, the rest is saved as given.
          const names = [[8, 'Win'], [2, 'Ctrl'], [1, 'Alt'], [4, 'Shift']].filter(([bit]) => message.mods & bit).map(([, n]) => n);
          const keys = message.vk == null ? null : [...names, message.vk === 32 ? '空格' : String.fromCharCode(message.vk)];
          const combo = keys && { mods: message.mods, vk: message.vk, keys, label: keys.join('+') };
          const text = combo && !(message.mods & 11) ? '快捷键需要包含 Ctrl、Alt 或 Win'
            : combo && message.mods === 3 && message.vk === 32 ? combo.label + ' 已经是“速览栅栏”的快捷键，请换一个' : null;
          if (!text) {
            state.windowsDesktopHotkey = { chosen: combo, registered: !!combo };
            state.settings.windowsDesktopHotkey = combo ? combo.label : null;
            setTimeout(window.testRefresh, 0);
          }
          setTimeout(() => receive({ data: { type: 'windowsDesktopHotkeyResult', ok: !text, text } }), 0);
        }
        if (message.name === 'addTemplate' && !state.rules.list.some(r => r.template === message.template)) {
          // Mirror the host: a new fence plus its rule at the top of the list.
          const id = 'tpl-' + message.template;
          state.fences.push({ id, title: message.template, kind: 'virtual', host: null, iconSize: 48, spacing: 'normal', spacingPx: 0, labelLines: 2, layout: 'icons', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', titleOnHover: 'default', portal: null });
          const idle = message.template === 'cleanup';
          const allOf = idle ? [{ cond: 'type', value: ['installers', 'archives'] }, { cond: 'idleDays', value: { min: 30 } }] : [{ cond: 'type', value: [message.template] }];
          const at = idle ? 0 : state.rules.list.findIndex(r => !r.allOf.some(c => c.cond === 'idleDays'));
          // 游戏 is one shortcut-target rule per kind of game shortcut.
          const games = message.template === 'games' && [['Steam 游戏', 'steam://rungameid/'], ['Steam 游戏', '\\steamapps\\common\\'], ['Epic 游戏', 'com.epicgames.launcher://apps/'], ['Epic 游戏', '\\Epic Games\\'], ['Xbox 游戏', '\\XboxGames\\']];
          const rules = games ? games.map(([name, value], i) => ({ id: 'rule-' + id + '-' + i, name, enabled: true, target: { fence: id }, allOf: [{ cond: 'shortcutTarget', value: { op: 'contains', value } }], priorityClass: 'target', template: 'games' }))
            : [{ id: 'rule-' + id, name: message.template, enabled: true, target: { fence: id }, allOf, priorityClass: 'type', template: message.template }];
          state.rules.list.splice(at < 0 ? state.rules.list.length : at, 0, ...rules);
        }
        if (message.type === 'setFence') {
          // Mirror the host: a portal flag lands in `portal`, everything else on the fence.
          const fence = state.fences.find(f => f.id === message.id);
          if (message.prop === 'portalNavigate') fence.portal.navigate = message.value;
          else if (message.prop === 'portalTitleIcon') (fence.portal || fence.smart).titleIcon = message.value;
          // 在哪个虚拟桌面显示: `all` or one desktop's id, kept as the list of ids.
          else if (message.prop === 'virtualDesktops') fence.virtualDesktops = message.value === 'all' ? [] : [message.value];
          else if (message.prop !== 'dockTop') fence[message.prop] = message.value;
          if (message.prop === 'spacingPx') fence.spacing = message.value <= -8 ? 'compact' : message.value >= 12 ? 'loose' : 'normal';
        }
        window.testShowFence = (id) => receive({ data: { type: 'showFence', id } });
        window.testShowFeedback = (crash) => receive({ data: { type: 'showFeedback', crash } });
        if (message.name === 'feedbackInfo') {
          // Mirror the host: what may be sent besides the user's text.
          // The host sizes the log for the message and says for how long a one.
          setTimeout(() => receive({ data: { type: 'feedbackInfo', url: 'https://feedback.test/feedback', app: '0.1.4', language: state.locale,
            crashed: !!state.crashed, system: { app: 'PecoFence 0.1.4 (ZIP / winget)', windows: 'build 26200' },
            log: 'INFO pecofence::app: config loaded fences=4', messageBytes: message.messageBytes ?? 0 } }), 0);
        }
        // Mirror the host: the system colour dialog answers with the colour picked there.
        if (message.name === 'pickColor') {
          state.pickedColors = { ...state.pickedColors, [message.target]: '123456' };
          setTimeout(() => receive({ data: { type: 'pickedColor', target: message.target, hex: '123456' } }), 0);
        }
        if (message.name === 'deleteSnapshot') state.snapshots = state.snapshots.filter(s => s.id !== message.id);
        if (['ready', 'patchSettings', 'setRules', 'setFence'].includes(message.type) || message.type === 'action') {
          setTimeout(window.testRefresh, 0);
        }
      },
    },
  };
}

// 智能栅栏编辑器: what the host sends the editor's page (`editor`), for a new fence, for 最近 7 天下载
// and for a search that uses every kind of row.
const editorDl = 'C:\\Users\\me\\Downloads';
const editorTemplates = [
  { key: 'downloads', title: '最近 7 天下载', spec: { template: 'downloads', folders: [editorDl], subfolders: true, allOf: [{ cond: 'filesOnly', value: null }], withinDays: 7, limit: 200 } },
  { key: 'screenshots', title: '最近一周的截图', spec: null },
  { key: 'large', title: '大于 1 GB 的文件', spec: { template: 'large', subfolders: true, allOf: [{ cond: 'filesOnly', value: null }, { cond: 'sizeMb', value: { min: 1024, max: null } }], limit: 200 } },
  { key: 'psd', title: '所有 .psd 文件', spec: { template: 'psd', subfolders: true, allOf: [{ cond: 'filesOnly', value: null }, { cond: 'ext', value: ['.psd'] }], limit: 200 } },
  { key: 'recent', title: '最近打开的文件', spec: { template: 'recent', folders: ['C:\\Users\\me\\Recent'], subfolders: false, allOf: [{ cond: 'filesOnly', value: null }, { cond: 'ext', value: ['.lnk'] }], limit: 40 } },
];
const editorFixtures = {
  // Saved by an older build or the CLI: plain files, a dotted extension, an origin condition.
  odd: { type: 'editor', fence: 'smart-odd', isNew: false, title: 'Odd', templates: editorTemplates, everything: { state: 'running' },
    spec: { folders: ['D:\\Odd'], subfolders: true, allOf: [{ cond: 'filesOnly', value: null, plain: true }, { cond: 'ext', value: ['.psd'] }, { cond: 'origin', value: 'publicDesktop' }], limit: 200 } },
  new: { type: 'editor', fence: null, isNew: true, title: '', spec: { subfolders: true, limit: 200 }, templates: editorTemplates, everything: { state: 'running' } },
  dl: { type: 'editor', fence: 'smart-dl', isNew: false, title: '最近 7 天下载', spec: editorTemplates[0].spec, templates: editorTemplates, everything: { state: 'running' } },
  every: { type: 'editor', fence: 'smart-all', isNew: false, title: 'Every row', templates: editorTemplates, everything: { state: 'running' },
    spec: { folders: ['D:\\Work', 'E:\\Archive'], subfolders: false, withinDays: 30, name: 'bg', everything: 'invoice|receipt', limit: 500,
      allOf: [
        { cond: 'type', value: ['images', 'music'] }, { cond: 'ext', value: ['pdf', 'tar.gz'] }, { cond: 'name', value: { op: 'startsWith', value: 'IMG_' } },
        { cond: 'sizeMb', value: { min: 1024, max: null } }, { cond: 'sizeMb', value: { min: 1.5, max: 20 } }, { cond: 'createdTime', value: { from_min: 1320, to_min: 360 } },
        { cond: 'createdWeekday', value: [5, 6] }, { cond: 'shortcutTarget', value: { op: 'contains', value: 'steam' } }, { cond: 'idleDays', value: { min: 45 } },
        { cond: 'glob', value: '*.bak' },
      ] } },
};

function editorBridge() {
  const state = JSON.parse(document.getElementById('fixture').textContent);
  window.testEditor = JSON.parse(document.getElementById('fixture-editor').textContent);
  window.testMessages = [];
  let receive;
  window.testReceive = data => receive({ data: structuredClone(data) });
  window.testState = state;
  delete Crypto.prototype.randomUUID;
  window.chrome = {
    webview: {
      addEventListener(_type, handler) { receive = handler; },
      postMessage(message) {
        window.testMessages.push(structuredClone(message));
        // Mirror the host: the page's language and theme first, then the fence.
        if (message.type === 'ready') setTimeout(() => { receive({ data: structuredClone(state) }); receive({ data: structuredClone(window.testEditor) }); }, 0);
        // The count: testCount(spec) gives the answer ({source, n, approx}); testHoldCount keeps
        // the answers back for the test to send (out of order, say).
        if (message.type === 'editorCount') {
          const answer = { type: 'editorCount', seq: message.seq, everything: { state: 'running' }, ...(window.testCount ? window.testCount(message.spec) : { source: 'everything', n: 37, approx: false }) };
          if (window.testHoldCount) window.testHeld = [...(window.testHeld || []), answer];
          else setTimeout(() => receive({ data: answer }), 0);
        }
        // The folder picker answers with testPickPath (null: cancelled).
        if (message.type === 'editorPickFolder') setTimeout(() => receive({ data: { type: 'editorFolder', row: message.row, path: window.testPickPath === undefined ? 'D:\\Picked' : window.testPickPath } }), 0);
      },
    },
  };
}

async function runTests() {
  const frame = document.querySelector('iframe');
  const results = document.getElementById('results');
  let passed = 0, failed = 0;
  const settle = () => new Promise(resolve => setTimeout(resolve, 40));
  const reset = async () => {
    await new Promise(resolve => { frame.onload = resolve; frame.src = '/settings'; });
    await settle();
    return frame.contentDocument;
  };
  const assert = (condition, message) => { if (!condition) throw new Error(message); };
  const test = async (name, run) => {
    const li = document.createElement('li');
    try { await run(); li.textContent = 'PASS — ' + name; li.className = 'pass'; passed++; }
    catch (error) { li.textContent = 'FAIL — ' + name + ': ' + error.message; li.className = 'fail'; failed++; }
    results.append(li);
  };
  const change = (doc, id, value) => {
    const el = doc.getElementById(id);
    el.value = value;
    el.dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true }));
  };
  // The new-rule form is a collapsed disclosure while rules exist; form tests open it first.
  const rulesPage = doc => { doc.querySelector('[data-page=rules]').click(); doc.getElementById('nrPanel').open = true; };
  const styleRadio = (doc, value) => doc.querySelector(`input[name=themeStyle][value=${value}]`);
  const visible = el => frame.contentWindow.getComputedStyle(el).visibility === 'visible';
  const rgbHex = rgb => rgb ? rgb.map(x => x.toString(16).padStart(2, '0')).join('').toUpperCase() : '';
  const ruleCount = doc => doc.querySelectorAll('#ruleList [data-row-id]').length;
  const add = async doc => { doc.getElementById('nrAdd').click(); await settle(); };
  const errorShown = doc => doc.getElementById('toast').classList.contains('error');
  const current = () => frame.contentWindow.testState;
  let doc = await reset();

  await test('All settings switches and selects have accessible names', async () => {
    for (const el of doc.querySelectorAll('[data-bind], [data-fence], #iconTint, #iconTintStrength, #fenceSel')) {
      const label = el.getAttribute('aria-label') || doc.getElementById(el.getAttribute('aria-labelledby'))?.textContent;
      assert(label?.trim(), 'Unnamed control: ' + (el.id || el.dataset.bind));
    }
  });
  await test('Notification icons render glyphs rather than escaped code text', async () => {
    const toast = doc.getElementById('toast');
    const icon = toast.querySelector('.ic');
    for (const [error, glyph] of [[false, 0xe73e], [true, 0xe783]]) {
      toast.classList.toggle('error', error);
      const content = frame.contentWindow.getComputedStyle(icon, '::before').content;
      assert(content.includes(String.fromCodePoint(glyph)) && !content.includes('\\\\'), 'Notification displays escape text instead of its icon');
    }
    toast.classList.remove('error');
  });
  await test('Single-monitor layout does not leave an empty multi-monitor section', async () => {
    const monitors = current().monitors;
    current().monitors = monitors.slice(0, 1);
    frame.contentWindow.testRefresh();
    assert(doc.getElementById('swapHeading').style.display === 'none' && doc.getElementById('swapCard').style.display === 'none', 'Empty display heading remains visible');
    current().monitors = monitors;
    frame.contentWindow.testRefresh();
    assert(doc.getElementById('swapHeading').style.display !== 'none' && doc.getElementById('swapCard').style.display !== 'none', 'Multi-monitor controls did not return');
  });
  await test('Every page fits the minimum settings window width', async () => {
    frame.style.width = '704px';
    for (const page of ['general', 'fences', 'rules', 'tools', 'layout', 'about']) {
      doc.querySelector(`[data-page=${page}]`).click();
      await settle();
      const main = doc.querySelector('main');
      assert(main.scrollWidth <= main.clientWidth + 1, page + ' overflows horizontally');
      for (const title of doc.querySelectorAll('.page.on .card .t')) {
        assert(title.getBoundingClientRect().height <= 60, page + ' crushes a card title into vertical text');
      }
    }
  });
  await test('Tools page: one switch, a new note, one clock, the examples and the guide', async () => {
    doc = await reset();
    doc.querySelector('[data-page=tools]').click();
    const page = doc.getElementById('page-tools');
    assert(page.querySelectorAll('.toggle').length === 1, 'The tools page has more than one switch');
    for (const gone of ['tools.notes', 'tools.clock', 'tools.scripts', 'tools.clockSeconds']) {
      assert(!doc.querySelector(`[data-bind="${gone}"]`), gone + ' still has a switch');
    }
    assert(!doc.getElementById('noteCount') && !doc.getElementById('newScriptExample'), 'The count line or 添加示例… is still there');
    const titles = [...page.querySelectorAll('.card.child .t')].map(t => t.textContent).join(' / ');
    assert(titles === '便签与待办 / 时钟 / 天气 / GitHub 仓库动态 / 电脑状态 / 用自己的脚本', 'Rows: ' + titles);
    const before = frame.contentWindow.testMessages.length;
    doc.querySelector('[data-bind="tools.enabled"]').click();
    const patch = frame.contentWindow.testMessages.slice(before).find(m => m.type === 'patchSettings');
    assert(patch && patch.settings.tools.enabled === true, 'The 小工具 switch not posted');
    const last = () => frame.contentWindow.testMessages.at(-1);
    doc.getElementById('newNote').click();
    assert(last().type === 'action' && last().name === 'newNote', 'New note not asked for: ' + JSON.stringify(last()));
    const clock = doc.getElementById('newClock');
    assert(!clock.disabled && clock.textContent === '添加', 'Clock button wrong without a clock: ' + clock.textContent);
    clock.click();
    assert(last().name === 'newClock', 'New clock not asked for');
    for (const [id, example] of [['addWeather', 'weather'], ['addGitHub', 'github'], ['addStatus', 'status']]) {
      doc.getElementById(id).click();
      assert(last().name === 'newScriptExample' && last().example === example, id + ' sent ' + JSON.stringify(last()));
    }
    doc.getElementById('newScript').click();
    assert(last().name === 'newScript', 'Own script not asked for');
    doc.getElementById('scriptGuide').click();
    assert(last().name === 'scriptGuide', 'Guide not asked for');
    // One clock at most: its button greys out while the desktop has one.
    current().fences.push({ ...current().fences[0], id: 'clock-a', title: 'Clock', tool: { kind: 'clock' } });
    current().fences.push({ ...current().fences[0], id: 'note-a', title: 'Note A', tool: { kind: 'note' } });
    frame.contentWindow.testRefresh();
    await settle();
    assert(clock.disabled && clock.textContent === '已在桌面上', 'Clock button not greyed with a clock: ' + clock.textContent);
    // A note is no rule target.
    rulesPage(doc);
    assert(![...doc.querySelectorAll('#nrTarget option')].some(o => o.value === 'note-a'), 'A note is offered as a rule target');
    current().fences.splice(-2);
    doc = await reset();
  });
  await test('文字大小 for notes and script fences, not for a clock or an ordinary fence', async () => {
    doc = await reset();
    current().fences.push({ ...current().fences[0], id: 'clock-a', title: 'Clock', tool: { kind: 'clock' }, textSize: 'normal' });
    current().fences.push({ ...current().fences[0], id: 'note-a', title: 'Note A', tool: { kind: 'note' }, textSize: 'large' });
    current().fences.push({ ...current().fences[0], id: 'script-a', title: 'Weather', tool: { kind: 'script' }, textSize: 'normal' });
    frame.contentWindow.testRefresh();
    await settle();
    doc.querySelector('[data-page=fences]').click();
    const card = doc.getElementById('fenceTextSize');
    change(doc, 'fenceSel', 'fence-a');
    assert(card.style.display === 'none', 'Text size offered for an ordinary fence');
    change(doc, 'fenceSel', 'clock-a');
    assert(card.style.display === 'none', 'Text size offered for a clock');
    change(doc, 'fenceSel', 'note-a');
    assert(card.style.display === '' && doc.getElementById('fenceTextSizeSel').value === 'large', 'Note text size not shown');
    const choices = [...doc.querySelectorAll('#fenceTextSizeSel option')].map(o => o.value + o.textContent).join();
    assert(choices === 'small小,normal标准,large大,extraLarge特大', 'Text size choices wrong: ' + choices);
    const before = frame.contentWindow.testMessages.length;
    change(doc, 'fenceTextSizeSel', 'extraLarge');
    const sent = frame.contentWindow.testMessages.slice(before).filter(m => m.type === 'setFence');
    assert(sent.length === 1 && sent[0].id === 'note-a' && sent[0].prop === 'textSize' && sent[0].value === 'extraLarge', 'Text size not posted: ' + JSON.stringify(sent));
    change(doc, 'fenceSel', 'script-a');
    assert(card.style.display === '' && doc.getElementById('fenceTextSizeSel').value === 'normal', 'Script text size not shown');
    current().fences.splice(-3);
    doc = await reset();
  });
  await test('「速览与搜索」 holds Peek, Everything and dialog jump, and their switches post', async () => {
    doc = await reset();
    const nav = [...doc.querySelectorAll('nav a[data-page]')].map(a => a.dataset.page).join(' ');
    assert(nav.startsWith('general peek '), 'The page is not right below 常规: ' + nav);
    const peek = doc.getElementById('page-peek');
    const heads = [...peek.querySelectorAll('h3')].map(h => h.textContent).join(' / ');
    assert(heads === '速览 / 搜索 / 文件对话框', 'Headings: ' + heads);
    for (const bind of ['peek.enabled', 'peek.dim', 'tools.everything', 'tools.dialogJump']) {
      assert(peek.querySelector(`[data-bind="${bind}"]`), bind + ' is not on 速览与搜索');
    }
    assert(peek.contains(doc.getElementById('dialogJumpDemo')) && peek.contains(doc.getElementById('peekHotkeyChoice')), 'The demo or the shortcut list is elsewhere');
    assert(!doc.querySelector('#page-general [data-bind="peek.enabled"]') && !doc.querySelector('#page-tools [data-bind="tools.dialogJump"]'), 'A switch is still on its old page');
    assert(!doc.querySelector('[data-bind="peek.doubleCtrl"]'), 'Double Ctrl still has its own switch');
    doc.querySelector('[data-page=peek]').click();
    for (const [path, read] of [['tools.dialogJump', s => s.tools.dialogJump], ['tools.everything', s => s.tools.everything]]) {
      const before = frame.contentWindow.testMessages.length;
      doc.querySelector(`[data-bind="${path}"]`).click();
      const patch = frame.contentWindow.testMessages.slice(before).find(m => m.type === 'patchSettings');
      assert(patch && read(patch.settings) === true, path + ' not posted');
      await settle();
      assert(doc.querySelector(`[data-bind="${path}"]`).classList.contains('on'), path + ' not shown on after the refresh');
    }
    doc = await reset();
  });
  await test('Everything card shows each state with its button and sends its action', async () => {
    doc = await reset();
    doc.querySelector('[data-page=peek]').click();
    current().settings.tools.everything = true;
    const card = doc.getElementById('everythingCard'), button = doc.getElementById('everythingAction');
    const cases = [
      [{ state: 'running', version: '1.5', pinyin: true }, 'Everything 1.5 正在运行', '拼音', null],
      [{ state: 'running', version: '1.4', pinyin: false }, 'Everything 1.4 正在运行', '1.5', ['升级到 1.5', 'installEverything']],
      [{ state: 'running', version: '1.4', pinyin: false, has15: true }, 'Everything 1.5 已经装好，现在运行的是 1.4', '登录 Windows', ['改用 1.5', 'useEverything15']],
      [{ state: 'installed' }, 'Everything 已经装好，还没有运行', '启动后', ['启动 Everything', 'startEverything']],
      [{ state: 'missing' }, '这台电脑还没有 Everything', 'winget', ['安装 Everything', 'installEverything']],
      [{ state: 'installing' }, '正在安装 Everything…', '自动启动', null],
    ];
    for (const [everything, status, help, action] of cases) {
      current().everything = everything;
      frame.contentWindow.testRefresh();
      await settle();
      const label = JSON.stringify(everything);
      assert(!card.hidden && !card.classList.contains('disabled'), 'Card hidden or disabled: ' + label);
      assert(doc.getElementById('everythingStatus').textContent === status, 'Wrong status for ' + label + ': ' + doc.getElementById('everythingStatus').textContent);
      assert(doc.getElementById('everythingHelp').textContent.includes(help), 'Wrong help for ' + label + ': ' + doc.getElementById('everythingHelp').textContent);
      assert(button.hidden === !action, 'Button shown wrongly for ' + label);
      if (!action) continue;
      assert(button.textContent === action[0] && !button.disabled, 'Wrong button for ' + label + ': ' + button.textContent);
      const before = frame.contentWindow.testMessages.length;
      button.click();
      const sent = frame.contentWindow.testMessages.slice(before);
      assert(sent.length === 1 && sent[0].type === 'action' && sent[0].name === action[1], 'Wrong action for ' + label + ': ' + JSON.stringify(sent));
      await settle();
    }
    // No state from the host (an older one): no card.
    delete current().everything;
    frame.contentWindow.testRefresh();
    assert(card.hidden, 'Card shown without a state');
    doc = await reset();
  });
  await test('Everything card is disabled while its switch is off', async () => {
    doc = await reset();
    doc.querySelector('[data-page=general]').click();
    const card = doc.getElementById('everythingCard'), button = doc.getElementById('everythingAction');
    assert(card.classList.contains('disabled') && button.disabled, 'Card usable while the switch is off');
    const before = frame.contentWindow.testMessages.length;
    button.click();
    assert(!frame.contentWindow.testMessages.slice(before).some(m => m.type === 'action'), 'A disabled button sent an action');
    doc.querySelector('[data-bind="tools.everything"]').click();
    assert(!card.classList.contains('disabled') && !button.disabled, 'Card still disabled right after the switch went on');
    await settle();
    assert(!card.classList.contains('disabled') && !button.disabled, 'Card disabled again after the refresh');
    doc.querySelector('[data-bind="tools.everything"]').click();
    await settle();
    assert(card.classList.contains('disabled') && button.disabled, 'Card usable after the switch went off');
    doc = await reset();
  });

  await test('Navigation resets content scroll and announces the current page', async () => {
    doc.querySelector('[data-page=general]').click();
    doc.querySelector('main').scrollTop = 900;
    doc.querySelector('[data-page=layout]').click();
    assert(doc.querySelector('main').scrollTop === 0, 'New page retained old scroll');
    assert(doc.querySelector('[aria-current=page]')?.dataset.page === 'layout', 'Wrong active page');
  });
  await test('Fence page lists every fence and reflects the selected fence', async () => {
    doc.querySelector('[data-page=fences]').click();
    const sel = doc.getElementById('fenceSel');
    assert(sel.options.length === 4, 'Fence list incomplete');
    assert([...sel.options].map(o => o.textContent).join('|') === 'Documents|Images（Documents 的标签页）|Desktop（桌面）|Portal（文件夹）', 'Fence labels wrong: ' + [...sel.options].map(o => o.textContent).join('|'));
    assert(doc.getElementById('fencePortalGroup').style.display === 'none', 'Portal options shown for an ordinary fence');
    assert(doc.getElementById('fenceHideEmpty').style.display === '', 'Hide-when-empty not offered for an ordinary fence');
    assert(doc.getElementById('fenceDesktopLayout').style.display === 'none', 'Desktop layout offered for an ordinary fence');
    assert(doc.getElementById('fenceFreePlacement').style.display === 'none', 'Free placement offered for an ordinary fence');
    const fenceHideNote = doc.querySelector('#fenceHideEmpty .s').textContent;
    assert(doc.getElementById('fenceOpenFolders').style.display === '' && !doc.querySelector('[data-fence=openFoldersInside]').classList.contains('on'), 'Open-folders-inside missing or on for an ordinary fence');
    assert(doc.getElementById('fenceTintTitleOnly').style.display === 'none', 'Title-only tint offered without a tint');
    assert(doc.getElementById('fenceTabColor').style.display === 'none', 'Tab colour offered for a fence without tabs');
    assert(doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint offered without a tint');
    const tintHint = doc.getElementById('fenceTitleColor').closest('.card').querySelector('.s');
    assert(!tintHint.hidden, 'Choose-a-tint hint missing without a tint');
    change(doc, 'fenceSel', 'inbox');
    assert(tintHint.hidden, 'Choose-a-tint hint shown although the fence has a tint');
    assert(doc.querySelector('[data-fence=locked]').classList.contains('on'), 'Lock state not shown');
    assert(doc.querySelector('[data-fence=expandUpward]').classList.contains('on'), 'Expand upward not shown');
    assert(doc.getElementById('fenceHideEmpty').style.display === '' && doc.querySelector('[data-fence=hideWhenEmpty]').classList.contains('on'), 'Inbox hide-when-empty not shown');
    assert(doc.getElementById('fenceDesktopLayout').style.display === '' && doc.querySelector('[data-fence=desktopLayout]').classList.contains('on'), 'Inbox desktop layout not shown');
    assert(doc.getElementById('fenceFreePlacement').style.display === '' && !doc.querySelector('[data-fence=freePlacement]').classList.contains('on'), 'Inbox free placement not shown, or on');
    assert(doc.querySelector('#fenceHideEmpty .s').textContent !== fenceHideNote, 'Inbox hide-when-empty note is the one of other fences');
    assert(doc.querySelector('[data-fence=openFoldersInside]').classList.contains('on'), 'Open-folders-inside state not shown');
    assert(doc.getElementById('fenceTintTitleOnly').style.display === '' && doc.querySelector('[data-fence=tintTitleOnly]').classList.contains('on'), 'Title-only tint not shown');
    assert(doc.querySelector('[data-fence=tint]').value === '0078D4', 'Palette tint not selected');
    assert(doc.querySelector('[data-fence=titleColor]').value === 'tint' && !doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint title not shown');
    assert(doc.querySelector('[data-fence=titleSize]').value === 'large', 'Title size not shown');
    assert(doc.querySelector('[data-fence=titleSize] [value=extraLarge]'), 'Extra large title size missing');
    assert(doc.querySelector('[data-fence=titleOnHover]').value === 'hover', 'Title on hover not shown');
    change(doc, 'fenceSel', 'fence-b');
    assert(doc.getElementById('fenceTabColor').style.display === '', 'Tab colour hidden for a tab');
    assert(doc.getElementById('fenceNameLines').style.display === '' && doc.getElementById('fenceNameLines').classList.contains('disabled'), 'Icon labels not greyed for a list fence');
    assert(doc.querySelector('#fenceTabColorSwatches [data-value=E74856]').getAttribute('aria-checked') === 'true', 'Tab colour swatch not selected');
    change(doc, 'fenceSel', 'portal');
    assert(doc.getElementById('fencePortalGroup').style.display === '', 'Portal options hidden for a portal');
    assert(doc.getElementById('fenceOpenFolders').style.display === 'none', 'Open-folders-inside offered for a portal');
    assert(doc.querySelector('[data-fence=portalNavigate]').classList.contains('on') && !doc.querySelector('[data-fence=portalTitleIcon]').classList.contains('on'), 'Portal flags wrong');
    assert(doc.querySelector('[data-fence=tint]').value === '123456' && doc.querySelector('[data-fence=titleColor]').value === 'ABCDEF', 'Imported custom colours not represented');
    assert(doc.querySelector('[data-fence=iconSize]').value === '32' && doc.querySelector('[data-fence=spacingPx]').value === '-12' && doc.querySelector('[data-fence=opacity]').value === 'clear', 'Layout values not shown');
    assert(doc.querySelector('[data-fence=iconAlign]').value === 'center', 'Icon alignment not shown');
    assert(!doc.getElementById('fenceNameLines').classList.contains('disabled') && doc.querySelector('[data-fence=labelLines]').value === '0', 'No-labels choice not shown');
    assert([...doc.querySelectorAll('#fenceLabelLines option')].map(o => o.value + o.textContent).join() === '0不显示,1一行,2两行,3三行', 'Icon label choices wrong');
  });
  await test('Fence controls post setFence and keep the selection across a host refresh', async () => {
    const messages = () => frame.contentWindow.testMessages.filter(m => m.type === 'setFence');
    change(doc, 'fenceSel', 'fence-a');
    const before = messages().length;
    doc.querySelector('[data-fence=locked]').click(); await settle();
    change(doc, 'fenceIconSize', '96');
    change(doc, 'fenceTint', 'E74856');
    change(doc, 'fenceTitle', '  Renamed  ');
    change(doc, 'fenceTitleOnHover', 'always');
    change(doc, 'fenceLabelLines', '0');
    await settle();
    const sent = messages().slice(before);
    assert(sent.length === 6, 'Expected six setFence messages, got ' + sent.length);
    assert(sent.every(m => m.id === 'fence-a'), 'Message addressed to the wrong fence');
    assert(sent[0].prop === 'locked' && sent[0].value === true, 'Lock toggle not posted');
    assert(sent[1].prop === 'iconSize' && sent[1].value === 96, 'Icon size not posted as a number');
    assert(sent[2].prop === 'tint' && sent[2].value === 'E74856', 'Tint not posted');
    assert(sent[3].prop === 'title' && sent[3].value === 'Renamed', 'Title not trimmed');
    assert(sent[4].prop === 'titleOnHover' && sent[4].value === 'always', 'Title on hover not posted');
    assert(sent[5].prop === 'labelLines' && sent[5].value === 0, 'Icon labels not posted as a number');
    assert(doc.getElementById('fenceSel').value === 'fence-a', 'Selection lost on refresh');
    assert(doc.querySelector('[data-fence=locked]').classList.contains('on') && doc.querySelector('[data-fence=tint]').value === 'E74856' && doc.querySelector('[data-fence=titleOnHover]').value === 'always' && doc.querySelector('[data-fence=labelLines]').value === '0', 'Refreshed state not reflected');
    assert(!doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint stays disabled after choosing a tint');
    change(doc, 'fenceTint', ''); await settle();
    assert(messages().at(-1).value === null, 'Clearing the tint did not post null');
    change(doc, 'fenceSel', 'fence-b');
    doc.querySelector('#fenceTabColorSwatches [data-value="0078D4"]').click(); await settle();
    assert(messages().at(-1).id === 'fence-b' && messages().at(-1).prop === 'tabColor' && messages().at(-1).value === '0078D4', 'Tab colour not posted');
    doc.querySelector('#fenceTabColorSwatches [data-value=""]').click(); await settle();
    assert(messages().at(-1).prop === 'tabColor' && messages().at(-1).value === null, 'Following the tint did not post null');
    change(doc, 'fenceSel', 'fence-a');
    assert(doc.querySelector('[data-fence=iconAlign]').value === 'left', 'Left icon alignment not shown');
    change(doc, 'fenceIconAlign', 'center'); await settle();
    assert(messages().at(-1).id === 'fence-a' && messages().at(-1).prop === 'iconAlign' && messages().at(-1).value === 'center', 'Icon alignment not posted');
  });
  await test('在哪个虚拟桌面显示 lists the desktops, shows the choice and posts it; none for the inbox or one desktop', async () => {
    doc.querySelector('[data-page=fences]').click();
    const messages = () => frame.contentWindow.testMessages.filter(m => m.type === 'setFence' && m.prop === 'virtualDesktops');
    change(doc, 'fenceSel', 'fence-a');
    const card = doc.getElementById('fenceVdCard'), sel = doc.getElementById('fenceVd');
    assert(card.style.display !== 'none', 'Card hidden with two desktops');
    const names = [...sel.options].map(o => o.textContent).join('|');
    assert(names === '所有虚拟桌面|Work（当前）|Games', 'Options: ' + names);
    assert(sel.value === '{B}', 'The desktop the fence is on is not chosen: ' + sel.value);
    change(doc, 'fenceVd', ''); await settle();
    assert(messages().at(-1).id === 'fence-a' && messages().at(-1).value === 'all', '所有虚拟桌面 not posted as all');
    assert(doc.getElementById('fenceVd').value === '', 'Every desktop not shown after the refresh');
    change(doc, 'fenceVd', '{A}'); await settle();
    assert(messages().at(-1).value === '{A}' && doc.getElementById('fenceVd').value === '{A}', 'A desktop not posted or not shown');
    change(doc, 'fenceSel', 'inbox');
    assert(doc.getElementById('fenceVdCard').style.display === 'none', 'The inbox offers a choice');
    const tabbed = current().fences.find(f => f.id === 'fence-b');
    tabbed.inboxWindow = true;
    change(doc, 'fenceSel', 'fence-b');
    frame.contentWindow.testRefresh(); await settle();
    assert(doc.getElementById('fenceVdCard').style.display === 'none', 'A tab in a window with the inbox offers a choice');
    delete tabbed.inboxWindow;
    const desktops = current().virtualDesktops;
    current().virtualDesktops = [];
    change(doc, 'fenceSel', 'fence-b');
    frame.contentWindow.testRefresh(); await settle();
    assert(doc.getElementById('fenceVdCard').style.display === 'none', 'Card shown with one desktop');
    current().virtualDesktops = desktops;
    change(doc, 'fenceSel', 'fence-a');
    frame.contentWindow.testRefresh(); await settle();
    assert(doc.getElementById('fenceVdCard').style.display !== 'none', 'A bound fence lost its card');
  });
  await test('每个虚拟桌面各用一套栅栏 is posted from the General page', async () => {
    doc.querySelector('[data-page=general]').click();
    const toggle = doc.querySelector('[data-bind=perDesktopFences]');
    assert(toggle && !toggle.classList.contains('on'), 'Switch missing or on by default');
    toggle.click(); await settle();
    assert(current().settings.perDesktopFences === true, 'Turning it on was not posted');
    toggle.click(); await settle();
    assert(current().settings.perDesktopFences === false, 'Turning it off was not posted');
  });
  await test('Icon labels grey out with a reason where no icons show, and stay live on a desktop-arranged inbox', async () => {
    doc.querySelector('[data-page=fences]').click();
    const card = doc.getElementById('fenceNameLines'), select = doc.getElementById('fenceLabelLines');
    const note = () => card.querySelector('.s').textContent, greyed = () => card.classList.contains('disabled') && select.disabled;
    const inbox = current().fences.find(f => f.id === 'inbox');
    change(doc, 'fenceSel', 'fence-b');
    assert(card.style.display === '' && greyed() && note() === '只用于图标视图', 'List fence: labels card not greyed with its reason: ' + note());
    change(doc, 'fenceSel', 'inbox');
    assert(!greyed() && !card.classList.contains('disabled') && note() === '不显示时只显示图标，鼠标悬停在图标上可以看到名称', 'Icons fence: labels card greyed');
    // Arranged like the desktop, the inbox shows icons whatever its own view.
    inbox.layout = 'details'; frame.contentWindow.testRefresh();
    assert(!card.classList.contains('disabled') && !select.disabled, 'Desktop-arranged inbox: labels card greyed');
    inbox.freePlacement = true; frame.contentWindow.testRefresh();
    assert(doc.querySelector('[data-fence=freePlacement]').classList.contains('on'), 'Free placement state not shown');
    inbox.desktopLayout = false; frame.contentWindow.testRefresh();
    assert(greyed() && note() === '只用于图标视图', 'Details inbox: labels card not greyed');
    // 自由摆放 is part of 像桌面一样排列: no card while that is off.
    assert(doc.getElementById('fenceFreePlacement').style.display === 'none', 'Free placement offered while the inbox is not arranged');
    // A 「桌面」 with tabs stays a fence and shows its own view.
    inbox.desktopLayout = true; inbox.tabbed = true; frame.contentWindow.testRefresh();
    assert(greyed(), 'Tabbed inbox in details: labels card not greyed');
    assert(doc.getElementById('fenceFreePlacement').style.display === 'none', 'Free placement offered for a tabbed inbox');
    delete inbox.tabbed; delete inbox.freePlacement; inbox.layout = 'icons'; frame.contentWindow.testRefresh();
    assert(!greyed(), 'Labels card stays greyed after the inbox shows icons again');
    change(doc, 'fenceSel', 'fence-a');
  });
  // A pointer drag on a slider: pointerdown, then one input event per position. `move`
  // alone continues a drag already under way.
  const move = (id, ...values) => {
    const input = doc.getElementById(id);
    for (const value of values) {
      input.value = value;
      input.dispatchEvent(new frame.contentWindow.Event('input', { bubbles: true }));
    }
    return input;
  };
  const slide = (id, ...values) => {
    doc.getElementById(id).dispatchEvent(new frame.contentWindow.PointerEvent('pointerdown', { bubbles: true }));
    return move(id, ...values);
  };
  // Chromium sends change after the release only when the value moved; `change: false`
  // leaves it out.
  const release = (id, { change = true } = {}) => {
    const input = doc.getElementById(id), win = frame.contentWindow;
    input.dispatchEvent(new win.PointerEvent('pointerup', { bubbles: true }));
    if (change) input.dispatchEvent(new win.Event('change', { bubbles: true }));
  };
  const keyOn = (id, type, key) => doc.getElementById(id).dispatchEvent(new frame.contentWindow.KeyboardEvent(type, { key, bubbles: true, cancelable: true }));
  const tap = (id, key) => { keyOn(id, 'keydown', key); keyOn(id, 'keyup', key); };
  const track = id => doc.getElementById(id).closest('.slider-track');
  const bubble = id => track(id).querySelector('.slider-bubble');
  const ticks = id => [...track(id).querySelectorAll('.slider-ticks > span')];
  const tickAt = id => ticks(id).map(s => parseFloat(s.style.left).toFixed(1)).join();
  const onTick = id => ticks(id).filter(s => s.classList.contains('on')).map(s => s.textContent).join();
  await test('Icon size sliders cover 24 to 128 px, snap to the menu sizes and label them', async () => {
    doc.querySelector('[data-page=general]').click();
    const input = doc.getElementById('iconSize'), preview = doc.getElementById('desktopPreview');
    assert(input.type === 'range' && input.min === '24' && input.max === '128', 'Default icon size is not a 24-128 slider');
    assert(tickAt('iconSize') === [32, 48, 64, 96].map(v => ((v - 24) / 104 * 100).toFixed(1)).join(), 'Ticks not at the menu sizes: ' + tickAt('iconSize'));
    assert(ticks('iconSize').map(s => s.textContent).join() === '小,中,较大,大', 'Ticks not labelled: ' + ticks('iconSize').map(s => s.textContent));
    assert(input.value === '48' && onTick('iconSize') === '中', 'Current tick not marked: ' + onTick('iconSize'));
    assert(input.getAttribute('aria-valuetext') === '中等图标 · 48 px', 'Preset name not announced: ' + input.getAttribute('aria-valuetext'));
    const patches = () => frame.contentWindow.testMessages.filter(m => m.type === 'patchSettings').length;
    const before = patches();
    // Closer than 4 px to a tick lands on it; elsewhere the thumb takes the nearest step of 4.
    for (const [raw, value] of [['30', 32], ['35.5', 32], ['28.5', 32], ['27.5', 28], ['37', 36], ['41', 40], ['44.5', 48], ['99', 96], ['100', 100], ['126.9', 128]]) {
      slide('iconSize', raw);
      assert(Number(input.value) === value, `${raw} should land on ${value}, got ${input.value}`);
    }
    assert(bubble('iconSize').textContent === '128 px' && input.getAttribute('aria-valuetext') === '128 px' && onTick('iconSize') === '', 'Bubble not updated while dragging: ' + bubble('iconSize').textContent);
    assert(preview.style.getPropertyValue('--preview-icon-size') === '40px', 'Preview did not follow the drag');
    assert(patches() === before, 'The default size was saved while dragging');
    slide('iconSize', '37');
    release('iconSize');
    await settle();
    assert(patches() === before + 1 && current().settings.iconSize === 36, 'The released value was not saved once');
    assert(input.value === '36' && bubble('iconSize').textContent === '36 px', 'Value lost on the host refresh');
    assert(preview.style.getPropertyValue('--preview-icon-size') === '22px', 'Preview not scaled between the menu sizes');
    // Keys move by one step, skip the snap and save on key-up.
    keyOn('iconSize', 'keydown', 'ArrowLeft');
    assert(input.value === '32' && patches() === before + 1, 'A held key saved before its release');
    keyOn('iconSize', 'keyup', 'ArrowLeft');
    assert(current().settings.iconSize === 32 && patches() === before + 2, 'Key-up did not save the size');
    tap('iconSize', 'ArrowRight');
    assert(current().settings.iconSize === 36 && input.value === '36', 'Arrow key snapped back to the tick');
    tap('iconSize', 'End');
    assert(current().settings.iconSize === 128, 'End did not reach the largest size');
    // 48 was on the drag's way: the host's value shows once its echoes can no longer arrive.
    current().settings.iconSize = 48;
    frame.contentWindow.testRefresh();
    assert(input.value === '128', 'An echo of a passed value moved the thumb');
    await new Promise(resolve => setTimeout(resolve, 600));
    frame.contentWindow.testRefresh();
    assert(input.value === '48', 'Host value not shown');
  });
  await test('A dragged slider shows its value in a bubble over the thumb', async () => {
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-a');
    const input = doc.getElementById('fenceIconSize'), tip = bubble('fenceIconSize');
    assert(!track('fenceIconSize').classList.contains('live') && tip.getAttribute('aria-hidden') === 'true', 'Bubble shown at rest');
    slide('fenceIconSize', '52');
    assert(track('fenceIconSize').classList.contains('live'), 'Bubble not shown during the drag');
    assert(tip.textContent === '52 px' && !tip.querySelector('small'), 'Bubble text off a preset: ' + tip.textContent);
    move('fenceIconSize', '47');
    assert(tip.querySelector('b').textContent === '48 px' && tip.querySelector('small').textContent === '中等图标', 'Bubble text on a preset: ' + tip.textContent);
    const thumbX = 10 + (input.getBoundingClientRect().width - 20) * (48 - 24) / 104;
    const tipX = tip.getBoundingClientRect().left + tip.getBoundingClientRect().width / 2 - input.getBoundingClientRect().left;
    assert(Math.abs(tipX - thumbX) < 1 && tip.getBoundingClientRect().bottom <= input.getBoundingClientRect().top + 8, `Bubble not over the thumb: ${tipX} vs ${thumbX}`);
    release('fenceIconSize');
    await settle();
    assert(!track('fenceIconSize').classList.contains('live'), 'Bubble stays after the release');
    change(doc, 'fenceSel', 'fence-b');
    slide('fenceSpacing', '1');
    assert(bubble('fenceSpacing').querySelector('b').textContent === '0 px' && bubble('fenceSpacing').querySelector('small').textContent === '标准', 'Spacing bubble: ' + bubble('fenceSpacing').textContent);
    release('fenceSpacing');
    await settle();
    // The bubble and the thumb follow the theme.
    const styleOf = el => frame.contentWindow.getComputedStyle(el);
    const dark = [styleOf(tip).backgroundColor, styleOf(input).getPropertyValue('--thumb').trim()];
    doc.documentElement.classList.add('light');
    const light = [styleOf(tip).backgroundColor, styleOf(input).getPropertyValue('--thumb').trim()];
    doc.documentElement.classList.remove('light');
    assert(dark.join() === 'rgb(44, 44, 44),#454545' && light.join() === 'rgb(249, 249, 249),#ffffff', `Bubble or thumb ignores the theme: ${dark} | ${light}`);
    await new Promise(resolve => setTimeout(resolve, 600));
    current().fences.find(f => f.id === 'fence-b').spacingPx = 16;
    change(doc, 'fenceSel', 'fence-a');
    current().fences.find(f => f.id === 'fence-a').iconSize = 48;
    frame.contentWindow.testRefresh();
  });
  await test('Fence sliders send live ticks at most ten times a second, then the release without the flag', async () => {
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-a');
    const fence = id => current().fences.find(f => f.id === id);
    fence('fence-a').iconSize = 48;
    frame.contentWindow.testRefresh();
    const sent = () => frame.contentWindow.testMessages.filter(m => m.type === 'setFence' && m.prop === 'iconSize');
    let before = sent().length;
    const input = slide('fenceIconSize', '52', '56', '61');
    assert(sent().length === before, 'Every input event posted at once');
    await settle();
    assert(sent().length === before + 1 && sent().at(-1).value === 64 && sent().at(-1).id === 'fence-a' && sent().at(-1).live === true, 'The latest position was not posted once as a live tick: ' + JSON.stringify(sent().slice(before)));
    slide('fenceIconSize', '69');
    // A host state from the earlier position must not move a held thumb.
    frame.contentWindow.testRefresh();
    assert(input.value === '68' && bubble('fenceIconSize').textContent === '68 px', 'Host refresh moved the thumb during a drag');
    release('fenceIconSize');
    assert(sent().length === before + 2 && sent().at(-1).value === 68 && !('live' in sent().at(-1)), 'The release was not posted at once without the live flag: ' + JSON.stringify(sent().slice(before)));
    await settle();
    assert(sent().length === before + 2, 'A throttled post fired after the release');
    fence('fence-a').iconSize = 64;
    frame.contentWindow.testRefresh();
    assert(input.value === '68', 'A late host echo of a value passed on the way moved the thumb');
    fence('fence-a').iconSize = 68;
    frame.contentWindow.testRefresh();
    // A held arrow key repeats: one post for the whole run, at its release.
    before = sent().length;
    for (let i = 0; i < 3; i++) keyOn('fenceIconSize', 'keydown', 'ArrowRight');
    assert(input.value === '80' && sent().length === before, 'Key repeats posted past the throttle');
    keyOn('fenceIconSize', 'keyup', 'ArrowRight');
    assert(sent().length === before + 1 && sent().at(-1).value === 80 && !sent().at(-1).live, 'Key-up did not post the size: ' + JSON.stringify(sent().slice(before)));
    await settle();
    // A drag back to its start gets no change event: the release still sends the start value,
    // so the host snaps the live ticks away.
    await new Promise(resolve => setTimeout(resolve, 600));
    before = sent().length;
    slide('fenceIconSize', '100', '120');
    await settle();
    assert(sent().at(-1).value === 120 && sent().at(-1).live, 'Drag not posted');
    slide('fenceIconSize', '81');
    release('fenceIconSize', { change: false });
    assert(sent().length === before + 2 && sent().at(-1).value === 80 && !sent().at(-1).live, 'Releasing on the start value did not post it: ' + JSON.stringify(sent().slice(before)));
    await settle();
    fence('fence-a').iconSize = 120;
    frame.contentWindow.testRefresh();
    assert(input.value === '80', 'A late echo moved the thumb after a drag back to its start');
    fence('fence-a').iconSize = 80;
    frame.contentWindow.testRefresh();
    // A drag that never left its start sends nothing.
    before = sent().length;
    slide('fenceIconSize', '81');
    release('fenceIconSize', { change: false });
    await settle();
    assert(sent().length === before, 'A drag that did not move posted');
    // Another fence shows its own size at once.
    change(doc, 'fenceSel', 'portal');
    assert(input.value === '32' && onTick('fenceIconSize') === '小', 'Portal size not shown: ' + input.value);
    change(doc, 'fenceSel', 'fence-a');
    assert(input.value === '80', 'Fence size not shown after switching back: ' + input.value);
    // A fence switch in the middle of a drag drops the drag instead of resizing the new fence.
    before = sent().length;
    slide('fenceIconSize', '100');
    change(doc, 'fenceSel', 'portal');
    move('fenceIconSize', '110');
    release('fenceIconSize');
    await settle();
    assert(input.value === '32' && sent().length === before, 'The rest of the drag resized another fence: ' + JSON.stringify(sent().slice(before)));
    change(doc, 'fenceSel', 'fence-a');
    // One whose live ticks already resized the old fence sends that fence its release first.
    await new Promise(resolve => setTimeout(resolve, 600));
    before = sent().length;
    slide('fenceIconSize', '100');
    await settle();
    change(doc, 'fenceSel', 'portal');
    move('fenceIconSize', '110');
    release('fenceIconSize');
    await settle();
    const tail = sent().slice(before).map(m => [m.id, m.value, !!m.live].join());
    assert(tail.join(' | ') === 'fence-a,100,true | fence-a,100,false' && input.value === '32', 'The old fence did not get its release: ' + tail.join(' | '));
    change(doc, 'fenceSel', 'fence-a');
    fence('fence-a').iconSize = 48;
    frame.contentWindow.testRefresh();
  });
  await test('The icon spacing slider covers -12 to +32 px in steps of 4 and labels the presets', async () => {
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-b');
    const input = doc.getElementById('fenceSpacing');
    const spacingSent = () => frame.contentWindow.testMessages.filter(m => m.type === 'setFence' && m.prop === 'spacingPx');
    const start = spacingSent().length;
    assert(input.type === 'range' && input.min === '-12' && input.max === '32', 'Icon spacing is not a -12 to 32 slider');
    assert(tickAt('fenceSpacing') === [-12, 0, 16].map(v => ((v + 12) / 44 * 100).toFixed(1)).join(), 'Ticks not at the presets: ' + tickAt('fenceSpacing'));
    assert(ticks('fenceSpacing').map(s => s.textContent).join() === '紧凑,标准,宽松', 'Presets not labelled');
    assert(input.closest('.card').querySelector('.s').textContent === '图标之间的距离，以“标准”为基准', 'Spacing help line missing');
    assert(input.value === '16' && onTick('fenceSpacing') === '宽松', 'Loose not shown: ' + onTick('fenceSpacing'));
    assert(input.getAttribute('aria-valuetext') === '宽松 · +16 px', 'Preset name not announced: ' + input.getAttribute('aria-valuetext'));
    // The thumb takes the nearest step of 4, so every preset is a stop of its own.
    for (const [raw, value] of [['-10.5', -12], ['1.9', 0], ['-1.9', 0], ['14.1', 16], ['13.9', 12], ['2.1', 4], ['9.9', 8], ['31', 32]]) {
      slide('fenceSpacing', raw);
      assert(Number(input.value) === value, `${raw} should land on ${value}, got ${input.value}`);
    }
    assert(bubble('fenceSpacing').textContent === '+32 px' && input.getAttribute('aria-valuetext') === '+32 px', 'Bubble not updated while dragging: ' + bubble('fenceSpacing').textContent);
    await settle();
    slide('fenceSpacing', '-11');
    assert(input.getAttribute('aria-valuetext') === '紧凑 · −12 px' && onTick('fenceSpacing') === '紧凑', 'Compact not shown');
    release('fenceSpacing');
    await settle();
    const sent = spacingSent().slice(start);
    assert(sent.at(-1).id === 'fence-b' && sent.at(-1).value === -12 && !sent.at(-1).live, 'Released value not posted: ' + JSON.stringify(sent.at(-1)));
    assert(sent.length > 1 && sent.slice(0, -1).every(m => m.live === true), 'A tick before the release was not live: ' + JSON.stringify(sent));
    assert(current().fences.find(f => f.id === 'fence-b').spacing === 'compact' && input.value === '-12', 'Value lost on the host refresh');
    tap('fenceSpacing', 'ArrowRight');
    tap('fenceSpacing', 'ArrowRight');
    tap('fenceSpacing', 'ArrowRight');
    assert(spacingSent().at(-1).value === 0 && input.value === '0', 'Arrow keys did not step by 4');
    await new Promise(resolve => setTimeout(resolve, 600));
    current().fences.find(f => f.id === 'fence-b').spacingPx = 16;
    change(doc, 'fenceSel', 'fence-a');
  });
  await test('A dragged slider keeps its row still and its tick labels apart in every window width and language', async () => {
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-a');
    for (const language of ['en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it']) {
      change(doc, 'language', language); await settle();
      for (const [id, values] of [['fenceIconSize', ['52', '128', '24', '48']], ['fenceSpacing', ['4', '-12', '32', '0']]]) {
        const input = doc.getElementById(id), card = input.closest('.card');
        for (const width of [704, 960]) {
          frame.style.width = width + 'px';
          await settle();
          const at = [];
          for (const value of values) {
            slide(id, value);
            at.push([card.getBoundingClientRect().height, input.getBoundingClientRect().left, input.getBoundingClientRect().width].join());
          }
          release(id);
          await settle();
          assert(new Set(at).size === 1, `${id} row moved during a drag at ${width}px: ${at.join(' | ')}`);
          // Tick labels keep 4 px apart, also beside the bold current one, and none leaves its
          // card. Each sits centred under its tick unless the card's edge pushed it in.
          const boxes = ticks(id).map(s => s.querySelector('b').getBoundingClientRect()), rail = track(id).getBoundingClientRect();
          const box = card.getBoundingClientRect();
          const spans = boxes.map(b => `${Math.round(b.left - rail.left)}..${Math.round(b.right - rail.left)}`).join(' ');
          assert(boxes.every((b, i) => i === 0 || boxes[i - 1].right + 4 <= b.left), `${id} tick labels overlap in ${language} at ${width}px: ${spans}`);
          assert(boxes.every(b => b.left >= box.left + 11.5 && b.right <= box.right - 11.5), `${id} tick label outside its card in ${language} at ${width}px: ${spans}`);
          const centred = ticks(id).map((s, i) => {
            const mark = s.getBoundingClientRect(), b = boxes[i];
            const edge = Math.abs(b.left - (box.left + 12)) < 1 || Math.abs(b.right - (box.right - 12)) < 1;
            return edge || Math.abs((b.left + b.right) / 2 - (mark.left + mark.right) / 2) < 1;
          });
          assert(centred.every(Boolean), `${id} tick label not centred under its tick in ${language} at ${width}px: ${spans}`);
          // The spacing bubble names the current preset with the word under its tick.
          const on = ticks(id).find(s => s.classList.contains('on'));
          assert(id !== 'fenceSpacing' || (on && bubble(id).querySelector('small').textContent === on.textContent), `Spacing bubble and tick disagree in ${language}`);
        }
      }
    }
    frame.style.width = '704px';
    change(doc, 'language', 'zh-CN'); await settle();
    const fence = current().fences.find(f => f.id === 'fence-a');
    fence.iconSize = 48; fence.spacingPx = 0; fence.spacing = 'normal';
    frame.contentWindow.testRefresh();
  });
  await test('A smart fence shows its search in one line, with 编辑条件…, 立即刷新 and Everything', async () => {
    const win = frame.contentWindow;
    try {
    const spec = { template: 'downloads', folders: ['C:\\Users\\me\\Downloads'], subfolders: true, allOf: [{ cond: 'filesOnly', value: null }], withinDays: 7, limit: 200 };
    const base = current().fences.find(f => f.id === 'fence-a');
    win.testState.fences.push({ ...structuredClone(base), id: 'smart', title: 'Recent downloads', smart: { spec, match: 'all', folders: spec.folders, source: 'everything', total: 6, shown: 6, query: 'q', titleIcon: true } });
    win.testRefresh(); await settle();
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-a');
    assert(doc.getElementById('fenceSmartGroup').style.display === 'none', 'Search conditions shown for an ordinary fence');
    change(doc, 'fenceSel', 'smart');
    assert(doc.getElementById('fenceSmartGroup').style.display === '', 'Search conditions hidden for a smart fence');
    assert(doc.getElementById('fenceOpenFolders').style.display === 'none', 'Open-folders-inside offered for a smart fence');
    const summary = () => doc.getElementById('smartSummary').textContent;
    assert(summary() === 'Downloads 文件夹（含子文件夹） · 最近 7 天 · 仅文件', 'Summary: ' + summary());
    assert(doc.getElementById('smartStatus').textContent.startsWith('现在显示 6 个文件 · 由 Everything 搜索') && doc.getElementById('smartEverythingBtn').hidden, 'Status wrong: ' + doc.getElementById('smartStatus').textContent);
    for (const gone of ['smartScope', 'smartDays', 'smartConds', 'smartLimit', 'smartName', 'smartEverything']) assert(!doc.getElementById(gone), gone + ' is still on the page');
    const before = win.testMessages.length;
    doc.getElementById('smartEdit').click();
    doc.getElementById('smartRefresh').click();
    doc.querySelector('#fenceSmartGroup [data-fence=portalTitleIcon]').click(); await settle();
    const got = win.testMessages.slice(before).filter(m => m.type === 'setFence' || m.type === 'action').map(m => m.type === 'setFence' ? `${m.prop}=${JSON.stringify(m.value)}` : `${m.name}:${m.id}`).join(' | ');
    assert(got === 'editSmart:smart | refreshSmart:smart | portalTitleIcon=false', 'Posted: ' + got);
    // A save in the editor comes back as new state: the line follows.
    const smart = win.testState.fences.find(f => f.id === 'smart');
    smart.smart.spec = { folders: ['D:\\Work', 'E:\\Archive\\'], subfolders: false, match: 'any', allOf: [{ cond: 'ext', value: ['pdf', 'docx'] }, { cond: 'name', value: { op: 'contains', value: '发票' } }], name: 'bg', limit: 200 };
    win.testRefresh(); await settle();
    assert(summary() === 'Work、Archive 文件夹 · 任一条件：扩展名：pdf、docx · 名称包含“发票” · 名称匹配“bg”', 'Summary after a save: ' + summary());
    smart.smart.spec = { allOf: [{ cond: 'ext', value: ['.psd'] }], everything: 'x', limit: 200 };
    win.testRefresh(); await settle();
    assert(summary() === '整台电脑 · 扩展名：.psd · Everything 搜索“x”', 'Whole computer: ' + summary());
    // Every drive without Everything: the card offers to start or install it.
    smart.smart.source = 'needsEverything'; smart.smart.shown = 0;
    win.testState.everything = { state: 'installed' };
    win.testRefresh(); await settle();
    const evBtn = doc.getElementById('smartEverythingBtn');
    assert(!evBtn.hidden && evBtn.textContent === '启动 Everything', 'Start button missing: ' + evBtn.textContent);
    evBtn.click();
    win.testState.everything = { state: 'missing' }; win.testRefresh(); await settle();
    assert(evBtn.textContent === '安装 Everything', 'Install button missing');
    evBtn.click(); await settle();
    const actions = win.testMessages.filter(m => m.type === 'action').slice(-2).map(m => m.name).join();
    assert(actions === 'startEverything,installEverything', 'Everything actions: ' + actions);
    } finally {
      win.testState.fences = win.testState.fences.filter(f => f.id !== 'smart');
      win.testState.everything = { state: 'missing' };
      win.testRefresh(); await settle();
      change(doc, 'fenceSel', 'fence-a');
    }
  });
  await test('showFence opens the fence page with that fence selected', async () => {
    doc.querySelector('[data-page=general]').click();
    frame.contentWindow.testShowFence('portal');
    await settle();
    assert(doc.querySelector('[aria-current=page]').dataset.page === 'fences', 'Fence page not shown');
    assert(doc.getElementById('fenceSel').value === 'portal', 'Requested fence not selected');
    assert(doc.activeElement.id === 'fenceSel', 'Fence selector not focused');
  });
  await test('Live item totals update without replacing a name being edited', async () => {
    doc.querySelector('[data-page=fences]').click();
    const title = doc.getElementById('fenceTitle');
    title.focus();
    title.value = '尚未提交的标题';
    const selected = doc.getElementById('fenceSel').value;
    frame.contentWindow.testSummary(53);
    assert(doc.getElementById('workspaceSummary').textContent === '4 个栅栏 · 53 个项目', 'Portal count not reflected in sidebar');
    assert(doc.getElementById('memDetail').textContent.includes('53 个项目'), 'About page count is stale');
    assert(title.value === '尚未提交的标题' && doc.activeElement === title, 'Live update replaced the input or focus');
    assert(doc.getElementById('fenceSel').value === selected, 'Live update changed selected fence');
    title.value = frame.contentWindow.testState.fences.find(f => f.id === selected).title;
    title.blur();
  });
  await test('Time and size fields retain normal dimensions', async () => {
    rulesPage(doc);
    change(doc, 'nrKind', 'size');
    const sizeHeight = doc.getElementById('nrMin').getBoundingClientRect().height;
    assert(sizeHeight >= 32 && sizeHeight === doc.getElementById('nrName').getBoundingClientRect().height, 'Size field does not match a full-size text field');
    change(doc, 'nrKind', 'time');
    assert(doc.getElementById('nrFrom').getBoundingClientRect().width >= 128, 'Time field uses checkbox width');
  });
  for (const [name, min, max] of [
    ['reversed size range', '10', '1'],
    ['negative size', '-1', '20'],
    ['partial numeric input', '10oops', '20'],
    ['infinite size', 'Infinity', '20'],
    ['empty size range', '', ''],
  ]) {
    await test('Rejects ' + name, async () => {
      change(doc, 'nrKind', 'size'); change(doc, 'nrMin', min); change(doc, 'nrMax', max);
      const count = ruleCount(doc);
      await add(doc);
      assert(ruleCount(doc) === count && errorShown(doc), 'Invalid rule was accepted');
      assert(['nrMin', 'nrMax'].includes(doc.activeElement.id), 'Invalid field was not focused');
    });
  }
  await test('Valid one-sided zero size range is accepted', async () => {
    change(doc, 'nrMin', ''); change(doc, 'nrMax', '0');
    const count = ruleCount(doc);
    await add(doc);
    assert(ruleCount(doc) === count + 1, 'Valid range was rejected');
    const value = current().rules.list[0].allOf[0].value;
    assert(value.min === null && value.max === 0, 'Blank or zero bound changed meaning');
    assert(current().rules.list[0].name === '大小 ≤ 0 MB', 'Unnamed rule not named after its condition');
  });
  await test('A new rule goes to the top of the list', async () => {
    const before = current().rules.list.map(r => r.id);
    change(doc, 'nrKind', 'ext'); change(doc, 'nrValue', 'pdf');
    await add(doc);
    const list = current().rules.list;
    assert(list[0].allOf[0].cond === 'ext' && list[0].allOf[0].value[0] === 'pdf', 'New rule is not first');
    assert(list.slice(1).map(r => r.id).join() === before.join(), 'Existing rules changed order');
    assert(doc.querySelector('#ruleList [data-row-id]').dataset.rowId === list[0].id, 'New rule not rendered first');
  });
  await test('A plain-files rule goes to the end of the list and is named after its condition', async () => {
    const before = current().rules.list.map(r => r.id);
    change(doc, 'nrKind', 'plain');
    assert(doc.getElementById('nrValueWrap').style.display === 'none', 'Plain files asks for a value');
    await add(doc);
    const list = current().rules.list, last = list[list.length - 1];
    assert(last.allOf.length === 1 && last.allOf[0].cond === 'plainFiles' && last.name === '普通文件', 'Plain-files rule not appended: ' + JSON.stringify(last));
    assert(list.slice(0, -1).map(r => r.id).join() === before.join(), 'Existing rules changed order');
    const rows = doc.querySelectorAll('#ruleList [data-row-id]');
    assert(rows[rows.length - 1].querySelector('.conds').textContent.startsWith('普通文件 → '), 'Plain-files rule described wrongly');
  });
  await test('Unmatched items can stay in their fence', async () => {
    const pick = async value => {
      const sel = doc.querySelector('[data-bind="rules.unmatched"]');
      sel.value = value; sel.dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true })); await settle();
    };
    assert(doc.querySelector('[data-bind="rules.unmatched"]').value === 'defaultTarget', 'Default is not back to the desktop fence');
    await pick('stay');
    const sent = frame.contentWindow.testMessages.filter(m => m.type === 'setRules').pop();
    assert(sent && sent.rules.unmatched === 'stay', 'setRules did not carry unmatched');
    assert(doc.querySelector('[data-bind="rules.unmatched"]').value === 'stay', 'Choice lost after refresh');
    await pick('defaultTarget');
  });
  for (const kind of ['ext', 'exact']) {
    await test('Rejects separator-only ' + kind + ' values', async () => {
      change(doc, 'nrKind', kind); change(doc, 'nrValue', ',，,');
      const count = ruleCount(doc);
      await add(doc);
      assert(ruleCount(doc) === count && errorShown(doc), 'Empty condition was accepted');
    });
  }
  await test('Missing time is rejected and overnight time is preserved', async () => {
    change(doc, 'nrKind', 'time'); change(doc, 'nrFrom', '');
    let count = ruleCount(doc);
    await add(doc);
    assert(ruleCount(doc) === count && errorShown(doc), 'Missing time became midnight');
    change(doc, 'nrFrom', '18:00'); change(doc, 'nrTo', '06:00');
    await add(doc);
    assert(ruleCount(doc) === count + 1, 'Overnight rule rejected');
    const value = current().rules.list[0].allOf[0].value;
    // The core reads `from_min` / `to_min`; any other spelling makes the host drop all rules.
    assert(value.from_min === 1080 && value.to_min === 360, 'Overnight bounds changed');
    assert(doc.querySelector('#ruleList .conds').textContent.includes('18:00 到 06:00'), 'Overnight rule described wrongly');
  });
  await test('Rule target and unfinished name survive a host refresh', async () => {
    change(doc, 'nrTarget', 'fence-b'); change(doc, 'nrName', 'Unfinished draft');
    frame.contentWindow.testRefresh();
    assert(doc.getElementById('nrTarget').value === 'fence-b', 'Target reset');
    assert(doc.getElementById('nrName').value === 'Unfinished draft', 'Draft lost');
    assert(![...doc.getElementById('nrTarget').options].some(o => o.value === 'portal'), 'Portal offered as a routing target');
  });
  await test('Reordering and toggling rules preserve keyboard focus', async () => {
    const down = doc.querySelector('[data-row-id="rule-a"] [data-act=down]');
    down.focus(); down.click(); await settle();
    assert(doc.activeElement.closest('[data-row-id]')?.dataset.rowId === 'rule-a', 'Reorder lost focus');
    const toggle = doc.querySelector('[data-row-id="rule-a"] [data-act=toggle]');
    toggle.focus();
    toggle.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key: ' ', bubbles: true }));
    await settle();
    assert(doc.activeElement.dataset.act === 'toggle', 'Toggle lost focus');
    assert(doc.activeElement.getAttribute('aria-checked') === 'false', 'Space did not toggle');
    assert(doc.getElementById('nrTarget').value === 'fence-b', 'Reorder reset draft target');
  });
  await test('Quick-add templates post addTemplate once and render the new rule', async () => {
    doc = await reset();
    rulesPage(doc);
    const before = ruleCount(doc);
    doc.querySelector('#templates [data-template=cleanup]').click(); await settle();
    const sent = frame.contentWindow.testMessages.filter(m => m.name === 'addTemplate');
    assert(sent.length === 1 && sent[0].template === 'cleanup', 'addTemplate not posted');
    assert(ruleCount(doc) === before + 1, 'Template rule not rendered');
    const conds = doc.querySelector('#ruleList [data-row-id] .conds').textContent;
    assert(conds.includes('30') && conds.includes('安装包'), 'Template conditions not labelled: ' + conds);
    doc.querySelector('#templates [data-template=cleanup]').click(); await settle();
    assert(ruleCount(doc) === before + 1, 'Second click duplicated the template rule');
    change(doc, 'nrKind', 'idle');
    assert(doc.getElementById('nrIdle').style.display === '', 'Idle-days input hidden');
    doc.getElementById('nrIdleDays').value = '0';
    await add(doc);
    assert(errorShown(doc), 'Zero idle days accepted');
    doc = await reset(); rulesPage(doc); // the template fence must not leak into later fixture-count checks
  });
  await test('The games template adds a shortcut-target rule per kind of game shortcut (feedback #83)', async () => {
    const chip = doc.querySelector('#templates [data-template=games]');
    assert(chip && chip.textContent.includes('游戏'), 'Games chip missing');
    const before = ruleCount(doc);
    chip.click(); await settle();
    assert(frame.contentWindow.testMessages.some(m => m.name === 'addTemplate' && m.template === 'games'), 'addTemplate games not posted');
    assert(ruleCount(doc) === before + 5, 'Games rules not rendered');
    const rows = [...doc.querySelectorAll('#ruleList [data-row-id]')].slice(0, 5);
    const conds = rows.map(r => r.querySelector('.conds').textContent);
    assert(conds[0].startsWith('快捷方式目标包含“steam://rungameid/”') && conds[4].startsWith('快捷方式目标包含“\\XboxGames\\”'), 'Games conditions not labelled: ' + conds.join(' | '));
    assert(rows[2].querySelector('.t').textContent === 'Epic 游戏', 'Games rule names not shown');
    // Each rule toggles and goes on its own, like any rule.
    rows[0].querySelector('[data-act=toggle]').click(); await settle();
    assert(current().rules.list[0].enabled === false && current().rules.list[1].enabled === true, 'Toggle did not act on one games rule');
    doc.querySelector('#ruleList [data-row-id] [data-act=del]').click(); await settle();
    assert(ruleCount(doc) === before + 4 && current().rules.list[0].allOf[0].value.value === '\\steamapps\\common\\', 'Delete did not remove one games rule');
    doc = await reset(); rulesPage(doc);
  });
  await test('Reaching a reorder boundary never focuses the opposite action', async () => {
    const up = doc.querySelector('[data-row-id="rule-a"] [data-act=up]');
    up.focus(); up.click(); await settle();
    assert(doc.querySelector('#ruleList').firstElementChild.dataset.rowId === 'rule-a', 'Rule did not move to the top');
    assert(!doc.activeElement.dataset.act, 'Focus switched to a different action');
    doc.activeElement.click(); await settle();
    assert(doc.querySelector('#ruleList').firstElementChild.dataset.rowId === 'rule-a', 'Repeated activation reversed the move');
  });
  await test('Removing the last rule focuses the new-rule field', async () => {
    while (doc.querySelector('#ruleList [data-act=del]')) {
      const button = doc.querySelector('#ruleList [data-act=del]');
      button.focus(); button.click(); await settle();
    }
    assert(doc.activeElement.id === 'nrName', 'Focus fell back to the document');
  });
  await test('Custom imported tint is represented and strength is disabled without tint', async () => {
    doc.querySelector('[data-page=general]').click();
    assert(doc.getElementById('iconTintStrength').disabled, 'Strength active without tint');
    current().settings.icons.tintRgb = [18, 52, 86];
    frame.contentWindow.testRefresh();
    assert(doc.getElementById('iconTint').value === '123456', 'Imported tint displayed as none');
    assert(!doc.getElementById('iconTintStrength').disabled, 'Tint strength disabled for custom tint');
  });
  await test('Long configuration paths wrap and can be selected', async () => {
    current().configPath = 'C:\\' + 'long-directory\\'.repeat(30) + 'config.json';
    frame.contentWindow.testRefresh();
    doc.querySelector('[data-page=about]').click();
    const main = doc.querySelector('main');
    assert(main.scrollWidth <= main.clientWidth + 1, 'Path forces horizontal scrolling');
    assert(frame.contentWindow.getComputedStyle(doc.getElementById('cfgPath')).userSelect === 'text', 'Path cannot be copied');
  });
  await test('Snapshot and backup restore controls retain focus on refresh', async () => {
    doc.querySelector('[data-page=layout]').click();
    for (const id of ['snapList', 'backupList']) {
      doc.querySelector('#' + id + ' [data-act=restore]').focus();
      frame.contentWindow.testRefresh();
      assert(doc.activeElement.closest('#' + id), id + ' lost focus');
    }
  });
  await test('Light theme updates without losing selected values', async () => {
    doc.querySelector('[data-page=general]').click();
    const theme = doc.querySelector('[data-bind=theme]');
    theme.value = 'light'; theme.dispatchEvent(new frame.contentWindow.Event('change'));
    await settle();
    assert(doc.documentElement.classList.contains('light'), 'Light theme not applied');
    assert(theme.value === 'light', 'Theme selection reset');
  });
  await test('Appearance preview reflects icon size, custom tint, Chameleon and host refreshes', async () => {
    const preview = doc.getElementById('desktopPreview');
    current().settings.iconSize = 96;
    current().settings.icons.tintRgb = [18, 52, 86];
    current().settings.icons.tintStrength = 0.9;
    current().settings.icons.chameleon = true;
    frame.contentWindow.testRefresh();
    assert(preview.style.getPropertyValue('--preview-icon-size') === '36px', 'Icon scale did not update');
    assert(preview.style.getPropertyValue('--preview-tint') === '#123456', 'Custom tint did not update');
    assert(preview.style.getPropertyValue('--preview-tint-strength') === '0.9', 'Tint strength did not update');
    assert(preview.classList.contains('chameleon'), 'Chameleon preview did not update');
    assert(doc.getElementById('workspaceSummary').textContent === '4 个栅栏 · 12 个项目', 'Workspace summary does not reflect host state');
  });
  await test('Liquid Glass persists independently of the colour mode and can switch back', async () => {
    const theme = doc.querySelector('[data-bind=theme]');
    const glass = styleRadio(doc, 'liquidGlass'), fluent = styleRadio(doc, 'fluent');
    for (const mode of ['dark', 'light', 'followWindowsMode', 'followAppMode']) {
      theme.value = mode; theme.dispatchEvent(new frame.contentWindow.Event('change'));
      glass.focus(); glass.click(); await settle();
      assert(current().settings.themeStyle === 'liquidGlass', 'Host did not receive the material');
      assert(current().settings.theme === mode && theme.value === mode, 'Style overwrote the colour preference');
      assert(doc.documentElement.classList.contains('liquid-glass'), 'Material not applied after host refresh');
      assert(glass.checked && !fluent.checked, 'Selected tile not shown after host refresh');
      assert(doc.activeElement === glass, 'Material change lost keyboard focus');
      assert(frame.contentWindow.getComputedStyle(doc.querySelector('.tile-glass .preview-title')).filter === 'none', 'Preview distorts foreground labels');
      fluent.click(); await settle();
      assert(!doc.documentElement.classList.contains('liquid-glass'), 'Material remained after switching back');
    }
    assert(doc.querySelector('.tile-glass .preview-glass') && !doc.querySelector('.tile-fluent .preview-glass'), 'Optical layer belongs to the Liquid Glass tile only');
  });
  await test('Older settings without a material field retain Fluent', async () => {
    delete current().settings.themeStyle;
    frame.contentWindow.testRefresh();
    assert(styleRadio(doc, 'fluent').checked, 'Missing material has no valid selection');
    assert(!doc.documentElement.classList.contains('liquid-glass'), 'Old settings changed the appearance');
  });
  await test('The 自定义 tile shows the theme files and picks the first; a built-in tile goes back', async () => {
    const select = doc.getElementById('customTheme'), row = doc.getElementById('customThemeRow');
    assert(row.hidden && styleRadio(doc, 'fluent').checked && !styleRadio(doc, 'custom').checked, 'A custom theme was chosen at first');
    styleRadio(doc, 'custom').click(); await settle();
    assert(!row.hidden && styleRadio(doc, 'custom').checked && current().settings.customTheme === 'Frosted Glass', 'The 自定义 tile did not pick the first theme file');
    assert([...select.options].map(o => o.value).join('|') === 'Frosted Glass|Warm' && !select.disabled && select.value === 'Frosted Glass', 'Theme files not listed');
    assert(current().settings.themeStyle === 'fluent', 'Choosing a custom theme changed the built-in style');
    assert(doc.documentElement.classList.contains('liquid-glass'), 'The page does not follow the theme base style');
    assert(doc.getElementById('customThemeError').hidden && doc.getElementById('customThemeHelp').textContent.includes('themes'), 'Help line or error wrong');
    change(doc, 'customTheme', 'Warm'); await settle();
    assert(current().settings.customTheme === 'Warm' && styleRadio(doc, 'custom').checked && !doc.documentElement.classList.contains('liquid-glass'), 'Another theme file not applied');
    styleRadio(doc, 'liquidGlass').click(); await settle();
    assert(current().settings.customTheme === null && current().settings.themeStyle === 'liquidGlass' && row.hidden, 'A tile click did not go back to the built-in style');
    assert(styleRadio(doc, 'liquidGlass').checked && !styleRadio(doc, 'custom').checked, 'The page did not show the built-in style again');
    styleRadio(doc, 'fluent').click(); await settle();
    assert(!doc.documentElement.classList.contains('liquid-glass'), 'Fluent did not come back');
  });
  await test('With no theme files, 自定义 says how to add one and changes no setting', async () => {
    const files = current().customThemes;
    current().customThemes = [];
    frame.contentWindow.testRefresh(); await settle();
    const before = frame.contentWindow.testMessages.length;
    styleRadio(doc, 'custom').click(); await settle();
    const select = doc.getElementById('customTheme'), help = doc.getElementById('customThemeHelp');
    assert(!doc.getElementById('customThemeRow').hidden && select.disabled && help.textContent.includes('打开主题文件夹'), 'No hint without theme files: ' + help.textContent);
    assert(!frame.contentWindow.testMessages.slice(before).some(m => m.type === 'patchSettings'), 'A setting changed without a theme file');
    frame.contentWindow.testRefresh(); await settle();
    assert(styleRadio(doc, 'custom').checked && !doc.getElementById('customThemeRow').hidden, 'A refresh closed the 自定义 tile');
    // 打开主题文件夹 puts the examples there: the first one is used, as a click on the tile would.
    current().customThemes = files;
    frame.contentWindow.testRefresh(); await settle();
    assert(current().settings.customTheme === 'Frosted Glass' && select.value === 'Frosted Glass' && !select.disabled, 'The first theme file that appeared was not used');
    styleRadio(doc, 'fluent').click(); await settle();
    assert(doc.getElementById('customThemeRow').hidden && current().settings.customTheme === null, 'Fluent did not close the row');
    // Theme files that appear while Fluent is chosen change nothing.
    current().customThemes = [];
    frame.contentWindow.testRefresh(); await settle();
    current().customThemes = files;
    frame.contentWindow.testRefresh(); await settle();
    assert(current().settings.customTheme === null && styleRadio(doc, 'fluent').checked, 'Theme files chose a theme on their own');
  });
  await test('A chosen theme file that does not load stays listed and says why', async () => {
    current().settings.customTheme = 'Gone';
    current().customThemeError = 'The system cannot find the file specified. (os error 2)';
    frame.contentWindow.testRefresh(); await settle();
    const select = doc.getElementById('customTheme'), error = doc.getElementById('customThemeError');
    assert(select.value === 'Gone', 'A chosen file that is gone is not listed');
    assert(!error.hidden && error.textContent.includes('os error 2'), 'The load error is not shown');
    assert(styleRadio(doc, 'custom').checked && !styleRadio(doc, 'fluent').checked, 'The 自定义 tile is not checked for a missing theme');
    styleRadio(doc, 'fluent').click(); await settle();
    assert(current().settings.customTheme === null && error.hidden, 'A built-in tile did not clear the theme and its error');
    assert(styleRadio(doc, 'fluent').checked, 'The built-in style is not checked again');
  });
  await test('The theme buttons ask the host for the folder and the guide', async () => {
    doc.getElementById('openThemes').click(); doc.getElementById('themeGuide').click(); await settle();
    const names = frame.contentWindow.testMessages.filter(m => m.type === 'action').map(m => m.name);
    assert(names.includes('openThemesFolder') && names.includes('openThemeGuide'), 'Buttons sent ' + names.join(', '));
  });
  await test('Switches show an On/Off caption that follows their state', async () => {
    doc.querySelector('[data-page=general]').click();
    const toggle = doc.querySelector('[data-bind=autostart]');
    const label = toggle.parentElement.querySelector('.switch-label');
    const shown = () => [...label.children].filter(visible).map(s => s.textContent);
    assert(shown().join() === (toggle.classList.contains('on') ? '开' : '关'), 'Caption does not match the switch: ' + shown());
    label.click(); await settle();
    assert(toggle.classList.contains('on') && shown().join() === '开', 'Clicking the caption did not toggle the switch');
    toggle.click(); await settle();
    assert(!current().settings.autostart && shown().join() === '关', 'Caption did not follow the switch');
  });
  await test('Hiding the title bar count and chevron is posted', async () => {
    doc.querySelector('[data-page=general]').click();
    const toggle = doc.querySelector('[data-bind="rollUp.hideCountAndChevron"]');
    assert(toggle && !toggle.classList.contains('on'), 'Switch missing or on by default');
    toggle.click(); await settle();
    assert(current().settings.rollUp.hideCountAndChevron === true, 'Turning it on was not posted');
    toggle.click(); await settle();
    assert(current().settings.rollUp.hideCountAndChevron === false, 'Turning it off was not posted');
  });
  await test('Dependent rows are disabled while their master switch is off', async () => {
    const guides = doc.querySelector('[data-bind="snapping.guideLines"]'), hotkey = doc.getElementById('peekHotkey');
    doc.querySelector('[data-bind="snapping.enabled"]').click(); await settle();
    assert(!current().settings.snapping.enabled && guides.closest('.card').classList.contains('disabled'), 'Guide lines stay active without snapping');
    const before = current().settings.snapping.guideLines;
    guides.click(); await settle();
    assert(current().settings.snapping.guideLines === before, 'A disabled switch still changed its setting');
    assert(guides.getAttribute('aria-disabled') === 'true' && guides.tabIndex === -1, 'Disabled switch is still announced as enabled');
    assert(!doc.querySelector('[data-bind="snapping.gapPx"]').disabled && !doc.querySelector('[data-bind="snapping.sizeToCells"]').closest('.card').classList.contains('disabled'), 'Independent snapping options were disabled');
    doc.querySelector('[data-bind="snapping.enabled"]').click(); await settle();
    assert(!guides.closest('.card').classList.contains('disabled') && guides.tabIndex === 0, 'Guide lines did not return');
    doc.querySelector('[data-bind="peek.enabled"]').click(); await settle();
    assert(hotkey.disabled && doc.getElementById('peekHotkeyWin').disabled && doc.getElementById('peekHotkeyChoice').disabled
      && !doc.querySelector('[data-bind="peek.dim"]').closest('.card').classList.contains('disabled'), 'Peek hotkey/dim dependency wrong');
    doc.querySelector('[data-bind="peek.enabled"]').click(); await settle();
    assert(!hotkey.disabled && !doc.getElementById('peekHotkeyChoice').disabled && !doc.getElementById('peekHotkeyCard').classList.contains('disabled'), 'Hotkey stayed disabled');
  });
  await test('Windows 桌面图标 is one list of three; 保持隐藏 shows only for 隐藏', async () => {
    const list = doc.getElementById('desktopIcons'), help = doc.getElementById('desktopIconsHelp'), keepCard = doc.getElementById('keepIconsCard');
    assert(!doc.querySelector('[data-bind=hideRealIcons]') && !doc.querySelector('[data-bind=hideInboxWhenIconsShown]'), 'The old switches are still there');
    // The mock's 「桌面」 is arranged like the desktop: it never shows beside the Windows icons.
    assert(list.value === 'showNoInbox' && list.querySelector('option[value=showWithInbox]').disabled && help.textContent.includes('像桌面一样排列'), 'An arranged 桌面 fence not explained: ' + list.value + ' ' + help.textContent);
    const inbox = current().fences.find(f => f.kind === 'inbox');
    inbox.desktopLayout = false;
    frame.contentWindow.testRefresh(); await settle();
    assert(!list.querySelector('option[value=showWithInbox]').disabled, 'Showing the 桌面 fence still greyed');
    assert(list.value === 'showWithInbox' && keepCard.hidden && help.textContent.includes('同时出现'), 'Shown icons with the Desktop fence not shown: ' + list.value);
    change(doc, 'desktopIcons', 'showNoInbox'); await settle();
    assert(!current().settings.hideRealIcons && current().settings.hideInboxWhenIconsShown && list.value === 'showNoInbox' && help.textContent.includes('不显示'), 'showNoInbox not posted');
    change(doc, 'desktopIcons', 'hide'); await settle();
    assert(current().settings.hideRealIcons && list.value === 'hide' && !keepCard.hidden, 'hide not posted or 保持隐藏 missing');
    const keep = doc.querySelector('[data-bind=keepIconsHidden]');
    keep.click(); await settle();
    assert(current().settings.keepIconsHidden && keep.getAttribute('aria-checked') === 'true', 'Keep switch did not post its setting');
    keep.click(); await settle();
    // Back to shown: the choice made before for the Desktop fence is not kept by accident.
    change(doc, 'desktopIcons', 'showWithInbox'); await settle();
    assert(!current().settings.hideRealIcons && !current().settings.hideInboxWhenIconsShown && keepCard.hidden, 'showWithInbox not posted');
    inbox.desktopLayout = true;
    frame.contentWindow.testRefresh(); await settle();
    const offer = doc.querySelector('[data-bind=offerNewFence]');
    assert(offer.getAttribute('aria-checked') === 'true', 'New fence offer not shown on');
    offer.click(); await settle();
    assert(current().settings.offerNewFence === false, 'New fence offer switch did not post');
    offer.click(); await settle();
    const tips = doc.querySelector('[data-bind=featureTips]');
    assert(tips.getAttribute('aria-checked') === 'true', 'Feature tips not shown on');
    tips.click(); await settle();
    assert(current().settings.featureTips === false, 'Feature tips switch did not post');
    tips.click(); await settle();
  });
  await test('收起的栅栏怎么展开 is one list for hover, click and double-click', async () => {
    const list = doc.getElementById('rollExpand');
    assert(!doc.querySelector('[data-bind="rollUp.hoverPeek"]') && !doc.querySelector('[data-bind="rollUp.clickToExpand"]'), 'The old switches are still there');
    assert(list.value === 'hover', 'Hover not shown: ' + list.value);
    // 单击标题时 keeps hoverPeek: a file dragged onto a rolled title still opens it.
    for (const [value, hover, click] of [['click', true, true], ['double', false, false], ['hover', true, false]]) {
      change(doc, 'rollExpand', value); await settle();
      const r = current().settings.rollUp;
      assert(r.hoverPeek === hover && r.clickToExpand === click && list.value === value, `${value} posted as hover ${r.hoverPeek}, click ${r.clickToExpand}`);
    }
    // A file with both on (an older page allowed it) shows what works: a click.
    current().settings.rollUp.clickToExpand = true;
    frame.contentWindow.testRefresh(); await settle();
    assert(list.value === 'click', 'Both on not shown as click');
    change(doc, 'rollExpand', 'hover'); await settle();
  });
  await test('常规 has the new headings in order, one card for the Windows desktop shortcut', async () => {
    doc.querySelector('[data-page=general]').click();
    const heads = [...doc.querySelectorAll('#page-general h3')].map(h => h.textContent).join(' / ');
    assert(heads === '语言 / 外观 / 图标 / 桌面与启动 / 隐藏栅栏 / 收起与展开 / 标题栏与标签页 / 吸附与对齐', 'Headings: ' + heads);
    const wd = doc.getElementById('wdHotkeyCard');
    assert(wd.contains(doc.getElementById('wdHotkey')) && !wd.classList.contains('child') && wd.querySelector('.t').textContent === '临时显示 Windows 桌面', 'The shortcut box is not in the 临时显示 Windows 桌面 card');
    assert(doc.getElementById('wdHotkeyHelp').textContent.includes('隐藏所有栅栏'), 'The card does not say what it does: ' + doc.getElementById('wdHotkeyHelp').textContent);
    const textColor = doc.querySelector('[data-bind=labelColor]').closest('.card');
    assert(textColor.querySelector('.t').textContent === '文字颜色' && textColor.querySelector('.s').textContent.includes('栅栏标题'), 'Text colour card not renamed');
    const firstCard = doc.querySelector('#page-general .card');
    assert(firstCard.querySelector('[data-bind=language]'), 'The language is not the first card');
  });
  const key = (target, k, keyCode) => target.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key: k, keyCode, bubbles: true, cancelable: true }));
  await test('查找设置 lists matching cards of every page and opens the one picked', async () => {
    doc = await reset();
    const box = doc.getElementById('settingsSearch'), list = doc.getElementById('settingsSearchList');
    const type = text => { box.focus(); box.value = text; box.dispatchEvent(new frame.contentWindow.Event('input', { bubbles: true })); };
    const rows = () => [...list.querySelectorAll('[role=option]')];
    const titles = () => rows().map(r => r.querySelector('b').textContent);
    type('隐藏');
    assert(!list.hidden && box.getAttribute('aria-expanded') === 'true', 'List not shown');
    assert(titles().includes('为空时自动隐藏') && titles().includes('隐藏标题栏右侧的项目数和折叠箭头'), 'Cards of other pages missing: ' + titles().join(' | '));
    // Title hits come before help-line hits: 临时显示 Windows 桌面 says 隐藏 only in its help line.
    assert(titles().indexOf('自动隐藏滚动条') < titles().indexOf('临时显示 Windows 桌面'), 'Order: ' + titles().join(' | '));
    const wd = rows()[titles().indexOf('临时显示 Windows 桌面')];
    assert(wd.querySelector('small').textContent.startsWith('常规 › 隐藏栅栏 · ') && wd.querySelector('small mark'), 'Path or snippet missing: ' + wd.textContent);
    // A card its page hides now (the keep switch while the Windows icons show) stays out.
    assert(!titles().includes('关闭 PecoFence 时保持 Windows 桌面图标隐藏'), 'A hidden card was listed');
    // ↓ and Enter open the pick on its page, tinted, with the focus on its control.
    const target = titles().indexOf('为空时自动隐藏');
    for (let i = 0; i < target; i++) key(box, 'ArrowDown', 40);
    assert(rows()[target].getAttribute('aria-selected') === 'true' && box.getAttribute('aria-activedescendant') === 'searchHit' + target, 'Arrow keys did not move');
    key(box, 'Enter', 13); await settle();
    const card = doc.querySelector('[data-fence="hideWhenEmpty"]').closest('.card');
    assert(doc.getElementById('page-fences').classList.contains('on') && card.classList.contains('found') && list.hidden, 'The pick did not open its page');
    assert(card.contains(doc.activeElement), 'Focus not on the picked card');
    // A list choice is found too, and a click opens its card.
    type('项目放在栅栏里');
    assert(rows().length === 1 && titles()[0] === 'Windows 桌面图标', 'A list choice was not found: ' + titles().join(' | '));
    rows()[0].dispatchEvent(new frame.contentWindow.MouseEvent('mousedown', { bubbles: true, cancelable: true }));
    rows()[0].click(); await settle();
    assert(doc.getElementById('page-general').classList.contains('on') && doc.getElementById('desktopIcons').closest('.card').classList.contains('found'), 'A click did not open the card');
    assert(doc.querySelectorAll('.card.found').length === 1, 'An earlier pick still carries its tint');
    await new Promise(resolve => setTimeout(resolve, 2600));
    assert(!doc.querySelector('.card.found, .card.fading'), 'The tint did not go');
    // Every word must occur; nothing found says so; Esc closes and clears.
    type('图标 快捷方式');
    assert(titles().includes('显示快捷方式箭头') && !titles().includes('图标着色'), 'Two words: ' + titles().join(' | '));
    type('zzzz');
    assert(!rows().length && !list.hidden && list.textContent.includes('zzzz'), 'No-match message missing');
    key(box, 'Escape', 27);
    assert(list.hidden && box.value === '', 'Esc did not clear');
  });
  await test('查找设置 matches pinyin, characters in order and other names', async () => {
    doc = await reset();
    const box = doc.getElementById('settingsSearch'), list = doc.getElementById('settingsSearchList');
    const type = text => { box.focus(); box.value = text; box.dispatchEvent(new frame.contentWindow.Event('input', { bubbles: true })); };
    const titles = () => [...list.querySelectorAll('[role=option] b')].map(b => b.textContent);
    const row = title => [...list.querySelectorAll('[role=option]')].find(r => r.querySelector('b').textContent === title);
    box.focus(); await settle();
    assert(frame.contentWindow.testPinyinAsked && frame.contentWindow.testPinyinAsked.includes('隐'), 'The page did not ask for pinyin');
    type('yc');
    assert(titles().includes('自动隐藏滚动条') && row('自动隐藏滚动条').querySelector('b mark').textContent === '隐藏', 'Pinyin initials: ' + titles().join(' | '));
    type('zdyc');
    assert(titles()[0] === '自动隐藏滚动条', 'Mixed pinyin: ' + titles().join(' | '));
    type('隐滚');
    assert(titles()[0] === '自动隐藏滚动条' && row('自动隐藏滚动条').querySelectorAll('b mark').length === 2, 'Characters in order: ' + titles().join(' | '));
    type('开机');
    assert(titles()[0] === '随 Windows 启动' && row('随 Windows 启动').querySelector('small mark').textContent === '开机', 'Other name: ' + titles().join(' | '));
    type('kaiji');
    assert(titles()[0] === '随 Windows 启动', 'Pinyin of another name: ' + titles().join(' | '));
    type('热键');
    assert(titles().includes('快捷键') && titles().includes('临时显示 Windows 桌面'), 'Hotkey names: ' + titles().join(' | '));
    // Exact and prefix hits rank above looser ones.
    type('快捷键');
    assert(titles()[0] === '快捷键', 'An exact title is not first: ' + titles().join(' | '));
    // A long help line is matched only as it is, not by scattered characters.
    type('桌图');
    assert(!titles().includes('自动隐藏滚动条'), 'Scattered characters matched a help line: ' + titles().join(' | '));
    key(box, 'Escape', 27);
  });
  await test('In English the search matches in-order letters and other names, without pinyin', async () => {
    doc = await reset();
    change(doc, 'language', 'en'); await settle();
    const box = doc.getElementById('settingsSearch'), list = doc.getElementById('settingsSearchList');
    const type = text => { box.focus(); box.value = text; box.dispatchEvent(new frame.contentWindow.Event('input', { bubbles: true })); };
    const titles = () => [...list.querySelectorAll('[role=option] b')].map(b => b.textContent);
    frame.contentWindow.testPinyinAsked = '';
    box.focus(); await settle();
    assert(!frame.contentWindow.testPinyinAsked, 'Pinyin asked in English');
    type('hdsb');
    assert(titles().includes('Automatically hide scrollbars'), 'In-order letters: ' + titles().join(' | '));
    type('hotkey');
    assert(titles().includes('Keyboard shortcut'), 'Other name in English: ' + titles().join(' | '));
    type('startup');
    assert(titles()[0] === 'Start with Windows', 'Start with Windows not first: ' + titles().join(' | '));
    // Another name that matches cleanly ranks above letters scattered over a title.
    type('dark mode');
    assert(titles()[0] === 'Color mode', 'Color mode not first: ' + titles().join(' | '));
    key(box, 'Escape', 27);
    change(doc, 'language', 'zh-CN'); await settle();
    doc.querySelector('[data-page=general]').click();
  });
  await test('Ctrl+F goes to 查找设置, and the search follows the language', async () => {
    doc = await reset();
    const box = doc.getElementById('settingsSearch');
    doc.querySelector('[data-page=rules]').click();
    doc.body.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key: 'f', keyCode: 70, ctrlKey: true, bubbles: true, cancelable: true }));
    assert(doc.activeElement === box, 'Ctrl+F did not focus the box');
    change(doc, 'language', 'en'); await settle();
    assert(box.placeholder === 'Find a setting', 'Placeholder not translated: ' + box.placeholder);
    box.value = 'scrollbar'; box.dispatchEvent(new frame.contentWindow.Event('input', { bubbles: true }));
    const titles = [...doc.querySelectorAll('#settingsSearchList [role=option] b')].map(b => b.textContent);
    assert(titles.includes('Automatically hide scrollbars'), 'English search failed: ' + titles.join(' | '));
    key(box, 'Escape', 27);
    change(doc, 'language', 'zh-CN'); await settle();
    doc.querySelector('[data-page=general]').click();
  });
  await test('Colour swatches drive the hidden selects and keep keyboard focus', async () => {
    const swatches = doc.getElementById('iconTintSwatches');
    const red = swatches.querySelector('[data-value=E74856]');
    red.focus(); red.click(); await settle();
    assert(rgbHex(current().settings.icons.tintRgb) === 'E74856', 'Swatch did not post the tint');
    assert(doc.getElementById('iconTint').value === 'E74856' && red.getAttribute('aria-checked') === 'true', 'Swatch state not reflected');
    assert(doc.activeElement === red && red.tabIndex === 0, 'Swatch lost focus after the host refresh');
    assert(!doc.getElementById('iconTintStrength').disabled, 'Strength unavailable with a tint');
    red.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true })); await settle();
    assert(doc.activeElement.dataset.value === '8764B8' && doc.getElementById('iconTint').value === '8764B8', 'Arrow key did not move the selection');
    swatches.querySelector('[data-value=""]').click(); await settle();
    assert(current().settings.icons.tintRgb === null && doc.getElementById('iconTintStrength').disabled, 'No-tint swatch did not clear the tint');
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-a');
    const blue = doc.querySelector('#fenceTintSwatches [data-value="0078D4"]');
    blue.click(); await settle();
    const sent = frame.contentWindow.testMessages.filter(m => m.type === 'setFence').at(-1);
    assert(sent.prop === 'tint' && sent.value === '0078D4', 'Fence tint swatch did not post setFence');
    assert(doc.querySelector('#fenceTintSwatches [data-value="0078D4"]').getAttribute('aria-checked') === 'true', 'Fence tint swatch not selected after refresh');
    doc.querySelector('#fenceTintSwatches [data-value=""]').click(); await settle();
  });
  await test('The custom colour swatch opens the colour dialog and stands for a colour off the palette', async () => {
    doc.querySelector('[data-page=general]').click();
    const swatches = doc.getElementById('iconTintSwatches');
    const custom = swatches.children[1];
    assert(custom.classList.contains('custom') && custom.getAttribute('aria-checked') === 'false', 'Custom swatch not right after no tint');
    const none = swatches.children[0];
    none.focus();
    none.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true })); await settle();
    assert(doc.activeElement === custom && !frame.contentWindow.testMessages.some(m => m.name === 'pickColor'), 'Arrow key did not just focus the custom swatch');
    custom.click(); await settle(); await settle();
    const asked = frame.contentWindow.testMessages.filter(m => m.name === 'pickColor').at(-1);
    assert(asked && asked.target === 'iconTint', 'Colour dialog not requested');
    assert(rgbHex(current().settings.icons.tintRgb) === '123456', 'Picked colour not posted');
    assert(custom.classList.contains('set') && custom.getAttribute('aria-checked') === 'true' && custom.dataset.value === '123456', 'Custom swatch does not show the picked colour');
    assert(swatches.children.length === doc.getElementById('iconTint').options.length, 'Picked colour added a second swatch');
    const keepsColour = (b) => b.classList.contains('set') && b.getAttribute('aria-checked') === 'false' && b.style.getPropertyValue('--swatch') === '#123456';
    swatches.querySelector('[data-value="60CDFF"]').click(); await settle();
    assert(keepsColour(custom), 'Custom swatch lost its colour when a palette colour was chosen');
    custom.click(); await settle();
    assert(frame.contentWindow.testMessages.filter(m => m.name === 'pickColor').at(-1).color === '123456', 'Colour dialog not opened on the kept colour');
    await settle();
    swatches.querySelector('[data-value=""]').click(); await settle();
    assert(keepsColour(custom), 'Custom swatch lost its colour when the tint was cleared');
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-a');
    doc.querySelector('#fenceTintSwatches .custom').click(); await settle(); await settle();
    const sent = frame.contentWindow.testMessages.filter(m => m.type === 'setFence').at(-1);
    assert(sent.prop === 'tint' && sent.value === '123456', 'Fence custom tint did not post setFence');
    assert(doc.querySelector('#fenceTintSwatches .custom').getAttribute('aria-checked') === 'true', 'Fence custom swatch not selected after refresh');
    doc.querySelector('#fenceTintSwatches [data-value=""]').click(); await settle();
    assert(keepsColour(doc.querySelector('#fenceTintSwatches .custom')), 'Fence custom swatch lost its colour');
  });
  await test('Title alignment is posted and previewed; fully transparent fences show their preset', async () => {
    doc.querySelector('[data-page=general]').click();
    const align = doc.querySelector('[data-bind=titleAlign]');
    assert(align.value === 'left', 'Saved alignment not shown');
    for (const value of ['center', 'right']) {
      align.value = value;
      align.dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true }));
      await settle();
      assert(current().settings.titleAlign === value, 'Alignment not posted: ' + value);
      const title = doc.querySelector('.tile-fluent .preview-title');
      assert(frame.contentWindow.getComputedStyle(title).textAlign === value, 'Preview title not aligned ' + value);
    }
    doc.querySelector('[data-page=fences]').click();
    change(doc, 'fenceSel', 'fence-b');
    assert(doc.querySelector('[data-fence=opacity]').value === 'transparent', 'Fully transparent preset not shown');
    change(doc, 'fenceSel', 'fence-a');
    change(doc, 'fenceOpacity', 'transparent'); await settle();
    const sent = frame.contentWindow.testMessages.filter(m => m.type === 'setFence').at(-1);
    assert(sent.id === 'fence-a' && sent.prop === 'opacity' && sent.value === 'transparent', 'Fully transparent not posted');
    assert(doc.querySelector('[data-fence=opacity]').value === 'transparent', 'Refreshed opacity not reflected');
  });
  await test('Tab bar position is shown and posted', async () => {
    doc.querySelector('[data-page=general]').click();
    const bar = doc.querySelector('[data-bind=tabBar]');
    assert(bar.value === 'top', 'Saved tab bar position not shown');
    for (const value of ['left', 'top']) {
      bar.value = value;
      bar.dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true }));
      await settle();
      assert(current().settings.tabBar === value, 'Tab bar position not posted: ' + value);
    }
  });
  await test('Icon name colour is shown and posted', async () => {
    doc.querySelector('[data-page=general]').click();
    const color = doc.querySelector('[data-bind=labelColor]');
    assert(color.value === 'auto', 'Saved icon name colour not shown');
    for (const value of ['white', 'black', 'auto']) {
      color.value = value;
      color.dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true }));
      await settle();
      assert(current().settings.labelColor === value, 'Icon name colour not posted: ' + value);
    }
  });
  await test('Rule rows are compact and the new-rule form opens when needed', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    const panel = doc.getElementById('nrPanel');
    assert(!panel.open, 'New-rule form open although rules exist');
    for (const row of doc.querySelectorAll('#ruleList .rule')) {
      assert(row.getBoundingClientRect().height <= 64, 'Rule row keeps dead space: ' + row.getBoundingClientRect().height);
    }
    change(doc, 'nrKind', 'ext'); change(doc, 'nrValue', ' ');
    await add(doc);
    assert(panel.open && doc.activeElement.id === 'nrValue', 'Validation error did not reveal the field');
  });
  // Rule editing (feedback #104): the row of a listed rule opens into the rule form in place.
  const edit = async (doc, id) => { doc.querySelector(`[data-row-id="${id}"] [data-act=edit]`).click(); await settle(); return doc.querySelector(`[data-row-id="${id}"] .redit`); };
  const setValue = (el, value) => { el.value = value; el.dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true })); };
  await test('A rule is edited in place: name, target and condition change; id, switch and place stay', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    current().rules.list[1].enabled = false;
    frame.contentWindow.testRefresh(); await settle();
    const form = await edit(doc, 'rule-b');
    assert(form && doc.activeElement === form.querySelector('.r-name'), 'Editor did not open with the name focused');
    assert(form.querySelector('.r-name').value === 'Images' && form.querySelector('.r-target').value === 'fence-b', 'Editor not filled');
    assert(![...form.querySelector('.r-target').options].some(o => o.value === 'portal'), 'Portal offered as a routing target');
    const blocks = form.querySelectorAll('.cond');
    assert(blocks.length === 1 && blocks[0].querySelector('.c-kind').value === 'ext' && blocks[0].querySelector('.c-text').value === 'png', 'Condition not filled');
    assert(blocks[0].querySelector('.c-del').hidden, 'The only condition can be removed');
    const main = doc.querySelector('main');
    assert(main.scrollWidth <= main.clientWidth + 1, 'Open editor overflows the narrow window');
    form.querySelector('.r-name').value = 'Pictures';
    setValue(form.querySelector('.r-target'), 'inbox');
    blocks[0].querySelector('.c-text').value = 'png, .jpg';
    form.querySelector('[data-act=save]').click(); await settle();
    const list = current().rules.list, r = list[1];
    assert(list.map(x => x.id).join() === 'rule-a,rule-b', 'Edited rule moved');
    assert(r.name === 'Pictures' && r.target === 'inbox' && r.enabled === false && r.priorityClass === 'type'
      && JSON.stringify(r.allOf) === '[{"cond":"ext","value":["png","jpg"]}]', 'Edit not saved: ' + JSON.stringify(r));
    assert(!doc.querySelector('#ruleList .redit'), 'Editor still open after saving');
    assert(doc.activeElement.dataset.act === 'edit' && doc.activeElement.closest('[data-row-id]').dataset.rowId === 'rule-b', 'Focus not back on the edit button');
    assert(doc.querySelector('[data-row-id="rule-b"] .conds').textContent.includes('png、jpg'), 'Row summary not updated');
  });
  await test('A host refresh keeps an open editor, its draft and focus; Cancel changes nothing', async () => {
    const form = await edit(doc, 'rule-a');
    form.querySelector('.r-name').value = 'Draft name';
    form.querySelector('.r-name').focus();
    frame.contentWindow.testRefresh(); await settle();
    assert(doc.querySelector('[data-row-id="rule-a"] .redit') === form && form.querySelector('.r-name').value === 'Draft name', 'Host refresh dropped the editor or its draft');
    assert(doc.activeElement === form.querySelector('.r-name'), 'Host refresh moved focus out of the editor');
    const before = JSON.stringify(current().rules);
    form.querySelector('[data-act=cancel]').click(); await settle();
    assert(JSON.stringify(current().rules) === before && current().rules.list[0].name === 'Documents', 'Cancel changed the rule');
    assert(!doc.querySelector('#ruleList .redit') && doc.activeElement.dataset.act === 'edit', 'Cancel did not close the editor onto the edit button');
  });
  await test('Every condition of a rule is shown and kept, one only the CLI writes as text', async () => {
    current().rules.list[0] = { id: 'rule-c', name: '待清理', enabled: true, target: { fence: 'fence-a' }, template: 'cleanup', priorityClass: 'type',
      allOf: [{ cond: 'type', value: ['installers', 'archives'] }, { cond: 'idleDays', value: { min: 30 } }, { cond: 'name', value: { op: 'is', value: 'a.zip' } }] };
    frame.contentWindow.testRefresh(); await settle();
    let form = await edit(doc, 'rule-c');
    const blocks = form.querySelectorAll('.cond');
    assert(blocks.length === 3, 'Not every condition shown');
    assert([...blocks[0].querySelectorAll('.c-cats input:checked')].map(i => i.value).join() === 'archives,installers', 'Type boxes not ticked');
    assert(blocks[1].querySelector('.c-kind').value === 'idle' && blocks[1].querySelector('.c-days').value === '30', 'Idle days not filled');
    assert(!blocks[2].querySelector('.c-kind') && blocks[2].textContent.includes('a.zip'), 'CLI-only condition not shown as its summary');
    assert([...blocks].every(b => !b.querySelector('.c-del').hidden), 'Conditions cannot be removed');
    form.querySelector('[data-act=save]').click(); await settle();
    let r = current().rules.list[0];
    assert(JSON.stringify(r.allOf) === '[{"cond":"type","value":["archives","installers"]},{"cond":"idleDays","value":{"min":30}},{"cond":"name","value":{"op":"is","value":"a.zip"}}]', 'Saving unchanged conditions changed them: ' + JSON.stringify(r.allOf));
    assert(r.id === 'rule-c' && r.template === 'cleanup' && r.priorityClass === 'type', 'Id, template tag or class lost');
    form = await edit(doc, 'rule-c');
    form.querySelectorAll('.cond')[2].querySelector('.c-del').click();
    form.querySelectorAll('.cond')[1].querySelector('.c-del').click();
    assert(form.querySelectorAll('.cond').length === 1 && form.querySelector('.cond .c-del').hidden && doc.activeElement === form.querySelector('.c-add'), 'Removing conditions misbehaved');
    form.querySelector('[data-act=save]').click(); await settle();
    r = current().rules.list[0];
    assert(r.allOf.length === 1 && r.allOf[0].cond === 'type' && r.name === '待清理', 'Removed conditions were saved');
  });
  await test('The new-rule form combines several conditions', async () => {
    rulesPage(doc);
    change(doc, 'nrKind', 'ext'); change(doc, 'nrValue', 'png');
    doc.getElementById('nrAddCond').click();
    const extra = doc.querySelector('#nrConds .cond:not(#nrCond)');
    assert(extra && !extra.querySelector('[id]:not(.c-hint)'), 'Added condition missing or reuses ids');
    assert(extra.querySelector('.c-text').getAttribute('aria-describedby') === extra.querySelector('.c-hint').id, 'Hint of the added condition not tied to its field');
    setValue(extra.querySelector('.c-kind'), 'startsWith');
    assert(extra.querySelector('.c-text').placeholder === 'IMG_' && extra.querySelector('.c-value').style.display === '', 'Added condition did not follow its kind');
    extra.querySelector('.c-text').value = 'IMG_';
    doc.getElementById('nrName').value = '';
    await add(doc);
    const r = current().rules.list[0];
    assert(JSON.stringify(r.allOf) === '[{"cond":"ext","value":["png"]},{"cond":"name","value":{"op":"startsWith","value":"IMG_"}}]' && r.priorityClass === 'type', 'Combined rule wrong: ' + JSON.stringify(r));
    assert(r.name === '扩展名：png，且 名称开头是“IMG_”', 'Unnamed rule not named after both conditions: ' + r.name);
    assert(!doc.querySelector('#nrConds .cond:not(#nrCond)'), 'Added condition left in the form');
  });
  await test('An editor error points at its own field and leaves the new-rule form closed', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    const form = await edit(doc, 'rule-a');
    form.querySelector('.c-text').value = ' ';
    form.querySelector('[data-act=save]').click(); await settle();
    assert(errorShown(doc) && doc.activeElement === form.querySelector('.c-text'), 'Error did not focus the editor field');
    assert(!doc.getElementById('nrPanel').open, 'Error opened the new-rule form');
    assert(current().rules.list[0].allOf[0].value[0] === 'txt', 'Invalid edit saved');
  });
  await test('A language switch keeps an open editor and its draft and translates it', async () => {
    const form = doc.querySelector('[data-row-id="rule-a"] .redit');
    form.querySelector('.c-text').value = 'md';
    form.querySelector('.c-add').click();
    doc.querySelector('[data-page=general]').click();
    change(doc, 'language', 'en'); await settle();
    doc.querySelector('[data-page=rules]').click();
    const now = doc.querySelector('[data-row-id="rule-a"] .redit'), text = frame.contentWindow.PecoFenceI18n.text;
    assert(now && now.querySelectorAll('.cond').length === 2 && now.querySelector('.c-text').value === 'md', 'Editor or draft lost');
    assert(doc.querySelector('[data-row-id="rule-a"] .re-head .t').textContent === text('编辑规则') && now.querySelector('[data-act=save]').textContent === text('保存'), 'Editor frame not translated');
    const added = now.querySelectorAll('.cond')[1];
    assert(added.querySelector('.c-kind option[value=ext]').textContent === text('扩展名（逗号分隔）') && added.querySelector('.c-cats label').textContent === text('程序'), 'Added condition not translated');
    assert(added.querySelector('.c-max').placeholder === text('不限') && added.querySelector('.c-text').getAttribute('aria-label') === text('条件值')
      && added.querySelector('.c-del').title === text('删除条件'), 'Placeholder or field name of the added condition not translated');
    change(doc, 'language', 'zh-CN'); await settle();
  });
  await test('Every page fits compact windows in both materials and colour modes', async () => {
    for (const width of [704, 480]) {
      frame.style.width = width + 'px';
      for (const material of ['fluent', 'liquidGlass']) {
      current().settings.themeStyle = material;
      for (const mode of ['dark', 'light']) {
        current().themeMode = mode;
        frame.contentWindow.testRefresh();
        for (const page of ['general', 'fences', 'rules', 'tools', 'layout', 'about']) {
          doc.querySelector(`[data-page=${page}]`).click();
          await settle();
          const main = doc.querySelector('main');
          assert(main.scrollWidth <= main.clientWidth + 1, `${page} overflows at ${width}px in ${material}/${mode}`);
          assert(doc.documentElement.scrollWidth <= width, `App overflows at ${width}px`);
        }
      }
      }
    }
    current().settings.themeStyle = 'fluent';
    frame.contentWindow.testRefresh();
    frame.style.width = '704px';
  });
  await test('Decorative icons are hidden from assistive technology after refresh', async () => {
    frame.contentWindow.testRefresh();
    for (const el of doc.querySelectorAll('.ic, .glyph')) {
      assert(el.getAttribute('aria-hidden') === 'true', 'Decorative glyph is announced');
    }
    assert(doc.getElementById('toast').getAttribute('role') === 'status', 'Status announcement missing');
  });
  // Rules the CLI wrote: `rule add --name-contains " (1)"` keeps the spaces, `--json` any value.
  const cliRule = allOf => { current().rules.list[0] = { id: 'rule-cli', name: 'From the CLI', enabled: true, target: { fence: 'fence-a' }, allOf, priorityClass: 'name' }; frame.contentWindow.testRefresh(); };
  await test('Values with spaces or quotes at either end are shown in quotes and saved unchanged', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    const allOf = [{ cond: 'name', value: { op: 'contains', value: ' (1)' } }, { cond: 'name', value: { op: 'endsWith', value: ' - 副本' } },
      { cond: 'name', value: { op: 'startsWith', value: '"draft"' } }, { cond: 'name', value: { op: 'notContains', value: 'backup' } },
      { cond: 'glob', value: ' *.tmp' }, { cond: 'shortcutTarget', value: { op: 'contains', value: ' steam ' } }];
    cliRule(allOf); await settle();
    const form = await edit(doc, 'rule-cli');
    const shown = [...form.querySelectorAll('.cond .c-text')].map(i => i.value);
    assert(JSON.stringify(shown) === JSON.stringify(['" (1)"', '" - 副本"', '\'"draft"\'', 'backup', '" *.tmp"', '" steam "']), 'Values not shown so they read back: ' + JSON.stringify(shown));
    form.querySelector('.r-name').value = 'Renamed';
    form.querySelector('[data-act=save]').click(); await settle();
    const r = current().rules.list[0];
    assert(r.name === 'Renamed' && JSON.stringify(r.allOf) === JSON.stringify(allOf), 'Saving changed the values: ' + JSON.stringify(r.allOf));
  });
  await test('Values the form would split or refuse stay as text and are kept, so the rule still saves', async () => {
    const odd = [{ cond: 'exactName', value: ['a,b.txt', 'c.txt'] }, { cond: 'idleDays', value: { min: 0 } },
      { cond: 'sizeMb', value: { min: null, max: null } }, { cond: 'createdTime', value: { from_min: 1500, to_min: 60 } }];
    const plain = [{ cond: 'exactName', value: ['todo.txt', 'notes.md'] }, { cond: 'idleDays', value: { min: 7 } }];
    cliRule([...odd, ...plain]); await settle();
    const form = await edit(doc, 'rule-cli'), blocks = [...form.querySelectorAll('.cond')];
    assert(blocks.slice(0, 4).every(b => !b.querySelector('.c-kind') && b.querySelector('.c-fixed')), 'A value the form cannot keep is editable');
    assert(blocks[0].textContent.includes('a,b.txt') && blocks[4].querySelector('.c-text').value === 'todo.txt, notes.md' && blocks[5].querySelector('.c-days').value === '7', 'Conditions not shown');
    form.querySelector('.r-name').value = 'Renamed again';
    form.querySelector('[data-act=save]').click(); await settle();
    const r = current().rules.list[0];
    assert(!doc.querySelector('#ruleList .redit') && !errorShown(doc), 'Save refused');
    assert(r.name === 'Renamed again' && JSON.stringify(r.allOf) === JSON.stringify([...odd, ...plain]), 'Saving changed the conditions: ' + JSON.stringify(r.allOf));
  });
  await test('Esc in an editor cancels like Cancel, except while an IME composes', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    const form = await edit(doc, 'rule-a'), name = form.querySelector('.r-name'), before = JSON.stringify(current().rules);
    name.value = 'Draft';
    const esc = isComposing => name.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true, isComposing }));
    esc(true); await settle();
    assert(doc.querySelector('[data-row-id="rule-a"] .redit') === form && name.value === 'Draft', 'Esc ending an IME composition closed the editor');
    esc(false); await settle();
    assert(!doc.querySelector('#ruleList .redit') && JSON.stringify(current().rules) === before, 'Esc did not cancel the edit');
    assert(doc.activeElement.dataset.act === 'edit' && doc.activeElement.closest('[data-row-id]').dataset.rowId === 'rule-a', 'Esc did not return focus to the edit button');
  });
  await test('One editor at a time: other edit buttons are off until it closes', async () => {
    const form = await edit(doc, 'rule-a');
    form.querySelector('.r-name').value = 'Draft';
    frame.contentWindow.testRefresh(); await settle();
    const other = doc.querySelector('[data-row-id="rule-b"] [data-act=edit]');
    assert(other.disabled, 'Another rule can be opened while one is being edited');
    other.click(); await settle();
    assert(doc.querySelector('[data-row-id="rule-a"] .redit') === form && form.querySelector('.r-name').value === 'Draft' && !doc.querySelector('[data-row-id="rule-b"] .redit'), 'The first draft was dropped');
    form.querySelector('[data-act=cancel]').click(); await settle();
    assert(!doc.querySelector('[data-row-id="rule-b"] [data-act=edit]').disabled, 'Edit buttons stay off after the editor closed');
  });
  await test('An editor whose fence is deleted shows the desktop fence, the same as when opened again', async () => {
    const form = await edit(doc, 'rule-b');
    current().fences = current().fences.filter(f => f.id !== 'fence-b');
    frame.contentWindow.testRefresh(); await settle();
    assert(form.querySelector('.r-target').value === 'inbox', 'Refresh picked another fence: ' + form.querySelector('.r-target').value);
    form.querySelector('[data-act=cancel]').click(); await settle();
    const again = await edit(doc, 'rule-b');
    assert(again.querySelector('.r-target').value === 'inbox', 'Reopened editor picked another fence: ' + again.querySelector('.r-target').value);
  });
  await test('A condition box with its cross lays out like the first one without it', async () => {
    doc = await reset();
    rulesPage(doc);
    change(doc, 'nrKind', 'ext');
    doc.getElementById('nrAddCond').click();
    const [first, added] = doc.querySelectorAll('#nrConds .cond');
    setValue(added.querySelector('.c-kind'), 'ext');
    assert(first.querySelector('.c-del').hidden && !added.querySelector('.c-del').hidden, 'Cross shown on the wrong box');
    const at = block => { const box = block.querySelector('.c-box').getBoundingClientRect(), value = block.querySelector('.c-text').getBoundingClientRect(); return [value.left - box.left, value.top - box.top, value.width].map(Math.round).join(); };
    const was = frame.style.width;
    for (const width of [704, 960]) {
      frame.style.width = width + 'px'; await settle();
      assert(at(first) === at(added), `Boxes differ at ${width}px: ${at(first)} vs ${at(added)}`);
    }
    const box = first.querySelector('.c-box').getBoundingClientRect(), value = first.querySelector('.c-text').getBoundingClientRect();
    assert(value.left > box.left + box.width / 3, 'Type and value no longer side by side in a wide window');
    frame.style.width = was;
  });
  await test('Idle days outside 1 to 36500 are refused with the range, in the editor and the new-rule form', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    const form = await edit(doc, 'rule-a'), before = JSON.stringify(current().rules);
    const block = form.querySelector('.cond');
    setValue(block.querySelector('.c-kind'), 'idle');
    const range = frame.contentWindow.PecoFenceI18n.format('闲置天数必须在 1 到 {0} 之间', 36500);
    for (const days of ['99999999999', '36501', '0']) {
      block.querySelector('.c-days').value = days;
      form.querySelector('[data-act=save]').click(); await settle();
      assert(errorShown(doc) && doc.getElementById('toastText').textContent === range && doc.activeElement === block.querySelector('.c-days'), days + ' idle days not refused with the range');
      assert(JSON.stringify(current().rules) === before && doc.querySelector('[data-row-id="rule-a"] .redit') === form, days + ' idle days saved or the editor closed');
    }
    block.querySelector('.c-days').value = '36500';
    form.querySelector('[data-act=save]').click(); await settle();
    assert(JSON.stringify(current().rules.list[0].allOf) === '[{"cond":"idleDays","value":{"min":36500}}]', '36500 idle days not saved');
    rulesPage(doc);
    change(doc, 'nrKind', 'idle'); change(doc, 'nrIdleDays', '99999999999');
    const count = ruleCount(doc);
    await add(doc);
    assert(ruleCount(doc) === count && errorShown(doc) && doc.activeElement.id === 'nrIdleDays', 'New-rule form accepted 99999999999 idle days');
  });
  await test('A rule list the host refuses comes back as stored, with an error instead of the saved toast', async () => {
    doc = await reset();
    doc.querySelector('[data-page=rules]').click();
    frame.contentWindow.testRefuseRules = true;
    doc.querySelector('[data-row-id="rule-a"] [data-act=toggle]').click(); await settle();
    const text = doc.getElementById('toastText').textContent;
    assert(errorShown(doc) && text === '没能保存这次改动，规则已恢复原样', 'No error after a refused list: ' + text);
    assert(doc.querySelector('[data-row-id="rule-a"] [data-act=toggle]').classList.contains('on'), 'The refused switch change stayed on the page');
    const form = await edit(doc, 'rule-a');
    form.querySelector('.r-name').value = 'Refused name';
    form.querySelector('[data-act=save]').click(); await settle();
    assert(errorShown(doc) && doc.querySelector('[data-row-id="rule-a"] .t').textContent === 'Documents', 'The refused rename stayed on the page');
    frame.contentWindow.testRefuseRules = false;
  });

  doc = await reset();
  rulesPage(doc);
  await test('Type and weekday rules validate empty groups and accept multiple selections', async () => {
    for (const [kind, group, values, label] of [
      ['type', 'nrCats', ['documents', 'images'], '类型：文档、图片'],
      ['weekday', 'nrWeek', ['0', '6'], '创建于周一、周日'],
    ]) {
      change(doc, 'nrKind', kind);
      const count = ruleCount(doc);
      await add(doc);
      assert(ruleCount(doc) === count && errorShown(doc), kind + ' accepted an empty group');
      for (const value of values) doc.querySelector(`#${group} input[value="${value}"]`).click();
      change(doc, 'nrName', '工作 <draft> & QA');
      await add(doc);
      assert(ruleCount(doc) === count + 1, kind + ' rejected selected values');
      assert(doc.querySelector('#ruleList .rule:first-child .t').textContent === '工作 <draft> & QA', 'Rule name lost punctuation');
      assert(doc.querySelector('#ruleList .rule:first-child .conds').textContent.includes(label), kind + ' displayed the wrong condition');
    }
  });
  await test('Name conditions reject empty values and describe accepted values', async () => {
    for (const [kind, value, label] of [
      ['contains', '截图', '名称包含“截图”'],
      ['notContains', '备份', '名称不包含“备份”'],
      ['startsWith', 'IMG_', '名称开头是“IMG_”'],
      ['endsWith', '_final', '名称结尾是“_final”'],
      ['exact', 'todo.txt， notes.md', '名称为：todo.txt、notes.md'],
      ['glob', '*.png', '匹配 *.png'],
    ]) {
      change(doc, 'nrKind', kind); change(doc, 'nrValue', ' ');
      const count = ruleCount(doc);
      await add(doc);
      assert(ruleCount(doc) === count && errorShown(doc), kind + ' accepted an empty condition');
      change(doc, 'nrValue', value);
      await add(doc);
      assert(ruleCount(doc) === count + 1, kind + ' rejected a valid condition');
      assert(doc.querySelector('#ruleList .rule:first-child .conds').textContent.includes(label), kind + ' displayed the wrong condition');
    }
  });
  await test('A value typed in quotes is saved without them (feedback #89)', async () => {
    assert(doc.getElementById('nrValueHint').textContent.includes('不用加引号'), 'The hint does not say quotes are not needed');
    assert(doc.getElementById('nrValue').getAttribute('aria-describedby') === 'nrValueHint', 'The hint is not tied to the input');
    for (const [kind, value, saved] of [
      ['contains', '"my report"', { cond: 'name', value: { op: 'contains', value: 'my report' } }],
      ['startsWith', "'my report'", { cond: 'name', value: { op: 'startsWith', value: 'my report' } }],
      ['endsWith', ' “my report” ', { cond: 'name', value: { op: 'endsWith', value: 'my report' } }],
      ['notContains', '‘old copy’', { cond: 'name', value: { op: 'notContains', value: 'old copy' } }],
      ['exact', `"my report.docx", 'notes 2.md'`, { cond: 'exactName', value: ['my report.docx', 'notes 2.md'] }],
      ['exact', '"a b.txt, c d.txt"', { cond: 'exactName', value: ['a b.txt', 'c d.txt'] }],
      ['glob', '"*my report*"', { cond: 'glob', value: '*my report*' }],
      ['target', '"Program Files"', { cond: 'shortcutTarget', value: { op: 'contains', value: 'Program Files' } }],
      ['ext', '"tar.gz"', { cond: 'ext', value: ['tar.gz'] }],
      // Quotes that are not one pair around the whole value stay.
      ['contains', `it's "draft`, { cond: 'name', value: { op: 'contains', value: `it's "draft` } }],
      ['contains', `"a" and "b"`, { cond: 'name', value: { op: 'contains', value: `"a" and "b"` } }],
    ]) {
      change(doc, 'nrKind', kind); change(doc, 'nrValue', value);
      await add(doc);
      const got = JSON.stringify(current().rules.list[0].allOf[0]);
      assert(got === JSON.stringify(saved), kind + ' ' + value + ' saved as ' + got);
    }
    change(doc, 'nrKind', 'contains'); change(doc, 'nrValue', '""');
    const count = ruleCount(doc);
    await add(doc);
    assert(ruleCount(doc) === count && errorShown(doc), 'A pair of quotes alone was accepted');
  });
  await test('Shortcut rules and file/folder filters can route to the desktop inbox', async () => {
    change(doc, 'nrTarget', 'inbox');
    for (const [kind, value, label] of [
      ['target', 'Program Files\\Steam', '快捷方式目标包含“Program Files\\Steam”'],
      ['files', '', '仅文件'],
      ['folders', '', '仅文件夹'],
    ]) {
      change(doc, 'nrKind', kind); change(doc, 'nrValue', value);
      const count = ruleCount(doc);
      await add(doc);
      assert(ruleCount(doc) === count + 1, kind + ' was not added');
      assert(doc.querySelector('#ruleList .rule:first-child .conds').textContent === label + ' → 放入“Desktop”', 'Wrong condition or destination');
      assert(current().rules.list[0].target === 'inbox', 'Inbox target serialized as a virtual fence');
    }
  });
  await test('Deleting the final snapshot preserves a useful keyboard focus target', async () => {
    doc.querySelector('[data-page=layout]').click();
    const button = doc.querySelector('#snapList [data-act=del]');
    button.focus(); button.click(); await settle();
    assert(doc.activeElement.id === 'snapName', 'Snapshot deletion lost focus');
    assert(doc.getElementById('snapList').textContent.includes('尚无快照'), 'Missing empty-state message');
  });
  await test('A same-display swap is rejected and distinct selections survive refresh', async () => {
    change(doc, 'swapB', 'one');
    const messages = () => frame.contentWindow.testMessages.filter(m => m.name === 'swapMonitors');
    doc.getElementById('swapDo').click(); await settle();
    assert(messages().length === 0 && errorShown(doc), 'Same-display swap was submitted');
    change(doc, 'swapA', 'two'); change(doc, 'swapB', 'one');
    frame.contentWindow.testRefresh();
    doc.getElementById('swapDo').click(); await settle();
    assert(messages().length === 1 && messages()[0].a === 'two' && messages()[0].b === 'one', 'Display selections reset or swapped incorrectly');
  });

  await test('The startup switch explains the Store states only Windows Settings or a policy can change', async () => {
    doc.querySelector('[data-page=general]').click();
    const note = doc.querySelector('[data-bind=autostart]').closest('.card').querySelector('.s');
    for (const [stateName, words] of [['DisabledByUser', '任务管理器'], ['DisabledByPolicy', '组策略关闭'], ['EnabledByPolicy', '组策略打开']]) {
      current().startupState = stateName;
      frame.contentWindow.testRefresh(); await settle();
      assert(!note.hidden && note.offsetHeight > 0 && note.textContent.includes(words), 'No note for ' + stateName);
    }
    for (const stateName of ['Enabled', 'Disabled', null]) {
      current().startupState = stateName;
      frame.contentWindow.testRefresh(); await settle();
      assert(note.hidden && !note.textContent, 'Note shown for ' + stateName);
    }
  });
  await test('The Store rating button appears only in the Store edition; the manual and support buttons post', async () => {
    doc.querySelector('[data-page=about]').click();
    const rate = doc.getElementById('rateStore'), star = doc.getElementById('starRepo'), manual = doc.getElementById('openManual');
    current().storeEdition = false;
    frame.contentWindow.testRefresh(); await settle();
    assert(rate.hidden && !star.hidden, 'Rating offered outside the Store edition');
    current().storeEdition = true;
    frame.contentWindow.testRefresh(); await settle();
    assert(!rate.hidden && rate.offsetWidth > 0, 'Rating button missing in the Store edition');
    const actions = () => frame.contentWindow.testMessages.filter(m => m.type === 'action').map(m => m.name);
    manual.click(); rate.click(); star.click(); await settle();
    assert(actions().slice(-3).join() === 'openManual,rateInStore,openRepository', 'About buttons did not post their actions');
    current().storeEdition = false;
    frame.contentWindow.testRefresh();
  });
  await test('Desktop icons can be restored, hidden again, and restored repeatedly', async () => {
    doc.querySelector('[data-page=about]').click();
    const restore = doc.getElementById('repairIcons'), hide = doc.getElementById('hideIconsAgain');
    for (let i = 0; i < 3; i++) {
      hide.click(); await settle();
      assert(current().settings.hideRealIcons && hide.disabled, 'Hide did not synchronize state');
      assert(doc.getElementById('desktopIcons').value === 'hide', 'General list out of sync');
      restore.click(); await settle();
      assert(!current().settings.hideRealIcons && !hide.disabled, 'Restore has no reverse entry');
      assert(doc.getElementById('desktopIconsStatus').textContent.includes('已显示'), 'Wrong visible state');
    }
  });
  await test('Desktop icon status follows Explorer even when the saved preference differs', async () => {
    current().settings.hideRealIcons = true;
    current().desktopIconsHidden = false;
    frame.contentWindow.testRefresh(); await settle();
    assert(!doc.getElementById('hideIconsAgain').disabled, 'Repair hidden state must remain available');
    assert(doc.getElementById('desktopIconsStatus').textContent.includes('已显示'), 'Preference presented as actual state');
    doc.getElementById('hideIconsAgain').click(); await settle();
    assert(current().desktopIconsHidden, 'Idempotent hide request was ignored');
  });

  // The page posts with fetch; the stub records each request and answers with `status`
  // (a thrown error stands for no connection).
  const stubFetch = (status) => {
    const requests = [];
    frame.contentWindow.fetch = async (url, init) => {
      requests.push({ url, init, body: JSON.parse(init.body) });
      if (status instanceof Error) throw status;
      return new frame.contentWindow.Response(null, { status });
    };
    return requests;
  };
  const feedbackPage = async doc => { doc.querySelector('[data-page=about]').click(); await settle(); };
  const feedbackOutcomes = () => frame.contentWindow.testMessages.filter(m => m.name === 'feedbackResult').map(m => m.status);

  await test('About page shows the update card for each edition and state', async () => {
    doc = await reset();
    doc.querySelector('[data-page=about]').click(); await settle();
    const card = doc.getElementById('updateCard'), button = doc.getElementById('updateAction');
    const status = doc.getElementById('updateStatus');
    const lastAction = () => frame.contentWindow.testMessages.filter(m => m.type === 'action').at(-1)?.name;
    const autoCard = doc.getElementById('updateAutoCard'), autoSwitch = doc.querySelector('[data-bind=autoCheckUpdates]');
    assert(card.hidden, 'Update card shown without update info (Store edition)');
    assert(autoCard.hidden, 'The automatic check switch shown in the Store edition');
    const show = async update => { current().update = update; frame.contentWindow.testUpdate(update); await settle(); };
    const installer = { edition: 'installer', version: '0.1.4', hasSetup: true };
    await show({ ...installer, state: 'idle', version: null });
    assert(!card.hidden && button.textContent === '检查更新' && !button.disabled && !button.classList.contains('accent') && status.textContent === '', 'Idle card wrong');
    assert(!autoCard.hidden && autoSwitch.getAttribute('aria-checked') === 'true', 'The automatic check switch not shown on');
    autoSwitch.click(); await settle();
    assert(current().settings.autoCheckUpdates === false, 'The automatic check switch did not post');
    autoSwitch.click(); await settle();
    assert(current().settings.autoCheckUpdates === true, 'The automatic check switch did not turn back on');
    button.click(); assert(lastAction() === 'checkUpdate', 'Check not requested');
    await show({ ...installer, state: 'checking', version: null });
    assert(button.disabled && status.textContent === '正在检查更新…', 'Checking state wrong');
    await show({ ...installer, state: 'available' });
    assert(status.textContent.startsWith('有新版本 0.1.4。') && button.textContent === '下载并安装' && button.classList.contains('accent'), 'Available (installer) wrong: ' + status.textContent);
    button.click(); assert(lastAction() === 'installUpdate', 'Install not requested');
    await show({ ...installer, state: 'downloading' });
    assert(button.disabled && button.textContent === '下载并安装' && status.textContent === '正在下载新版本…', 'Downloading state wrong');
    await show({ ...installer, state: 'available', hasSetup: false });
    assert(button.textContent === '打开下载页', 'A release without setup offered an install');
    await show({ ...installer, edition: 'zip', state: 'available' });
    assert(status.textContent === '有新版本 0.1.4' && button.textContent === '打开下载页', 'ZIP card wrong: ' + status.textContent);
    button.click(); assert(lastAction() === 'openReleasePage', 'Release page not requested');
    await show({ ...installer, state: 'downloadFailed', version: null });
    assert(status.textContent === '下载失败，请稍后再试' && button.textContent === '检查更新', 'Download failure wrong');
    await show({ ...installer, state: 'checkFailed', version: null });
    assert(status.textContent === '检查更新失败，请稍后再试' && button.textContent === '检查更新', 'Check failure wrong');
    frame.contentWindow.testRefresh(); await settle();
    assert(status.textContent === '检查更新失败，请稍后再试', 'State refresh lost the update card');
    change(doc, 'language', 'en'); await settle();
    assert(status.textContent === "Couldn't check for updates. Try again later." && button.textContent === 'Check for updates', 'English card wrong: ' + status.textContent);
  });

  await test('Feedback page shows what will be sent and posts the report', async () => {
    doc = await reset();
    await feedbackPage(doc);
    assert(doc.getElementById('page-about').classList.contains('on'), 'About page (with the feedback form) not shown');
    assert(frame.contentWindow.testMessages.some(m => m.name === 'feedbackInfo'), 'Diagnostics were not requested');
    const preview = doc.getElementById('fbPreview').textContent;
    assert(preview.includes('windows: build 26200') && preview.includes('config loaded fences=4'), 'Preview misses the diagnostics: ' + preview);
    assert(doc.getElementById('fbCrashNote').hidden, 'Crash note shown without a crash');
    const requests = stubFetch(201);
    doc.getElementById('fbSend').click(); await settle();
    assert(errorShown(doc) && !requests.length, 'An empty message was sent');
    doc.getElementById('fbMessage').value = '  Fences vanish after sleep  ';
    doc.getElementById('fbEmail').value = 'not-an-email';
    doc.getElementById('fbSend').click(); await settle();
    assert(errorShown(doc) && !requests.length, 'An invalid email was accepted');
    doc.getElementById('fbEmail').value = 'jane@example.com';
    doc.querySelector('input[name=fbKind][value=idea]').checked = true;
    doc.getElementById('fbSend').click(); await settle();
    assert(requests.length === 1, 'Report not posted');
    const { url, init, body } = requests[0];
    assert(url === 'https://feedback.test/feedback' && init.method === 'POST' && init.headers['Content-Type'] === 'application/json', 'Wrong request');
    assert(body.kind === 'idea' && body.message === 'Fences vanish after sleep' && body.email === 'jane@example.com', 'Wrong report: ' + init.body);
    assert(body.app === '0.1.4' && body.language === 'zh-CN' && body.crash === false, 'Wrong facts: ' + init.body);
    assert(body.system.windows === 'build 26200' && body.log.includes('config loaded'), 'Diagnostics missing');
    assert(feedbackOutcomes().at(-1) === 201, 'Outcome not reported to the host');
    assert(!errorShown(doc) && doc.getElementById('fbMessage').value === '', 'Sent message was not cleared');
  });
  await test('The log is sized again for the message before a report goes out', async () => {
    doc = await reset();
    await feedbackPage(doc);
    const asked = () => frame.contentWindow.testMessages.filter(m => m.name === 'feedbackInfo');
    assert(asked().at(-1).messageBytes === 0, 'First request does not say the message is empty');
    const requests = stubFetch(201);
    doc.getElementById('fbMessage').value = '栅栏在外接显示器后错位';
    const before = asked().length;
    doc.getElementById('fbSend').click(); await settle();
    assert(asked().length === before + 1 && asked().at(-1).messageBytes === 33, 'Not asked again with the UTF-8 length: ' + JSON.stringify(asked().at(-1)));
    assert(requests.length === 1 && requests[0].body.log.includes('config loaded'), 'Report not sent after the new log arrived');
  });
  await test('Turning the diagnostic log off sends only the message and version', async () => {
    doc = await reset();
    await feedbackPage(doc);
    doc.getElementById('fbAttach').click(); await settle();
    assert(doc.getElementById('fbAttach').getAttribute('aria-checked') === 'false', 'Switch state not announced');
    assert(doc.getElementById('fbPreviewBox').hidden, 'Preview still offered with the log off');
    const requests = stubFetch(201);
    doc.getElementById('fbMessage').value = 'Just an idea';
    doc.getElementById('fbSend').click(); await settle();
    const body = requests[0].body;
    assert(body.system === null && body.log === null && body.language === null && body.email === null, 'Diagnostics sent with the log off: ' + requests[0].init.body);
    assert(body.app === '0.1.4', 'Version missing');
  });
  await test('Failed sends keep the message and say why', async () => {
    doc = await reset();
    await feedbackPage(doc);
    doc.getElementById('fbMessage').value = 'Keep me';
    for (const [status, text] of [[429, '频繁'], [502, '502'], [new Error('offline'), '网络']]) {
      stubFetch(status);
      doc.getElementById('fbSend').click(); await settle();
      assert(errorShown(doc) && doc.getElementById('toastText').textContent.includes(text), 'Unexpected message for ' + status + ': ' + doc.getElementById('toastText').textContent);
      assert(doc.getElementById('fbMessage').value === 'Keep me', 'Message lost after ' + status);
      assert(!doc.getElementById('fbSend').disabled, 'Send stays disabled after ' + status);
    }
    assert(feedbackOutcomes().join() === '429,502,0', 'Outcomes not reported: ' + feedbackOutcomes().join());
  });
  await test('A crash report opens the feedback page with Problem selected and the crash note', async () => {
    doc = await reset();
    current().crashed = true;
    frame.contentWindow.testShowFeedback(true); await settle();
    assert(doc.getElementById('page-about').classList.contains('on'), 'About page (with the feedback form) not shown');
    assert(doc.querySelector('input[name=fbKind][value=bug]').checked, 'Problem not selected');
    assert(!doc.getElementById('fbCrashNote').hidden, 'Crash note hidden');
    assert(doc.activeElement === doc.getElementById('fbMessage'), 'Message field not focused');
    const heading = doc.getElementById('feedbackHeading').getBoundingClientRect(), view = doc.querySelector('main').getBoundingClientRect();
    assert(heading.top >= view.top - 1 && heading.top < view.top + view.height / 2, 'Feedback section not scrolled into view');
    const requests = stubFetch(201);
    doc.getElementById('fbMessage').value = 'It crashed while dragging';
    doc.getElementById('fbSend').click(); await settle();
    assert(requests[0].body.crash === true, 'Crash flag not sent');
    assert(doc.getElementById('fbCrashNote').hidden, 'Crash note stays after the report went out');
    current().crashed = false;
  });
  await test('Script errors reach the host as their type and position, without the message', async () => {
    doc = await reset();
    const failing = doc.createElement('script');
    failing.textContent = "throw new TypeError('C:/Users/jane.doe/taxes.pdf')";
    doc.body.append(failing);
    await settle();
    const sent = frame.contentWindow.testMessages.filter(m => m.name === 'pageError');
    assert(sent.length === 1 && sent[0].kind === 'TypeError' && sent[0].line > 0, 'Script error not reported: ' + JSON.stringify(sent));
    assert(!JSON.stringify(sent).includes('jane'), 'The error message reached the host: ' + JSON.stringify(sent));
  });

  const hotkeyText = () => [...doc.querySelectorAll('#peekHotkeyKeys kbd')].map(k => k.textContent).join('+');
  const press = (target, key, keyCode, mods = {}) => target.dispatchEvent(new frame.contentWindow.KeyboardEvent('keydown', { key, keyCode, bubbles: true, cancelable: true, ...mods }));
  const sent = name => win().testMessages.filter(m => m.name === name);
  const win = () => frame.contentWindow;
  const choice = () => doc.getElementById('peekHotkeyChoice');
  const choiceText = () => choice().selectedOptions[0]?.textContent;
  // Picks an entry of the shortcut list the way a user does (the list sends `change`).
  const choose = value => { choice().focus(); choice().value = value; choice().dispatchEvent(new frame.contentWindow.Event('change', { bubbles: true })); };
  await test('The Peek shortcut list records a combination through 自定义… and shows what the host saved', async () => {
    doc = await reset();
    doc.querySelector('[data-page=peek]').click();
    const box = doc.getElementById('peekHotkey'), edit = doc.getElementById('peekHotkeyEdit'), help = doc.getElementById('peekHotkeyHelp');
    assert([...choice().options].map(o => o.textContent).join('|') === 'Ctrl + Alt + 空格|Win + Alt + 空格|Ctrl + Shift + 空格|双击 Ctrl|自定义…', 'Choices wrong: ' + [...choice().options].map(o => o.textContent).join('|'));
    assert(choiceText() === 'Ctrl + Alt + 空格' && edit.hidden && !doc.getElementById('peekHotkeyWin').checked, 'Active shortcut not shown: ' + choiceText());
    assert(help.textContent.includes('双击 Ctrl') && help.textContent.includes('自定义…'), 'Help does not mention the other choices: ' + help.textContent);
    choose('custom'); await settle();
    assert(!edit.hidden && box.classList.contains('recording') && doc.activeElement === box && sent('peekHotkeyRecording').at(-1)?.on === true, 'Recording did not start');
    assert(choice().value === 'custom' && help.textContent.includes('Esc'), 'Recording not explained: ' + help.textContent);
    press(box, 'Control', 17, { ctrlKey: true });
    assert(hotkeyText() === 'Ctrl', 'Held modifiers are not shown: ' + hotkeyText());
    press(box, 'k', 75, { ctrlKey: true, altKey: true }); await settle();
    const set = sent('setPeekHotkey').at(-1);
    assert(set && set.mods === 3 && set.vk === 75, 'Wrong combination sent: ' + JSON.stringify(set));
    assert(!box.classList.contains('recording') && edit.hidden && choice().value === '3:75' && choiceText() === 'Ctrl + Alt + K', 'Saved shortcut not shown: ' + choiceText());
    assert(current().settings.peek.customHotkey === 'Ctrl+Alt+K', 'Shortcut not saved');
    assert(choice().options.length === 6, 'The recorded combination is not in the list');
  });
  await test('A refused shortcut keeps recording with the reason; Esc cancels', async () => {
    doc = await reset();
    doc.querySelector('[data-page=peek]').click();
    const box = doc.getElementById('peekHotkey'), help = doc.getElementById('peekHotkeyHelp'), winBox = doc.getElementById('peekHotkeyWin');
    choose('custom'); await settle();
    press(box, 'k', 75); await settle();
    assert(help.classList.contains('error') && help.textContent.includes('Ctrl、Alt 或 Win') && box.classList.contains('recording'), 'No-modifier refusal not shown: ' + help.textContent);
    // Win comes from the checkbox: Windows keeps Win+Space before the page sees it.
    winBox.click(); await settle();
    assert(winBox.checked && box.classList.contains('recording') && doc.activeElement === box && hotkeyText() === 'Win', 'Win checkbox did not start a Win recording');
    press(box, ' ', 32, { shiftKey: true }); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 12, 'Win was not added');
    assert(help.textContent.includes('已被 Windows 或其他程序占用') && box.classList.contains('recording') && choice().value === 'custom', 'Taken combination not reported: ' + help.textContent);
    press(box, 'Escape', 27); await settle();
    assert(!box.classList.contains('recording') && sent('peekHotkeyRecording').at(-1).on === false, 'Esc did not end the recording');
    assert(!winBox.checked && doc.getElementById('peekHotkeyEdit').hidden && choice().value === '3:32' && !help.classList.contains('error'), 'Cancel did not restore the shortcut');
    assert(current().settings.peek.customHotkey == null, 'A refused shortcut was saved');
  });
  await test('Keys that stop arriving during a recording suggest the combination is taken', async () => {
    doc = await reset();
    const box = doc.getElementById('peekHotkey'), help = doc.getElementById('peekHotkeyHelp');
    const quiet = () => new Promise(resolve => setTimeout(resolve, 3200));
    choose('custom'); await quiet();
    assert(!help.classList.contains('caution') && help.textContent.includes('Esc'), 'Hint shown before any key: ' + help.textContent);
    // Another program's hotkey keeps the key itself: only the modifiers arrive.
    press(box, 'Control', 17, { ctrlKey: true }); await quiet();
    assert(help.classList.contains('caution') && help.textContent.includes('其他程序占用') && box.classList.contains('recording'), 'Silence not explained: ' + help.textContent);
    press(box, 'Alt', 18, { ctrlKey: true, altKey: true });
    assert(!help.classList.contains('caution') && help.textContent.includes('Esc'), 'Hint stayed after a key: ' + help.textContent);
    await quiet();
    assert(help.classList.contains('caution'), 'Hint did not come back');
    press(box, 'Escape', 27); await settle();
    assert(!box.classList.contains('recording') && !help.classList.contains('caution'), 'Esc left the hint: ' + help.textContent);
    // A refusal of the previous key does not hide the hint for the next, taken combination.
    choose('custom'); await settle();
    press(box, 'k', 75); await settle();
    assert(help.classList.contains('error'), 'No-modifier refusal not shown: ' + help.textContent);
    press(box, 'Control', 17, { ctrlKey: true }); await quiet();
    assert(help.classList.contains('caution') && help.textContent.includes('其他程序占用'), 'Old refusal hid the hint: ' + help.textContent);
    press(box, 'Escape', 27); await settle();
  });
  await test('Combinations from the list set the shortcut; a fallback says which one is in use', async () => {
    doc = await reset();
    doc.querySelector('[data-page=peek]').click();
    choose('9:32'); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 9 && choice().value === '9:32' && hotkeyText() === 'Win+Alt+空格', 'Choice not applied: ' + choiceText());
    assert(doc.activeElement === choice() && doc.getElementById('peekHotkeyEdit').hidden, 'The list lost the focus or the recording box showed');
    // A taken combination from the list: the list stays on the shortcut in use and says why.
    win().testTaken = '6:32';
    choose('6:32'); await settle();
    win().testTaken = null;
    const help = doc.getElementById('peekHotkeyHelp');
    assert(sent('setPeekHotkey').at(-1).mods === 6 && choice().value === '9:32' && help.classList.contains('error') && help.textContent.includes('已被 Windows 或其他程序占用'), 'Refused choice not reported: ' + help.textContent);
    // A shortcut with Win does not tick the box for the next recording.
    const box = doc.getElementById('peekHotkey'), winBox = doc.getElementById('peekHotkeyWin');
    choose('custom'); await settle();
    assert(!winBox.checked && !help.classList.contains('error'), 'Win checkbox ticked or old error left at a new recording');
    winBox.click(); await settle();
    assert(winBox.checked && hotkeyText() === 'Win', 'Ticking Win during a recording failed');
    winBox.click(); await settle();
    assert(!winBox.checked && box.classList.contains('recording') && doc.activeElement === box, 'Clearing Win ended the recording');
    press(box, 'k', 75, { ctrlKey: true, altKey: true }); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 3 && choiceText() === 'Ctrl + Alt + K', 'Win stayed in the next recording');
    // A 0.2.0 file with Win+Space: Windows holds it, Ctrl+Alt+Space works instead.
    current().peekHotkey.chosen = { mods: 8, vk: 32, keys: ['Win', '空格'], label: 'Win+空格' };
    current().peekHotkey.active = current().peekHotkey.presets[0];
    win().testRefresh(); await settle();
    assert(choiceText() === 'Win + 空格' && help.classList.contains('caution') && help.textContent.includes('Win+空格') && help.textContent.includes('Ctrl+Alt+空格'), 'Fallback not explained: ' + help.textContent);
    current().peekHotkey.active = null;
    win().testRefresh(); await settle();
    assert(help.classList.contains('error') && choiceText() === 'Win + 空格', 'All-taken state not shown: ' + help.textContent);
  });
  await test('双击 Ctrl is one of the shortcut choices; a combination replaces it', async () => {
    doc = await reset();
    const box = doc.getElementById('peekHotkey'), edit = doc.getElementById('peekHotkeyEdit'), help = doc.getElementById('peekHotkeyHelp');
    const before = win().testMessages.length;
    choose('double');
    const patch = win().testMessages.slice(before).find(m => m.type === 'patchSettings');
    assert(patch && patch.settings.peek.doubleCtrl === true && !win().testMessages.slice(before).some(m => m.name === 'setPeekHotkey'), 'Double Ctrl not posted as peek.doubleCtrl');
    await settle();
    assert(choice().value === 'double' && edit.hidden && help.textContent.includes('按两下') && !help.className.includes('error'), 'Double Ctrl not shown: ' + choice().value + ' ' + help.textContent);
    // The combination is not registered meanwhile: that is no error.
    current().peekHotkey.active = null;
    win().testRefresh(); await settle();
    assert(choice().value === 'double' && !help.classList.contains('error'), 'An unregistered combination reported as taken: ' + help.textContent);
    // 自定义… and Esc go back to 双击 Ctrl.
    choose('custom'); await settle();
    assert(!edit.hidden && box.classList.contains('recording'), 'Recording did not start from 双击 Ctrl');
    press(box, 'Escape', 27); await settle();
    assert(choice().value === 'double' && edit.hidden && current().settings.peek.doubleCtrl === true, 'Esc did not go back to 双击 Ctrl');
    // Peek off: the list is off too.
    doc.querySelector('[data-bind="peek.enabled"]').click(); await settle();
    assert(choice().disabled && choice().value === 'double', 'List usable while Peek is off');
    doc.querySelector('[data-bind="peek.enabled"]').click(); await settle();
    // A combination from the list replaces it.
    choose('6:32'); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 6 && current().settings.peek.doubleCtrl === false && choice().value === '6:32', 'A combination did not replace 双击 Ctrl');
    assert(!help.textContent.includes('按两下'), 'Help still describes 双击 Ctrl: ' + help.textContent);
  });
  await test('The Windows desktop shortcut starts unset, records a combination and clears', async () => {
    doc = await reset();
    const box = doc.getElementById('wdHotkey'), clear = doc.getElementById('wdHotkeyClear'), help = doc.getElementById('wdHotkeyHelp');
    const wdText = () => [...doc.querySelectorAll('#wdHotkeyKeys kbd')].map(k => k.textContent).join('+');
    assert(wdText() === '' && doc.getElementById('wdHotkeyKeys').textContent.includes('未设置') && clear.hidden, 'An unset shortcut is not shown as unset');
    box.focus(); box.click(); await settle();
    assert(box.classList.contains('recording') && sent('windowsDesktopHotkeyRecording').at(-1)?.on === true, 'Recording did not start');
    // Peek's shortcut is refused and the recording goes on.
    press(box, ' ', 32, { ctrlKey: true, altKey: true }); await settle();
    assert(help.classList.contains('error') && help.textContent.includes('速览栅栏') && box.classList.contains('recording'), 'Peek shortcut not refused: ' + help.textContent);
    press(box, 'd', 68, { ctrlKey: true, altKey: true }); await settle();
    const set = sent('setWindowsDesktopHotkey').at(-1);
    assert(set && set.mods === 3 && set.vk === 68, 'Wrong combination sent: ' + JSON.stringify(set));
    assert(!box.classList.contains('recording') && wdText() === 'Ctrl+Alt+D' && !clear.hidden && !help.classList.contains('error'), 'Saved shortcut not shown: ' + wdText());
    assert(current().settings.windowsDesktopHotkey === 'Ctrl+Alt+D', 'Shortcut not saved');
    // Taken by another program after a restart: said under the box.
    current().windowsDesktopHotkey.registered = false;
    win().testRefresh(); await settle();
    assert(help.classList.contains('error') && help.textContent.includes('Ctrl+Alt+D'), 'Taken shortcut not explained: ' + help.textContent);
    clear.click(); await settle();
    assert(sent('setWindowsDesktopHotkey').at(-1).vk === undefined && clear.hidden && wdText() === '', 'Clear did not remove the shortcut');
    assert(current().settings.windowsDesktopHotkey === null, 'Clear not saved');
  });
  await test('The AI card copies a prompt that names this edition\'s CLI and opens the guide', async () => {
    doc = await reset();
    current().cliCommand = 'C:\\PecoFence\\pecofence-cli.exe';
    win().testRefresh(); await settle();
    let copied = '';
    doc.execCommand = cmd => { copied = cmd === 'copy' ? doc.querySelector('textarea[readonly]')?.value || '' : ''; return true; };
    doc.getElementById('copyAiPrompt').click(); await settle();
    assert(copied.includes('`C:\\PecoFence\\pecofence-cli.exe skill`') && copied.includes('PecoFence'), 'Prompt does not name the CLI: ' + copied);
    assert(!doc.querySelector('textarea[readonly]') && doc.getElementById('toast').classList.contains('on'), 'Copy left its field or showed no toast');
    doc.getElementById('openCliDocs').click(); await settle();
    assert(sent('openCliDocs').length === 1, 'CLI guide not opened');
  });
  await test('All eleven languages switch live without changing user names or draft inputs', async () => {
    doc = await reset();
    const win = frame.contentWindow;
    current().fences[0].title = '名称';
    current().rules.list[0].name = '保存快照';
    current().snapshots[0].name = '桌面 {1}';
    win.testRefresh();
    doc.getElementById('nrName').value = 'My rule 中文 {0}';
    change(doc, 'nrKind', 'type');
    doc.querySelector('#nrCats input').checked = true;
    for (const language of ['en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it', 'zh-CN', 'system']) {
      change(doc, 'language', language);
      await settle();
      const locale = language === 'system' ? 'en' : language;
      assert(doc.documentElement.lang === locale, 'Wrong document language: ' + language);
      assert(doc.querySelector('[data-page=general]').textContent.includes(win.PecoFenceI18n.text('常规')), 'Navigation was not translated: ' + language);
      const sizeTicks = [...doc.querySelectorAll('#page-fences .slider-ticks b')].slice(0, 4).map(b => b.textContent);
      assert(sizeTicks.join() === ['小', '中', '较大', '大'].map(k => win.PecoFenceI18n.text('图标大小\x04' + k)).join() && !sizeTicks.join().includes('\x04'), 'Icon size ticks not translated: ' + language);
      assert(doc.getElementById('nrName').value === 'My rule 中文 {0}' && doc.querySelector('#nrCats input').checked, 'Draft rule input lost: ' + language);
      assert(current().fences[0].title === '名称' && current().rules.list[0].name === '保存快照' && current().snapshots[0].name === '桌面 {1}', 'User data changed: ' + language);
      assert(doc.querySelector('#ruleList .t').textContent === '保存快照', 'User rule name translated: ' + language);
      assert(win.PecoFenceI18n.format('{0} 个栅栏', 'title {1}').includes('title {1}'), 'User placeholder was reinterpreted');
      for (const page of ['general', 'fences', 'rules', 'tools', 'layout', 'about']) {
        doc.querySelector(`[data-page=${page}]`).click();
        const main = doc.querySelector('main');
        assert(main.scrollWidth <= main.clientWidth + 1, language + '/' + page + ' overflows');
        for (const control of doc.querySelectorAll('.page.on select,.page.on button')) {
          const box = control.getBoundingClientRect();
          assert(box.right <= main.getBoundingClientRect().right + 1, language + '/' + page + ' control outside content');
        }
      }
    }
  });

  // ---- 智能栅栏编辑器 (the page in the editor's own window, with a mock host) ----
  const openEditor = async (query) => {
    await new Promise(resolve => { frame.onload = resolve; frame.src = '/editor?' + query; });
    await settle(); await settle();
    return frame.contentDocument;
  };
  const ew = () => frame.contentWindow;
  const posted = (type, since = 0) => ew().testMessages.slice(since).filter(m => m.type === type);
  const rowsOf = d => [...d.querySelectorAll('#seWindow .se-row')];
  const kindsOf = d => rowsOf(d).map(r => r.dataset.kind).join(',');
  const seKey = (d, k, opts = {}) => d.activeElement.dispatchEvent(new (ew().KeyboardEvent)('keydown', { key: k, bubbles: true, cancelable: true, ...opts }));
  const sePick = (el, value) => { el.value = value; el.dispatchEvent(new (ew().Event)('change', { bubbles: true })); };
  const seType = (el, text) => { el.focus(); el.value = text; el.dispatchEvent(new (ew().Event)('input', { bubbles: true })); };
  const saved = (since = 0) => posted('editorSave', since).at(-1);
  await test('Smart fence editor: a new fence starts blank and Enter creates it only when every row is filled in', async () => {
    doc = await openEditor('fence=new');
    const w = doc.getElementById('seWindow');
    assert(doc.body.classList.contains('se-only') && ew().getComputedStyle(doc.querySelector('.app')).display === 'none', 'The settings show behind the editor');
    assert(w.querySelector('h2').textContent === '新建智能栅栏' && w.querySelector('.se-ok').textContent === '创建', 'Not the new-fence editor');
    assert(doc.activeElement === w.querySelector('.se-name') && w.querySelector('.se-name').value === '', 'The name box does not have the cursor');
    assert(kindsOf(doc) === 'where,name' && w.querySelector('.se-where').value === 'all', 'Rows of a new fence: ' + kindsOf(doc));
    assert(w.querySelectorAll('[data-preset]').length === 5 && w.querySelector('[data-preset=screenshots]').disabled, 'Templates wrong (a template without its folder must be off)');
    assert(!posted('editorDirty').length, 'Unsaved before any change');
    seKey(doc, 'Enter'); await settle();
    assert(!saved() && w.querySelector('.se-msg').textContent === '请输入关键字' && rowsOf(doc)[1].classList.contains('bad'), 'An empty row was saved: ' + w.querySelector('.se-msg').textContent);
    assert(doc.activeElement === rowsOf(doc)[1].querySelector('.se-text'), 'The cursor is not in the empty box');
    seType(rowsOf(doc)[1].querySelector('.se-text'), ' 发票 ');
    assert(posted('editorDirty').length === 1 && posted('editorDirty')[0].dirty === true, 'editorDirty not posted once');
    assert(!w.querySelector('.se-msg').textContent && !rowsOf(doc)[1].classList.contains('bad'), 'The error stays after typing');
    seKey(doc, 'Enter'); await settle();
    const s = saved();
    assert(s && s.title === '' && JSON.stringify(s.spec) === JSON.stringify({ template: null, folders: [], subfolders: true, match: 'all', allOf: [{ cond: 'name', value: { op: 'contains', value: '发票' } }], withinDays: null, name: null, everything: null, limit: 200 }), 'Saved: ' + JSON.stringify(s));
  });
  await test('Smart fence editor: a template fills the rows and the name, and keeps a name the user typed', async () => {
    doc = await openEditor('fence=new');
    const w = doc.getElementById('seWindow'), name = w.querySelector('.se-name');
    w.querySelector('[data-preset=downloads]').click(); await settle();
    assert(name.value === '最近 7 天下载' && kindsOf(doc) === 'where,sub,time,type', 'Downloads template: ' + name.value + ' ' + kindsOf(doc));
    assert(rowsOf(doc)[0].querySelector('.se-path').textContent === 'C:\\Users\\me\\Downloads' && rowsOf(doc)[2].querySelector('.se-days').value === '7', 'Folder or days wrong');
    w.querySelector('.se-ok').click(); await settle();
    assert(JSON.stringify(saved().spec) === JSON.stringify({ template: 'downloads', folders: ['C:\\Users\\me\\Downloads'], subfolders: true, match: 'all', allOf: [{ cond: 'filesOnly', value: null }], withinDays: 7, name: null, everything: null, limit: 200 }), 'Saved: ' + JSON.stringify(saved().spec));
    w.querySelector('[data-preset=recent]').click(); await settle();
    assert(name.value === '最近打开的文件' && w.querySelector('.se-limit select').value === '40' && rowsOf(doc)[1].querySelector('.se-sub').value === 'off', 'The second template did not take the name, limit or subfolders');
    seType(name, 'My big files');
    w.querySelector('[data-preset=large]').click(); await settle();
    assert(name.value === 'My big files' && kindsOf(doc) === 'where,type,size', 'A typed name was replaced or rows wrong: ' + kindsOf(doc));
    const size = rowsOf(doc)[2];
    assert(size.querySelector('.se-n1').value === '1' && size.querySelector('.se-unit').value === 'gb', '1024 MB not shown as 1 GB');
    w.querySelector('.se-ok').click(); await settle();
    assert(saved().title === 'My big files' && JSON.stringify(saved().spec.allOf) === '[{"cond":"filesOnly","value":null},{"cond":"sizeMb","value":{"min":1024,"max":null}}]' && saved().spec.folders.length === 0, 'Saved: ' + JSON.stringify(saved()));
  });
  await test('Smart fence editor: every kind of row reads and saves back unchanged', async () => {
    doc = await openEditor('fence=every');
    const w = doc.getElementById('seWindow');
    assert(w.querySelector('h2').textContent === '编辑智能栅栏' && w.querySelector('.se-ok').textContent === '好' && !w.querySelector('.se-presets'), 'Not the edit editor');
    assert(kindsOf(doc) === 'where,where,sub,time,type,ext,name,size,size,created,weekday,target,idle,keep,name,everything', 'Rows: ' + kindsOf(doc));
    assert(w.querySelector('.se-all').value === 'all' && w.querySelector('.se-always').hidden, '全部 not shown, or the note of 任一 shown');
    assert(rowsOf(doc)[13].querySelector('.se-kind').disabled && rowsOf(doc)[13].textContent.includes('*.bak'), 'A condition the editor has no row for is not kept as it is');
    w.querySelector('.se-ok').click(); await settle();
    const want = { template: null, folders: ['D:\\Work', 'E:\\Archive'], subfolders: false, match: 'all', allOf: ew().testEditor.spec.allOf, withinDays: 30, name: 'bg', everything: 'invoice|receipt', limit: 500 };
    assert(JSON.stringify(saved().spec) === JSON.stringify(want), 'Changed on the way: ' + JSON.stringify(saved().spec));
    assert(saved().title === 'Every row' && !posted('editorDirty').length, 'Title or dirty wrong');
  });
  await test('Smart fence editor: kind menus, + and -, the once-only kinds, 文件 alone and 介于', async () => {
    doc = await openEditor('fence=dl');
    const w = doc.getElementById('seWindow');
    const time = rowsOf(doc)[2];
    assert(rowsOf(doc)[3].querySelector('.se-kind option[value=time]').disabled && !time.querySelector('.se-kind option[value=time]').disabled, '时间 offered twice');
    rowsOf(doc)[3].querySelector('.se-plus').click(); await settle();
    assert(kindsOf(doc) === 'where,sub,time,type,name' && doc.activeElement === rowsOf(doc)[4].querySelector('.se-text'), '+ did not add a name row with the cursor');
    sePick(rowsOf(doc)[4].querySelector('.se-kind'), 'ext'); await settle();
    assert(kindsOf(doc) === 'where,sub,time,type,ext', 'Kind change: ' + kindsOf(doc));
    seType(rowsOf(doc)[4].querySelector('.se-text'), '.PNG, jpg  tar.gz');
    sePick(rowsOf(doc)[3].querySelector('.se-kind'), 'size'); await settle();
    sePick(rowsOf(doc)[3].querySelector('.se-op'), 'between'); await settle();
    const size = rowsOf(doc)[3];
    assert(size.querySelectorAll('input[type=number]').length === 2, '介于 has no second number');
    size.querySelector('.se-n1').value = '5'; size.querySelector('.se-n2').value = '2';
    w.querySelector('.se-ok').click(); await settle();
    assert(w.querySelector('.se-msg').textContent === '最大大小不能小于最小大小' && !saved(), 'A wrong size range was saved');
    size.querySelector('.se-n2').value = '2'; size.querySelectorAll('.se-unit')[1].value = 'gb';
    sePick(rowsOf(doc)[0].querySelector('.se-where'), 'all'); await settle();
    assert(kindsOf(doc) === 'where,time,size,ext', '整台电脑 left other 位置 or 子文件夹 rows: ' + kindsOf(doc));
    w.querySelector('.se-ok').click(); await settle();
    const s = saved().spec;
    assert(s.folders.length === 0 && JSON.stringify(s.allOf) === '[{"cond":"sizeMb","value":{"min":5,"max":2048}},{"cond":"ext","value":["PNG","jpg","tar.gz"]}]' && s.template === 'downloads', 'Saved: ' + JSON.stringify(s));
    // - removes a row, never the last one (the 位置 row stays).
    while (rowsOf(doc).length > 1) { rowsOf(doc).at(-1).querySelector('.se-minus').click(); await settle(); }
    rowsOf(doc)[0].querySelector('.se-minus').click(); await settle();
    assert(rowsOf(doc).length === 1 && rowsOf(doc)[0].dataset.kind === 'where', 'The last row went');
    // 文件 goes alone among the types.
    rowsOf(doc)[0].querySelector('.se-plus').click(); await settle();
    sePick(rowsOf(doc)[1].querySelector('.se-kind'), 'type'); await settle();
    const chip = v => rowsOf(doc)[1].querySelector(`.se-cats input[value=${v}]`);
    const tick = v => { chip(v).checked = true; chip(v).dispatchEvent(new (ew().Event)('change', { bubbles: true })); };
    tick('images'); tick('music'); tick('files');
    assert(chip('files').checked && !chip('images').checked && !chip('music').checked && chip('files').parentElement.classList.contains('on'), '文件 kept other types');
    tick('video');
    assert(!chip('files').checked && chip('video').checked, 'Another type kept 文件');
  });
  await test('Smart fence editor: 位置 asks the host for a folder; several folders are searched together', async () => {
    doc = await openEditor('fence=new');
    const w = doc.getElementById('seWindow');
    const mark = ew().testMessages.length;
    sePick(rowsOf(doc)[0].querySelector('.se-where'), 'folder'); await settle(); await settle();
    assert(posted('editorPickFolder', mark).length === 1, 'No folder picker asked for');
    assert(rowsOf(doc)[0].querySelector('.se-path').textContent === 'D:\\Picked', 'The picked folder is not shown');
    rowsOf(doc)[0].querySelector('.se-plus').click(); await settle();
    ew().testPickPath = null;
    sePick(rowsOf(doc)[1].querySelector('.se-kind'), 'where'); await settle();
    rowsOf(doc)[1].querySelector('.se-pick').click(); await settle(); await settle();
    assert(rowsOf(doc)[1].querySelector('.se-path').classList.contains('none'), 'A cancelled pick filled the row');
    w.querySelector('.se-ok').click(); await settle();
    assert(w.querySelector('.se-msg').textContent === '请选择文件夹' && doc.activeElement === rowsOf(doc)[1].querySelector('.se-pick'), 'A row without its folder was not pointed at');
    ew().testPickPath = 'E:\\Two';
    rowsOf(doc)[1].querySelector('.se-pick').click(); await settle(); await settle();
    seType(rowsOf(doc)[2].querySelector('.se-text'), 'x');
    sePick(w.querySelector('.se-all'), 'any'); await settle();
    w.querySelector('.se-ok').click(); await settle();
    const s = saved().spec;
    assert(JSON.stringify(s.folders) === '["D:\\\\Picked","E:\\\\Two"]' && s.match === 'any' && s.subfolders === true, 'Saved: ' + JSON.stringify(s));
  });
  await test('Smart fence editor: Esc cancels, Enter on a button is that button, a host error shows', async () => {
    doc = await openEditor('fence=dl');
    const w = doc.getElementById('seWindow');
    w.querySelector('.se-cancel').focus();
    seKey(doc, 'Enter'); await settle();
    assert(!saved(), 'Enter on 取消 saved');
    rowsOf(doc)[0].querySelector('.se-kind').focus();
    seKey(doc, 'Enter'); await settle();
    assert(saved(), 'Enter on a menu did not save');
    seKey(doc, 'Escape'); await settle();
    assert(posted('editorCancel').length === 1, 'Esc did not cancel');
    w.querySelector('.se-cancel').click(); await settle();
    assert(posted('editorCancel').length === 2, '取消 did not cancel');
    ew().testReceive({ type: 'editorError', text: '找不到文件夹 D:\\Gone' }); await settle();
    assert(w.querySelector('.se-msg').textContent === '找不到文件夹 D:\\Gone', 'Host error not shown');
    const errors = ew().testMessages.filter(m => m.name === 'pageError');
    assert(!errors.length, 'Page error: ' + JSON.stringify(errors));
  });
  await test('Smart fence editor: 任一 with Everything syntax refuses a row only PecoFence tests, and says which', async () => {
    doc = await openEditor('fence=new');
    const w = doc.getElementById('seWindow');
    sePick(rowsOf(doc)[1].querySelector('.se-kind'), 'everything'); await settle();
    seType(rowsOf(doc)[1].querySelector('.se-text'), 'invoice');
    rowsOf(doc)[1].querySelector('.se-plus').click(); await settle();
    sePick(rowsOf(doc)[2].querySelector('.se-kind'), 'idle'); await settle();
    sePick(w.querySelector('.se-all'), 'any'); await settle();
    await new Promise(r => setTimeout(r, 420));
    const asked = posted('editorCount').length;
    assert(w.querySelector('.se-count').textContent.includes('Everything 语法不能和这一行一起用'), 'The count does not say why: ' + w.querySelector('.se-count').textContent);
    w.querySelector('.se-ok').click(); await settle();
    assert(!saved() && w.querySelector('.se-msg').textContent === '选「任一」时，Everything 语法不能和这一行一起用' && rowsOf(doc)[2].classList.contains('bad'), 'Saved or the row not marked: ' + w.querySelector('.se-msg').textContent);
    for (const [kind, ok] of [['target', false], ['ext', true]]) {
      sePick(rowsOf(doc)[2].querySelector('.se-kind'), kind); await settle();
      if (kind === 'target' || kind === 'ext') seType(rowsOf(doc)[2].querySelector('.se-text'), 'x');
      w.querySelector('.se-ok').click(); await settle();
      assert(!!saved() === ok, kind + ': saved=' + !!saved());
    }
    // 全部 with the same rows is fine.
    const n = posted('editorSave').length;
    sePick(rowsOf(doc)[2].querySelector('.se-kind'), 'idle'); await settle();
    sePick(w.querySelector('.se-all'), 'all'); await settle();
    w.querySelector('.se-ok').click(); await settle();
    assert(posted('editorSave').length === n + 1 && saved().spec.match === 'all', '全部 was refused');
    assert(asked >= 0 && !ew().testMessages.some(m => m.name === 'pageError'), 'page error');
  });
  await test('Smart fence editor: the live count follows the search, a moment after the last change', async () => {
    doc = await openEditor('fence=dl');
    const w = doc.getElementById('seWindow'), line = () => w.querySelector('.se-count').textContent;
    await new Promise(r => setTimeout(r, 50));
    assert(posted('editorCount').length === 1 && line().includes('现在找到 37 个文件') && line().includes('由 Everything 搜索'), 'First count: ' + line());
    assert(JSON.stringify(posted('editorCount')[0].spec.folders) === '["C:\\\\Users\\\\me\\\\Downloads"]', 'Counted another search');
    // Three quick changes are one request.
    rowsOf(doc)[3].querySelector('.se-plus').click();
    for (const text of ['r', 're', 'rep']) seType(rowsOf(doc)[4].querySelector('.se-text'), text);
    await new Promise(r => setTimeout(r, 120));
    assert(posted('editorCount').length === 1 && w.querySelector('.se-count').classList.contains('busy') === false, 'Asked before the typing paused');
    await new Promise(r => setTimeout(r, 400));
    assert(posted('editorCount').length === 2 && posted('editorCount')[1].spec.allOf.at(-1).value.value === 'rep', 'Not one request after typing: ' + posted('editorCount').length);
    // Only the newest answer shows.
    ew().testHoldCount = true;
    seType(rowsOf(doc)[4].querySelector('.se-text'), 'repo'); await new Promise(r => setTimeout(r, 420));
    seType(rowsOf(doc)[4].querySelector('.se-text'), 'report'); await new Promise(r => setTimeout(r, 420));
    const [older, newer] = ew().testHeld;
    ew().testReceive({ ...newer, n: 4 }); await settle();
    ew().testReceive({ ...older, n: 99 }); await settle();
    assert(line().includes('现在找到 4 个文件') && !line().includes('99'), 'An older answer replaced the newer one: ' + line());
    ew().testHoldCount = false;
    // Only the newest files checked, the folder read and where it stopped, more than the
    // limit, Everything needed (for the whole computer, or for the syntax of a folder search).
    ew().testCount = () => ({ source: 'everything', n: 1234, checkedNewest: 5000 });
    seType(rowsOf(doc)[4].querySelector('.se-text'), 'x'); await new Promise(r => setTimeout(r, 420));
    assert(line().includes('现在找到 1234 个文件') && line().includes('只检查了最新的 5000 个') && line().includes('栅栏里显示最新的 200 个') && !line().includes('约'), 'Newest checked: ' + line());
    ew().testCount = () => ({ source: 'folder', n: 6, readOnly: 20000 });
    seType(rowsOf(doc)[4].querySelector('.se-text'), 'y'); await new Promise(r => setTimeout(r, 420));
    assert(line().includes('现在找到 6 个文件') && line().includes('Everything 没有运行') && line().includes('只读了其中的 20000 个'), 'Folder read: ' + line());
    ew().testCount = () => ({ source: 'needsEverything', n: 0 });
    rowsOf(doc)[4].querySelector('.se-plus').click(); await settle();
    sePick(rowsOf(doc)[5].querySelector('.se-kind'), 'everything'); await settle();
    seType(rowsOf(doc)[5].querySelector('.se-text'), 'invoice'); await new Promise(r => setTimeout(r, 420));
    assert(line().includes('用了 Everything 语法，需要先打开 Everything'), 'Folder with Everything syntax: ' + line());
    rowsOf(doc)[5].querySelector('.se-minus').click(); await settle();
    sePick(rowsOf(doc)[0].querySelector('.se-where'), 'all'); await new Promise(r => setTimeout(r, 420));
    assert(line().includes('要搜索整台电脑，需要先打开 Everything'), 'Everything needed: ' + line());
    // An unfinished row is not counted.
    const asked = posted('editorCount').length;
    seType(rowsOf(doc).at(-1).querySelector('.se-text'), ''); await new Promise(r => setTimeout(r, 420));
    assert(posted('editorCount').length === asked && line().includes('填好条件后'), 'An unfinished search was counted: ' + line());
    ew().testCount = null;
  });
  await test('Smart fence editor: unchanged rows keep their conditions as saved; an origin row is named; a location stays', async () => {
    doc = await openEditor('fence=odd');
    const w = doc.getElementById('seWindow');
    await new Promise(r => setTimeout(r, 420));
    assert(w.querySelector('.se-count').textContent.includes('「来源」不能作为智能栅栏的条件'), 'The count does not name the origin row: ' + w.querySelector('.se-count').textContent);
    w.querySelector('.se-ok').click(); await settle();
    const origin = rowsOf(doc).find(r => r.dataset.kind === 'keep' && r.textContent.includes('来源'));
    assert(!saved() && origin.classList.contains('bad') && w.querySelector('.se-msg').textContent.includes('请删掉这一行'), 'The origin row was not named');
    origin.querySelector('.se-minus').click(); await settle();
    w.querySelector('.se-ok').click(); await settle();
    assert(JSON.stringify(saved().spec.allOf) === '[{"cond":"filesOnly","value":null,"plain":true},{"cond":"ext","value":[".psd"]}]', 'Unchanged rows changed: ' + JSON.stringify(saved().spec.allOf));
    // A changed row is read again.
    seType(rowsOf(doc).find(r => r.dataset.kind === 'ext').querySelector('.se-text'), 'psd, ai');
    w.querySelector('.se-ok').click(); await settle();
    assert(JSON.stringify(saved().spec.allOf[1]) === '{"cond":"ext","value":["psd","ai"]}', 'A changed row was not read: ' + JSON.stringify(saved().spec.allOf));
    // The only 位置 row cannot be removed; turned into another kind, 整台电脑 comes back on top.
    const where = rowsOf(doc)[0];
    assert(where.dataset.kind === 'where' && where.querySelector('.se-minus').disabled, 'The only location row can be removed');
    sePick(where.querySelector('.se-kind'), 'name'); await settle();
    assert(rowsOf(doc)[0].dataset.kind === 'where' && rowsOf(doc)[0].querySelector('.se-where').value === 'all', 'No location row after a kind change: ' + kindsOf(doc));
  });
  await test('Smart fence editor: a change made elsewhere stays said until 好 or 取消; plain files count as a rough row', async () => {
    doc = await openEditor('fence=dl');
    const w = doc.getElementById('seWindow');
    ew().testReceive({ type: 'editorNotice', text: '这个栅栏的条件刚在别处改过，按「好」会换成这里的条件' }); await settle();
    seType(w.querySelector('.se-name'), 'typed after');
    await new Promise(r => setTimeout(r, 420));
    assert(w.querySelector('.se-msg').textContent.includes('刚在别处改过'), 'The notice went with the next edit: ' + w.querySelector('.se-msg').textContent);
    // A smart spec writes plain files as plainFiles: with 任一 and Everything syntax it is refused.
    ew().testReceive({ type: 'editor', fence: 'p', isNew: false, title: 'P', templates: [], everything: { state: 'running' },
      spec: { folders: ['D:\\P'], subfolders: true, match: 'any', allOf: [{ cond: 'plainFiles', value: null }], everything: 'invoice', limit: 200 } }); await settle();
    await new Promise(r => setTimeout(r, 420));
    assert(w.querySelector('.se-count').textContent.includes('Everything 语法不能和这一行一起用') && !w.querySelector('.se-msg').textContent, 'plainFiles not refused, or an old notice kept: ' + w.querySelector('.se-count').textContent);
    w.querySelector('.se-ok').click(); await settle();
    assert(rowsOf(doc).find(r => r.dataset.kind === 'type').classList.contains('bad'), 'The plain files row is not marked');
  });
  await test('Smart fence editor: a late count of the fence shown before does not show on the next one', async () => {
    doc = await openEditor('fence=dl');
    const w = doc.getElementById('seWindow');
    await new Promise(r => setTimeout(r, 100));
    ew().testHoldCount = true; ew().testHeld = [];
    seType(w.querySelector('.se-name'), 'x'); await new Promise(r => setTimeout(r, 420));
    const late = ew().testHeld.at(-1);
    ew().testReceive({ type: 'editor', fence: 'other', isNew: false, title: 'Other', spec: { folders: ['D:\\Other'], subfolders: true, limit: 200 }, templates: [], everything: { state: 'running' } }); await settle();
    ew().testReceive({ ...late, n: 99 }); await settle();
    assert(w.querySelector('.se-name').value === 'Other' && !w.querySelector('.se-count').textContent.includes('99'), 'The old fence\'s count shows on the new one: ' + w.querySelector('.se-count').textContent);
    const mine = ew().testHeld.at(-1);
    ew().testReceive({ ...mine, n: 5 }); await settle();
    assert(w.querySelector('.se-count').textContent.includes('现在找到 5 个文件'), 'The new fence\'s count does not show');
    ew().testHoldCount = false;
  });
  await test('Smart fence editor: fits its window in every language and keeps the rows when the language changes', async () => {
    doc = await openEditor('fence=every');
    frame.style.width = '680px'; frame.style.height = '560px';
    try {
      const w = doc.getElementById('seWindow');
      seType(w.querySelector('.se-name'), 'Kept name');
      rowsOf(doc)[6].querySelector('.se-text').value = 'kept text';
      for (const language of ['en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it', 'zh-CN']) {
        const catalogs = JSON.parse(doc.getElementById('fixture-catalogs').textContent);
        ew().testReceive({ ...ew().testState, locale: language, translations: catalogs[language] || {} }); await settle();
        const w2 = doc.getElementById('seWindow');
        assert(w2.scrollWidth <= w2.clientWidth + 1, language + ': the editor overflows sideways');
        for (const el of w2.querySelectorAll('.se-row select, .se-row button, .se-foot button, .se-foot select')) {
          const box = el.getBoundingClientRect();
          assert(box.right <= w2.getBoundingClientRect().right + 1, language + ': a control is outside the window');
        }
        assert(w2.querySelector('.se-foot .se-ok').getBoundingClientRect().bottom <= 560, language + ': 好 is below the window');
        assert(w2.querySelector('.se-name').value === 'Kept name' && rowsOf(doc)[6].querySelector('.se-text').value === 'kept text' && rowsOf(doc).length === 16, language + ': the rows were not kept');
        assert(w2.querySelector('h2').textContent === ew().PecoFenceI18n.text('编辑智能栅栏'), language + ': title not translated');
      }
    } finally { frame.style.width = ''; frame.style.height = ''; }
    doc = await reset();
  });

  // The settings window review of 2026-11-12.
  await test('Drop-down lists: the open list is styled by the page, a closed list keeps the system keys, scrollbars are thin', async () => {
    doc = await reset();
    if (!win().CSS.supports('appearance', 'base-select')) return;
    const align = doc.querySelector('[data-bind=titleAlign]'), css = el => win().getComputedStyle(el);
    assert(css(align).appearance === 'base-select', 'The list is not styled by the page');
    assert(css(align.options[0]).minHeight === '36px', 'Rows of an open list are not 36 px: ' + css(align.options[0]).minHeight);
    assert(css(doc.getElementById('iconTint')).display === 'none', 'A hidden list shows');
    assert(align.getBoundingClientRect().height === 32 && align.getBoundingClientRect().width === 160, 'The closed box changed size');
    align.focus();
    press(align, 'ArrowDown'); assert(align.value === 'center', 'Arrow down on a closed list: ' + align.value);
    press(align, 'End'); assert(align.value === 'right', 'End on a closed list: ' + align.value);
    press(align, 'ArrowDown'); assert(align.value === 'right', 'Arrow down past the last choice: ' + align.value);
    press(align, 'ArrowUp'); press(align, 'Home'); assert(align.value === 'left', 'Home on a closed list: ' + align.value);
    assert(win().testMessages.at(-1).settings.titleAlign === 'left' && win().testMessages.filter(m => m.type === 'patchSettings').length === 4, 'Each key that changed the choice saves once');
    press(align, 'ArrowDown', 40, { altKey: true }); assert(align.value === 'left', 'Alt+Down belongs to the list (it opens it)');
    for (const sel of ['nav', '.search-list', 'textarea', '.fb-preview']) {
      assert(win().getComputedStyle(doc.querySelector(sel), '::-webkit-scrollbar').width === '12px', sel + ' has the default scrollbar');
    }
  });

  await test('查找设置: the selected row is tinted in both modes, data rows are not listed, common words find their setting', async () => {
    const rgb = text => text.match(/[\d.]+/g).slice(0, 3).map(Number);
    const lum = c => { const [r, g, b] = rgb(c).map(v => { v /= 255; return v <= .03928 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4; }); return .2126 * r + .7152 * g + .0722 * b; };
    const ratio = (a, b) => (Math.max(lum(a), lum(b)) + .05) / (Math.min(lum(a), lum(b)) + .05);
    for (const mode of ['dark', 'light']) {
      doc = await reset();
      change(doc, 'language', 'zh-CN');
      doc.querySelector('[data-bind=theme]').value = mode; doc.querySelector('[data-bind=theme]').dispatchEvent(new (win().Event)('change', { bubbles: true })); await settle();
      const box = doc.getElementById('settingsSearch'), list = doc.getElementById('settingsSearchList');
      const type = text => { box.focus(); box.value = text; box.dispatchEvent(new (win().Event)('input', { bubbles: true })); };
      const titles = () => [...list.querySelectorAll('[role=option] b')].map(b => b.textContent);
      type('图标');
      const rows = list.querySelectorAll('[role=option]'), bg = el => win().getComputedStyle(el).backgroundColor;
      // The list's own fill shows through an unselected row.
      const contrast = ratio(bg(rows[0]), win().getComputedStyle(list).backgroundColor);
      assert(contrast >= 1.2, mode + ': the selected row is ' + contrast.toFixed(2) + ' : 1 against the list');
      assert(bg(rows[1]) === 'rgba(0, 0, 0, 0)', mode + ': an unselected row has a fill');
      if (mode === 'light') continue;
      type('快照');
      assert(titles().includes('把当前布局存为快照') && !titles().includes('Before changes'), 'A snapshot is listed: ' + titles().join(' | '));
      type('2026');
      assert(!titles().length, 'A daily backup is listed: ' + titles().join(' | '));
      type('Documents');
      assert(!titles().includes('Documents'), 'A rule is listed: ' + titles().join(' | '));
      for (const [word, title] of [['帮助', '使用手册'], ['教程', '使用手册'], ['关于', '使用手册'], ['版本号', '使用手册'], ['命令行', '让 AI 助手整理桌面'], ['CLI', '让 AI 助手整理桌面'],
        ['终端', '让 AI 助手整理桌面'], ['撤销', '把当前布局存为快照'], ['dark', '颜色模式'], ['light', '颜色模式'], ['导出', '导出 / 导入配置文件'], ['写法说明', '用自己的脚本']]) {
        type(word);
        assert(titles().includes(title), word + ' finds: ' + titles().join(' | '));
      }
      box.dispatchEvent(new (win().KeyboardEvent)('keydown', { key: 'Escape', keyCode: 27, bubbles: true, cancelable: true }));
    }
    for (const [language, word, title] of [['en', 'help', 'Manual'], ['en', 'command line', 'Let an AI assistant tidy your desktop'], ['ja', 'ヘルプ', null], ['de', 'Hilfe', null], ['ru', 'справка', null]]) {
      change(doc, 'language', language); await settle();
      const box = doc.getElementById('settingsSearch');
      box.focus(); box.value = word; box.dispatchEvent(new (win().Event)('input', { bubbles: true }));
      const found = [...doc.querySelectorAll('#settingsSearchList [role=option] b')].map(b => b.textContent);
      assert(found.length && (!title || found.includes(title)), language + ' ' + word + ' finds: ' + found.join(' | '));
    }
    doc = await reset();
  });

  await test('Keyboard: the nav is one Tab stop, arrow keys move in it, Enter or Space shows the page', async () => {
    doc = await reset();
    const rows = [...doc.querySelectorAll('nav a')], stops = () => rows.filter(a => a.tabIndex === 0).map(a => a.dataset.page).join();
    assert(stops() === 'general', 'Tab stops in the nav at the start: ' + stops());
    rows[0].focus();
    press(rows[0], 'ArrowDown', 40); assert(doc.activeElement === rows[1], 'Arrow down did not move to the next row');
    press(rows[1], 'End', 35); assert(doc.activeElement === rows[6], 'End did not move to the last row');
    press(rows[6], 'ArrowDown', 40); assert(doc.activeElement === rows[6], 'Arrow down left the last row');
    press(rows[6], 'Home', 36); press(rows[0], 'ArrowUp', 38); assert(doc.activeElement === rows[0], 'Home / Arrow up at the first row');
    assert(doc.getElementById('page-general').classList.contains('on') && stops() === 'general', 'Arrow keys alone changed the page');
    press(rows[0], 'ArrowDown', 40); press(rows[1], 'ArrowDown', 40);
    press(rows[2], ' ', 32);
    assert(doc.getElementById('page-fences').classList.contains('on'), 'Space did not show the page');
    assert(stops() === 'fences' && doc.activeElement === rows[2], 'The Tab stop did not follow the page: ' + stops());
    // What Tab reaches next: the first control after the row that can take the focus.
    const next = [...doc.querySelectorAll('nav a, main input, main select, main button, main [tabindex]')].filter(el => el.tabIndex >= 0 && el.getClientRects().length);
    assert(next[next.indexOf(rows[2]) + 1] === doc.getElementById('fenceSel'), 'Tab after the nav does not reach the page');
    doc.getElementById('settingsSearch').focus();
    assert(next.indexOf(rows[2]) === 0, 'More than one Tab stop between the search box and the page');
    doc.querySelector('[data-page=general]').click();
    assert(stops() === 'general', 'A click did not move the Tab stop: ' + stops());
  });

  await test('栅栏: the swatches of 色调 and 标签页颜色 stay beside their text at the default window width', async () => {
    doc = await reset();
    frame.style.width = '944px';
    try {
      win().testState.tintPalette = ['红', '橙', '黄', '绿', '青', '蓝', '紫', '粉', '灰'].map((name, i) => ({ name, hex: ['E74856', 'F7630C', 'FCE100', '10893E', '00B7C3', '0078D4', '8764B8', 'E3008C', '7A7574'][i] }));
      win().testRefresh(); await settle();
      doc.querySelector('[data-page=fences]').click();
      for (const language of ['zh-CN', 'en', 'de', 'ru', 'ja']) {
        change(doc, 'language', language); await settle();
        for (const fence of ['fence-a', 'fence-b']) {
          change(doc, 'fenceSel', fence); await settle();
          for (const id of ['fenceTintSwatches', 'fenceTabColorSwatches']) {
            const row = doc.getElementById(id), card = row.closest('.card');
            if (!card.getClientRects().length) continue;
            assert(row.children.length === 11, id + ' has ' + row.children.length + ' swatches');
            const text = card.querySelector('.l').getBoundingClientRect(), box = row.getBoundingClientRect();
            assert(box.height < 40 && box.top < text.bottom && box.left >= text.right, language + ' ' + id + ': the swatches wrap under the text');
          }
        }
      }
    } finally { frame.style.width = ''; }
    doc = await reset();
  });

  await test('主题风格: a tile caption that wraps breaks after the dot, never inside the Chinese name', async () => {
    doc = await reset();
    try {
      const text = [...doc.querySelector('.tile-glass .cap').childNodes].find(n => n.nodeType === 3);
      const lineOf = i => { const r = doc.createRange(); r.setStart(text, i); r.setEnd(text, i + 1); return Math.round(r.getBoundingClientRect().top); };
      const at = text.nodeValue.indexOf('液态玻璃');
      assert(at > 0, 'Caption changed: ' + text.nodeValue);
      for (let width = 720; width <= 1000; width += 10) {
        frame.style.width = width + 'px'; await settle();
        assert(lineOf(at) === lineOf(at + 3), width + ' px: 液态玻璃 is split over two lines');
      }
    } finally { frame.style.width = ''; }
  });

  await test('The nav at the smallest window size: no sideways scrolling, the search box fits, long page names keep their height', async () => {
    doc = await reset();
    frame.style.width = '704px'; frame.style.height = '441px';
    try {
      for (const language of ['zh-CN', 'de', 'ru', 'fr']) {
        change(doc, 'language', language); await settle();
        const nav = doc.querySelector('nav'), box = doc.getElementById('settingsSearch');
        assert(win().getComputedStyle(nav).overflowX === 'hidden' && nav.scrollWidth <= nav.clientWidth, language + ': the nav scrolls sideways');
        assert(box.getBoundingClientRect().right <= nav.getBoundingClientRect().left + nav.clientWidth, language + ': the search box is cut');
        assert(win().getComputedStyle(box).paddingRight === '32px', language + ': the text runs under the magnifier');
        for (const a of nav.querySelectorAll('a')) assert(a.scrollHeight <= a.clientHeight, language + ': ' + a.textContent.trim() + ' is squeezed');
        if (language === 'zh-CN') assert(nav.scrollHeight <= nav.clientHeight, 'zh-CN: the nav needs a scrollbar (' + nav.scrollHeight + ' in ' + nav.clientHeight + ')');
      }
    } finally { frame.style.width = ''; frame.style.height = ''; }
    doc = await reset();
  });

  document.getElementById('summary').textContent = `${passed} passed; ${failed} failed`;
  document.title = failed ? 'FAIL — settings UI regressions' : 'PASS — settings UI regressions';
  window.testResults = { passed, failed };
}

const server = http.createServer(async (request, response) => {
  try {
    const url = new URL(request.url, 'http://127.0.0.1');
    response.setHeader('Content-Type', 'text/html; charset=utf-8');
    if (url.pathname === '/i18n.js') {
      response.setHeader('Content-Type', 'text/javascript; charset=utf-8');
      response.end(await readFile(new URL('../ui/i18n.js', import.meta.url), 'utf8'));
      return;
    }
    if (url.pathname === '/editor') {
      // The smart fence editor: ?fence=new|dl|every, &language=, &mode=light.
      const language = url.searchParams.get('language') || 'zh-CN';
      const initial = structuredClone(fixture);
      initial.settings.language = language;
      initial.locale = language;
      initial.translations = catalogs[language] || {};
      if (url.searchParams.get('mode') === 'light') { initial.settings.theme = 'light'; initial.themeMode = 'light'; initial.accent = '#005FB8'; }
      const editor = editorFixtures[url.searchParams.get('fence') || 'new'];
      const safeJson = value => JSON.stringify(value).replace(/</g, '\\u003c');
      let html = await readFile(settingsUrl, 'utf8');
      html = html.replace('<!-- PECOFENCE_LOCALE -->', `<script>window.PECOFENCE_LOCALE=${safeJson({ locale: language, translations: initial.translations })};window.PECOFENCE_MODE='smartEditor';</script>`);
      const setup = `<script type="application/json" id="fixture">${safeJson(initial)}</script><script type="application/json" id="fixture-editor">${safeJson(editor)}</script><script type="application/json" id="fixture-catalogs">${safeJson(catalogs)}</script><script>(${editorBridge})();</script>`;
      response.end(html.replace('<body>', '<body>' + setup).replace('<html lang="zh-CN">', '<html lang="zh-CN" class="no-mica">'));
      return;
    }
    if (url.pathname === '/settings' || url.pathname === '/preview') {
      let html = await readFile(settingsUrl, 'utf8');
      const initial = structuredClone(fixture);
      const language = url.searchParams.get('language') || 'zh-CN';
      initial.settings.language = language;
      initial.locale = language;
      initial.translations = catalogs[language] || {};
      if (url.pathname === '/preview') {
        html = html.replace('<title>PecoFence 设置</title>', '<title>PecoFence · 设计预览</title>');
        if (url.searchParams.get('style') === 'liquidGlass') initial.settings.themeStyle = 'liquidGlass';
        if (url.searchParams.get('mode') === 'light') {
          initial.settings.theme = 'light';
          initial.themeMode = 'light';
          initial.accent = '#005FB8';
        }
      }
      const safeJson = value => JSON.stringify(value).replace(/</g, '\\u003c');
      html = html.replace('<!-- PECOFENCE_LOCALE -->', `<script>window.PECOFENCE_LOCALE=${safeJson({locale: language, translations: initial.translations})};</script>`);
      const setup = `<script type="application/json" id="fixture">${safeJson(initial)}</script><script type="application/json" id="fixture-catalogs">${safeJson(catalogs)}</script><script>(${bridge})();</script>`;
      response.end(html.replace('<body>', '<body>' + setup).replace('<html lang="zh-CN">', '<html lang="zh-CN" class="no-mica">'));
    } else {
      response.end(`<!doctype html><meta charset="utf-8"><title>Settings UI regressions</title>
        <style>body{font:15px system-ui;margin:24px;background:#fafafa;color:#222}iframe{width:960px;height:480px;border:1px solid #888}.pass{color:#176923}.fail{color:#b31d16}li{margin:6px 0}</style>
        <h1>Settings UI regressions</h1><p id="summary">Running…</p><ol id="results"></ol><iframe title="Settings under test"></iframe>
        <script>(${runTests})().catch(e=>{document.getElementById('summary').textContent=e.stack;document.title='FAIL — test runner';});</script>`);
    }
  } catch (error) {
    response.statusCode = 500;
    response.end(String(error));
  }
});
server.listen(port, '127.0.0.1', () => console.log(`Settings UI tests: http://127.0.0.1:${port}`));
