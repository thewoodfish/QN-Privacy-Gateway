const terminal = document.getElementById('terminal');
const statusLight = document.getElementById('status-light');
const statusText = document.getElementById('status-text');
const scrollLock = document.getElementById('scroll-lock');
const pauseBtn = document.getElementById('pause-btn');
const clearBtn = document.getElementById('clear-btn');
const filterInfo = document.getElementById('filter-info');
const filterWarn = document.getElementById('filter-warn');
const filterError = document.getElementById('filter-error');
const filterMethod = document.getElementById('filter-method');
const filterHash = document.getElementById('filter-hash');

const metricRequests = document.getElementById('metric-requests');
const metricUnique = document.getElementById('metric-unique');
const metricHitRatio = document.getElementById('metric-hit-ratio');
const metricLatency = document.getElementById('metric-latency');
const metricP95 = document.getElementById('metric-p95');

const MAX_LINES = 800;
const LATENCY_WINDOW = 200;

let paused = false;
let logBuffer = [];
let latencyBuffer = [];
let lastLatency = null;

function setStatus(connected) {
  statusLight.classList.toggle('connected', connected);
  statusText.textContent = connected ? 'CONNECTED' : 'DISCONNECTED';
}

function formatLine(event) {
  const ts = event.ts || '';
  const method = event.method ? ` ${event.method}` : '';
  const hash = event.request_hash ? ` ${event.request_hash.slice(0, 10)}…` : '';
  const latency = event.latency_ms != null ? ` ${event.latency_ms}ms` : '';
  const note = event.note ? ` :: ${event.note}` : '';
  return `${ts} [${event.event}] ${event.level}${method}${hash}${latency}${note}`;
}

function matchesFilters(event) {
  const level = event.level || '';
  if (level === 'INFO' && !filterInfo.checked) return false;
  if (level === 'WARN' && !filterWarn.checked) return false;
  if (level === 'ERROR' && !filterError.checked) return false;

  const methodFilter = filterMethod.value.trim().toLowerCase();
  if (methodFilter && (!event.method || !event.method.toLowerCase().includes(methodFilter))) {
    return false;
  }

  const hashFilter = filterHash.value.trim().toLowerCase();
  if (hashFilter && (!event.request_hash || !event.request_hash.toLowerCase().includes(hashFilter))) {
    return false;
  }

  return true;
}

function renderLogs() {
  terminal.innerHTML = '';
  const fragment = document.createDocumentFragment();
  for (const event of logBuffer) {
    if (!matchesFilters(event)) continue;
    const line = document.createElement('div');
    line.className = `log-line log-${event.level.toLowerCase()}`;
    line.textContent = formatLine(event);
    fragment.appendChild(line);
  }
  terminal.appendChild(fragment);
  if (!paused) {
    terminal.scrollTop = terminal.scrollHeight;
  }
}

function pushLog(event) {
  logBuffer.push(event);
  if (logBuffer.length > MAX_LINES) {
    logBuffer.shift();
  }

  if (event.latency_ms != null) {
    lastLatency = event.latency_ms;
    latencyBuffer.push(event.latency_ms);
    if (latencyBuffer.length > LATENCY_WINDOW) {
      latencyBuffer.shift();
    }
    updateLatencyStats();
  }

  if (paused) return;

  if (!matchesFilters(event)) return;

  const line = document.createElement('div');
  line.className = `log-line log-${event.level.toLowerCase()}`;
  line.textContent = formatLine(event);
  terminal.appendChild(line);

  if (terminal.childElementCount > MAX_LINES) {
    terminal.removeChild(terminal.firstChild);
  }

  const atBottom = terminal.scrollHeight - terminal.scrollTop - terminal.clientHeight < 20;
  if (atBottom) {
    terminal.scrollTop = terminal.scrollHeight;
  }

  scrollLock.classList.toggle('visible', !atBottom);
}

function updateLatencyStats() {
  metricLatency.textContent = lastLatency == null ? '-' : `${lastLatency}ms`;
  if (!latencyBuffer.length) {
    metricP95.textContent = '-';
    return;
  }
  const sorted = [...latencyBuffer].sort((a, b) => a - b);
  const idx = Math.floor(0.95 * (sorted.length - 1));
  metricP95.textContent = `${sorted[idx]}ms`;
}

function fetchMetrics() {
  fetch('/metrics')
    .then((res) => res.json())
    .then((data) => {
      const requests = data.requests_total || 0;
      const unique = data.unique_request_hashes || 0;
      const hits = data.cache_hits || 0;
      const misses = data.cache_misses || 0;
      const ratio = hits + misses > 0 ? Math.round((hits / (hits + misses)) * 100) : 0;
      metricRequests.textContent = requests;
      metricUnique.textContent = unique;
      metricHitRatio.textContent = `${ratio}%`;
    })
    .catch(() => {});
}

function connectSse() {
  const source = new EventSource('/events');

  source.onopen = () => setStatus(true);
  source.onerror = () => setStatus(false);

  source.addEventListener('log', (evt) => {
    try {
      const event = JSON.parse(evt.data);
      pushLog(event);
    } catch (_) {}
  });
}

pauseBtn.addEventListener('click', () => {
  paused = !paused;
  pauseBtn.textContent = paused ? 'RESUME' : 'PAUSE';
  if (!paused) {
    renderLogs();
  }
});

clearBtn.addEventListener('click', () => {
  logBuffer = [];
  terminal.innerHTML = '';
});

[filterInfo, filterWarn, filterError, filterMethod, filterHash].forEach((el) => {
  el.addEventListener('input', () => {
    renderLogs();
  });
});

terminal.addEventListener('scroll', () => {
  const atBottom = terminal.scrollHeight - terminal.scrollTop - terminal.clientHeight < 20;
  scrollLock.classList.toggle('visible', !atBottom);
});

connectSse();
fetchMetrics();
setInterval(fetchMetrics, 2000);
