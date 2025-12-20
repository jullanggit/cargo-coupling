// =====================================================
// cargo-coupling Web Visualization - Main Entry Point
// =====================================================

import { CONFIG, state, setGraphData, setSelectedNode } from './state.js';
import { setupLanguageToggle, updateUILanguage } from './i18n.js';
import { initCytoscape, applyLayout, centerOnNode, focusOnNode, highlightNeighbors, highlightDependencyPath, clearHighlights } from './graph.js';
import {
    updateHeaderStats,
    updateFooterStats,
    setupFilters,
    setupSearch,
    setupLayoutSelector,
    setupExportButtons,
    setupKeyboardShortcuts,
    setupCenterModeToggle,
    setupLegendToggle,
    setupResizableSidebar,
    showNodeDetails,
    showEdgeDetails,
    clearDetails,
    showBlastRadius,
    clearBlastRadius
} from './ui.js';
import { showItemGraph, hideItemGraph } from './item-graph.js';
import {
    populateCriticalIssues,
    populateHotspots,
    populateModuleRankings,
    setupModuleRankingSorting,
    populateIssueList,
    setupAnalysisButtons,
    enableAnalysisButtons,
    setupClusterView,
    populatePathFinderSelects,
    setupJobButtons,
    updateWhatBreaksButton,
    setSelectNodeCallback
} from './features.js';

// =====================================================
// Initialization
// =====================================================

async function init() {
    try {
        await loadConfig();
        const data = await fetchGraphData();
        setGraphData(data);

        initGraph(data);
        initUI(data);
        initJobFeatures();

    } catch (error) {
        console.error('Failed to initialize:', error);
        document.getElementById('cy').innerHTML =
            `<div style="padding: 2rem; color: #ef4444;">Failed to load graph data: ${error.message}</div>`;
    }
}

function initGraph(data) {
    initCytoscape(
        data,
        // Node tap handler
        (node) => selectNode(node),
        // Edge tap handler
        (edge) => {
            highlightDependencyPath(edge);
            showEdgeDetails(edge.data());
        },
        // Background tap handler
        () => clearSelection()
    );
}

function initUI(data) {
    setupLanguageToggle(() => {
        // Re-populate dynamic content on language change
        populateCriticalIssues();
    });

    updateHeaderStats(data.summary, data);
    updateFooterStats(data.summary);

    setupFilters();
    setupSearch();
    setupLayoutSelector();
    setupExportButtons();
    setupAnalysisButtons();
    setupClusterView();
    populateIssueList();
    setupResizableSidebar();

    setupKeyboardShortcuts({
        onEscape: () => clearSelection()
    });

    setupAutoHideTriggers();
    setupViewToggle();
    setupLegendToggle();
    setupCenterModeToggle();

    // Set up callback for feature module
    setSelectNodeCallback((node) => selectNode(node));
}

function initJobFeatures() {
    populateCriticalIssues();
    populateHotspots();
    populateModuleRankings();
    setupModuleRankingSorting();
    setupJobButtons();
    populatePathFinderSelects();
}

// =====================================================
// Data Loading
// =====================================================

async function loadConfig() {
    try {
        const response = await fetch(CONFIG.configPath);
        if (response.ok) {
            const serverConfig = await response.json();
            if (serverConfig.api_endpoint) {
                CONFIG.apiEndpoint = serverConfig.api_endpoint;
            }
        }
    } catch (e) {
        console.log('Using default config');
    }
}

async function fetchGraphData() {
    const url = CONFIG.apiEndpoint + CONFIG.graphPath;
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP ${response.status} from ${url}`);
    }
    return response.json();
}

// =====================================================
// Node Selection
// =====================================================

function selectNode(node) {
    setSelectedNode(node);

    if (state.centerMode) {
        centerOnNode(node, true);
    } else {
        focusOnNode(node);
    }

    highlightNeighbors(node);
    showNodeDetails(node.data());
    enableAnalysisButtons(true);
    showBlastRadius(node);
    updateWhatBreaksButton();
    showItemGraph(node.id());
}

function clearSelection() {
    setSelectedNode(null);

    if (state.cy) {
        state.cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');
        state.cy.fit(undefined, 50);
    }

    clearDetails();
    enableAnalysisButtons(false);
    hideItemGraph();
    clearBlastRadius();
    updateWhatBreaksButton();

    const jobResult = document.getElementById('job-result');
    if (jobResult) jobResult.innerHTML = '';
}

// =====================================================
// View Toggle (Graph / Tree)
// =====================================================

let currentView = 'graph';

function setupViewToggle() {
    document.getElementById('view-graph')?.addEventListener('click', () => {
        currentView = 'graph';
        document.getElementById('cy').style.display = 'block';
        document.getElementById('tree-view')?.style.setProperty('display', 'none');
        document.querySelectorAll('.view-toggle button').forEach(b => b.classList.remove('active'));
        document.getElementById('view-graph')?.classList.add('active');
    });

    document.getElementById('view-tree')?.addEventListener('click', () => {
        currentView = 'tree';
        document.getElementById('cy').style.display = 'none';
        const treeView = document.getElementById('tree-view');
        if (treeView) {
            treeView.style.display = 'block';
            populateTreeView();
        }
        document.querySelectorAll('.view-toggle button').forEach(b => b.classList.remove('active'));
        document.getElementById('view-tree')?.classList.add('active');
    });
}

function populateTreeView() {
    const container = document.getElementById('tree-view');
    if (!container || !state.graphData) return;

    // Group by crate
    const crates = {};
    state.graphData.nodes.forEach(node => {
        const crate = node.id.split('::')[0] || 'root';
        if (!crates[crate]) crates[crate] = [];
        crates[crate].push(node);
    });

    container.innerHTML = Object.entries(crates).map(([crate, nodes]) => `
        <div class="tree-crate">
            <div class="tree-crate-header">${escapeHtml(crate)}</div>
            <div class="tree-modules">
                ${nodes.map(n => `
                    <div class="tree-module" data-node-id="${n.id}">
                        <span class="tree-module-name">${escapeHtml(n.label)}</span>
                        <span class="tree-module-stats">
                            ${n.metrics?.fn_count || 0}fn ${n.metrics?.type_count || 0}ty ${n.metrics?.impl_count || 0}impl
                        </span>
                    </div>
                `).join('')}
            </div>
        </div>
    `).join('');

    container.querySelectorAll('.tree-module').forEach(item => {
        item.addEventListener('click', () => {
            const nodeId = item.dataset.nodeId;
            const node = state.cy?.getElementById(nodeId);
            if (node?.length) {
                // Switch to graph view and select
                document.getElementById('view-graph')?.click();
                setTimeout(() => selectNode(node), 100);
            }
        });
    });
}

// =====================================================
// Auto-hide Triggers
// =====================================================

function setupAutoHideTriggers() {
    const sidebar = document.querySelector('.sidebar');
    const toggleBtn = document.getElementById('sidebar-toggle');

    if (toggleBtn && sidebar) {
        toggleBtn.addEventListener('click', () => {
            sidebar.classList.toggle('collapsed');
            toggleBtn.textContent = sidebar.classList.contains('collapsed') ? '◀' : '▶';
        });
    }
}

// =====================================================
// Utilities
// =====================================================

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text || '';
    return div.innerHTML;
}

// =====================================================
// Start Application
// =====================================================

document.addEventListener('DOMContentLoaded', init);

// Export for debugging
window.cargoCoupling = {
    state,
    selectNode,
    clearSelection,
    applyLayout
};
