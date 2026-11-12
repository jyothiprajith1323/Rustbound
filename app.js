// Rustbound: The Systems Frontier - Web Interactive Showcase & Simulator
// HUD/Sci-Fi FUI redesign — all original simulation logic preserved and improved
(function () {
  'use strict';

  // ============================================================
  // MISSION DEFINITIONS & FIRMWARE
  // ============================================================
  const MISSIONS = {
    1: {
      id: 1,
      name: 'Cold Boot',
      machine: 'Aux Generator Unit #01',
      badge: 'Syntax & Telemetry',
      badgeClass: 'badge-orange',
      description: 'Initialize the emergency power turbine by polling hardware telemetry and submitting the ignition state transition packet.',
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
      initialTelemetry: { rpm: 0, temp: 24, stability: 12, power: 0, status: 'STANDBY' },
      targetCondition: (s) => s.rpm >= 2800 && s.power >= 95
    },
    2: {
      id: 2,
      name: 'Subterranean Extraction',
      machine: 'Quarry Drill Core #04',
      badge: 'Thermal Throttling',
      badgeClass: 'badge-warning',
      description: 'Drill geothermal ore deposits while dynamically throttling motor torque to prevent thermal runaway beyond 85°C.',
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
      initialTelemetry: { rpm: 1200, temp: 35, stability: 98, depth: 0, status: 'DRILLING' },
      targetCondition: (s) => s.depth >= 100
    },
    3: {
      id: 3,
      name: 'Single Owner Custody',
      machine: 'Extraction & Logistics Grid',
      badge: 'Linear Types & Move',
      badgeClass: 'badge-cyan',
      description: 'Transfer ResourceChunk<Ore> tokens from the Drill through the Conveyor into Storage without duplicating or dropping linear tokens.',
      code: `// Single Owner Custody: Zero-Leak Linear Pipeline
#![no_std]
use rustbound_api::prelude::*;

pub fn tick(
    drill: &mut QuarryDrill,
    conveyor: &mut ConveyorBelt,
    bay: &mut StorageBay
) {
    // 1. Extract linear custody token from drill hopper
    if let Some(ore_chunk) = drill.try_extract_ore() {
        // ore_chunk: ResourceChunk<CobaltOre>
        // Cannot be cloned or dropped!
        
        // 2. Move linear token into conveyor belt intake
        let transit_token = conveyor.intake(ore_chunk);
        
        // 3. Advance conveyor and deposit into secure storage
        if let Some(arrived_chunk) = conveyor.advance_and_pop(transit_token) {
            bay.deposit(arrived_chunk);
        }
    }
}`,
      initialTelemetry: { drillOre: 10, conveyorItems: 0, storedOre: 0, custodyLeaks: 0, status: 'PIPELINE_ACTIVE' },
      targetCondition: (s) => s.storedOre >= 5
    }
  };

  // ============================================================
  // STATE
  // ============================================================
  let activeMissionId = 1;
  let isSimulating   = false;
  let simInterval    = null;
  let simTick        = 0;
  let simState       = JSON.parse(JSON.stringify(MISSIONS[1].initialTelemetry));

  // Reduced-motion preference
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // ============================================================
  // CANVAS FACTORY VISUALIZATION
  // ============================================================
  const canvas = document.getElementById('factory-canvas');
  const ctx    = canvas ? canvas.getContext('2d') : null;
  let   animId = null;

  // Conveyor particles
  const particles = [
    { progress: 0.10, id: 'CHUNK_#001' },
    { progress: 0.45, id: 'CHUNK_#002' },
    { progress: 0.80, id: 'CHUNK_#003' }
  ];

  function getCanvasSize() {
    const rect = canvas.getBoundingClientRect();
    return { w: rect.width, h: rect.height };
  }

  function resizeCanvas() {
    if (!canvas) return;
    const dpr  = window.devicePixelRatio || 1;
    const { w, h } = getCanvasSize();
    canvas.width  = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  // ResizeObserver for responsive canvas
  if (canvas && typeof ResizeObserver !== 'undefined') {
    const ro = new ResizeObserver(() => {
      resizeCanvas();
      if (!animId && !prefersReducedMotion) {
        animId = requestAnimationFrame(drawFactory);
      }
    });
    ro.observe(canvas);
  } else if (canvas) {
    window.addEventListener('resize', resizeCanvas);
  }
  resizeCanvas();

  // ---- Draw helpers ----

  function drawGrid(w, h) {
    const gs = 40;
    ctx.strokeStyle = '#111827';
    ctx.lineWidth = 1;
    for (let x = 0; x < w; x += gs) {
      ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
    }
    for (let y = 0; y < h; y += gs) {
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
    }
    // Crosshair markers at machine positions
    const positions = [[w * 0.10, h * 0.60], [w * 0.28, h * 0.32], [w * 0.84, h * 0.32]];
    ctx.strokeStyle = 'rgba(88,166,255,0.18)';
    ctx.lineWidth = 1;
    positions.forEach(([cx, cy]) => {
      ctx.beginPath(); ctx.moveTo(cx - 12, cy); ctx.lineTo(cx + 12, cy); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(cx, cy - 12); ctx.lineTo(cx, cy + 12); ctx.stroke();
      ctx.beginPath(); ctx.arc(cx, cy, 6, 0, Math.PI * 2); ctx.stroke();
    });
  }

  function drawPowerConduit(w, h) {
    ctx.strokeStyle = 'rgba(0,229,176,0.20)';
    ctx.lineWidth = 2;
    ctx.setLineDash([6, 4]);
    ctx.beginPath();
    ctx.moveTo(w * 0.10 + 46, h * 0.60);
    ctx.lineTo(w * 0.10 + 46, h * 0.46);
    ctx.lineTo(w * 0.84 - 46, h * 0.46);
    ctx.lineTo(w * 0.84 - 46, h * 0.60);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  function drawMachineBase(x, y, w, h, label, color) {
    // Shadow
    ctx.fillStyle = 'rgba(0,0,0,0.4)';
    ctx.fillRect(x + 3, y + 3, w, h);
    // Body
    ctx.fillStyle = '#0d1117';
    ctx.strokeStyle = color || '#1e2d40';
    ctx.lineWidth = 1.5;
    ctx.fillRect(x, y, w, h);
    ctx.strokeRect(x, y, w, h);
    // Corner accent marks
    const mark = 6;
    ctx.strokeStyle = color || '#2a3f5a';
    ctx.lineWidth = 1;
    [[x, y], [x + w, y], [x, y + h], [x + w, y + h]].forEach(([cx, cy]) => {
      const sx = cx === x ? 1 : -1;
      const sy = cy === y ? 1 : -1;
      ctx.beginPath(); ctx.moveTo(cx + sx * mark, cy); ctx.lineTo(cx, cy); ctx.lineTo(cx, cy + sy * mark); ctx.stroke();
    });
    // Label plate
    if (label) {
      ctx.fillStyle = '#0d1117';
      ctx.strokeStyle = color || '#1e2d40';
      ctx.lineWidth = 1;
      const lw = label.length * 5.5 + 10;
      ctx.fillRect(x + (w - lw) / 2, y + h + 4, lw, 14);
      ctx.strokeRect(x + (w - lw) / 2, y + h + 4, lw, 14);
      ctx.fillStyle = '#484f58';
      ctx.font = '8.5px "Fira Code", monospace';
      ctx.textAlign = 'center';
      ctx.fillText(label, x + w / 2, y + h + 14);
      ctx.textAlign = 'left';
    }
  }

  function drawGenerator(x, y, w, h, time) {
    const color = simState.rpm > 2000 ? '#00e5b0' : '#e36a47';
    drawMachineBase(x, y, w, h, 'GENERATOR', color);

    const cx = x + w / 2, cy = y + h / 2;
    const angle = time * 0.003 * (simState.rpm ? simState.rpm / 1000 : 0.4);

    // Glow halo
    if (simState.rpm > 500) {
      const alpha = Math.min(0.25, simState.rpm / 12000);
      ctx.fillStyle = `rgba(0,229,176,${alpha})`;
      ctx.beginPath(); ctx.arc(cx, cy, 28, 0, Math.PI * 2); ctx.fill();
    }

    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(angle);

    // Outer ring
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.shadowColor = color;
    ctx.shadowBlur = simState.rpm > 1000 ? 10 : 0;
    ctx.beginPath(); ctx.arc(0, 0, 22, 0, Math.PI * 2); ctx.stroke();

    // 4 blades
    for (let i = 0; i < 4; i++) {
      ctx.rotate(Math.PI / 2);
      ctx.beginPath(); ctx.moveTo(0, 4); ctx.lineTo(0, 18); ctx.stroke();
    }
    ctx.shadowBlur = 0;

    // Center hub
    ctx.fillStyle = color;
    ctx.beginPath(); ctx.arc(0, 0, 4, 0, Math.PI * 2); ctx.fill();
    ctx.restore();

    // RPM readout
    ctx.fillStyle = '#484f58';
    ctx.font = '7px "Fira Code", monospace';
    ctx.textAlign = 'center';
    ctx.fillText(`${Math.round(simState.rpm)} RPM`, cx, y + h - 6);
    ctx.textAlign = 'left';
  }

  function drawDrill(x, y, w, h, time) {
    const isHot = simState.temp > 70;
    const color = isHot ? '#f85149' : '#e36a47';
    drawMachineBase(x, y, w, h, 'DRILL CORE', color);

    const cx = x + w / 2;
    const offset = Math.sin(time * 0.025) * (isSimulating ? 7 : 2);

    // Drill head shaft
    ctx.fillStyle = isHot ? 'rgba(248,81,73,0.9)' : 'rgba(227,106,71,0.9)';
    ctx.fillRect(cx - 7, y + h * 0.4 + offset, 14, h * 0.5);

    // Drill tip
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.moveTo(cx - 7, y + h * 0.4 + h * 0.5 + offset);
    ctx.lineTo(cx + 7, y + h * 0.4 + h * 0.5 + offset);
    ctx.lineTo(cx, y + h * 0.4 + h * 0.5 + 10 + offset);
    ctx.closePath(); ctx.fill();

    // Heat aura
    if (simState.temp > 60) {
      const alpha = Math.min(0.3, (simState.temp - 60) / 100);
      ctx.fillStyle = `rgba(248,81,73,${alpha})`;
      ctx.beginPath();
      ctx.arc(cx, y + h * 0.75 + offset, 22, 0, Math.PI * 2);
      ctx.fill();
    }

    // Temp readout
    ctx.fillStyle = isHot ? '#f85149' : '#484f58';
    ctx.font = '7px "Fira Code", monospace';
    ctx.textAlign = 'center';
    ctx.fillText(`${Math.round(simState.temp)}°C`, cx, y + h - 5);
    ctx.textAlign = 'left';
  }

  function drawConveyor(x, y, len, h, time) {
    // Track body
    ctx.fillStyle = '#080b12';
    ctx.strokeStyle = '#1e2d40';
    ctx.lineWidth = 1.5;
    ctx.fillRect(x, y, len, h);
    ctx.strokeRect(x, y, len, h);

    // Belt stripes (animated)
    const stripePeriod = 20;
    const stripeOffset = (time * 0.08) % stripePeriod;
    ctx.fillStyle = 'rgba(42,63,90,0.4)';
    for (let i = -1; i < Math.ceil(len / stripePeriod) + 1; i++) {
      const sx = x + i * stripePeriod + stripeOffset;
      ctx.fillRect(sx, y + 1, 8, h - 2);
    }

    // Rollers
    const rollerCount = Math.floor(len / 36);
    const spacing = len / rollerCount;
    for (let i = 0; i <= rollerCount; i++) {
      const rx = x + i * spacing;
      ctx.fillStyle = '#1e2d40';
      ctx.beginPath();
      ctx.arc(rx, y + h / 2, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#2a3f5a';
      ctx.lineWidth = 1;
      ctx.stroke();
    }

    // Label
    ctx.fillStyle = '#484f58';
    ctx.font = '7.5px "Fira Code", monospace';
    ctx.textAlign = 'center';
    ctx.fillText('CONVEYOR LOGISTICS — MOVE SEMANTICS', x + len / 2, y + h + 14);
    ctx.textAlign = 'left';
  }

  function drawStorageBay(x, y, w, h, time) {
    const color = '#58a6ff';
    drawMachineBase(x, y, w, h, 'STORAGE BAY', color);

    // Vault door
    ctx.strokeStyle = simState.storedOre >= 5 ? '#00e5b0' : '#2a3f5a';
    ctx.lineWidth = 1.5;
    if (simState.storedOre >= 5) {
      ctx.shadowColor = '#00e5b0'; ctx.shadowBlur = 8;
    }
    ctx.strokeRect(x + 10, y + 20, w - 20, h - 36);
    ctx.shadowBlur = 0;

    // Stored ore cubes
    const stored = Math.min(simState.storedOre || 0, 9);
    for (let i = 0; i < stored; i++) {
      const col = i % 3;
      const row = Math.floor(i / 3);
      const cx = x + 14 + col * 14;
      const cy = y + h - 16 - row * 15;
      ctx.fillStyle = '#e36a47';
      ctx.shadowColor = '#e36a47'; ctx.shadowBlur = 4;
      ctx.fillRect(cx, cy, 11, 11);
      ctx.shadowBlur = 0;
    }

    // Count readout
    ctx.fillStyle = '#58a6ff';
    ctx.font = '7px "Fira Code", monospace';
    ctx.textAlign = 'center';
    ctx.fillText(`${simState.storedOre || 0} ORE`, x + w / 2, y + 16);
    ctx.textAlign = 'left';
  }

  function drawFactory(time) {
    if (!canvas || !ctx) return;
    const { w, h } = getCanvasSize();

    // Background
    ctx.fillStyle = '#080b12';
    ctx.fillRect(0, 0, w, h);

    drawGrid(w, h);
    drawPowerConduit(w, h);

    // Machine positions (relative to canvas size)
    const gx = w * 0.06,  gy = h * 0.35,  gw = w * 0.09, gh = h * 0.45;
    const dx = w * 0.22,  dy = h * 0.08,  dw = w * 0.10, dh = h * 0.65;
    const sx = w * 0.76,  sy = h * 0.08,  sw = w * 0.18, sh = h * 0.65;
    const cvx = dx + dw + 4, cvy = dy + dh * 0.3;
    const cvLen = sx - cvx - 4, cvH = 22;

    drawGenerator(gx, gy, gw, gh, time);
    drawDrill(dx, dy, dw, dh, time);
    drawConveyor(cvx, cvy, cvLen, cvH, time);
    drawStorageBay(sx, sy, sw, sh, time);

    // Ore packets along conveyor
    if (isSimulating || activeMissionId === 3) {
      particles.forEach(p => {
        p.progress += 0.006;
        if (p.progress > 1.0) p.progress = 0;
        const px = cvx + p.progress * cvLen;
        const py = cvy + cvH / 2 - 5;

        ctx.fillStyle = '#e36a47';
        ctx.shadowColor = '#e36a47'; ctx.shadowBlur = 10;
        ctx.fillRect(px - 5, py, 10, 10);
        ctx.shadowBlur = 0;

        ctx.fillStyle = '#00e5b0';
        ctx.font = '6.5px "Fira Code", monospace';
        ctx.fillText('TOKEN', px - 10, py - 4);
      });
    }

    if (!prefersReducedMotion) {
      animId = requestAnimationFrame(drawFactory);
    }
  }

  // Start canvas animation
  if (canvas && !prefersReducedMotion) {
    animId = requestAnimationFrame(drawFactory);
  } else if (canvas && prefersReducedMotion) {
    // Draw a static frame
    requestAnimationFrame((t) => { drawFactory(t); cancelAnimationFrame(animId); animId = null; });
  }

  // ============================================================
  // DOM REFS
  // ============================================================
  const missionTabs     = document.querySelectorAll('.mission-tab');
  const codeEditor      = document.getElementById('code-editor');
  const missionTitle    = document.getElementById('mission-title');
  const missionDesc     = document.getElementById('mission-desc');
  const targetMachine   = document.getElementById('target-machine');
  const btnDeploy       = document.getElementById('btn-deploy');
  const btnReset        = document.getElementById('btn-reset');
  const terminalBody    = document.getElementById('terminal-body');
  const progressFill    = document.getElementById('progress-fill');
  const mainHeader      = document.getElementById('main-header');
  const hamburgerBtn    = document.getElementById('hamburger-btn');
  const mobileNav       = document.getElementById('mobile-nav');

  // ============================================================
  // MOBILE NAV TOGGLE
  // ============================================================
  if (hamburgerBtn && mainHeader && mobileNav) {
    hamburgerBtn.addEventListener('click', () => {
      const isOpen = mainHeader.classList.toggle('nav-open');
      hamburgerBtn.setAttribute('aria-expanded', isOpen ? 'true' : 'false');
      mobileNav.setAttribute('aria-hidden', isOpen ? 'false' : 'true');

      const openIcon  = hamburgerBtn.querySelector('.hamburger-open');
      const closeIcon = hamburgerBtn.querySelector('.hamburger-close');
      if (openIcon)  openIcon.hidden  = isOpen;
      if (closeIcon) closeIcon.hidden = !isOpen;
    });

    // Close on nav link click
    mobileNav.querySelectorAll('.mobile-nav-link').forEach(link => {
      link.addEventListener('click', () => {
        mainHeader.classList.remove('nav-open');
        hamburgerBtn.setAttribute('aria-expanded', 'false');
        mobileNav.setAttribute('aria-hidden', 'true');
        const openIcon  = hamburgerBtn.querySelector('.hamburger-open');
        const closeIcon = hamburgerBtn.querySelector('.hamburger-close');
        if (openIcon)  openIcon.hidden  = false;
        if (closeIcon) closeIcon.hidden = true;
      });
    });

    // Close on outside click
    document.addEventListener('click', (e) => {
      if (mainHeader.classList.contains('nav-open') &&
          !mainHeader.contains(e.target)) {
        mainHeader.classList.remove('nav-open');
        hamburgerBtn.setAttribute('aria-expanded', 'false');
        mobileNav.setAttribute('aria-hidden', 'true');
      }
    });
  }

  // ============================================================
  // SMOOTH SCROLL ON NAV LINKS
  // ============================================================
  document.querySelectorAll('a[href^="#"]').forEach(a => {
    a.addEventListener('click', (e) => {
      const id = a.getAttribute('href').slice(1);
      const target = document.getElementById(id);
      if (target) {
        e.preventDefault();
        target.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    });
  });

  // ============================================================
  // SCROLL-DRIVEN ACTIVE NAV LINKS (IntersectionObserver)
  // ============================================================
  const desktopNavLinks = document.querySelectorAll('.desktop-nav .nav-link');

  const sectionObserver = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const id = entry.target.id;
        desktopNavLinks.forEach(link => {
          const href = link.getAttribute('href');
          link.classList.toggle('active', href === `#${id}`);
        });
      }
    });
  }, { threshold: 0.25 });

  ['simulator', 'lore', 'architecture', 'blueprint', 'quickstart'].forEach(id => {
    const el = document.getElementById(id);
    if (el) sectionObserver.observe(el);
  });

  // ============================================================
  // SYNTAX HIGHLIGHTER (Rust)
  // ============================================================
  function highlightRust(code) {
    let s = code
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Comments first (before anything else replaces slashes)
    s = s.replace(/(\/\/.*$)/gm, '<span class="syn-comment">$1</span>');
    // Attributes
    s = s.replace(/(#!\[.*?\])/g, '<span class="syn-macro">$1</span>');
    // Keywords
    s = s.replace(/\b(pub|fn|let|mut|if|else|use|struct|enum|match|return|impl|for|while|loop|type|const|static|unsafe|async|await|dyn|trait|where|mod|self|super|crate|in|ref|move|extern|Some|None)\b/g, '<span class="syn-kw">$1</span>');
    // Types
    s = s.replace(/\b(Generator|QuarryDrill|ConveyorBelt|StorageBay|StatusCode|TargetRpm|Torque|DepthRate|ResourceChunk|CobaltOre|Option|Result|String|Vec|HashMap|bool|u8|u16|u32|u64|i8|i16|i32|i64|f32|f64|usize|isize)\b/g, '<span class="syn-type">$1</span>');
    // Numbers
    s = s.replace(/\b(\d+(\.\d+)?)\b/g, '<span class="syn-num">$1</span>');
    // Strings
    s = s.replace(/(&quot;.*?&quot;)/g, '<span class="syn-str">$1</span>');

    return s;
  }

  // ============================================================
  // TERMINAL LOGGER
  // ============================================================
  function logTerminal(msg, type = 'sim') {
    if (!terminalBody) return;
    const line = document.createElement('div');
    line.className = `log-line log-${type}`;
    const ts = new Date().toISOString().substring(11, 19);
    line.textContent = `[${ts}] ${msg}`;
    terminalBody.appendChild(line);
    terminalBody.scrollTop = terminalBody.scrollHeight;
  }

  // ============================================================
  // PROGRESS BAR UPDATE
  // ============================================================
  function updateProgress() {
    if (!progressFill) return;
    const mission = MISSIONS[activeMissionId];
    let pct = 0;
    if (activeMissionId === 1) {
      pct = Math.min(100, ((simState.rpm / 3000) * 60) + ((simState.power / 100) * 40));
    } else if (activeMissionId === 2) {
      pct = Math.min(100, (simState.depth / 100) * 100);
    } else if (activeMissionId === 3) {
      pct = Math.min(100, (simState.storedOre / 5) * 100);
    }
    progressFill.style.width = pct.toFixed(1) + '%';
  }

  // ============================================================
  // GAUGES
  // ============================================================
  function updateGauges() {
    const container = document.getElementById('gauges-container');
    if (!container) return;

    if (activeMissionId === 1) {
      const rpmPct  = Math.min(100, (simState.rpm / 3000) * 100);
      const rpmWarn = simState.rpm > 2900;
      container.innerHTML = `
        <div class="gauge-card">
          <div class="gauge-header">
            <span>TURBINE VELOCITY</span>
            <span class="gauge-value ${rpmWarn ? 'warning' : ''}">${Math.round(simState.rpm)} RPM</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill ${rpmWarn ? 'warning' : ''}" style="width:${rpmPct}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>GRID SYNCHRONIZATION</span>
            <span class="gauge-value">${Math.round(simState.power)}%</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width:${simState.power}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>CORE STABILITY</span>
            <span class="gauge-value">${simState.stability}%</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width:${simState.stability}%"></div>
          </div>
        </div>
        <div class="custody-box">
          <div class="custody-box-title">FERRIC ARBITER SAFETY STATE</div>
          <div>STATUS: ${simState.status}</div>
          <div>ISOLATION: NONE (SANDBOX SECURE)</div>
        </div>`;
    } else if (activeMissionId === 2) {
      const isHot   = simState.temp > 70;
      const tempPct = Math.min(100, (simState.temp / 100) * 100);
      const depthPct = Math.min(100, (simState.depth / 100) * 100);
      container.innerHTML = `
        <div class="gauge-card">
          <div class="gauge-header">
            <span>CORE TEMPERATURE</span>
            <span class="gauge-value ${isHot ? 'danger' : ''}">${Math.round(simState.temp)}°C</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill ${isHot ? 'danger' : ''}" style="width:${tempPct}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>BOREHOLE DEPTH</span>
            <span class="gauge-value">${Math.round(simState.depth)} m</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width:${depthPct}%"></div>
          </div>
        </div>
        <div class="custody-box">
          <div class="custody-box-title">THERMAL GOVERNOR STATUS</div>
          <div>MOTOR STATE: ${simState.temp > 75 ? 'THROTTLED (COOLING)' : 'FULL POWER'}</div>
          <div>RUNAWAY RISK: 0.00% (FERRIC VALIDATED)</div>
        </div>`;
    } else if (activeMissionId === 3) {
      const orePct = (simState.storedOre / 5) * 100;
      container.innerHTML = `
        <div class="gauge-card">
          <div class="gauge-header">
            <span>STORED ORE CHUNKS</span>
            <span class="gauge-value">${simState.storedOre} / 5</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width:${orePct}%"></div>
          </div>
        </div>
        <div class="gauge-card">
          <div class="gauge-header">
            <span>DRILL HOPPER</span>
            <span class="gauge-value">${simState.drillOre} chunks</span>
          </div>
          <div class="gauge-bar-bg">
            <div class="gauge-bar-fill" style="width:${(simState.drillOre / 10) * 100}%"></div>
          </div>
        </div>
        <div class="custody-box">
          <div class="custody-box-title">LINEAR CUSTODY TRACKER</div>
          <div>ACTIVE TOKENS IN FLIGHT: ${simState.conveyorItems}</div>
          <div>USE-AFTER-FREE ERRORS: 0</div>
          <div>DOUBLE-FREE DETECTIONS: 0</div>
          <div>OWNERSHIP LEAKS: ${simState.custodyLeaks}</div>
        </div>`;
    }

    updateProgress();
  }

  // ============================================================
  // SIMULATION LOGIC
  // ============================================================
  function startSimulation() {
    if (isSimulating) return;
    isSimulating = true;

    if (btnDeploy) {
      btnDeploy.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
        SIMULATING (20 HZ)...`;
      btnDeploy.style.background = 'var(--success)';
    }

    logTerminal('Invoking subprocess: rustc --target wasm32-unknown-unknown', 'rustc');
    setTimeout(() => {
      logTerminal('rustc compilation successful. Output: firmware.wasm (18.4 KB)', 'rustc');
      logTerminal('Wasmi 2.0 VM loaded bytecode. Fuel budget: 50,000 instructions/tick.', 'arbiter');
    }, 250);

    simInterval = setInterval(() => {
      simTick++;

      if (activeMissionId === 1) {
        if (simState.rpm < 3000) simState.rpm = Math.min(3000, simState.rpm + 85);
        if (simState.rpm >= 2800 && simState.power < 100) simState.power = Math.min(100, simState.power + 5);
        if (simState.stability < 98) simState.stability = Math.min(98, simState.stability + 3);
        if (simState.power >= 100) simState.status = 'ONLINE // SYNCHRONIZED';
        if (simTick % 10 === 0) {
          logTerminal(`[TICK ${simTick}] Generator RPM: ${Math.round(simState.rpm)} | Grid: ${Math.round(simState.power)}%`, 'sim');
        }
      } else if (activeMissionId === 2) {
        if (simState.temp > 75) {
          simState.temp = Math.max(35, simState.temp - 2.5);
          if (simTick % 8 === 0) logTerminal(`[TICK ${simTick}] Thermal governor active. Purging coolant... Temp: ${Math.round(simState.temp)}°C`, 'arbiter');
        } else {
          simState.temp = Math.min(85, simState.temp + 1.8);
          simState.depth = Math.min(100, simState.depth + 1.4);
          if (simTick % 8 === 0) logTerminal(`[TICK ${simTick}] Drillhead boring borehole. Depth: ${Math.round(simState.depth)}m`, 'sim');
        }
      } else if (activeMissionId === 3) {
        if (simTick % 15 === 0) {
          if (simState.storedOre < 5 && simState.drillOre > 0) {
            simState.drillOre--;
            simState.conveyorItems = Math.min(3, simState.conveyorItems + 1);
            setTimeout(() => {
              simState.conveyorItems = Math.max(0, simState.conveyorItems - 1);
              simState.storedOre++;
              logTerminal(`[TICK ${simTick}] Move semantics verified: ResourceChunk<CobaltOre> moved to StorageBay.`, 'ok');
            }, 400);
          }
        }
      }

      updateGauges();

      if (MISSIONS[activeMissionId].targetCondition(simState)) {
        stopSimulation(true);
      }
    }, 100);
  }

  function stopSimulation(completed) {
    if (simInterval) clearInterval(simInterval);
    isSimulating = false;

    if (completed) {
      logTerminal(`[OBJECTIVE COMPLETE] Mission 0${activeMissionId} objectives verified by Ferric Arbiter!`, 'ok');
      if (btnDeploy) {
        btnDeploy.innerHTML = `
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20,6 9,17 4,12"/></svg>
          MISSION COMPLETE`;
        btnDeploy.style.background  = 'var(--accent-cyan)';
        btnDeploy.style.color       = 'var(--bg-deepest)';
        btnDeploy.style.borderColor = 'var(--accent-cyan)';
      }
      if (progressFill) progressFill.style.width = '100%';
    } else {
      if (btnDeploy) {
        btnDeploy.innerHTML = `
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="5,3 19,12 5,21"/></svg>
          Deploy Firmware (F5)`;
        btnDeploy.style.background  = '';
        btnDeploy.style.color       = '';
        btnDeploy.style.borderColor = '';
      }
    }
  }

  // ============================================================
  // MISSION LOADER
  // ============================================================
  function loadMission(id) {
    activeMissionId = id;
    const mission = MISSIONS[id];

    stopSimulation(false);
    simState = JSON.parse(JSON.stringify(mission.initialTelemetry));
    simTick  = 0;

    // Update tab UI
    missionTabs.forEach(t => {
      const active = parseInt(t.dataset.mission) === id;
      t.classList.toggle('active', active);
      t.setAttribute('aria-selected', active ? 'true' : 'false');
    });

    // Update panel text
    if (missionTitle)  missionTitle.textContent  = `Mission 0${id}: ${mission.name}`;
    if (missionDesc)   missionDesc.textContent   = mission.description;
    if (targetMachine) targetMachine.textContent = mission.machine;

    // Load + highlight code
    if (codeEditor) codeEditor.innerHTML = highlightRust(mission.code);

    updateGauges();
    updateProgress();
    logTerminal(`Mission 0${id} [${mission.name}] loaded. Awaiting firmware deployment.`, 'arbiter');
  }

  // ============================================================
  // COPY BUTTONS (Quickstart terminal)
  // ============================================================
  document.querySelectorAll('.copy-btn').forEach(btn => {
    btn.addEventListener('click', async () => {
      const text = btn.dataset.clipboard;
      if (!text) return;

      try {
        await navigator.clipboard.writeText(text);
      } catch {
        // Fallback
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.opacity = '0';
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
      }

      const copyIcon  = btn.querySelector('.copy-icon');
      const checkIcon = btn.querySelector('.check-icon');
      if (copyIcon)  copyIcon.hidden  = true;
      if (checkIcon) checkIcon.hidden = false;
      btn.style.color = 'var(--success)';

      setTimeout(() => {
        if (copyIcon)  copyIcon.hidden  = false;
        if (checkIcon) checkIcon.hidden = true;
        btn.style.color = '';
      }, 1800);
    });
  });

  // ============================================================
  // EVENT LISTENERS
  // ============================================================
  missionTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      loadMission(parseInt(tab.dataset.mission));
    });
  });

  if (btnDeploy) {
    btnDeploy.addEventListener('click', () => {
      if (!isSimulating) {
        startSimulation();
      } else {
        stopSimulation(false);
      }
    });
  }

  if (btnReset) {
    btnReset.addEventListener('click', () => {
      loadMission(activeMissionId);
    });
  }

  // F5 hotkey
  window.addEventListener('keydown', e => {
    if (e.key === 'F5') {
      e.preventDefault();
      if (btnDeploy) btnDeploy.click();
    }
  });

  // ============================================================
  // INITIAL LOAD
  // ============================================================
  loadMission(1);

})();
