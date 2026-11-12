// Rustbound: The Systems Frontier - Web Interactive Showcase & Simulator
(function() {
  'use strict';

  // --- MISSION DEFINITIONS & FIRMWARE ---
  const MISSIONS = {
    1: {
      id: 1,
      name: "Cold Boot",
      machine: "Aux Generator Unit #01",
      badge: "Syntax & Telemetry",
      description: "Initialize the emergency power turbine by polling hardware telemetry and submitting the ignition state transition packet.",
      code: `// Cold Boot Firmware: Aux Generator Ignition
#![no_std]
use rustbound_api::prelude::*;

pub fn tick(gen: &mut Generator) {
    let telemetry = gen.poll_telemetry();
    
    // Check if the generator is idling or offline
    if telemetry.rpm < 500 && telemetry.status_code == StatusCode::Standby {
        // Issue ignition command with safe 3000 RPM baseline
        gen.command_ignite(TargetRpm(3000));
    } else if telemetry.rpm >= 2800 {
        // Lock grid synchronization relay
        gen.engage_power_grid();
    }
}`,
      initialTelemetry: {
        rpm: 0,
        temp: 24,
        stability: 12,
        power: 0,
        status: "STANDBY"
      },
      targetCondition: (state) => state.rpm >= 2800 && state.power >= 95
    },
    2: {
      id: 2,
      name: "Subterranean Extraction",
      machine: "Quarry Drill Core #04",
      badge: "Control Flow & Thermals",
      description: "Drill geothermal ore deposits while dynamically throttling motor torque to prevent thermal runaway beyond 85°C.",
      code: `// Subterranean Extraction Firmware: Thermal Governor
#![no_std]
use rustbound_api::prelude::*;

pub fn tick(drill: &mut QuarryDrill) {
    let telemetry = drill.poll_telemetry();

    // Prevent thermal runaway: throttle if temp exceeds 80°C
    if telemetry.core_temperature_c > 80.0 {
        drill.set_motor_torque(Torque::Idle);
        drill.engage_coolant_purge();
    } else if telemetry.core_temperature_c < 45.0 {
        // Temperature safe: resume aggressive borehole boring
        drill.set_motor_torque(Torque::High);
        drill.advance_drillhead(DepthRate::Fast);
    }
}`,
      initialTelemetry: {
        rpm: 1200,
        temp: 35,
        stability: 98,
        depth: 0,
        status: "DRILLING"
      },
      targetCondition: (state) => state.depth >= 100
    },
    3: {
      id: 3,
      name: "Single Owner Custody",
      machine: "Extraction & Logistics Grid",
      badge: "Ownership & Linear Types",
      description: "Transfer ResourceChunk<Ore> tokens from the Drill through the Conveyor into Storage without duplicating or dropping linear tokens.",
      code: `// Single Owner Custody: Zero-Leak Linear Pipeline
#![no_std]
use rustbound_api::prelude::*;

pub fn tick(drill: &mut QuarryDrill, conveyor: &mut ConveyorBelt, bay: &mut StorageBay) {
    // 1. Extract linear custody token from drill hopper
    if let Some(ore_chunk) = drill.try_extract_ore() {
        // ore_chunk: ResourceChunk<CobaltOre> (Cannot be cloned or dropped!)
        
        // 2. Move linear token into conveyor belt intake
        let transit_token = conveyor.intake(ore_chunk);
        
        // 3. Advance conveyor and deposit into secure storage
        if let Some(arrived_chunk) = conveyor.advance_and_pop(transit_token) {
            bay.deposit(arrived_chunk);
        }
    }
}`,
      initialTelemetry: {
        drillOre: 10,
        conveyorItems: 0,
        storedOre: 0,
        custodyLeaks: 0,
        status: "PIPELINE_ACTIVE"
      },
      targetCondition: (state) => state.storedOre >= 5
    }
  };

  let activeMissionId = 1;
  let isSimulating = false;
  let simInterval = null;
  let simTick = 0;
  let simState = JSON.parse(JSON.stringify(MISSIONS[1].initialTelemetry));

  // --- CANVAS FACTORY VISUALIZATION ---
  const canvas = document.getElementById('factory-canvas');
  const ctx = canvas.getContext('2d');
  let animationFrameId = null;

  // Resize canvas to match display container
  function resizeCanvas() {
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * window.devicePixelRatio;
    canvas.height = rect.height * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
  }
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();

  // Conveyor particles
  const particles = [
    { x: 260, y: 160, progress: 0.1, id: 'CHUNK_#001' },
    { x: 380, y: 160, progress: 0.45, id: 'CHUNK_#002' },
    { x: 500, y: 160, progress: 0.8, id: 'CHUNK_#003' }
  ];

  function drawFactory(time) {
    const width = canvas.width / window.devicePixelRatio;
    const height = canvas.height / window.devicePixelRatio;

    // Clear background
    ctx.fillStyle = '#0a0d14';
    ctx.fillRect(0, 0, width, height);

    // Floor grid lines
    ctx.strokeStyle = '#161d2b';
    ctx.lineWidth = 1;
    const gridSize = 40;
    for (let x = 0; x < width; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    for (let y = 0; y < height; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Power conduit glows
    ctx.strokeStyle = 'rgba(0, 255, 204, 0.25)';
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(80, 260);
    ctx.lineTo(200, 260);
    ctx.lineTo(200, 160);
    ctx.lineTo(650, 160);
    ctx.lineTo(650, 260);
    ctx.lineTo(760, 260);
    ctx.stroke();

    // Machine 1: Aux Generator (Left)
    drawGenerator(80, 210, time);

    // Machine 2: Quarry Drill (Center-Left)
    drawDrill(200, 100, time);

    // Machine 3: Conveyor Belt (Connecting center)
    drawConveyor(270, 140, 360, time);

    // Machine 4: Storage Bay (Right)
    drawStorageBay(670, 100, time);

    // Animate and draw ore packets along conveyor
    if (isSimulating || activeMissionId === 3) {
      particles.forEach(p => {
        p.progress += 0.008;
        if (p.progress > 1.0) p.progress = 0;
        const px = 270 + p.progress * 360;
        const py = 155;

        // Draw ore chunk
        ctx.fillStyle = '#e27d60';
        ctx.shadowColor = '#e27d60';
        ctx.shadowBlur = 8;
        ctx.fillRect(px - 6, py - 6, 12, 12);
        ctx.shadowBlur = 0;

        // Owner token badge
        ctx.fillStyle = '#00ffcc';
        ctx.font = '8px monospace';
        ctx.fillText("TOKEN", px - 12, py - 10);
      });
    }

    animationFrameId = requestAnimationFrame(drawFactory);
  }

  function drawGenerator(x, y, time) {
    // Base platform
    ctx.fillStyle = '#171d28';
    ctx.strokeStyle = '#2d3748';
    ctx.lineWidth = 2;
    ctx.fillRect(x, y, 90, 80);
    ctx.strokeRect(x, y, 90, 80);

    // Rotating Turbine Core
    const angle = time * 0.003 * (simState.rpm ? simState.rpm / 1000 : 0.5);
    ctx.save();
    ctx.translate(x + 45, y + 40);
    ctx.rotate(angle);
    ctx.strokeStyle = simState.rpm > 2000 ? '#00ffcc' : '#e27d60';
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(0, 0, 24, 0, Math.PI * 2);
    ctx.stroke();
    // Rotor blades
    for (let i = 0; i < 4; i++) {
      ctx.rotate(Math.PI / 2);
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(0, 20);
      ctx.stroke();
    }
    ctx.restore();

    // Machine label
    ctx.fillStyle = '#8b949e';
    ctx.font = '10px monospace';
    ctx.fillText("GENERATOR", x + 12, y + 100);
  }

  function drawDrill(x, y, time) {
    // Gantry tower
    ctx.fillStyle = '#1b2230';
    ctx.strokeStyle = '#2d3748';
    ctx.lineWidth = 2;
    ctx.fillRect(x, y, 60, 110);
    ctx.strokeRect(x, y, 60, 110);

    // Reciprocating drillhead
    const drillOffset = Math.sin(time * 0.02) * (isSimulating ? 8 : 2);
    ctx.fillStyle = simState.temp > 70 ? '#f85149' : '#e27d60';
    ctx.fillRect(x + 22, y + 50 + drillOffset, 16, 50);

    // Heat glow
    if (simState.temp > 60) {
      ctx.fillStyle = 'rgba(248, 81, 73, 0.2)';
      ctx.beginPath();
      ctx.arc(x + 30, y + 90, 25, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.fillStyle = '#8b949e';
    ctx.font = '10px monospace';
    ctx.fillText("DRILL CORE", x + 2, y + 130);
  }

  function drawConveyor(x, y, len, time) {
    // Belt track
    ctx.fillStyle = '#121721';
    ctx.strokeStyle = '#263245';
    ctx.lineWidth = 2;
    ctx.fillRect(x, y, len, 24);
    ctx.strokeRect(x, y, len, 24);

    // Rollers
    const rollerCount = 10;
    const spacing = len / rollerCount;
    for (let i = 0; i <= rollerCount; i++) {
      const rx = x + i * spacing;
      ctx.fillStyle = '#263245';
      ctx.beginPath();
      ctx.arc(rx, y + 12, 4, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.fillStyle = '#8b949e';
    ctx.font = '10px monospace';
    ctx.fillText("CONVEYOR LOGISTICS (MOVE SEMANTICS)", x + 60, y + 42);
  }

  function drawStorageBay(x, y, time) {
    // Vault chamber
    ctx.fillStyle = '#171d28';
    ctx.strokeStyle = '#2d3748';
    ctx.lineWidth = 2;
    ctx.fillRect(x, y, 80, 110);
    ctx.strokeRect(x, y, 80, 110);

    // Vault door hatch
    ctx.strokeStyle = '#00ffcc';
    ctx.lineWidth = 2;
    ctx.strokeRect(x + 15, y + 30, 50, 60);

    // Stored cubes indicator
    const stored = simState.storedOre || 2;
    for (let i = 0; i < Math.min(stored, 8); i++) {
      const cx = x + 22 + (i % 3) * 13;
      const cy = y + 70 - Math.floor(i / 3) * 14;
      ctx.fillStyle = '#e27d60';
      ctx.fillRect(cx, cy, 10, 10);
    }

    ctx.fillStyle = '#8b949e';
    ctx.font = '10px monospace';
    ctx.fillText("STORAGE BAY", x + 5, y + 130);
  }

  animationFrameId = requestAnimationFrame(drawFactory);

  // --- UI & MISSION CONTROLLER ---
  const missionTabs = document.querySelectorAll('.mission-tab');
  const codeEditor = document.getElementById('code-editor');
  const missionTitle = document.getElementById('mission-title');
  const missionDesc = document.getElementById('mission-desc');
  const targetMachine = document.getElementById('target-machine');
  const btnDeploy = document.getElementById('btn-deploy');
  const btnReset = document.getElementById('btn-reset');
  const terminalBody = document.getElementById('terminal-body');

  // Syntax highlighter for Rust code
  function highlightRust(code) {
    let escaped = code
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Comments
    escaped = escaped.replace(/(\/\/.*$)/gm, '<span class="syn-comment">$1</span>');
    // Attributes
    escaped = escaped.replace(/(#!\[.*?\])/g, '<span class="syn-macro">$1</span>');
    // Keywords
    escaped = escaped.replace(/\b(pub|fn|let|mut|if|else|use|struct|enum|match|return)\b/g, '<span class="syn-kw">$1</span>');
    // Types
    escaped = escaped.replace(/\b(Generator|QuarryDrill|ConveyorBelt|StorageBay|StatusCode|TargetRpm|Torque|DepthRate|ResourceChunk|CobaltOre|Some|None|Option)\b/g, '<span class="syn-type">$1</span>');
    // Numbers
    escaped = escaped.replace(/\b(\d+(\.\d+)?)\b/g, '<span class="syn-num">$1</span>');

    return escaped;
  }

  function logTerminal(msg, type = 'sim') {
    const line = document.createElement('div');
    line.className = `log-line log-${type}`;
    line.textContent = `[${new Date().toISOString().substring(11, 19)}] ${msg}`;
    terminalBody.appendChild(line);
    terminalBody.scrollTop = terminalBody.scrollHeight;
  }

  function loadMission(id) {
    activeMissionId = id;
    const mission = MISSIONS[id];

    // Reset simulation
    stopSimulation();
    simState = JSON.parse(JSON.stringify(mission.initialTelemetry));
    simTick = 0;

    // Update Tab UI
    missionTabs.forEach(t => t.classList.toggle('active', parseInt(t.dataset.mission) === id));

    // Update text
    missionTitle.textContent = `Mission 0${id}: ${mission.name}`;
    missionDesc.textContent = mission.description;
    targetMachine.textContent = mission.machine;

    // Load and highlight code
    codeEditor.innerHTML = highlightRust(mission.code);

    updateGauges();
    logTerminal(`Mission 0${id} [${mission.name}] loaded. Waiting for firmware deployment.`, 'arbiter');
  }

  function updateGauges() {
    const gaugesContainer = document.getElementById('gauges-container');
    if (!gaugesContainer) return;

    if (activeMissionId === 1) {
      gaugesContainer.innerHTML = `
        <div class="gauge-card">
          <div class="gauge-header">
            <span>TURBINE VELOCITY</span>
            <span class="gauge-value">${Math.round(simState.rpm)} RPM</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width: ${Math.min(100, (simState.rpm / 3000) * 100)}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>GRID SYNCHRONIZATION</span>
            <span class="gauge-value">${Math.round(simState.power)}%</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width: ${simState.power}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>CORE STABILITY</span>
            <span class="gauge-value">${simState.stability}%</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width: ${simState.stability}%"></div>
          </div>
        </div>
        <div class="custody-box">
          <div class="custody-box-title">FERRIC ARBITER SAFETY STATE</div>
          <div>STATUS: ${simState.status}</div>
          <div>ISOLATION: NONE (SANDBOX SECURE)</div>
        </div>
      `;
    } else if (activeMissionId === 2) {
      const isHot = simState.temp > 70;
      gaugesContainer.innerHTML = `
        <div class="gauge-card">
          <div class="gauge-header">
            <span>CORE TEMPERATURE</span>
            <span class="gauge-value ${isHot ? 'danger' : ''}">${Math.round(simState.temp)}°C</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill ${isHot ? 'danger' : ''}" style="width: ${Math.min(100, (simState.temp / 100) * 100)}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>BOREHOLE DEPTH</span>
            <span class="gauge-value">${Math.round(simState.depth)} m</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width: ${Math.min(100, (simState.depth / 100) * 100)}%"></div>
          </div>
        </div>
        <div class="custody-box">
          <div class="custody-box-title">THERMAL GOVERNOR STATUS</div>
          <div>MOTOR STATE: ${simState.temp > 75 ? 'THROTTLED (COOLING)' : 'FULL POWER'}</div>
          <div>RUNAWAY RISK: 0.00% (FERRIC VALIDATED)</div>
        </div>
      `;
    } else if (activeMissionId === 3) {
      gaugesContainer.innerHTML = `
        <div class="gauge-card">
          <div class="gauge-header">
            <span>STORED ORE CHUNKS</span>
            <span class="gauge-value">${simState.storedOre} / 5</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width: ${(simState.storedOre / 5) * 100}%"></div>
          </div>
        </div>
        <div class="custody-box">
          <div class="custody-box-title">LINEAR CUSTODY TRACKER</div>
          <div>ACTIVE TOKENS IN FLIGHT: 3</div>
          <div>USE-AFTER-FREE ERRORS: 0</div>
          <div>DOUBLE-FREE DETECTIONS: 0</div>
          <div>OWNERSHIP LEAKS: 0</div>
        </div>
      `;
    }
  }

  function startSimulation() {
    if (isSimulating) return;
    isSimulating = true;
    btnDeploy.textContent = "Simulating (20 Hz)...";
    btnDeploy.style.background = "#3fb950";

    logTerminal("Invoking subprocess: rustc --target wasm32-unknown-unknown", "rustc");
    setTimeout(() => {
      logTerminal("rustc compilation successful. Output: firmware.wasm (18.4 KB)", "rustc");
      logTerminal("Wasmi 2.0 VM loaded bytecode. Fuel budget: 50,000 instructions/tick.", "arbiter");
    }, 250);

    simInterval = setInterval(() => {
      simTick++;

      if (activeMissionId === 1) {
        // Cold boot: accelerate RPM to 3000
        if (simState.rpm < 3000) simState.rpm += 85;
        if (simState.rpm >= 2800 && simState.power < 100) simState.power += 5;
        if (simState.power >= 100) simState.status = "ONLINE // SYNCHRONIZED";

        if (simTick % 10 === 0) {
          logTerminal(`[TICK ${simTick}] Generator RPM: ${Math.round(simState.rpm)} | Grid: ${Math.round(simState.power)}%`, "sim");
        }
      } else if (activeMissionId === 2) {
        // Subterranean extraction: advance depth, cycle temperature
        if (simState.temp > 75) {
          simState.temp -= 2.5; // Cooling
          if (simTick % 8 === 0) logTerminal(`[TICK ${simTick}] Thermal governor active. Purging coolant... Temp: ${Math.round(simState.temp)}°C`, "arbiter");
        } else {
          simState.temp += 1.8; // Drilling heat
          simState.depth += 1.4;
          if (simTick % 8 === 0) logTerminal(`[TICK ${simTick}] Drillhead boring borehole. Depth: ${Math.round(simState.depth)}m`, "sim");
        }
      } else if (activeMissionId === 3) {
        // Ownership transfer: accumulate stored ore
        if (simTick % 15 === 0) {
          if (simState.storedOre < 5) {
            simState.storedOre++;
            logTerminal(`[TICK ${simTick}] Move semantics verified: ResourceChunk<CobaltOre> moved to StorageBay.`, "ok");
          }
        }
      }

      updateGauges();

      // Check win condition
      if (MISSIONS[activeMissionId].targetCondition(simState)) {
        stopSimulation();
        logTerminal(`[OBJECTIVE COMPLETE] Mission 0${activeMissionId} objectives verified by Ferric Arbiter!`, "ok");
        btnDeploy.textContent = "Mission Completed!";
        btnDeploy.style.background = "#00ffcc";
        btnDeploy.style.color = "#000";
      }
    }, 100);
  }

  function stopSimulation() {
    if (simInterval) clearInterval(simInterval);
    isSimulating = false;
    btnDeploy.textContent = "Deploy Firmware (F5)";
    btnDeploy.style.background = "";
    btnDeploy.style.color = "";
  }

  // Event Listeners
  missionTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const id = parseInt(tab.dataset.mission);
      loadMission(id);
    });
  });

  btnDeploy.addEventListener('click', () => {
    if (!isSimulating) {
      startSimulation();
    } else {
      stopSimulation();
    }
  });

  btnReset.addEventListener('click', () => {
    loadMission(activeMissionId);
  });

  // Hotkey F5
  window.addEventListener('keydown', (e) => {
    if (e.key === 'F5') {
      e.preventDefault();
      btnDeploy.click();
    }
  });

  // Initial Load
  loadMission(1);

})();
