// Rust Idle Game - Web Edition (JavaScript Engine)

const RESOURCES = {
    gold: { name: 'Gold', icon: 'fa-coins', color: 'var(--gold)' },
    wood: { name: 'Wood', icon: 'fa-tree', color: 'var(--wood)' },
    stone: { name: 'Stone', icon: 'fa-mountain', color: 'var(--stone)' },
    food: { name: 'Food', icon: 'fa-wheat-awn', color: 'var(--food)' },
    science: { name: 'Science', icon: 'fa-flask', color: 'var(--science)' },
    magic: { name: 'Magic', icon: 'fa-wand-magic-sparkles', color: 'var(--magic)' }
};

const BUILDINGS = {
    mine: {
        id: 'mine',
        name: 'Gold Mine',
        desc: 'Produces gold over time.',
        baseCost: { gold: 50, wood: 20 },
        costGrowth: 1.15,
        production: { gold: 1.0 },
        maxLevel: 100
    },
    lumber_mill: {
        id: 'lumber_mill',
        name: 'Lumber Mill',
        desc: 'Produces wood over time.',
        baseCost: { gold: 100, wood: 50 },
        costGrowth: 1.15,
        production: { wood: 2.0 },
        maxLevel: 100,
        prerequisite: 'wood_clicker'
    },
    quarry: {
        id: 'quarry',
        name: 'Quarry',
        desc: 'Produces stone over time.',
        baseCost: { gold: 500, wood: 200, stone: 50 },
        costGrowth: 1.2,
        production: { stone: 1.0 },
        maxLevel: 50,
        prerequisite: 'gold_multiplier'
    },
    farm: {
        id: 'farm',
        name: 'Farm',
        desc: 'Produces food over time.',
        baseCost: { gold: 200, wood: 100 },
        costGrowth: 1.15,
        production: { food: 5.0 },
        maxLevel: 100
    },
    laboratory: {
        id: 'laboratory',
        name: 'Laboratory',
        desc: 'Produces science over time.',
        baseCost: { gold: 2000, science: 100 },
        costGrowth: 1.25,
        production: { science: 2.0 },
        maxLevel: 50,
        prerequisite: 'science_unlock'
    },
    wizard_tower: {
        id: 'wizard_tower',
        name: 'Wizard Tower',
        desc: 'Produces magic over time.',
        baseCost: { gold: 10000, magic: 50, science: 500 },
        costGrowth: 1.3,
        production: { magic: 1.0 },
        maxLevel: 25,
        prerequisite: 'magic_unlock'
    }
};


const PRESTIGE_PERKS = {
    perk_click: {
        id: 'perk_click',
        name: 'Golden Overload',
        desc: '+2 Click Power per level.',
        cost: 2,
        costScaling: 1.5,
        maxLevel: 10,
        effect: { type: 'click_power', amount: 2 }
    },
    perk_speed: {
        id: 'perk_speed',
        name: 'Temporal Warp',
        desc: '+15% Global Production per level.',
        cost: 5,
        costScaling: 2.0,
        maxLevel: 10,
        effect: { type: 'global_mult', amount: 0.15 }
    },
    perk_cheap: {
        id: 'perk_cheap',
        name: "Architect's Blessing",
        desc: '-2% Building cost scaling per level.',
        cost: 10,
        costScaling: 2.5,
        maxLevel: 5,
        effect: { type: 'cost_reduction', amount: 0.02 }
    },
    perk_offline: {
        id: 'perk_offline',
        name: 'Astral Efficiency',
        desc: '+20% Offline progress efficiency per level.',
        cost: 4,
        costScaling: 1.8,
        maxLevel: 5,
        effect: { type: 'offline_eff', amount: 0.20 }
    },
    perk_alchemy: {
        id: 'perk_alchemy',
        name: "Philosopher's Stone",
        desc: '+100% Gold Production per level.',
        cost: 15,
        costScaling: 3.0,
        maxLevel: 5,
        effect: { type: 'gold_mult', amount: 1.0 }
    }
};

const UPGRADES = {
    gold_clicker: {
        id: 'gold_clicker',
        name: 'Golden Touch',
        desc: 'Clicking generates +1 gold per level.',
        cost: { gold: 10 },
        effect: { type: 'click_power', amount: 1 },
        maxLevel: 10
    },
    gold_multiplier: {
        id: 'gold_multiplier',
        name: 'Alchemy',
        desc: 'Gold production +50% per level.',
        cost: { gold: 100 },
        effect: { type: 'resource_multiplier', resource: 'gold', mult: 1.5 },
        maxLevel: 5
    },
    wood_clicker: {
        id: 'wood_clicker',
        name: 'Forestry',
        desc: 'Unlocks Lumber Mill & wood gathering.',
        cost: { gold: 50, wood: 20 },
        effect: { type: 'unlock', feature: 'wood' },
        maxLevel: 1
    },
    science_unlock: {
        id: 'science_unlock',
        name: 'Academy',
        desc: 'Unlocks Laboratory and science research.',
        cost: { gold: 1000, stone: 200 },
        effect: { type: 'unlock', feature: 'science' },
        maxLevel: 1
    },
    magic_unlock: {
        id: 'magic_unlock',
        name: 'Arcane Mastery',
        desc: 'Unlocks Wizard Tower and magic generation.',
        cost: { gold: 5000, science: 200 },
        effect: { type: 'unlock', feature: 'magic' },
        maxLevel: 1
    }
};

// Game State
let game = {
    resources: { gold: 0, wood: 0, stone: 0, food: 0, science: 0, magic: 0 },
    rates: { gold: 0.1, wood: 0.05, stone: 0.02, food: 0, science: 0, magic: 0 },
    buildings: {},
    upgrades: {},
    prestige: { points: 0, totalEarnedGold: 0, unspentPoints: 0, spentPoints: 0 },
    stats: {
        timePlayed: 0,
        totalClicks: 0,
        buildingsBuilt: 0,
        totalGained: { gold: 0, wood: 0, stone: 0, food: 0, science: 0, magic: 0 },
        totalSpent: { gold: 0, wood: 0, stone: 0, food: 0, science: 0, magic: 0 }
    },
    settings: {
        autoSave: true,
        scientificNotation: false
    }
};

// Initialize state
function initGameState() {
    for (let id in BUILDINGS) {
        game.buildings[id] = { level: 0, totalBuilt: 0 };
    }
    for (let id in UPGRADES) {
        game.upgrades[id] = { level: 0 };
    }
    if (!game.perks) {
        game.perks = {};
    }
    for (let id in PRESTIGE_PERKS) {
        if (!game.perks[id]) {
            game.perks[id] = { level: 0 };
        }
    }
}

// Format numbers
function formatNum(num) {
    if (isNaN(num) || num === null) return '0';
    if (game.settings.scientificNotation && num >= 1e6) {
        return num.toExponential(2);
    }
    if (num >= 1e12) return (num / 1e12).toFixed(2) + 'T';
    if (num >= 1e9) return (num / 1e9).toFixed(2) + 'B';
    if (num >= 1e6) return (num / 1e6).toFixed(2) + 'M';
    if (num >= 1e3) return (num / 1e3).toFixed(2) + 'k';
    if (num < 10 && num > 0 && num % 1 !== 0) return num.toFixed(2);
    return Math.floor(num).toLocaleString();
}

// Global production multiplier from prestige points
function getGlobalMultiplier() {
    let base = 1.0 + (game.prestige.points * 0.1);
    let speedLvl = game.perks && game.perks.perk_speed ? game.perks.perk_speed.level : 0;
    if (speedLvl > 0) {
        base += speedLvl * 0.15;
    }
    return base;
}

// Click Power calculation
function getClickPower() {
    let base = 1;
    let goldClickerLevel = game.upgrades.gold_clicker ? game.upgrades.gold_clicker.level : 0;
    let perkClickLvl = game.perks && game.perks.perk_click ? game.perks.perk_click.level : 0;
    return base + goldClickerLevel + (perkClickLvl * 2);
}

// Calculate resource rates
function recalculateRates() {
    // Base rates
    let newRates = { gold: 0.1, wood: 0.05, stone: 0.02, food: 0, science: 0, magic: 0 };

    // Add building production
    for (let bId in BUILDINGS) {
        let b = BUILDINGS[bId];
        let lvl = game.buildings[bId].level;
        if (lvl > 0) {
            for (let r in b.production) {
                newRates[r] = (newRates[r] || 0) + (b.production[r] * lvl);
            }
        }
    }

    // Apply upgrade multipliers
        let alchemyLvl = game.upgrades.gold_multiplier ? game.upgrades.gold_multiplier.level : 0;
        if (alchemyLvl > 0) {
            newRates.gold *= Math.pow(1.5, alchemyLvl);
        }

        // Apply perk_alchemy
        let perkAlchemyLvl = game.perks && game.perks.perk_alchemy ? game.perks.perk_alchemy.level : 0;
        if (perkAlchemyLvl > 0) {
            newRates.gold *= (1.0 + (perkAlchemyLvl * 1.0));
        }

        // Apply global multiplier
    let globalMult = getGlobalMultiplier();
    for (let r in newRates) {
        newRates[r] *= globalMult;
    }

    game.rates = newRates;
}

// Toast notification
function showToast(message) {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => {
        toast.remove();
    }, 3000);
}

// UI State tracking
let uiInitialized = false;
let buildingCards = {};
let upgradeCards = {};

// Initialize static UI elements (called once)
function initUI() {
    // Resources list - build once
    buildResourcesList();
    
    // Buildings Grid - build once
    buildBuildingsGrid();
    
    // Upgrades Grid - build once
    buildUpgradesGrid();
    
    // Perks - build once
    buildPerksContainer();
    
    uiInitialized = true;
}

function buildResourcesList() {
    const resList = document.getElementById('resource-list');
    resList.innerHTML = '';
    for (let rKey in RESOURCES) {
        let rDef = RESOURCES[rKey];
        const item = document.createElement('div');
        item.className = 'resource-item';
        item.id = `resource-item-${rKey}`;
        item.innerHTML = `
            <div class="resource-info" style="color: ${rDef.color}">
                <i class="fa-solid ${rDef.icon}"></i>
                <span>${rDef.name}</span>
            </div>
            <div class="resource-values">
                <span class="resource-amount">0</span>
                <span class="resource-rate">+0/s</span>
            </div>
        `;
        resList.appendChild(item);
    }
}

function buildBuildingsGrid() {
    const bGrid = document.getElementById('buildings-grid');
    bGrid.innerHTML = '';
    buildingCards = {};
    
    for (let bId in BUILDINGS) {
        let b = BUILDINGS[bId];
        let state = game.buildings[bId];
        
        // Check prerequisites - still build but hide if locked
        let isLocked = b.prerequisite && (!game.upgrades[b.prerequisite] || game.upgrades[b.prerequisite].level === 0);
        
        const card = document.createElement('div');
        card.className = 'entity-card' + (isLocked ? ' locked' : '');
        card.id = `building-card-${bId}`;
        card.innerHTML = `
            <div class="entity-header">
                <div class="entity-title">
                    <h4>${b.name}</h4>
                    <span class="level-display">Level ${state.level} / ${b.maxLevel}</span>
                </div>
                <div class="entity-level">Lvl ${state.level}</div>
            </div>
            <div class="entity-desc">${b.desc}</div>
            <div class="entity-footer">
                <div class="entity-costs" id="building-costs-${bId}"></div>
                <button class="btn btn-primary building-btn" id="building-btn-${bId}" onclick="buyBuilding('${bId}')">
                    Build
                </button>
            </div>
        `;
        bGrid.appendChild(card);
        buildingCards[bId] = { card, b, state };
    }
}

function buildUpgradesGrid() {
    const uGrid = document.getElementById('upgrades-grid');
    uGrid.innerHTML = '';
    upgradeCards = {};
    
    for (let uId in UPGRADES) {
        let u = UPGRADES[uId];
        let state = game.upgrades[uId];
        
        const card = document.createElement('div');
        card.className = 'entity-card';
        card.id = `upgrade-card-${uId}`;
        card.innerHTML = `
            <div class="entity-header">
                <div class="entity-title">
                    <h4>${u.name}</h4>
                    <span>Research</span>
                </div>
                <div class="entity-level" id="upgrade-level-${uId}">${state.level}/${u.maxLevel}</div>
            </div>
            <div class="entity-desc">${u.desc}</div>
            <div class="entity-footer">
                <div class="entity-costs" id="upgrade-costs-${uId}"></div>
                <button class="btn btn-primary upgrade-btn" id="upgrade-btn-${uId}" onclick="buyUpgrade('${uId}')">
                    Research
                </button>
            </div>
        `;
        uGrid.appendChild(card);
        upgradeCards[uId] = { card, u, state };
    }
}

function buildPerksContainer() {
    const perksContainer = document.getElementById('perks-container');
    if (!perksContainer) return;
    perksContainer.innerHTML = '';
    
    for (let perkId in PRESTIGE_PERKS) {
        let perk = PRESTIGE_PERKS[perkId];
        let state = game.perks ? game.perks[perkId] : { level: 0 };
        if (!game.perks) game.perks = {};
        if (!game.perks[perkId]) game.perks[perkId] = { level: 0 };
        state = game.perks[perkId];

        const card = document.createElement('div');
        card.className = 'entity-card perk-card';
        card.id = `perk-card-${perkId}`;
        card.innerHTML = `
            <div class="entity-header">
                <div class="entity-title">
                    <h4>${perk.name}</h4>
                    <span class="perk-desc">${perk.desc}</span>
                </div>
                <div class="entity-level" id="perk-level-${perkId}">Lvl ${state.level} / ${perk.maxLevel}</div>
            </div>
            <div class="entity-footer">
                <div class="entity-costs" id="perk-costs-${perkId}"></div>
                <button class="btn btn-primary perk-btn" id="perk-btn-${perkId}" onclick="buyPrestigePerk('${perkId}')">
                    Purchase
                </button>
            </div>
        `;
        perksContainer.appendChild(card);
    }
}

// Update dynamic values only (called every tick)
function updateUI() {
    if (!uiInitialized) {
        initUI();
    }
    
    // Header stats
    document.getElementById('header-prestige-points').textContent = formatNum(game.prestige.points);
    document.getElementById('header-multiplier').textContent = (getGlobalMultiplier() * 100).toFixed(0) + '%';
    document.getElementById('click-power-label').textContent = `+${formatNum(getClickPower())} Gold`;

    // Update Resources (only amounts and rates)
    for (let rKey in RESOURCES) {
        let item = document.getElementById(`resource-item-${rKey}`);
        if (!item) continue;
        
        let rDef = RESOURCES[rKey];
        let amount = game.resources[rKey] || 0;
        let rate = game.rates[rKey] || 0;

        // Hide locked advanced resources
        if ((rKey === 'science' || rKey === 'magic' || rKey === 'stone') && amount === 0 && rate === 0) {
            let unlockReq = rKey === 'science' ? 'science_unlock' : (rKey === 'magic' ? 'magic_unlock' : 'wood_clicker');
            if (game.upgrades[unlockReq] && game.upgrades[unlockReq].level === 0) {
                item.style.display = 'none';
                continue;
            }
        }
        item.style.display = '';

        item.querySelector('.resource-amount').textContent = formatNum(amount);
        item.querySelector('.resource-rate').textContent = `+${formatNum(rate)}/s`;
    }

    // Update Buildings Grid (only costs, affordability, button state, level)
    for (let bId in BUILDINGS) {
        let b = BUILDINGS[bId];
        let state = game.buildings[bId];
        let cardData = buildingCards[bId];
        
        if (!cardData) continue;
        
        let card = cardData.card;
        
        // Check prerequisites - show/hide card
        let isLocked = b.prerequisite && (!game.upgrades[b.prerequisite] || game.upgrades[b.prerequisite].level === 0);
        card.classList.toggle('locked', isLocked);
        
        let costMultiplier = Math.pow(b.costGrowth, state.level);
        let canAfford = true;
        let costHtml = '';

        for (let r in b.baseCost) {
            let cost = Math.floor(b.baseCost[r] * costMultiplier);
            let hasEnough = (game.resources[r] || 0) >= cost;
            if (!hasEnough) canAfford = false;
            let rDef = RESOURCES[r];
            costHtml += `
                <div class="cost-item ${hasEnough ? 'affordable' : 'unaffordable'}">
                    <i class="fa-solid ${rDef ? rDef.icon : 'fa-coins'}"></i>
                    <span>${formatNum(cost)}</span>
                </div>
            `;
        }

        // Update only dynamic parts
        document.getElementById(`building-costs-${bId}`).innerHTML = costHtml;
        document.getElementById(`building-btn-${bId}`).disabled = !canAfford || state.level >= b.maxLevel;
        card.querySelector('.level-display').textContent = `Level ${state.level} / ${b.maxLevel}`;
        card.querySelector('.entity-level').textContent = `Lvl ${state.level}`;
        
        // Update max level display
        if (state.level >= b.maxLevel) {
            document.getElementById(`building-btn-${bId}`).textContent = 'Maxed';
        } else {
            document.getElementById(`building-btn-${bId}`).textContent = 'Build';
        }
    }

    // Update Upgrades Grid (only costs, affordability, button state, level)
    for (let uId in UPGRADES) {
        let u = UPGRADES[uId];
        let state = game.upgrades[uId];
        let cardData = upgradeCards[uId];
        
        if (!cardData) continue;
        
        if (state.level >= u.maxLevel) {
            // Hide maxed upgrades
            cardData.card.style.display = 'none';
            continue;
        }
        
        cardData.card.style.display = '';
        
        let canAfford = true;
        let costHtml = '';
        for (let r in u.cost) {
            let cost = u.cost[r];
            let hasEnough = (game.resources[r] || 0) >= cost;
            if (!hasEnough) canAfford = false;
            let rDef = RESOURCES[r];
            costHtml += `
                <div class="cost-item ${hasEnough ? 'affordable' : 'unaffordable'}">
                    <i class="fa-solid ${rDef ? rDef.icon : 'fa-coins'}"></i>
                    <span>${formatNum(cost)}</span>
                </div>
            `;
        }

        // Update only dynamic parts
        document.getElementById(`upgrade-costs-${uId}`).innerHTML = costHtml;
        document.getElementById(`upgrade-btn-${uId}`).disabled = !canAfford;
        document.getElementById(`upgrade-level-${uId}`).textContent = `${state.level}/${u.maxLevel}`;
    }

    // Update Perks (only costs, affordability, button state, level)
    const perksContainer = document.getElementById('perks-container');
    if (perksContainer) {
        let unspentPoints = game.prestige.unspentPoints !== undefined ? game.prestige.unspentPoints : 0;
        document.getElementById('unspent-prestige-points').textContent = formatNum(unspentPoints);

        for (let perkId in PRESTIGE_PERKS) {
            let perk = PRESTIGE_PERKS[perkId];
            let state = game.perks ? game.perks[perkId] : { level: 0 };
            if (!game.perks) game.perks = {};
            if (!game.perks[perkId]) game.perks[perkId] = { level: 0 };
            state = game.perks[perkId];

            let cost = Math.floor(perk.cost * Math.pow(perk.costScaling, state.level));
            let canAfford = unspentPoints >= cost && state.level < perk.maxLevel;
            let isMaxed = state.level >= perk.maxLevel;

            // Update only dynamic parts
            document.getElementById(`perk-costs-${perkId}`).innerHTML = `
                <div class="cost-item ${canAfford ? 'affordable' : 'unaffordable'}">
                    <i class="fa-solid fa-star text-warning"></i>
                    <span>${isMaxed ? 'MAX' : formatNum(cost)} PP</span>
                </div>
            `;
            document.getElementById(`perk-btn-${perkId}`).disabled = !canAfford || isMaxed;
            document.getElementById(`perk-btn-${perkId}`).textContent = isMaxed ? 'Maxed' : 'Purchase';
            document.getElementById(`perk-level-${perkId}`).textContent = `Lvl ${state.level} / ${perk.maxLevel}`;
            
            // Update maxed visual state
            let perkCard = document.getElementById(`perk-card-${perkId}`);
            if (perkCard) {
                perkCard.classList.toggle('maxed', isMaxed);
            }
        }
    }

    // Prestige calculation
    let totalGold = game.stats.totalGained.gold || 0;
    let pendingPrestige = Math.floor(Math.sqrt(totalGold) / 10);
    document.getElementById('pending-prestige-points').textContent = formatNum(pendingPrestige);
    document.getElementById('prestige-btn').disabled = pendingPrestige <= game.prestige.points;

    // Statistics Tab
    const statsGrid = document.getElementById('stats-grid');
    statsGrid.innerHTML = `
        <div class="stat-box">
            <span>Time Played</span>
            <strong>${Math.floor(game.stats.timePlayed)}s</strong>
        </div>
        <div class="stat-box">
            <span>Total Clicks</span>
            <strong>${formatNum(game.stats.totalClicks)}</strong>
        </div>
        <div class="stat-box">
            <span>Buildings Built</span>
            <strong>${formatNum(game.stats.buildingsBuilt)}</strong>
        </div>
        <div class="stat-box">
            <span>Lifetime Gold Earned</span>
            <strong>${formatNum(game.stats.totalGained.gold)}</strong>
        </div>
        <div class="stat-box">
            <span>Prestige Count</span>
            <strong>${formatNum(game.prestige.points)}</strong>
        </div>
    `;
}

// Actions
function clickGold() {
    let power = getClickPower();
    game.resources.gold = (game.resources.gold || 0) + power;
    game.stats.totalClicks++;
    game.stats.totalGained.gold = (game.stats.totalGained.gold || 0) + power;
    game.prestige.totalEarnedGold = (game.prestige.totalEarnedGold || 0) + power;
    updateUI();
}

function buyBuilding(bId) {
    let b = BUILDINGS[bId];
    let state = game.buildings[bId];
    if (state.level >= b.maxLevel) return;

    let costMultiplier = Math.pow(b.costGrowth, state.level);
    let costs = {};
    for (let r in b.baseCost) {
        costs[r] = Math.floor(b.baseCost[r] * costMultiplier);
        if ((game.resources[r] || 0) < costs[r]) return;
    }

    for (let r in costs) {
        game.resources[r] -= costs[r];
        game.stats.totalSpent[r] = (game.stats.totalSpent[r] || 0) + costs[r];
    }

    state.level++;
    state.totalBuilt++;
    game.stats.buildingsBuilt++;
    recalculateRates();
    updateUI();
    showToast(`Constructed ${b.name} (Level ${state.level})`);
}

function buyUpgrade(uId) {
    let u = UPGRADES[uId];
    let state = game.upgrades[uId];
    if (state.level >= u.maxLevel) return;

    for (let r in u.cost) {
        if ((game.resources[r] || 0) < u.cost[r]) return;
    }

    for (let r in u.cost) {
        game.resources[r] -= u.cost[r];
        game.stats.totalSpent[r] = (game.stats.totalSpent[r] || 0) + u.cost[r];
    }

    state.level++;
    recalculateRates();
    updateUI();
    showToast(`Researched ${u.name}!`);
}

function performPrestige() {
    let totalGold = game.stats.totalGained.gold || 0;
    let pendingPrestige = Math.floor(Math.sqrt(totalGold) / 10);
    let gained = pendingPrestige - game.prestige.points;
    if (gained <= 0) return;

    game.prestige.points = pendingPrestige;
    game.prestige.unspentPoints = (game.prestige.unspentPoints || 0) + gained;

    // Reset resources and buildings
    game.resources = { gold: 0, wood: 0, stone: 0, food: 0, science: 0, magic: 0 };
    initGameState();
    recalculateRates();
    updateUI();
    showToast(`Ascended! Gained ${gained} Prestige Points.`);
    saveGame();
}

// Save / Load
function saveGame() {
    localStorage.setItem('rust_idle_save', JSON.stringify(game));
    showToast('Game saved successfully');
}

function loadGame() {
    let saved = localStorage.getItem('rust_idle_save');
    if (saved) {
        try {
            let data = JSON.parse(saved);
            // Merge with default to ensure new fields are present
            game = Object.assign({}, game, data);
            recalculateRates();
            showToast('Game loaded successfully');
        } catch (e) {
            console.error('Failed to load save:', e);
        }
    }
}

function exportSave() {
    let json = JSON.stringify(game);
    let encoded = btoa(unescape(encodeURIComponent(json)));
    document.getElementById('modal-title').textContent = 'Export Save Code';
    document.getElementById('save-code-textarea').value = encoded;
    document.getElementById('save-modal').classList.add('active');
}

function importSavePrompt() {
    document.getElementById('modal-title').textContent = 'Import Save Code';
    document.getElementById('save-code-textarea').value = '';
    document.getElementById('save-modal').classList.add('active');
}

// Game Loop Tick (every 100ms)
function gameTick() {
    let dt = 0.1; // 100ms
    game.stats.timePlayed += dt;

    for (let r in game.rates) {
        let inc = (game.rates[r] || 0) * dt;
        if (inc > 0) {
            game.resources[r] = (game.resources[r] || 0) + inc;
            game.stats.totalGained[r] = (game.stats.totalGained[r] || 0) + inc;
            if (r === 'gold') {
                game.prestige.totalEarnedGold = (game.prestige.totalEarnedGold || 0) + inc;
            }
        }
    }

    updateUI();
}

// Auto-save timer (every 30s)
setInterval(() => {
    if (game.settings.autoSave) {
        saveGame();
    }
}, 30000);

// Event Listeners
document.addEventListener('DOMContentLoaded', () => {
    initGameState();
    loadGame();
    recalculateRates();
    updateUI();

    // Start loop
    setInterval(gameTick, 100);

    // Click button
    document.getElementById('click-gold-btn').addEventListener('click', clickGold);
    document.getElementById('prestige-btn').addEventListener('click', performPrestige);
    document.getElementById('save-btn').addEventListener('click', saveGame);
    document.getElementById('export-btn').addEventListener('click', exportSave);
    document.getElementById('import-btn').addEventListener('click', importSavePrompt);
    document.getElementById('reset-btn').addEventListener('click', () => {
        if (confirm('Are you sure you want to hard reset all progress?')) {
            localStorage.removeItem('rust_idle_save');
            location.reload();
        }
    });

    // Settings
    const autoSaveCheckbox = document.getElementById('setting-autosave');
    autoSaveCheckbox.checked = game.settings.autoSave;
    autoSaveCheckbox.addEventListener('change', (e) => {
        game.settings.autoSave = e.target.checked;
    });

    const scientificCheckbox = document.getElementById('setting-scientific');
    scientificCheckbox.checked = game.settings.scientificNotation;
    scientificCheckbox.addEventListener('change', (e) => {
        game.settings.scientificNotation = e.target.checked;
        updateUI();
    });

    document.getElementById('settings-export-btn').addEventListener('click', exportSave);
    document.getElementById('settings-import-btn').addEventListener('click', importSavePrompt);

    // Tabs
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
        });
    });

    // Modal
    document.getElementById('modal-close-btn').addEventListener('click', () => {
        document.getElementById('save-modal').classList.remove('active');
    });

    document.getElementById('modal-action-btn').addEventListener('click', () => {
        let textarea = document.getElementById('save-code-textarea');
        let title = document.getElementById('modal-title').textContent;
        if (title.includes('Export')) {
            textarea.select();
            navigator.clipboard.writeText(textarea.value);
            showToast('Save code copied to clipboard!');
        } else {
            try {
                let json = decodeURIComponent(escape(atob(textarea.value.trim())));
                let data = JSON.parse(json);
                game = Object.assign({}, game, data);
                recalculateRates();
                updateUI();
                saveGame();
                document.getElementById('save-modal').classList.remove('active');
                showToast('Game imported successfully!');
            } catch (e) {
                alert('Invalid save code!');
            }
        }
    });
});


function buyPrestigePerk(perkId) {
    let perk = PRESTIGE_PERKS[perkId];
    if (!game.perks) game.perks = {};
    if (!game.perks[perkId]) game.perks[perkId] = { level: 0 };
    let perkState = game.perks[perkId];
    
    if (perkState.level >= perk.maxLevel) return;
    
    // Calculate cost based on level
    let cost = Math.floor(perk.cost * Math.pow(perk.costScaling, perkState.level));
    let unspent = game.prestige.unspentPoints !== undefined ? game.prestige.unspentPoints : game.prestige.points;
    
    if (unspent < cost) {
        showToast('Not enough Prestige Points!');
        return;
    }
    
    if (game.prestige.unspentPoints !== undefined) {
        game.prestige.unspentPoints -= cost;
    } else {
        game.prestige.points -= cost; // fallback
    }
    game.prestige.spentPoints = (game.prestige.spentPoints || 0) + cost;
    
    perkState.level++;
    recalculateRates();
    updateUI();
    showToast(`Purchased perk: ${perk.name}!`);
}
