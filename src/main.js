// Ping Connectivity Monitor - Frontend JavaScript

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// Application State
const state = {
    targets: [],
    stats: {},
    recentPings: {},
    isRunning: false,
    selectedStatsTarget: null,
    chart: null,
    chartData: {},
    colors: [
        '#3b82f6', // blue
        '#10b981', // green
        '#f59e0b', // yellow
        '#ef4444', // red
        '#8b5cf6', // purple
        '#ec4899', // pink
        '#06b6d4', // cyan
        '#f97316', // orange
    ]
};

// DOM Elements
const elements = {
    connectionStatus: document.getElementById('connection-status'),
    statusText: document.querySelector('.status-text'),
    targetsList: document.getElementById('targets-list'),
    chartTargetSelect: document.getElementById('chart-target-select'),
    statsTargetSelect: document.getElementById('stats-target-select'),
    latencyChart: document.getElementById('latency-chart'),
    startBtn: document.getElementById('start-btn'),
    stopBtn: document.getElementById('stop-btn'),
    resetBtn: document.getElementById('reset-btn'),
    logsBtn: document.getElementById('logs-btn'),
    settingsBtn: document.getElementById('settings-btn'),
    addTargetBtn: document.getElementById('add-target-btn'),
    settingsModal: document.getElementById('settings-modal'),
    addTargetModal: document.getElementById('add-target-modal'),
    closeSettings: document.getElementById('close-settings'),
    closeAddTarget: document.getElementById('close-add-target'),
    cancelAddTarget: document.getElementById('cancel-add-target'),
    confirmAddTarget: document.getElementById('confirm-add-target'),
    saveSettingsBtn: document.getElementById('save-settings-btn'),
    settingsTargetsList: document.getElementById('settings-targets-list'),
    presetButtons: document.getElementById('preset-buttons'),
    newTargetAddress: document.getElementById('new-target-address'),
    newTargetLabel: document.getElementById('new-target-label'),
    quickTargetAddress: document.getElementById('quick-target-address'),
    quickTargetLabel: document.getElementById('quick-target-label'),
    pingInterval: document.getElementById('ping-interval'),
    pingTimeout: document.getElementById('ping-timeout'),
    statMin: document.getElementById('stat-min'),
    statMax: document.getElementById('stat-max'),
    statAvg: document.getElementById('stat-avg'),
    statJitter: document.getElementById('stat-jitter'),
    statLoss: document.getElementById('stat-loss'),
    statTotal: document.getElementById('stat-total'),
};

// Initialize the application
async function init() {
    console.log('Initializing Ping Connectivity Monitor...');
    
    // Load initial data
    await loadTargets();
    await loadConfig();
    await loadPresets();
    
    // Initialize chart
    initChart();
    
    // Set up event listeners
    setupEventListeners();
    
    // Listen for Tauri events
    await setupTauriListeners();
    
    console.log('Initialization complete');
}

// Load targets from backend
async function loadTargets() {
    try {
        state.targets = await invoke('get_targets');
        renderTargets();
        updateTargetSelects();
    } catch (error) {
        console.error('Failed to load targets:', error);
    }
}

// Load configuration
async function loadConfig() {
    try {
        const config = await invoke('get_config');
        elements.pingInterval.value = config.ping_interval_ms;
        elements.pingTimeout.value = config.timeout_ms;
    } catch (error) {
        console.error('Failed to load config:', error);
    }
}

// Load preset targets
async function loadPresets() {
    try {
        const presets = await invoke('get_preset_targets');
        renderPresets(presets);
    } catch (error) {
        console.error('Failed to load presets:', error);
    }
}

// Initialize Chart.js
function initChart() {
    const ctx = elements.latencyChart.getContext('2d');
    
    state.chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: {
                duration: 0
            },
            interaction: {
                intersect: false,
                mode: 'index'
            },
            plugins: {
                legend: {
                    display: true,
                    position: 'top',
                    labels: {
                        color: '#9ca3af',
                        usePointStyle: true,
                        padding: 20
                    }
                },
                tooltip: {
                    backgroundColor: '#1f2937',
                    titleColor: '#e5e7eb',
                    bodyColor: '#9ca3af',
                    borderColor: '#374151',
                    borderWidth: 1
                }
            },
            scales: {
                x: {
                    display: true,
                    grid: {
                        color: '#374151',
                        drawBorder: false
                    },
                    ticks: {
                        color: '#6b7280',
                        maxTicksLimit: 10
                    }
                },
                y: {
                    display: true,
                    beginAtZero: true,
                    grid: {
                        color: '#374151',
                        drawBorder: false
                    },
                    ticks: {
                        color: '#6b7280',
                        callback: (value) => value + ' ms'
                    }
                }
            }
        }
    });
}

// Set up event listeners
function setupEventListeners() {
    // Control buttons
    elements.startBtn.addEventListener('click', startPinging);
    elements.stopBtn.addEventListener('click', stopPinging);
    elements.resetBtn.addEventListener('click', resetStatistics);
    elements.logsBtn.addEventListener('click', openLogs);
    elements.settingsBtn.addEventListener('click', openSettings);
    elements.addTargetBtn.addEventListener('click', openAddTargetModal);
    
    // Modal controls
    elements.closeSettings.addEventListener('click', closeSettings);
    elements.closeAddTarget.addEventListener('click', closeAddTargetModal);
    elements.cancelAddTarget.addEventListener('click', closeAddTargetModal);
    elements.confirmAddTarget.addEventListener('click', addTargetFromModal);
    elements.saveSettingsBtn.addEventListener('click', saveSettings);
    
    // Modal backdrop clicks
    elements.settingsModal.querySelector('.modal-backdrop').addEventListener('click', closeSettings);
    elements.addTargetModal.querySelector('.modal-backdrop').addEventListener('click', closeAddTargetModal);
    
    // Add target in settings
    document.getElementById('add-new-target-btn').addEventListener('click', addTargetFromSettings);
    
    // Stats target select
    elements.statsTargetSelect.addEventListener('change', (e) => {
        state.selectedStatsTarget = e.target.value;
        updateStatsDisplay();
    });
    
    // Chart target select
    elements.chartTargetSelect.addEventListener('change', () => {
        rebuildChart();
    });
}

// Set up Tauri event listeners
async function setupTauriListeners() {
    // Listen for ping results
    await listen('ping-result', (event) => {
        const result = event.payload;
        handlePingResult(result);
    });
    
    // Listen for stats updates
    await listen('stats-update', (event) => {
        const stats = event.payload;
        handleStatsUpdate(stats);
    });
}

// Handle incoming ping result
function handlePingResult(result) {
    console.log('Received ping result:', result);
    
    // Update recent pings for this target
    if (!state.recentPings[result.target]) {
        state.recentPings[result.target] = [];
    }
    
    state.recentPings[result.target].push(result);
    
    // Keep only last 100 pings per target
    if (state.recentPings[result.target].length > 100) {
        state.recentPings[result.target].shift();
    }
    
    // Update chart data
    updateChartData(result);
    
    // Update target card latency
    updateTargetLatency(result);
}

// Handle stats update
function handleStatsUpdate(stats) {
    console.log('Received stats update:', stats);
    
    // Store stats by target
    stats.forEach(stat => {
        state.stats[stat.target] = stat;
    });
    
    // Update stats display
    updateStatsDisplay();
}

// Update chart with new data
function updateChartData(result) {
    const target = result.target;
    
    // Initialize chart data for this target if needed
    if (!state.chartData[target]) {
        state.chartData[target] = {
            labels: [],
            data: []
        };
    }
    
    // Add new data point
    const time = new Date(result.timestamp).toLocaleTimeString();
    state.chartData[target].labels.push(time);
    state.chartData[target].data.push(result.latency_ms || null);
    
    // Keep only last 50 points
    if (state.chartData[target].labels.length > 50) {
        state.chartData[target].labels.shift();
        state.chartData[target].data.shift();
    }
    
    // Rebuild chart datasets
    rebuildChart();
}

// Rebuild chart with current data
function rebuildChart() {
    const selectedTarget = elements.chartTargetSelect.value;
    const datasets = [];
    let labels = [];
    
    const enabledTargets = state.targets.filter(t => t.enabled);
    
    enabledTargets.forEach((target, index) => {
        const data = state.chartData[target.address];
        if (!data) return;
        
        if (selectedTarget === 'all' || selectedTarget === target.address) {
            // Use the longest label array
            if (data.labels.length > labels.length) {
                labels = data.labels;
            }
            
            datasets.push({
                label: target.label,
                data: data.data,
                borderColor: state.colors[index % state.colors.length],
                backgroundColor: state.colors[index % state.colors.length] + '20',
                borderWidth: 2,
                tension: 0.3,
                fill: false,
                pointRadius: 0,
                pointHoverRadius: 4
            });
        }
    });
    
    state.chart.data.labels = labels;
    state.chart.data.datasets = datasets;
    state.chart.update('none');
}

// Update target latency display
function updateTargetLatency(result) {
    const targetCard = document.querySelector(`[data-target="${result.target}"]`);
    if (!targetCard) return;
    
    const latencyEl = targetCard.querySelector('.target-latency');
    if (!latencyEl) return;
    
    if (result.success && result.latency_ms !== null) {
        const latency = result.latency_ms.toFixed(1);
        latencyEl.innerHTML = `<span class="latency-dot"></span>${latency} ms`;
        
        // Set color class based on latency
        latencyEl.classList.remove('good', 'medium', 'bad', 'offline');
        if (result.latency_ms < 50) {
            latencyEl.classList.add('good');
        } else if (result.latency_ms < 100) {
            latencyEl.classList.add('medium');
        } else {
            latencyEl.classList.add('bad');
        }
    } else {
        latencyEl.innerHTML = `<span class="latency-dot"></span>--`;
        latencyEl.classList.remove('good', 'medium', 'bad');
        latencyEl.classList.add('offline');
    }
}

// Update statistics display
function updateStatsDisplay() {
    const targetAddress = state.selectedStatsTarget || 
        (state.targets.length > 0 ? state.targets[0].address : null);
    
    if (!targetAddress) return;
    
    const stats = state.stats[targetAddress];
    if (!stats) return;
    
    elements.statMin.textContent = stats.min_latency_ms !== null 
        ? stats.min_latency_ms.toFixed(1) : '--';
    elements.statMax.textContent = stats.max_latency_ms !== null 
        ? stats.max_latency_ms.toFixed(1) : '--';
    elements.statAvg.textContent = stats.avg_latency_ms !== null 
        ? stats.avg_latency_ms.toFixed(1) : '--';
    elements.statJitter.textContent = stats.jitter_ms !== null 
        ? stats.jitter_ms.toFixed(1) : '--';
    elements.statLoss.textContent = stats.packet_loss_percent.toFixed(1);
    elements.statTotal.textContent = stats.total_pings;
}

// Render targets list
function renderTargets() {
    elements.targetsList.innerHTML = state.targets.map((target, index) => `
        <div class="target-card" data-target="${target.address}">
            <input type="checkbox" class="target-checkbox" 
                   ${target.enabled ? 'checked' : ''} 
                   onchange="toggleTarget('${target.id}')">
            <div class="target-info">
                <div class="target-address">${target.address}</div>
                <div class="target-label">${target.label}</div>
            </div>
            <div class="target-latency offline">
                <span class="latency-dot"></span>--
            </div>
        </div>
    `).join('');
}

// Update target select dropdowns
function updateTargetSelects() {
    const options = state.targets.map(t => 
        `<option value="${t.address}">${t.label} (${t.address})</option>`
    ).join('');
    
    elements.chartTargetSelect.innerHTML = `<option value="all">All Targets</option>${options}`;
    elements.statsTargetSelect.innerHTML = options;
    
    if (state.targets.length > 0 && !state.selectedStatsTarget) {
        state.selectedStatsTarget = state.targets[0].address;
    }
}

// Render settings targets list
function renderSettingsTargets() {
    elements.settingsTargetsList.innerHTML = state.targets.map(target => `
        <div class="settings-target-item">
            <input type="checkbox" class="target-checkbox" 
                   ${target.enabled ? 'checked' : ''} 
                   onchange="toggleTarget('${target.id}')">
            <div class="target-info">
                <div class="target-address">${target.address}</div>
                <div class="target-label">${target.label}</div>
            </div>
            <button class="btn-icon" onclick="removeTarget('${target.id}')" title="Remove">
                🗑
            </button>
        </div>
    `).join('');
}

// Render preset buttons
function renderPresets(presets) {
    elements.presetButtons.innerHTML = presets.map(preset => {
        const isAdded = state.targets.some(t => t.address === preset.address);
        return `
            <button class="preset-btn ${isAdded ? 'added' : ''}" 
                    onclick="addPreset('${preset.address}', '${preset.label}')"
                    ${isAdded ? 'disabled' : ''}>
                ${preset.label}
            </button>
        `;
    }).join('');
}

// Control functions
async function startPinging() {
    try {
        await invoke('start_pinging');
        state.isRunning = true;
        updateControlButtons();
        updateConnectionStatus('running');
    } catch (error) {
        console.error('Failed to start pinging:', error);
        alert('Failed to start pinging: ' + error);
    }
}

async function stopPinging() {
    try {
        await invoke('stop_pinging');
        state.isRunning = false;
        updateControlButtons();
        updateConnectionStatus('stopped');
    } catch (error) {
        console.error('Failed to stop pinging:', error);
    }
}

async function resetStatistics() {
    try {
        await invoke('reset_statistics');
        state.stats = {};
        state.recentPings = {};
        state.chartData = {};
        
        // Reset chart
        state.chart.data.labels = [];
        state.chart.data.datasets = [];
        state.chart.update();
        
        // Reset stats display
        elements.statMin.textContent = '--';
        elements.statMax.textContent = '--';
        elements.statAvg.textContent = '--';
        elements.statJitter.textContent = '--';
        elements.statLoss.textContent = '--';
        elements.statTotal.textContent = '--';
        
        // Reset target latencies
        document.querySelectorAll('.target-latency').forEach(el => {
            el.innerHTML = '<span class="latency-dot"></span>--';
            el.className = 'target-latency offline';
        });
    } catch (error) {
        console.error('Failed to reset statistics:', error);
    }
}

async function openLogs() {
    try {
        await invoke('open_log_directory');
    } catch (error) {
        console.error('Failed to open logs:', error);
        alert('Failed to open log directory: ' + error);
    }
}

// Update control buttons state
function updateControlButtons() {
    elements.startBtn.disabled = state.isRunning;
    elements.stopBtn.disabled = !state.isRunning;
}

// Update connection status indicator
function updateConnectionStatus(status) {
    elements.connectionStatus.classList.remove('connected', 'disconnected', 'running');
    
    switch (status) {
        case 'running':
            elements.connectionStatus.classList.add('running');
            elements.statusText.textContent = 'Running';
            break;
        case 'connected':
            elements.connectionStatus.classList.add('connected');
            elements.statusText.textContent = 'Connected';
            break;
        default:
            elements.connectionStatus.classList.add('disconnected');
            elements.statusText.textContent = 'Stopped';
    }
}

// Modal functions
function openSettings() {
    renderSettingsTargets();
    elements.settingsModal.classList.remove('hidden');
}

function closeSettings() {
    elements.settingsModal.classList.add('hidden');
}

function openAddTargetModal() {
    elements.quickTargetAddress.value = '';
    elements.quickTargetLabel.value = '';
    elements.addTargetModal.classList.remove('hidden');
}

function closeAddTargetModal() {
    elements.addTargetModal.classList.add('hidden');
}

// Target management
async function toggleTarget(id) {
    try {
        await invoke('toggle_target', { id });
        await loadTargets();
        renderSettingsTargets();
    } catch (error) {
        console.error('Failed to toggle target:', error);
    }
}

async function removeTarget(id) {
    if (!confirm('Are you sure you want to remove this target?')) return;
    
    try {
        await invoke('remove_target', { id });
        await loadTargets();
        renderSettingsTargets();
        await loadPresets();
    } catch (error) {
        console.error('Failed to remove target:', error);
    }
}

async function addTargetFromModal() {
    const address = elements.quickTargetAddress.value.trim();
    const label = elements.quickTargetLabel.value.trim() || address;
    
    if (!address) {
        alert('Please enter an address');
        return;
    }
    
    try {
        await invoke('add_target', { address, label });
        await loadTargets();
        closeAddTargetModal();
    } catch (error) {
        console.error('Failed to add target:', error);
        alert('Failed to add target: ' + error);
    }
}

async function addTargetFromSettings() {
    const address = elements.newTargetAddress.value.trim();
    const label = elements.newTargetLabel.value.trim() || address;
    
    if (!address) {
        alert('Please enter an address');
        return;
    }
    
    try {
        await invoke('add_target', { address, label });
        elements.newTargetAddress.value = '';
        elements.newTargetLabel.value = '';
        await loadTargets();
        renderSettingsTargets();
        await loadPresets();
    } catch (error) {
        console.error('Failed to add target:', error);
        alert('Failed to add target: ' + error);
    }
}

async function addPreset(address, label) {
    try {
        await invoke('add_target', { address, label });
        await loadTargets();
        await loadPresets();
        renderSettingsTargets();
    } catch (error) {
        console.error('Failed to add preset:', error);
    }
}

async function saveSettings() {
    try {
        const intervalMs = parseInt(elements.pingInterval.value) || 1000;
        const timeoutMs = parseInt(elements.pingTimeout.value) || 5000;
        
        await invoke('set_ping_interval', { intervalMs });
        
        const config = await invoke('get_config');
        config.ping_interval_ms = intervalMs;
        config.timeout_ms = timeoutMs;
        await invoke('save_config', { config });
        
        closeSettings();
    } catch (error) {
        console.error('Failed to save settings:', error);
        alert('Failed to save settings: ' + error);
    }
}

// Make functions available globally for onclick handlers
window.toggleTarget = toggleTarget;
window.removeTarget = removeTarget;
window.addPreset = addPreset;

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', init);
