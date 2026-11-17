// pdf2md-gui frontend: vanilla JS over the Tauri IPC bridge.
// No build step, no npm. Requires window.__TAURI__ (i.e. run inside the app).

const SCHEMES = ['sunset', 'harvest', 'ember', 'desert', 'sky'];

const state = {
  pdfPath: '',
  markdown: '',
  converting: false,
  themeChoice: 'auto',
  scheme: 'sunset',
};

function tauri() {
  if (!window.__TAURI__) {
    throw new Error('請用桌面版開啟此頁面');
  }
  return window.__TAURI__;
}

function $(id) {
  return document.getElementById(id);
}

function setStatus(text, isError) {
  const el = $('status');
  el.textContent = text;
  el.classList.toggle('error', !!isError);
}

function setPreviewStatus(text, isError) {
  const el = $('previewStatus');
  if (!el) {
    return;
  }
  el.textContent = text;
  el.classList.toggle('error', !!isError);
}

function setSavePath(text) {
  const el = $('savePath');
  if (el) {
    el.textContent = text;
  }
}

function currentOptions() {
  return {
    compact: $('optCompact').checked,
    pageNumbers: $('optPages').checked,
    detectOnly: $('optDetect').checked,
    analyze: $('optAnalyze').checked,
    pageSpec: $('optPageSpec').value.trim(),
    password: $('optPassword').value,
  };
}

function refreshCliPreview() {
  if (!state.pdfPath) {
    $('cliPreview').textContent = 'pdf2md <請先選擇 PDF>';
    return;
  }
  const output = $('optOutput').value.trim();
  tauri().core.invoke('cli_preview', {
    path: state.pdfPath,
    output,
    opts: currentOptions(),
  }).then((cmd) => {
    $('cliPreview').textContent = cmd;
  }).catch((err) => {
    setStatus('指令預覽失敗：' + err, true);
  });
}

function acceptPdfPath(path, via) {
  state.pdfPath = path;
  $('filePath').textContent = path;
  $('btnConvert').disabled = false;
  // Each new PDF resets the output to its own stem so the CLI preview
  // always points at "<new-name>.md".
  if (path.toLowerCase().endsWith('.pdf')) {
    $('optOutput').value = path.slice(0, -4) + '.md';
  }
  setStatus(via + '：' + path, false);
  setSavePath('尚未儲存：轉換後按「儲存 .md…」會寫到此位置');
  refreshCliPreview();
}

async function pickPdf() {
  try {
    const path = await tauri().core.invoke('pick_pdf');
    if (path) {
      acceptPdfPath(path, '已選擇');
    }
  } catch (err) {
    setStatus('選擇檔案失敗：' + err, true);
  }
}

async function convert() {
  if (!state.pdfPath || state.converting) {
    return;
  }
  state.converting = true;
  $('btnConvert').disabled = true;
  setStatus('轉換中…', false);
  try {
    const report = await tauri().core.invoke('convert_pdf', {
      path: state.pdfPath,
      opts: currentOptions(),
    });
    renderReport(report);
    state.markdown = report.markdown || '';
    renderMarkdown();
    const pendingOutput = $('optOutput').value.trim();
    if (pendingOutput) {
      setSavePath('儲存位置：' + pendingOutput + '（按「儲存 .md…」寫檔）');
    } else {
      setSavePath('尚未儲存：按「儲存 .md…」時會請你選擇位置');
    }
    if (report.needsOcr) {
      setStatus('此 PDF 為「' + report.pdfType + '」，無可用文字層，需要 OCR 才能轉換。', true);
      setPreviewStatus('此 PDF 為「' + report.pdfType + '」，無可用文字層，需要 OCR 才能轉換。', true);
    } else if (!state.markdown.trim()) {
      setStatus('轉換完成，但沒有抽出文字（可能為掃描頁或加密 PDF）。', true);
      setPreviewStatus('轉換完成，但沒有抽出文字（可能為掃描頁或加密 PDF）。', true);
    } else {
      setStatus('轉換完成：' + state.markdown.length + ' 字元。', false);
      setPreviewStatus('轉換完成：' + state.markdown.length + ' 字元，可複製全文或儲存 .md。', false);
    }
    switchTab('preview', true);
  } catch (err) {
    setStatus('轉換失敗：' + err, true);
  } finally {
    state.converting = false;
    $('btnConvert').disabled = !state.pdfPath;
  }
}

function renderReport(report) {
  const rows = [
    ['類型', report.pdfType],
    ['信心分數', Math.round(report.confidence * 100) + '%'],
    ['總頁數', String(report.pageCount)],
    ['處理時間', report.processingTimeMs + ' ms'],
    ['版面', report.layout],
  ];
  if (report.pagesNeedingOcr && report.pagesNeedingOcr.length > 0) {
    rows.push(['需 OCR 頁面', report.pagesNeedingOcr.join('、')]);
  }
  for (const warning of report.warnings || []) {
    rows.push(['警告', warning]);
  }
  $('report').innerHTML = rows
    .map(([k, v]) => '<dt>' + escapeHtml(k) + '</dt><dd>' + escapeHtml(String(v)) + '</dd>')
    .join('');
  $('ocrNotice').hidden = !report.needsOcr;
}

function renderMarkdown() {
  $('markdown').textContent = state.markdown;
}

function escapeHtml(text) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (err) {
    // Fallback for contexts without async clipboard permission.
    const area = document.createElement('textarea');
    area.value = text;
    document.body.appendChild(area);
    area.select();
    let ok = false;
    try {
      ok = document.execCommand('copy');
    } catch (ignored) {
      ok = false;
    }
    document.body.removeChild(area);
    return ok;
  }
}

async function saveMarkdown() {
  if (!state.markdown) {
    setStatus('目前沒有可儲存的 Markdown，請先轉換。', true);
    setPreviewStatus('目前沒有可儲存的 Markdown，請先轉換。', true);
    return;
  }
  try {
    let output = $('optOutput').value.trim();
    if (!output) {
      const stem = state.pdfPath.split(/[\\/]/).pop().replace(/\.pdf$/i, '');
      const picked = await tauri().core.invoke('pick_save_path', {
        defaultName: (stem || 'output') + '.md',
      });
      if (!picked) {
        return;
      }
      output = picked;
      $('optOutput').value = output;
    }
    await tauri().core.invoke('save_markdown', { path: output, content: state.markdown });
    setStatus('已儲存：' + output, false);
    setPreviewStatus('已儲存：' + output, false);
    setSavePath('儲存位置：' + output);
  } catch (err) {
    setStatus('寫檔失敗：' + err, true);
    setPreviewStatus('寫檔失敗：' + err, true);
  }
}

const TAB_ORDER = ['upload', 'preview', 'about'];

function switchTab(name, focusTab) {
  if (!TAB_ORDER.includes(name)) {
    return;
  }
  for (const button of document.querySelectorAll('.tabs [role="tab"]')) {
    const selected = button.dataset.tab === name;
    button.classList.toggle('active', selected);
    button.setAttribute('aria-selected', selected ? 'true' : 'false');
    button.tabIndex = selected ? 0 : -1;
    if (selected && focusTab) {
      button.focus();
    }
    if (selected) {
      button.scrollIntoView({ block: 'nearest', inline: 'nearest', container: 'nearest' });
    }
  }
  for (const section of document.querySelectorAll('main section')) {
    section.hidden = section.id !== 'tab-' + name;
  }
}

function focusTabByOffset(current, offset) {
  const index = TAB_ORDER.indexOf(current);
  const next = TAB_ORDER[(index + offset + TAB_ORDER.length) % TAB_ORDER.length];
  switchTab(next, true);
}

function resolveTheme() {
  if (state.themeChoice === 'dark') {
    return 'dark';
  }
  if (state.themeChoice === 'light') {
    return 'light';
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme() {
  const resolved = resolveTheme();
  document.documentElement.setAttribute('data-theme', resolved);
  const meta = document.querySelector('meta[name="color-scheme"]');
  if (meta) {
    meta.content = state.themeChoice === 'auto' ? 'light dark' : state.themeChoice;
  }
}

function loadStored(key) {
  try {
    return localStorage.getItem(key);
  } catch (ignored) {
    return null;
  }
}

function store(key, value) {
  try {
    localStorage.setItem(key, value);
  } catch (ignored) {
    // Private mode etc: choice still applies for this session.
  }
}

function setThemeChoice(choice) {
  state.themeChoice = choice === 'dark' || choice === 'light' ? choice : 'auto';
  if (state.themeChoice === 'auto') {
    try { localStorage.removeItem('pdf2md.theme'); } catch (ignored) {}
  } else {
    store('pdf2md.theme', state.themeChoice);
  }
  applyTheme();
  for (const button of document.querySelectorAll('.theme-switch button')) {
    const active = button.dataset.themeChoice === state.themeChoice;
    button.classList.toggle('active', active);
    button.setAttribute('aria-pressed', active ? 'true' : 'false');
  }
}

function setSchemeChoice(scheme) {
  if (!SCHEMES.includes(scheme)) {
    scheme = 'sunset';
  }
  state.scheme = scheme;
  document.documentElement.setAttribute('data-scheme', scheme);
  store('pdf2md.scheme', scheme);
  const picker = document.getElementById('schemeSelect');
  if (picker) {
    picker.value = scheme;
  }
}

function bindEvents() {
  for (const button of document.querySelectorAll('.tabs [role="tab"]')) {
    button.addEventListener('click', () => switchTab(button.dataset.tab, false));
    button.addEventListener('keydown', (event) => {
      const key = event.key;
      if (key === 'ArrowRight' || key === 'ArrowLeft' || key === 'Home' || key === 'End') {
        event.preventDefault();
        if (key === 'Home') {
          switchTab(TAB_ORDER[0], true);
        } else if (key === 'End') {
          switchTab(TAB_ORDER[TAB_ORDER.length - 1], true);
        } else {
          focusTabByOffset(button.dataset.tab, key === 'ArrowRight' ? 1 : -1);
        }
      }
    });
  }
  for (const button of document.querySelectorAll('.theme-switch button')) {
    button.addEventListener('click', () => setThemeChoice(button.dataset.themeChoice));
  }
  $('schemeSelect').addEventListener('change', (event) => {
    setSchemeChoice(event.target.value);
  });
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);
  const zone = $('dropzone');
  zone.addEventListener('click', pickPdf);
  zone.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      pickPdf();
    }
  });
  $('btnConvert').addEventListener('click', convert);
  $('btnCopyCmd').addEventListener('click', async () => {
    const ok = await copyText($('cliPreview').textContent);
    setStatus(ok ? '已複製 CLI 指令。' : '複製失敗。', !ok);
  });
  $('btnCopyMd').addEventListener('click', async () => {
    const ok = await copyText(state.markdown);
    setStatus(ok ? '已複製全文。' : '複製失敗。', !ok);
    setPreviewStatus(ok ? '已複製全文。' : '複製失敗。', !ok);
  });
  $('btnSaveMd').addEventListener('click', saveMarkdown);
  for (const id of ['optCompact', 'optPages', 'optDetect', 'optAnalyze']) {
    $(id).addEventListener('change', refreshCliPreview);
  }
  for (const id of ['optPageSpec', 'optPassword', 'optOutput']) {
    $(id).addEventListener('input', refreshCliPreview);
  }
  // Cosmetic drag-over highlight only; file data arrives via backend events.
  zone.addEventListener('dragenter', (e) => { e.preventDefault(); zone.classList.add('over'); });
  zone.addEventListener('dragover', (e) => { e.preventDefault(); });
  zone.addEventListener('dragleave', () => zone.classList.remove('over'));
  zone.addEventListener('drop', (e) => {
    e.preventDefault();
    zone.classList.remove('over');
  });
  for (const panel of document.querySelectorAll('main section[data-panel]')) {
    panel.addEventListener('beforematch', () => {
      switchTab(panel.dataset.panel, true);
    });
  }
  window.addEventListener('storage', (event) => {
    if (event.key === 'pdf2md.theme') {
      setThemeChoice(event.newValue || 'auto');
    }
    if (event.key === 'pdf2md.scheme') {
      setSchemeChoice(event.newValue || 'sunset');
    }
  });
}

async function init() {
  bindEvents();
  // Restore the persisted look: scheme + theme survive restarts until changed.
  setSchemeChoice(loadStored('pdf2md.scheme') || 'sunset');
  setThemeChoice(loadStored('pdf2md.theme') || 'auto');
  try {
    const version = await tauri().core.invoke('app_version');
    $('appVersion').textContent = 'v' + version;
    $('aboutVersion').textContent = 'v' + version;
    await tauri().event.listen('pdf-dropped', (event) => {
      acceptPdfPath(event.payload, '已接收拖放檔案');
    });
    await tauri().event.listen('pdf-drop-rejected', (event) => {
      setStatus('拖放未接受：' + event.payload, true);
    });
  } catch (err) {
    setStatus(String(err && err.message ? err.message : err), true);
  }
}

document.addEventListener('DOMContentLoaded', init);
