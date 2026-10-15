// Dependency-free browser regression suite for the actual WebView settings document.
// Run: node scripts/test-settings-ui.mjs
// Open the printed localhost address. The mock bridge never writes application settings.
import http from 'node:http';
import { readFile } from 'node:fs/promises';

const settingsUrl = new URL('../ui/settings.html', import.meta.url);
const languageTags = ['en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru'];
const catalogs = Object.fromEntries(await Promise.all(languageTags.map(async language =>
  [language, JSON.parse(await readFile(new URL('../locales/' + language + '.json', import.meta.url), 'utf8'))])));
const port = Number(process.env.PECOFENCE_UI_TEST_PORT || 43190);
const fixture = {
  type: 'state',
  locale: 'zh-CN', translations: {},
  settings: {
    language: 'zh-CN',
    autostart: false, hideRealIcons: false, hideInboxWhenIconsShown: false, offerNewFence: true, quickHide: { enabled: true },
    showDesktop: 'keepVisible', theme: 'dark', themeStyle: 'fluent', titleAlign: 'left', tabBar: 'top', backdrop: 'acrylic', iconSize: 48,
    icons: { tintRgb: null, tintStrength: 0.6, chameleon: false },
    rollUp: { hoverPeek: true, clickToExpand: false, pushNeighbors: true, titleOnHover: false, hideInactiveScrollbar: false, switchTabOnHover: false },
    snapping: { enabled: true, gapPx: 8, sizeToCells: false, guideLines: true }, peek: { enabled: true, dim: true, hotkey: 'ctrlAltSpace' },
  },
  rules: {
    keepUpdated: true, defaultTarget: 'inbox',
    list: [
      { id: 'rule-a', name: 'Documents', enabled: true, target: { fence: 'fence-a' }, allOf: [{ cond: 'ext', value: ['txt'] }], priorityClass: 'type' },
      { id: 'rule-b', name: 'Images', enabled: true, target: { fence: 'fence-b' }, allOf: [{ cond: 'ext', value: ['png'] }], priorityClass: 'type' },
    ],
  },
  fences: [
    { id: 'fence-a', title: 'Documents', kind: 'virtual', host: null, iconSize: 48, spacing: 'normal', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', titleOnHover: 'default', portal: null },
    { id: 'fence-b', title: 'Images', kind: 'virtual', host: 'Documents', iconSize: 96, spacing: 'loose', autoHeight: true, locked: false, excludeFromQuickHide: false, opacity: 'transparent', tint: null, tabColor: 'E74856', titleColor: 'theme', titleSize: 'normal', titleOnHover: 'default', portal: null },
    { id: 'inbox', title: 'Desktop', kind: 'inbox', host: null, iconSize: 48, spacing: 'normal', autoHeight: false, locked: true, excludeFromQuickHide: true, opacity: 'solid', tint: '0078D4', tintTitleOnly: true, titleColor: 'tint', titleSize: 'large', titleOnHover: 'hover', hideWhenEmpty: true, portal: null },
    { id: 'portal', title: 'Portal', kind: 'portal', host: null, iconSize: 32, spacing: 'compact', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'clear', tint: '123456', titleColor: 'ABCDEF', titleSize: 'small', titleOnHover: 'always', portal: { navigate: true, titleIcon: false } },
  ],
  tintPalette: [{ name: '红', hex: 'E74856' }, { name: '蓝', hex: '0078D4' }, { name: '灰', hex: '7A7574' }],
  snapshots: [{ id: 'snapshot-a', name: 'Before changes', date: '2026-09-09', fenceCount: 4 }],
  backups: [{ name: '2026-09-09', path: 'C:\\PecoFence\\backups\\2026-09-09.json' }],
  monitors: [{ id: 'one', label: 'Display 1' }, { id: 'two', label: 'Display 2' }],
  desktopIconsHidden: false,
  version: 'test', configPath: 'C:\\PecoFence\\config.json', memoryMb: 28, itemCount: 12,
  themeMode: 'dark', accent: '#60CDFF',
  peekHotkey: {
    chosen: { mods: 3, vk: 32, keys: ['Ctrl', 'Alt', '空格'], label: 'Ctrl+Alt+空格' },
    active: { mods: 3, vk: 32, keys: ['Ctrl', 'Alt', '空格'], label: 'Ctrl+Alt+空格' },
    presets: [
      { mods: 3, vk: 32, keys: ['Ctrl', 'Alt', '空格'], label: 'Ctrl+Alt+空格' },
      { mods: 9, vk: 32, keys: ['Win', 'Alt', '空格'], label: 'Win+Alt+空格' },
      { mods: 6, vk: 32, keys: ['Ctrl', 'Shift', '空格'], label: 'Ctrl+Shift+空格' },
    ],
  },
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
        }
        if (message.name === 'repairIcons' || message.name === 'hideDesktopIcons') {
          state.settings.hideRealIcons = message.name === 'hideDesktopIcons';
          state.desktopIconsHidden = state.settings.hideRealIcons;
        }
        if (message.type === 'setRules') state.rules = structuredClone(message.rules);
        if (message.name === 'setPeekHotkey') {
          const names = [[8, 'Win'], [2, 'Ctrl'], [1, 'Alt'], [4, 'Shift']].filter(([bit]) => message.mods & bit).map(([, n]) => n);
          const keys = [...names, message.vk === 32 ? '空格' : String.fromCharCode(message.vk)];
          const combo = { mods: message.mods, vk: message.vk, keys, label: keys.join('+') };
          // Win+Shift+Space stands for a combination Windows holds.
          const text = !(message.mods & 11) ? '快捷键需要包含 Ctrl、Alt 或 Win'
            : message.mods === 12 && message.vk === 32 ? combo.label + ' 已被 Windows 或其他程序占用，请换一个' : null;
          if (!text) {
            state.peekHotkey.chosen = state.peekHotkey.active = combo;
            state.settings.peek.customHotkey = message.mods === 3 && message.vk === 32 ? null : combo.label;
            setTimeout(window.testRefresh, 0);
          }
          setTimeout(() => receive({ data: { type: 'peekHotkeyResult', ok: !text, text } }), 0);
        }
        if (message.name === 'addTemplate' && !state.rules.list.some(r => r.template === message.template)) {
          // Mirror the host: a new fence plus its rule at the top of the list.
          const id = 'tpl-' + message.template;
          state.fences.push({ id, title: message.template, kind: 'virtual', host: null, iconSize: 48, spacing: 'normal', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', titleOnHover: 'default', portal: null });
          const idle = message.template === 'cleanup';
          const allOf = idle ? [{ cond: 'type', value: ['installers', 'archives'] }, { cond: 'idleDays', value: { min: 30 } }] : [{ cond: 'type', value: [message.template] }];
          const at = idle ? 0 : state.rules.list.findIndex(r => !r.allOf.some(c => c.cond === 'idleDays'));
          state.rules.list.splice(at < 0 ? state.rules.list.length : at, 0, { id: 'rule-' + id, name: message.template, enabled: true, target: { fence: id }, allOf, priorityClass: 'type', template: message.template });
        }
        if (message.type === 'setFence') {
          // Mirror the host: a portal flag lands in `portal`, everything else on the fence.
          const fence = state.fences.find(f => f.id === message.id);
          if (message.prop === 'portalNavigate') fence.portal.navigate = message.value;
          else if (message.prop === 'portalTitleIcon') fence.portal.titleIcon = message.value;
          else if (message.prop !== 'dockTop') fence[message.prop] = message.value;
        }
        window.testShowFence = (id) => receive({ data: { type: 'showFence', id } });
        window.testShowFeedback = (crash) => receive({ data: { type: 'showFeedback', crash } });
        if (message.name === 'feedbackInfo') {
          // Mirror the host: what may be sent besides the user's text.
          setTimeout(() => receive({ data: { type: 'feedbackInfo', url: 'https://feedback.test/feedback', app: '0.1.4', language: state.locale,
            crashed: !!state.crashed, system: { app: 'PecoFence 0.1.4 (ZIP / winget)', windows: 'build 26200' },
            log: 'INFO pecofence::app: config loaded fences=4' } }), 0);
        }
        if (message.name === 'deleteSnapshot') state.snapshots = state.snapshots.filter(s => s.id !== message.id);
        if (['ready', 'patchSettings', 'setRules', 'setFence'].includes(message.type) || message.type === 'action') {
          setTimeout(window.testRefresh, 0);
        }
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
    for (const page of ['general', 'fences', 'rules', 'layout', 'about']) {
      doc.querySelector(`[data-page=${page}]`).click();
      await settle();
      const main = doc.querySelector('main');
      assert(main.scrollWidth <= main.clientWidth + 1, page + ' overflows horizontally');
      for (const title of doc.querySelectorAll('.page.on .card .t')) {
        assert(title.getBoundingClientRect().height <= 60, page + ' crushes a card title into vertical text');
      }
    }
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
    assert(doc.getElementById('fenceHideEmpty').style.display === 'none', 'Hide-when-empty offered for an ordinary fence');
    assert(doc.getElementById('fenceTintTitleOnly').style.display === 'none', 'Title-only tint offered without a tint');
    assert(doc.getElementById('fenceTabColor').style.display === 'none', 'Tab colour offered for a fence without tabs');
    assert(doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint offered without a tint');
    const tintHint = doc.getElementById('fenceTitleColor').closest('.card').querySelector('.s');
    assert(!tintHint.hidden, 'Choose-a-tint hint missing without a tint');
    change(doc, 'fenceSel', 'inbox');
    assert(tintHint.hidden, 'Choose-a-tint hint shown although the fence has a tint');
    assert(doc.querySelector('[data-fence=locked]').classList.contains('on'), 'Lock state not shown');
    assert(doc.getElementById('fenceHideEmpty').style.display === '' && doc.querySelector('[data-fence=hideWhenEmpty]').classList.contains('on'), 'Inbox hide-when-empty not shown');
    assert(doc.getElementById('fenceTintTitleOnly').style.display === '' && doc.querySelector('[data-fence=tintTitleOnly]').classList.contains('on'), 'Title-only tint not shown');
    assert(doc.querySelector('[data-fence=tint]').value === '0078D4', 'Palette tint not selected');
    assert(doc.querySelector('[data-fence=titleColor]').value === 'tint' && !doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint title not shown');
    assert(doc.querySelector('[data-fence=titleSize]').value === 'large', 'Title size not shown');
    assert(doc.querySelector('[data-fence=titleSize] [value=extraLarge]'), 'Extra large title size missing');
    assert(doc.querySelector('[data-fence=titleOnHover]').value === 'hover', 'Title on hover not shown');
    change(doc, 'fenceSel', 'fence-b');
    assert(doc.getElementById('fenceTabColor').style.display === '', 'Tab colour hidden for a tab');
    assert(doc.querySelector('#fenceTabColorSwatches [data-value=E74856]').getAttribute('aria-checked') === 'true', 'Tab colour swatch not selected');
    change(doc, 'fenceSel', 'portal');
    assert(doc.getElementById('fencePortalGroup').style.display === '', 'Portal options hidden for a portal');
    assert(doc.querySelector('[data-fence=portalNavigate]').classList.contains('on') && !doc.querySelector('[data-fence=portalTitleIcon]').classList.contains('on'), 'Portal flags wrong');
    assert(doc.querySelector('[data-fence=tint]').value === '123456' && doc.querySelector('[data-fence=titleColor]').value === 'ABCDEF', 'Imported custom colours not represented');
    assert(doc.querySelector('[data-fence=iconSize]').value === '32' && doc.querySelector('[data-fence=spacing]').value === 'compact' && doc.querySelector('[data-fence=opacity]').value === 'clear', 'Layout values not shown');
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
    await settle();
    const sent = messages().slice(before);
    assert(sent.length === 5, 'Expected five setFence messages, got ' + sent.length);
    assert(sent.every(m => m.id === 'fence-a'), 'Message addressed to the wrong fence');
    assert(sent[0].prop === 'locked' && sent[0].value === true, 'Lock toggle not posted');
    assert(sent[1].prop === 'iconSize' && sent[1].value === 96, 'Icon size not posted as a number');
    assert(sent[2].prop === 'tint' && sent[2].value === 'E74856', 'Tint not posted');
    assert(sent[3].prop === 'title' && sent[3].value === 'Renamed', 'Title not trimmed');
    assert(sent[4].prop === 'titleOnHover' && sent[4].value === 'always', 'Title on hover not posted');
    assert(doc.getElementById('fenceSel').value === 'fence-a', 'Selection lost on refresh');
    assert(doc.querySelector('[data-fence=locked]').classList.contains('on') && doc.querySelector('[data-fence=tint]').value === 'E74856' && doc.querySelector('[data-fence=titleOnHover]').value === 'always', 'Refreshed state not reflected');
    assert(!doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint stays disabled after choosing a tint');
    change(doc, 'fenceTint', ''); await settle();
    assert(messages().at(-1).value === null, 'Clearing the tint did not post null');
    change(doc, 'fenceSel', 'fence-b');
    doc.querySelector('#fenceTabColorSwatches [data-value="0078D4"]').click(); await settle();
    assert(messages().at(-1).id === 'fence-b' && messages().at(-1).prop === 'tabColor' && messages().at(-1).value === '0078D4', 'Tab colour not posted');
    doc.querySelector('#fenceTabColorSwatches [data-value=""]').click(); await settle();
    assert(messages().at(-1).prop === 'tabColor' && messages().at(-1).value === null, 'Following the tint did not post null');
    change(doc, 'fenceSel', 'fence-a');
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
    assert(doc.querySelector('#ruleList .conds').textContent.includes('18:00–06:00'), 'Overnight rule described wrongly');
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
    assert(hotkey.disabled && doc.getElementById('peekHotkeyWin').disabled && [...doc.querySelectorAll('#peekPresets button')].every(b => b.disabled)
      && !doc.querySelector('[data-bind="peek.dim"]').closest('.card').classList.contains('disabled'), 'Peek hotkey/dim dependency wrong');
    doc.querySelector('[data-bind="peek.enabled"]').click(); await settle();
    assert(!hotkey.disabled && !doc.getElementById('peekPresetCard').classList.contains('disabled'), 'Hotkey stayed disabled');
  });
  await test('Hiding the desktop fence is available only while the Windows icons show', async () => {
    const icons = doc.querySelector('[data-bind=hideRealIcons]'), inbox = doc.querySelector('[data-bind=hideInboxWhenIconsShown]');
    assert(!current().settings.hideRealIcons && !inbox.closest('.card').classList.contains('disabled') && inbox.tabIndex === 0, 'Unavailable while the Windows icons show');
    inbox.click(); await settle();
    assert(current().settings.hideInboxWhenIconsShown && inbox.getAttribute('aria-checked') === 'true', 'Switch did not post its setting');
    icons.click(); await settle();
    assert(current().settings.hideRealIcons && inbox.closest('.card').classList.contains('disabled') && inbox.getAttribute('aria-disabled') === 'true', 'Still available with the Windows icons hidden');
    inbox.click(); await settle();
    assert(current().settings.hideInboxWhenIconsShown, 'A disabled switch still changed its setting');
    icons.click(); await settle();
    assert(!inbox.closest('.card').classList.contains('disabled'), 'Did not return with the Windows icons');
    inbox.click(); await settle();
    const offer = doc.querySelector('[data-bind=offerNewFence]');
    assert(offer.getAttribute('aria-checked') === 'true', 'New fence offer not shown on');
    offer.click(); await settle();
    assert(current().settings.offerNewFence === false, 'New fence offer switch did not post');
    offer.click(); await settle();
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
  await test('Every page fits compact windows in both materials and colour modes', async () => {
    for (const width of [704, 480]) {
      frame.style.width = width + 'px';
      for (const material of ['fluent', 'liquidGlass']) {
      current().settings.themeStyle = material;
      for (const mode of ['dark', 'light']) {
        current().themeMode = mode;
        frame.contentWindow.testRefresh();
        for (const page of ['general', 'fences', 'rules', 'layout', 'about']) {
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
      assert(doc.querySelector('[data-bind=hideRealIcons]').getAttribute('aria-checked') === 'true', 'General toggle out of sync');
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
    assert(card.hidden, 'Update card shown without update info (Store edition)');
    const show = async update => { current().update = update; frame.contentWindow.testUpdate(update); await settle(); };
    const installer = { edition: 'installer', version: '0.1.4', hasSetup: true };
    await show({ ...installer, state: 'idle', version: null });
    assert(!card.hidden && button.textContent === '检查更新' && !button.disabled && !button.classList.contains('accent') && status.textContent === '', 'Idle card wrong');
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
  await test('The Peek shortcut box records a combination and shows what the host saved', async () => {
    doc = await reset();
    const box = doc.getElementById('peekHotkey');
    assert(hotkeyText() === 'Ctrl+Alt+空格' && !doc.getElementById('peekHotkeyWin').checked, 'Active shortcut not shown: ' + hotkeyText());
    assert(doc.getElementById('peekHotkeyHelp').textContent.includes('包含 Win 键'), 'Help does not mention the Win checkbox');
    box.focus(); box.click(); await settle();
    assert(box.classList.contains('recording') && sent('peekHotkeyRecording').at(-1)?.on === true, 'Recording did not start');
    press(box, 'Control', 17, { ctrlKey: true });
    assert(hotkeyText() === 'Ctrl', 'Held modifiers are not shown: ' + hotkeyText());
    press(box, 'k', 75, { ctrlKey: true, altKey: true }); await settle();
    const set = sent('setPeekHotkey').at(-1);
    assert(set && set.mods === 3 && set.vk === 75, 'Wrong combination sent: ' + JSON.stringify(set));
    assert(!box.classList.contains('recording') && hotkeyText() === 'Ctrl+Alt+K', 'Saved shortcut not shown: ' + hotkeyText());
    assert(current().settings.peek.customHotkey === 'Ctrl+Alt+K', 'Shortcut not saved');
    assert(![...doc.querySelectorAll('#peekPresets button')].some(b => b.getAttribute('aria-pressed') === 'true'), 'A quick choice still looks selected');
  });
  await test('A refused shortcut keeps recording with the reason; Esc cancels', async () => {
    doc = await reset();
    const box = doc.getElementById('peekHotkey'), help = doc.getElementById('peekHotkeyHelp'), winBox = doc.getElementById('peekHotkeyWin');
    box.focus(); box.click(); await settle();
    press(box, 'k', 75); await settle();
    assert(help.classList.contains('error') && help.textContent.includes('Ctrl、Alt 或 Win') && box.classList.contains('recording'), 'No-modifier refusal not shown: ' + help.textContent);
    // Win comes from the checkbox: Windows keeps Win+Space before the page sees it.
    winBox.click(); await settle();
    assert(winBox.checked && box.classList.contains('recording') && doc.activeElement === box && hotkeyText() === 'Win', 'Win checkbox did not start a Win recording');
    press(box, ' ', 32, { shiftKey: true }); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 12, 'Win was not added');
    assert(help.textContent.includes('已被 Windows 或其他程序占用') && box.classList.contains('recording'), 'Taken combination not reported: ' + help.textContent);
    press(box, 'Escape', 27); await settle();
    assert(!box.classList.contains('recording') && sent('peekHotkeyRecording').at(-1).on === false, 'Esc did not end the recording');
    assert(!winBox.checked && hotkeyText() === 'Ctrl+Alt+空格' && !help.classList.contains('error'), 'Cancel did not restore the shortcut');
    assert(current().settings.peek.customHotkey == null, 'A refused shortcut was saved');
  });
  await test('Keys that stop arriving during a recording suggest the combination is taken', async () => {
    doc = await reset();
    const box = doc.getElementById('peekHotkey'), help = doc.getElementById('peekHotkeyHelp');
    const quiet = () => new Promise(resolve => setTimeout(resolve, 3200));
    box.focus(); box.click(); await quiet();
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
    box.focus(); box.click(); await settle();
    press(box, 'k', 75); await settle();
    assert(help.classList.contains('error'), 'No-modifier refusal not shown: ' + help.textContent);
    press(box, 'Control', 17, { ctrlKey: true }); await quiet();
    assert(help.classList.contains('caution') && help.textContent.includes('其他程序占用'), 'Old refusal hid the hint: ' + help.textContent);
    press(box, 'Escape', 27); await settle();
  });
  await test('Quick choices set the shortcut; a fallback says which one is in use', async () => {
    doc = await reset();
    const presets = [...doc.querySelectorAll('#peekPresets button')];
    assert(presets.length === 3 && presets[0].getAttribute('aria-pressed') === 'true' && presets[1].textContent === 'Win + Alt + 空格', 'Quick choices wrong');
    presets[1].focus(); presets[1].click(); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 9 && hotkeyText() === 'Win+Alt+空格', 'Quick choice not applied: ' + hotkeyText());
    assert(presets[1].getAttribute('aria-pressed') === 'true' && doc.activeElement === presets[1], 'Selected quick choice lost its state or focus');
    // A shortcut with Win does not tick the box for the next recording.
    const box = doc.getElementById('peekHotkey'), winBox = doc.getElementById('peekHotkeyWin');
    assert(!winBox.checked, 'Win checkbox ticked outside a recording');
    box.focus(); box.click(); await settle();
    winBox.click(); await settle();
    assert(winBox.checked && hotkeyText() === 'Win', 'Ticking Win during a recording failed');
    winBox.click(); await settle();
    assert(!winBox.checked && box.classList.contains('recording') && doc.activeElement === box, 'Clearing Win ended the recording');
    press(box, 'k', 75, { ctrlKey: true, altKey: true }); await settle();
    assert(sent('setPeekHotkey').at(-1).mods === 3 && hotkeyText() === 'Ctrl+Alt+K', 'Win stayed in the next recording');
    // A 0.2.0 file with Win+Space: Windows holds it, Ctrl+Alt+Space works instead.
    current().peekHotkey.chosen = { mods: 8, vk: 32, keys: ['Win', '空格'], label: 'Win+空格' };
    current().peekHotkey.active = current().peekHotkey.presets[0];
    win().testRefresh(); await settle();
    const help = doc.getElementById('peekHotkeyHelp');
    assert(hotkeyText() === 'Ctrl+Alt+空格' && help.classList.contains('caution') && help.textContent.includes('Win+空格') && help.textContent.includes('Ctrl+Alt+空格'), 'Fallback not explained: ' + help.textContent);
    current().peekHotkey.active = null;
    win().testRefresh(); await settle();
    assert(help.classList.contains('error') && hotkeyText() === 'Win+空格', 'All-taken state not shown: ' + help.textContent);
  });
  await test('All ten languages switch live without changing user names or draft inputs', async () => {
    doc = await reset();
    const win = frame.contentWindow;
    current().fences[0].title = '名称';
    current().rules.list[0].name = '保存快照';
    current().snapshots[0].name = '桌面 {1}';
    win.testRefresh();
    doc.getElementById('nrName').value = 'My rule 中文 {0}';
    change(doc, 'nrKind', 'type');
    doc.querySelector('#nrCats input').checked = true;
    for (const language of ['en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'zh-CN', 'system']) {
      change(doc, 'language', language);
      await settle();
      const locale = language === 'system' ? 'en' : language;
      assert(doc.documentElement.lang === locale, 'Wrong document language: ' + language);
      assert(doc.querySelector('[data-page=general]').textContent.includes(win.PecoFenceI18n.text('常规')), 'Navigation was not translated: ' + language);
      assert(doc.getElementById('nrName').value === 'My rule 中文 {0}' && doc.querySelector('#nrCats input').checked, 'Draft rule input lost: ' + language);
      assert(current().fences[0].title === '名称' && current().rules.list[0].name === '保存快照' && current().snapshots[0].name === '桌面 {1}', 'User data changed: ' + language);
      assert(doc.querySelector('#ruleList .t').textContent === '保存快照', 'User rule name translated: ' + language);
      assert(win.PecoFenceI18n.format('{0} 个栅栏', 'title {1}').includes('title {1}'), 'User placeholder was reinterpreted');
      for (const page of ['general', 'fences', 'rules', 'layout', 'about']) {
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
