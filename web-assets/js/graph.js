// =====================================================
// Cytoscape Graph Module
// =====================================================

import { state, setCy, setCurrentLayout } from './state.js';
import { STABLE_CRATES, isExternalCrate, estimateVolatility } from './utils.js';

/**
 * Initialize Cytoscape graph with data
 */
export function initCytoscape(data, onNodeTap, onEdgeTap, onBackgroundTap) {
    const elements = buildElements(data);

    const cy = cytoscape({
        container: document.getElementById('cy'),
        elements: elements,
        style: getCytoscapeStyle(),
        layout: getLayoutConfig('cose'),
        minZoom: 0.2,
        maxZoom: 3,
        wheelSensitivity: 0.3,
        pixelRatio: 'auto'
    });

    setCy(cy);
    setupGraphEventHandlers(cy, onNodeTap, onEdgeTap, onBackgroundTap);
    return cy;
}

/**
 * Build Cytoscape elements from graph data
 */
export function buildElements(data) {
    const nodes = data.nodes.map(node => {
        const crate = node.id.split('::')[0];
        const items = node.items || [];

        // Count items by kind (prefer API values, fallback to counting items)
        const fnCount = node.metrics?.fn_count ?? items.filter(i => i.kind === 'fn').length;
        const typeCount = node.metrics?.type_count ?? items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
        const implCount = node.metrics?.impl_count ?? ((node.metrics?.trait_impl_count || 0) + (node.metrics?.inherent_impl_count || 0));

        const statsStr = `${fnCount}fn ${typeCount}ty ${implCount}impl`;

        return {
            data: {
                id: node.id,
                label: node.label,
                crate: crate,
                ...node.metrics,
                file_path: node.file_path,
                in_cycle: node.in_cycle,
                items: node.items,
                fn_count: fnCount,
                type_count: typeCount,
                impl_count: implCount,
                stats_label: statsStr
            }
        };
    });

    // Aggregate edges between same source-target pairs
    const edgeMap = new Map();

    data.edges.forEach(edge => {
        const key = `${edge.source}->${edge.target}`;
        const dims = edge.dimensions || {};

        if (!edgeMap.has(key)) {
            edgeMap.set(key, {
                source: edge.source,
                target: edge.target,
                strength: dims.strength?.value ?? 0.5,
                strengthLabel: dims.strength?.label ?? 'Model',
                distance: dims.distance?.label ?? 'DifferentModule',
                volatility: dims.volatility?.label ?? 'Low',
                balance: dims.balance?.value ?? 0.5,
                balanceLabel: dims.balance?.label ?? 'Acceptable',
                classification: dims.balance?.classification ?? '',
                classificationJa: dims.balance?.classification_ja ?? '',
                issue: edge.issue,
                inCycle: edge.in_cycle,
                location: edge.location,
                count: 1
            });
        } else {
            const existing = edgeMap.get(key);
            existing.strength = Math.max(existing.strength, dims.strength?.value ?? 0.5);
            existing.balance = Math.min(existing.balance, dims.balance?.value ?? 0.5);
            existing.inCycle = existing.inCycle || edge.in_cycle;
            existing.issue = existing.issue || edge.issue;
            existing.count++;
            if ((dims.strength?.value ?? 0) > existing.strength) {
                existing.strengthLabel = dims.strength?.label ?? existing.strengthLabel;
            }
        }
    });

    const edges = Array.from(edgeMap.entries()).map(([key, data], idx) => ({
        data: {
            id: `e${idx}`,
            source: data.source,
            target: data.target,
            strength: data.strength,
            strengthLabel: data.strengthLabel,
            distance: data.distance,
            volatility: data.volatility,
            balance: data.balance,
            balanceLabel: data.balanceLabel,
            classification: data.classification,
            classificationJa: data.classificationJa,
            issue: data.issue,
            inCycle: data.inCycle,
            location: data.location,
            count: data.count
        }
    }));

    return [...nodes, ...edges];
}

/**
 * Get Cytoscape style configuration
 */
export function getCytoscapeStyle() {
    return [
        // Node styles
        {
            selector: 'node',
            style: {
                'label': node => {
                    const label = node.data('label') || '';
                    const fn = node.data('fn_count') || 0;
                    const ty = node.data('type_count') || 0;
                    const impl = node.data('impl_count') || 0;
                    if (fn === 0 && ty === 0 && impl === 0) return label;
                    return `${label}\n${fn}fn ${ty}ty ${impl}impl`;
                },
                'text-valign': 'center',
                'text-halign': 'center',
                'text-wrap': 'wrap',
                'text-max-width': '120px',
                'background-color': node => getHealthColor(node.data('health')),
                'border-width': 2,
                'border-color': '#475569',
                'color': '#f8fafc',
                'font-size': '9px',
                'text-outline-color': '#0f172a',
                'text-outline-width': 2,
                'width': node => 40 + (node.data('couplings_out') || 0) * 2,
                'height': node => 40 + (node.data('couplings_out') || 0) * 2
            }
        },
        // Edge styles - base
        {
            selector: 'edge',
            style: {
                'width': edge => 1 + edge.data('strength') * 4,
                'line-color': edge => getEdgeColorByAnalysis(edge.data()),
                'target-arrow-color': edge => getEdgeColorByAnalysis(edge.data()),
                'target-arrow-shape': 'triangle',
                'arrow-scale': 1.5,
                'curve-style': 'bezier',
                'opacity': 0.7,
                'line-style': edge => getDistanceStyle(edge.data('distance'))
            }
        },
        // Critical coupling edges
        {
            selector: 'edge[strengthLabel="Intrusive"][distance="DifferentCrate"], edge[strengthLabel="Intrusive"][distance="DifferentModule"], edge[strengthLabel="Functional"][distance="DifferentCrate"]',
            style: {
                'line-color': '#ef4444',
                'target-arrow-color': '#ef4444',
                'width': edge => 2 + edge.data('strength') * 5,
                'opacity': 0.9
            }
        },
        // Good coupling edges
        {
            selector: 'edge[strengthLabel="Intrusive"][distance="SameModule"], edge[strengthLabel="Functional"][distance="SameModule"], edge[strengthLabel="Contract"][distance="DifferentModule"], edge[strengthLabel="Contract"][distance="DifferentCrate"]',
            style: {
                'line-color': '#22c55e',
                'target-arrow-color': '#22c55e',
                'opacity': 0.6
            }
        },
        // Edges with issues
        {
            selector: 'edge[issue]',
            style: {
                'width': edge => 3 + edge.data('strength') * 4,
                'opacity': 0.85
            }
        },
        // Cycle edges
        {
            selector: 'edge[?inCycle]',
            style: {
                'line-color': '#dc2626',
                'target-arrow-color': '#dc2626',
                'width': 3,
                'line-style': 'solid'
            }
        },
        // Highlighted state
        {
            selector: '.highlighted',
            style: {
                'opacity': 1,
                'border-width': 3,
                'border-color': '#3b82f6'
            }
        },
        // Dimmed state
        {
            selector: '.dimmed',
            style: { 'opacity': 0.15 }
        },
        // Hidden state
        {
            selector: '.hidden',
            style: { 'display': 'none' }
        },
        // Dependency highlighting
        {
            selector: '.dependency-source',
            style: {
                'border-color': '#22c55e',
                'border-width': 4
            }
        },
        {
            selector: '.dependency-target',
            style: {
                'border-color': '#ef4444',
                'border-width': 4
            }
        },
        // Hover state
        {
            selector: '.hover',
            style: {
                'border-color': '#3b82f6',
                'border-width': 3
            }
        },
        // Search match
        {
            selector: '.search-match',
            style: {
                'border-color': '#eab308',
                'border-width': 4,
                'background-color': '#eab308'
            }
        }
    ];
}

/**
 * Get layout configuration by name
 */
export function getLayoutConfig(name) {
    const configs = {
        cose: {
            name: 'cose',
            animate: true,
            animationDuration: 500,
            nodeRepulsion: 8000,
            idealEdgeLength: 100,
            edgeElasticity: 100,
            gravity: 0.25,
            numIter: 1000
        },
        dagre: {
            name: 'dagre',
            rankDir: 'TB',
            nodeSep: 50,
            rankSep: 80,
            edgeSep: 10,
            animate: true,
            animationDuration: 500,
            fit: true,
            padding: 50
        },
        concentric: {
            name: 'concentric',
            animate: true,
            animationDuration: 500,
            concentric: node => node.data('couplings_in') || 0,
            levelWidth: () => 2
        },
        grid: { name: 'grid', animate: true, animationDuration: 500, rows: 5 }
    };
    return configs[name] || configs.cose;
}

/**
 * Apply layout to graph
 */
export function applyLayout(name) {
    if (!state.cy) return;
    setCurrentLayout(name);
    state.cy.layout(getLayoutConfig(name)).run();
}

/**
 * Center view on a specific node with optional re-layout
 */
export function centerOnNode(node, useRelayout = false) {
    if (!state.cy) return;

    if (useRelayout) {
        // Re-layout with selected node at center
        const layout = state.cy.layout({
            name: 'concentric',
            concentric: function(n) {
                if (n.id() === node.id()) return 10;
                if (node.neighborhood().contains(n)) return 5;
                return 1;
            },
            levelWidth: function() { return 2; },
            animate: true,
            animationDuration: 500
        });
        layout.run();
    } else {
        // Just animate to center the node
        state.cy.animate({
            center: { eles: node },
            zoom: Math.max(state.cy.zoom(), 1),
            duration: 400,
            easing: 'ease-out-cubic'
        });
    }
}

/**
 * Focus on node with zoom
 */
export function focusOnNode(node) {
    if (!state.cy) return;
    state.cy.animate({
        center: { eles: node },
        zoom: 1.5,
        duration: 400,
        easing: 'ease-out-cubic'
    });
}

/**
 * Highlight neighbors of a node
 */
export function highlightNeighbors(node) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');
    node.neighborhood().removeClass('dimmed').addClass('highlighted');
}

/**
 * Highlight dependency path for an edge
 */
export function highlightDependencyPath(edge) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');

    const source = state.cy.getElementById(edge.data('source'));
    const target = state.cy.getElementById(edge.data('target'));

    source.removeClass('dimmed').addClass('dependency-source');
    target.removeClass('dimmed').addClass('dependency-target');
    edge.removeClass('dimmed').addClass('highlighted');

    state.cy.fit(source.union(target).union(edge), 100);
}

/**
 * Clear all highlights
 */
export function clearHighlights() {
    if (state.cy) {
        state.cy.elements().removeClass('highlighted dimmed dependency-source dependency-target search-match');
    }
}

/**
 * Setup graph event handlers
 */
function setupGraphEventHandlers(cy, onNodeTap, onEdgeTap, onBackgroundTap) {
    cy.on('tap', 'node', function(evt) {
        if (onNodeTap) onNodeTap(evt.target);
    });

    cy.on('tap', 'edge', function(evt) {
        if (onEdgeTap) onEdgeTap(evt.target);
    });

    cy.on('tap', function(evt) {
        if (evt.target === cy && onBackgroundTap) {
            onBackgroundTap();
        }
    });

    // Hover effects
    cy.on('mouseover', 'node', function(evt) {
        evt.target.addClass('hover');
    });

    cy.on('mouseout', 'node', function(evt) {
        evt.target.removeClass('hover');
    });
}

// =====================================================
// Helper Functions
// =====================================================

export function getHealthColor(health) {
    const colors = { good: '#22c55e', needs_review: '#eab308', critical: '#ef4444' };
    return colors[health] || '#64748b';
}

export function getBalanceColor(balance) {
    if (balance >= 0.8) return '#22c55e';
    if (balance >= 0.4) return '#eab308';
    return '#ef4444';
}

function getDistanceStyle(distance) {
    if (distance === 'SameModule' || distance === 'SameFunction') return 'solid';
    if (distance === 'DifferentModule') return 'dashed';
    return 'dotted';
}

function getStrengthName(value) {
    if (value >= 0.75) return 'Intrusive';
    if (value >= 0.5) return 'Functional';
    if (value >= 0.25) return 'Model';
    return 'Contract';
}

/**
 * Analyze coupling based on Khononov's balance formula
 */
export function analyzeCoupling(strength, distance, volatility, targetName = '') {
    const isStrongCoupling = ['Intrusive', 'Functional'].includes(strength);
    const isWeakCoupling = ['Model', 'Contract'].includes(strength);
    const isClose = ['SameFunction', 'SameModule'].includes(distance);
    const isFar = ['DifferentModule', 'DifferentCrate'].includes(distance);

    const effectiveVolatility = estimateVolatility(targetName, volatility);
    const isHighVolatility = effectiveVolatility === 'High';
    const isLowVolatility = effectiveVolatility === 'Low';
    const isMediumVolatility = effectiveVolatility === 'Medium';

    const isStableExternal = isExternalCrate(targetName) && isLowVolatility;
    const hasModularity = (isStrongCoupling && isClose) || (isWeakCoupling && isFar);

    // Case 1: Strong + Far
    if (isStrongCoupling && isFar) {
        if (isLowVolatility) {
            return {
                status: 'good',
                icon: '🔒',
                statusText: 'Stable External Dependency',
                action: null
            };
        }
        if (isMediumVolatility) {
            return {
                status: 'acceptable',
                icon: '⚠️',
                statusText: 'Global Complexity (Medium)',
                action: 'Consider introducing trait for abstraction or reducing distance'
            };
        }
        return {
            status: 'critical',
            icon: '❌',
            statusText: 'Global Complexity + Cascading Changes',
            action: 'Introduce trait to invert dependency (DIP) or move closer'
        };
    }

    // Case 2: Strong + Close = High Cohesion (Good)
    if (isStrongCoupling && isClose) {
        return {
            status: 'good',
            icon: '✅',
            statusText: 'High Cohesion',
            action: null
        };
    }

    // Case 3: Weak + Far = Loose Coupling (Good)
    if (isWeakCoupling && isFar) {
        return {
            status: 'good',
            icon: '✅',
            statusText: 'Loose Coupling',
            action: null
        };
    }

    // Case 4: Weak + Close = Local Complexity (potential over-abstraction)
    if (isWeakCoupling && isClose) {
        return {
            status: 'acceptable',
            icon: '🤔',
            statusText: 'Local Complexity',
            action: 'Direct access may be simpler within same module'
        };
    }

    return {
        status: 'good',
        icon: '✅',
        statusText: 'Balanced',
        action: null
    };
}

function getEdgeColorByAnalysis(data) {
    const strength = data.strengthLabel || getStrengthName(data.strength);
    const distance = data.distance || 'Unknown';
    const volatility = data.volatility || 'Medium';
    const target = data.target || '';

    const analysis = analyzeCoupling(strength, distance, volatility, target);

    switch (analysis.status) {
        case 'good': return '#22c55e';
        case 'acceptable': return '#eab308';
        case 'critical': return '#ef4444';
        default: return '#64748b';
    }
}
