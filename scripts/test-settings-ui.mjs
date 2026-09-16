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
    autostart: false, hideRealIcons: false, quickHide: { enabled: true },
    showDesktop: 'keepVisible', theme: 'dark', themeStyle: 'fluent', backdrop: 'acrylic', iconSize: 48,
    icons: { tintRgb: null, tintStrength: 0.6, chameleon: false },
    rollUp: { hoverPeek: true, clickToExpand: false, titleOnHover: false, hideInactiveScrollbar: false },
    snapping: { enabled: true }, peek: { enabled: true, dim: true, hotkey: 'ctrlAltSpace' },
  },
  rules: {
    keepUpdated: true, defaultTarget: 'inbox',
    list: [
      { id: 'rule-a', name: 'Documents', enabled: true, target: { fence: 'fence-a' }, allOf: [{ cond: 'ext', value: ['txt'] }], priorityClass: 'type' },
      { id: 'rule-b', name: 'Images', enabled: true, target: { fence: 'fence-b' }, allOf: [{ cond: 'ext', value: ['png'] }], priorityClass: 'type' },
    ],
  },
  fences: [
    { id: 'fence-a', title: 'Documents', kind: 'virtual', host: null, iconSize: 48, spacing: 'normal', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', portal: null },
    { id: 'fence-b', title: 'Images', kind: 'virtual', host: 'Documents', iconSize: 96, spacing: 'loose', autoHeight: true, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', portal: null },
    { id: 'inbox', title: 'Desktop', kind: 'inbox', host: null, iconSize: 48, spacing: 'normal', autoHeight: false, locked: true, excludeFromQuickHide: true, opacity: 'solid', tint: '0078D4', titleColor: 'tint', titleSize: 'large', portal: null },
    { id: 'portal', title: 'Portal', kind: 'portal', host: null, iconSize: 32, spacing: 'compact', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'clear', tint: '123456', titleColor: 'ABCDEF', titleSize: 'small', portal: { navigate: true, titleIcon: false } },
  ],
  tintPalette: [{ name: '红', hex: 'E74856' }, { name: '蓝', hex: '0078D4' }, { name: '灰', hex: '7A7574' }],
  snapshots: [{ id: 'snapshot-a', name: 'Before changes', date: '2026-09-09', fenceCount: 4 }],
  backups: [{ name: '2026-09-09', path: 'C:\\PecoFence\\backups\\2026-09-09.json' }],
  monitors: [{ id: 'one', label: 'Display 1' }, { id: 'two', label: 'Display 2' }],
  desktopIconsHidden: false,
  version: 'test', configPath: 'C:\\PecoFence\\config.json', memoryMb: 28, itemCount: 12,
  themeMode: 'dark', accent: '#60CDFF',
};

function bridge() {
  const state = JSON.parse(document.getElementById('fixture').textContent);
  const catalogs = JSON.parse(document.getElementById('fixture-catalogs').textContent);
  window.testMessages = [];
  let receive;
  window.testSummary = (itemCount) => receive({ data: { type: 'workspaceSummary', fenceCount: state.fences.length, itemCount } });
  window.testRefresh = () => receive({ data: structuredClone(state) });
  window.testState = state;
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
        if (message.name === 'addTemplate' && !state.rules.list.some(r => r.template === message.template)) {
          // Mirror the host: a new fence plus its rule at the top of the list.
          const id = 'tpl-' + message.template;
          state.fences.push({ id, title: message.template, kind: 'virtual', host: null, iconSize: 48, spacing: 'normal', autoHeight: false, locked: false, excludeFromQuickHide: false, opacity: 'default', tint: null, titleColor: 'theme', titleSize: 'normal', portal: null });
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
  const rulesPage = doc => doc.querySelector('[data-page=rules]').click();
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
    assert([...sel.options].map(o => o.textContent).join('|') === 'Documents|Images（Documents 的标签页）|Desktop（桌面）|Portal（门户）', 'Fence labels wrong: ' + [...sel.options].map(o => o.textContent).join('|'));
    assert(doc.getElementById('fencePortalGroup').style.display === 'none', 'Portal options shown for an ordinary fence');
    assert(doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint offered without a tint');
    change(doc, 'fenceSel', 'inbox');
    assert(doc.querySelector('[data-fence=locked]').classList.contains('on'), 'Lock state not shown');
    assert(doc.querySelector('[data-fence=tint]').value === '0078D4', 'Palette tint not selected');
    assert(doc.querySelector('[data-fence=titleColor]').value === 'tint' && !doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint title not shown');
    assert(doc.querySelector('[data-fence=titleSize]').value === 'large', 'Title size not shown');
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
    await settle();
    const sent = messages().slice(before);
    assert(sent.length === 4, 'Expected four setFence messages, got ' + sent.length);
    assert(sent.every(m => m.id === 'fence-a'), 'Message addressed to the wrong fence');
    assert(sent[0].prop === 'locked' && sent[0].value === true, 'Lock toggle not posted');
    assert(sent[1].prop === 'iconSize' && sent[1].value === 96, 'Icon size not posted as a number');
    assert(sent[2].prop === 'tint' && sent[2].value === 'E74856', 'Tint not posted');
    assert(sent[3].prop === 'title' && sent[3].value === 'Renamed', 'Title not trimmed');
    assert(doc.getElementById('fenceSel').value === 'fence-a', 'Selection lost on refresh');
    assert(doc.querySelector('[data-fence=locked]').classList.contains('on') && doc.querySelector('[data-fence=tint]').value === 'E74856', 'Refreshed state not reflected');
    assert(!doc.querySelector('[data-fence=titleColor] [value=tint]').disabled, 'Follow-tint stays disabled after choosing a tint');
    change(doc, 'fenceTint', ''); await settle();
    assert(messages().at(-1).value === null, 'Clearing the tint did not post null');
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
    const value = current().rules.list.at(-1).allOf[0].value;
    assert(value.min === null && value.max === 0, 'Blank or zero bound changed meaning');
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
    const value = current().rules.list.at(-1).allOf[0].value;
    assert(value.fromMin === 1080 && value.toMin === 360, 'Overnight bounds changed');
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
    const select = doc.getElementById('themeStyle');
    const theme = doc.querySelector('[data-bind=theme]');
    for (const mode of ['dark', 'light', 'followWindowsMode', 'followAppMode']) {
      theme.value = mode; theme.dispatchEvent(new frame.contentWindow.Event('change'));
      select.focus();
      change(doc, 'themeStyle', 'liquidGlass'); await settle();
      assert(current().settings.themeStyle === 'liquidGlass', 'Host did not receive the material');
      assert(current().settings.theme === mode && theme.value === mode, 'Style overwrote the colour preference');
      assert(doc.documentElement.classList.contains('liquid-glass'), 'Material not applied after host refresh');
      assert(doc.activeElement === select, 'Material change lost keyboard focus');
      assert(frame.contentWindow.getComputedStyle(doc.querySelector('.preview-title')).filter === 'none', 'Preview distorts foreground labels');
      change(doc, 'themeStyle', 'fluent'); await settle();
      assert(!doc.documentElement.classList.contains('liquid-glass'), 'Material remained after switching back');
      assert(frame.contentWindow.getComputedStyle(doc.querySelector('.preview-glass')).display === 'none', 'Optical layer remained visible');
    }
  });
  await test('Older settings without a material field retain Fluent', async () => {
    delete current().settings.themeStyle;
    frame.contentWindow.testRefresh();
    assert(doc.getElementById('themeStyle').value === 'fluent', 'Missing material has no valid selection');
    assert(!doc.documentElement.classList.contains('liquid-glass'), 'Old settings changed the appearance');
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
      ['weekday', 'nrWeek', ['0', '6'], '创建于 周一、周日'],
    ]) {
      change(doc, 'nrKind', kind);
      const count = ruleCount(doc);
      await add(doc);
      assert(ruleCount(doc) === count && errorShown(doc), kind + ' accepted an empty group');
      for (const value of values) doc.querySelector(`#${group} input[value="${value}"]`).click();
      change(doc, 'nrName', '工作 <draft> & QA');
      await add(doc);
      assert(ruleCount(doc) === count + 1, kind + ' rejected selected values');
      assert(doc.querySelector('#ruleList .rule:last-child .t').textContent === '工作 <draft> & QA', 'Rule name lost punctuation');
      assert(doc.querySelector('#ruleList .rule:last-child .conds').textContent.includes(label), kind + ' displayed the wrong condition');
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
      assert(doc.querySelector('#ruleList .rule:last-child .conds').textContent.includes(label), kind + ' displayed the wrong condition');
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
      assert(doc.querySelector('#ruleList .rule:last-child .conds').textContent === label + ' → 放入 “Desktop”', 'Wrong condition or destination');
      assert(current().rules.list.at(-1).target === 'inbox', 'Inbox target serialized as a virtual fence');
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
        html = html.replace('<title>PecoFence 设置</title>', '<title>PecoFence · 设计预览</title>')
          .replace('更改即时生效', '交互预览 · 不修改桌面');
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
