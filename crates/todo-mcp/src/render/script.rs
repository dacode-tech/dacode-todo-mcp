//! The pages' scripts (U58's split out of `page.rs`, paying that
//! file's recorded file-size debt): the two scripts every render
//! carries — the click-detail handler and the lazy navigation —
//! hand-built, no dependencies. Both ride both pages (each is served
//! standalone); the live-update connection init is generated per
//! render from the page's slug and stays with the assembly that
//! emits it (`page`'s `live_script`).

/// The click-detail script (U54's rework of the Python tool's JS): one
/// delegated listener on the document — delegation is what makes the
/// handler resident across the lazy swaps, where per-node attachment
/// would die with every swapped-in SVG. The node data is parsed per
/// click from the inert `#detail-data` element, so the fields are
/// always the swapped-in view's own; on the epic page the listener is
/// inert (no `.node` elements, no data element) and harmless.
pub(super) const SCRIPT: &str = r#"document.addEventListener('click', event => {
  const n = event.target.closest('.node');
  if (!n) return;
  const source = document.getElementById('detail-data');
  if (!source) return;
  const data = JSON.parse(source.textContent);
  const id = n.dataset.id, d = data.nodes[id];
  const box = document.getElementById('detail');
  if (!d) return;
  const rel = data.edges.filter(e => e.from === id || e.to === id)
    .map(e => `<div>${e.from} ${e.from === id ? '→' : '←'} ${e.to}</div>`).join('');
  box.innerHTML = `<b>${id}</b> · ${d.kind || 'decision'} · ${d.status}<br>`
    + `${d.title || ''}`
    + (d.verify ? `<br><br><b>verify:</b> ${d.verify}` : '')
    + (d.evidence ? `<br><b>evidence:</b> ${d.evidence}` : '')
    + (d.note ? `<br><b>note:</b> ${d.note}` : '')
    + (d.resolution ? `<br><b>resolution:</b> ${d.resolution}` : '')
    + `<br><br><b>edges</b><br>${rel || 'none drawn'}`;
  box.style.display = 'block';
});"#;

/// The lazy navigation script (U54), hand-built — no dependencies. It
/// rides both pages (each is served standalone): the sidebar's epic
/// links are intercepted (`.sepic a` — the repo-name rows and the
/// switcher are deliberately NOT intercepted: a repo click is a full
/// page load that resets the page's script assembly), their page is
/// fetched, and the parts that differ between views — the `<title>`,
/// the `<style>` (each page carries its own complete stylesheet, so
/// a swapped-in view serves its own — the epic page's marker rule
/// would otherwise render under the repo page's), the switcher nav,
/// the `.cols` region — are swapped in place. The resident script
/// element is never replaced and never re-executes (the inert-JSON
/// DATA design exists so nothing here needs re-running); the
/// interceptor is re-attached after every swap because the
/// swapped-in links carry no listeners. Concurrent fetches are
/// sequenced by a generation counter: each `fetchView` bumps it, and
/// a superseded fetch abandons after every `await` — the
/// last-initiated navigation wins, so the DOM can never show a view
/// older than the URL. The WS push and the popstate handler re-fetch
/// the CURRENT view through the same path — no pushState on either,
/// only on intercepted clicks, so the history stays one entry per
/// user navigation. The WS subscription follows the view's repo:
/// `followView()` runs after every fetch+swap — same-repo swaps never
/// reconnect (no churn), a repo change closes the old connection
/// before opening the new one (never two live). A failed fetch (a
/// non-OK response or a network error) falls back to a full
/// navigation: the browser shows the server's named 404/500 body,
/// never a silent no-op. Modifier-keyed clicks (new tab, window) are
/// left to the browser.
pub(super) const LAZY_SCRIPT: &str = r#"
let fetchGen = 0;
function swapView(doc) {
  document.title = doc.title;
  const style = document.querySelector('style');
  const freshStyle = doc.querySelector('style');
  if (style && freshStyle) style.replaceWith(freshStyle);
  const switcher = document.querySelector('nav.switch');
  const freshSwitcher = doc.querySelector('nav.switch');
  if (switcher && freshSwitcher) switcher.replaceWith(freshSwitcher);
  const cols = document.querySelector('.cols');
  const freshCols = doc.querySelector('.cols');
  if (cols && freshCols) {
    cols.replaceWith(freshCols);
    intercept();
  }
}
async function fetchView(url, push) {
  // Last-initiated navigation wins: a superseded fetch abandons after every await.
  const gen = ++fetchGen;
  try {
    const response = await fetch(url);
    if (gen !== fetchGen) return;
    if (!response.ok) { location.assign(url); return; }
    const text = await response.text();
    if (gen !== fetchGen) return;
    swapView(new DOMParser().parseFromString(text, 'text/html'));
    if (push) history.pushState(null, '', url);
    // The URL is the view's own by now (pushState for clicks, popstate
    // moved it earlier) — the subscription can follow it.
    followView();
  } catch (e) {
    if (gen === fetchGen) location.assign(url);
  }
}
function intercept() {
  document.querySelectorAll('.sepic a').forEach(a => a.addEventListener('click', event => {
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    fetchView(a.href, true);
  }));
}
function connectWs() {
  try {
    liveWs = new WebSocket('ws://' + location.host + '/ws?repo=' + liveRepo);
    liveWs.onmessage = () => fetchView(location.pathname + location.search, false);
  } catch (e) {}
}
function currentRepo() {
  const parts = location.pathname.split('/');
  if (parts[1] === 'repo' && parts[2]) return parts[2];
  const repo = new URLSearchParams(location.search).get('repo');
  if (repo) return repo;
  const cur = document.querySelector('nav.switch .cur');
  return cur ? cur.textContent : null;
}
function followView() {
  // The subscription follows the view: close the old connection before opening the new.
  const repo = currentRepo();
  if (!repo || repo === liveRepo) return;
  if (liveWs) liveWs.close();
  liveRepo = repo;
  connectWs();
}
window.addEventListener('popstate', () => fetchView(location.pathname + location.search, false));
intercept();"#;
