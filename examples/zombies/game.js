/**
 * Zombie Chase Game
 *
 * A simple game demonstrating libavoid-rust pathfinding.
 * Survive increasing rounds while zombies chase you using optimal paths!
 */

import init, {
  Router,
  Point,
  Rectangle,
  ConnRef,
  ConnEnd,
  ShapeRef
} from '../web/pkg/libavoid.js';

// Game constants
const CANVAS_WIDTH = 800;
const CANVAS_HEIGHT = 600;
const OBSTACLE_COUNT = 20;
const OBSTACLE_MIN_SIZE = 40;
const OBSTACLE_MAX_SIZE = 100;
const PLAYER_SPEED = 150; // pixels per second
const BASE_ZOMBIE_SPEED = 50;  // pixels per second
const ZOMBIE_SPEED_VARIATION = 0.05; // +/- 5% speed variation per zombie
const BASE_ROUND_DURATION = 10; // seconds for round 1
const ROUND_TIME_INCREMENT = 5; // additional seconds per round
const ZOMBIE_SPEED_MULTIPLIER = 1.10; // 10% faster each round

// Zombie wobble/stumble effect
const ZOMBIE_WOBBLE_AMOUNT = 3; // max pixels of wobble offset
const ZOMBIE_WOBBLE_SPEED = 8; // wobble frequency
const CATCH_DISTANCE = 20;
const SCARED_DISTANCE = 50; // Distance at which player looks scared
const ENTITY_SIZE = 24;
const APPLE_SIZE = 20;
const APPLE_COLLECT_DISTANCE = 25;
const STARTING_HEALTH = 5;
const MAX_HEALTH = 20;
const DAMAGE_INTERVAL_MS = 250; // Take damage every 250ms while touching zombie

// Player emoji states
const EMOJI_NORMAL = '\u{1F642}';  // 🙂
const EMOJI_SCARED = '\u{1F633}';  // 😳
const EMOJI_DEAD = '\u{1F480}';    // 💀

// Zombie emoji variants
const ZOMBIE_EMOJIS = [
  '\u{1F9DF}',           // 🧟
  '\u{1F9DF}\u200D\u2640\uFE0F',  // 🧟‍♀️
  '\u{1F9DF}\u200D\u2642\uFE0F',  // 🧟‍♂️
];

// Apple emoji
const EMOJI_APPLE = '\u{1F34E}';  // 🍎

const SPAWN_MARGIN = 50;
const MIN_SPAWN_DISTANCE = 150; // Minimum distance between player and zombie spawn

// Routing constants
const POLY_LINE_ROUTING = 1;
const ORTHOGONAL_ROUTING = 2;

// Use polyline for more direct zombie paths
const ZOMBIE_ROUTING = POLY_LINE_ROUTING;

// SVG namespace
const SVG_NS = 'http://www.w3.org/2000/svg';

// Zombie colors for visual distinction
const ZOMBIE_COLORS = [
  '#e94560', '#ff6b6b', '#ffa502', '#ff4757', '#ee5a24',
  '#c44569', '#f78fb3', '#cf6a87', '#e77f67', '#fa983a'
];

class ZombieGame {
  constructor() {
    this.router = null;
    this.obstacles = [];
    this.player = { x: 0, y: 0 };
    this.zombies = []; // Array of zombie objects
    this.apples = []; // Collectible apples on the map

    this.keysPressed = new Set();
    this.gameRunning = false;
    this.timeRemaining = BASE_ROUND_DURATION;
    this.lastFrameTime = 0;
    this.lastDamageTime = 0; // Track when player last took damage

    this.round = 1;
    this.health = STARTING_HEALTH;
    this.showPaths = true; // Debug paths on by default

    this.svg = null;
    this.playerElement = null;
  }

  async init() {
    await init();

    document.getElementById('loading').style.display = 'none';
    document.getElementById('game-container').style.display = 'block';
    document.getElementById('hud').style.display = 'flex';

    this.svg = document.getElementById('game-canvas');
    this.setupInputHandlers();
    this.startNewGame();
  }

  setupInputHandlers() {
    document.addEventListener('keydown', (e) => {
      const key = e.key.toLowerCase();
      // Support both WASD and arrow keys
      if (['w', 'a', 's', 'd'].includes(key)) {
        e.preventDefault();
        this.keysPressed.add(key);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        this.keysPressed.add('w');
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        this.keysPressed.add('s');
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        this.keysPressed.add('a');
      } else if (e.key === 'ArrowRight') {
        e.preventDefault();
        this.keysPressed.add('d');
      } else if (e.key === ' ' || e.key === 'Enter') {
        // Space or Enter to advance from round complete screen
        e.preventDefault();
        if (!this.gameRunning && this.countdownTimer) {
          // Round complete - advance to next round
          this.nextRound();
        } else if (!this.gameRunning) {
          // Game over - restart
          this.restart();
        }
      }
    });

    document.addEventListener('keyup', (e) => {
      const key = e.key.toLowerCase();
      this.keysPressed.delete(key);
      // Also handle arrow keys on release
      if (e.key === 'ArrowUp') this.keysPressed.delete('w');
      if (e.key === 'ArrowDown') this.keysPressed.delete('s');
      if (e.key === 'ArrowLeft') this.keysPressed.delete('a');
      if (e.key === 'ArrowRight') this.keysPressed.delete('d');
    });

    // Debug checkbox handler
    const checkbox = document.getElementById('show-paths');
    if (checkbox) {
      checkbox.checked = this.showPaths;
      checkbox.addEventListener('change', (e) => {
        this.showPaths = e.target.checked;
        this.updatePathVisibility();
      });
    }
  }

  updatePathVisibility() {
    this.zombies.forEach(z => {
      if (z.pathElement) {
        z.pathElement.style.display = this.showPaths ? 'block' : 'none';
      }
    });
  }

  startNewGame() {
    this.clearCountdownTimer();
    this.round = 1;
    this.health = STARTING_HEALTH;
    this.startRound();
  }

  clearCountdownTimer() {
    if (this.countdownTimer) {
      clearInterval(this.countdownTimer);
      this.countdownTimer = null;
    }
  }

  startRound() {
    this.clearSvg();
    this.createRouter();
    this.generateObstacles();
    this.spawnEntities();
    this.spawnApples();
    this.drawObstacles();
    this.createEntityElements();
    this.updateAllPaths();

    // Calculate time for this round
    this.timeRemaining = BASE_ROUND_DURATION + (this.round - 1) * ROUND_TIME_INCREMENT;
    this.gameRunning = true;
    this.lastFrameTime = performance.now();
    this.lastDamageTime = 0;

    document.getElementById('game-over').style.display = 'none';
    this.updateHUD();

    requestAnimationFrame((t) => this.gameLoop(t));
  }

  restart() {
    this.startNewGame();
  }

  nextRound() {
    this.clearCountdownTimer();
    this.round++;
    this.startRound();
  }

  createRouter() {
    this.router = new Router(ZOMBIE_ROUTING);
  }

  generateObstacles() {
    this.obstacles = [];

    for (let i = 0; i < OBSTACLE_COUNT; i++) {
      let obstacle;
      let attempts = 0;
      const maxAttempts = 100;

      do {
        const width = OBSTACLE_MIN_SIZE + Math.random() * (OBSTACLE_MAX_SIZE - OBSTACLE_MIN_SIZE);
        const height = OBSTACLE_MIN_SIZE + Math.random() * (OBSTACLE_MAX_SIZE - OBSTACLE_MIN_SIZE);
        const x = SPAWN_MARGIN + Math.random() * (CANVAS_WIDTH - width - SPAWN_MARGIN * 2);
        const y = SPAWN_MARGIN + Math.random() * (CANVAS_HEIGHT - height - SPAWN_MARGIN * 2);

        obstacle = { x, y, width, height };
        attempts++;
      } while (this.overlapsExisting(obstacle) && attempts < maxAttempts);

      if (attempts < maxAttempts) {
        this.obstacles.push(obstacle);

        // Add to router
        const centerX = obstacle.x + obstacle.width / 2;
        const centerY = obstacle.y + obstacle.height / 2;
        const rect = new Rectangle(new Point(centerX, centerY), obstacle.width, obstacle.height);
        const shape = new ShapeRef(this.router, rect.toPolygon());
        this.router.addShape(shape);
        obstacle.shapeRef = shape;
      }
    }

    // Process transaction after adding all shapes
    this.router.processTransaction();
  }

  overlapsExisting(newObs) {
    const padding = 10;
    for (const obs of this.obstacles) {
      if (!(newObs.x + newObs.width + padding < obs.x ||
            newObs.x > obs.x + obs.width + padding ||
            newObs.y + newObs.height + padding < obs.y ||
            newObs.y > obs.y + obs.height + padding)) {
        return true;
      }
    }
    return false;
  }

  spawnEntities() {
    // Spawn player
    this.player = this.findSpawnPoint();

    // Spawn zombies (one per round)
    this.zombies = [];
    const zombieCount = this.round;
    const baseSpeed = BASE_ZOMBIE_SPEED * Math.pow(ZOMBIE_SPEED_MULTIPLIER, this.round - 1);

    for (let i = 0; i < zombieCount; i++) {
      let pos;
      let attempts = 0;
      do {
        pos = this.findSpawnPoint();
        attempts++;
      } while (this.distance(this.player, pos) < MIN_SPAWN_DISTANCE && attempts < 100);

      // Each zombie gets +/- 5% speed variation
      const speedVariation = 1 + (Math.random() * 2 - 1) * ZOMBIE_SPEED_VARIATION;
      const zombieSpeed = baseSpeed * speedVariation;

      this.zombies.push({
        x: pos.x,
        y: pos.y,
        path: [],
        pathIndex: 0,
        pathProgress: 0,
        speed: zombieSpeed,
        color: ZOMBIE_COLORS[i % ZOMBIE_COLORS.length],
        emoji: ZOMBIE_EMOJIS[Math.floor(Math.random() * ZOMBIE_EMOJIS.length)],
        element: null,
        pathElement: null,
        // Wobble state - random phase so zombies don't wobble in sync
        wobblePhase: Math.random() * Math.PI * 2,
        wobbleTime: 0
      });
    }
  }

  spawnApples() {
    this.apples = [];
    // Number of apples is round +/- 1
    const baseCount = this.round;
    const variation = Math.floor(Math.random() * 3) - 1; // -1, 0, or 1
    const appleCount = Math.max(1, baseCount + variation);

    for (let i = 0; i < appleCount; i++) {
      const pos = this.findSpawnPoint();
      this.apples.push({
        x: pos.x,
        y: pos.y,
        element: null
      });
    }
  }

  findSpawnPoint() {
    let attempts = 0;
    while (attempts < 1000) {
      const x = SPAWN_MARGIN + Math.random() * (CANVAS_WIDTH - SPAWN_MARGIN * 2);
      const y = SPAWN_MARGIN + Math.random() * (CANVAS_HEIGHT - SPAWN_MARGIN * 2);

      if (!this.isInsideObstacle(x, y)) {
        return { x, y };
      }
      attempts++;
    }
    return { x: CANVAS_WIDTH / 2, y: CANVAS_HEIGHT / 2 };
  }

  isInsideObstacle(x, y, padding = ENTITY_SIZE / 2) {
    for (const obs of this.obstacles) {
      if (x >= obs.x - padding &&
          x <= obs.x + obs.width + padding &&
          y >= obs.y - padding &&
          y <= obs.y + obs.height + padding) {
        return true;
      }
    }
    return false;
  }

  distance(a, b) {
    return Math.sqrt((a.x - b.x) ** 2 + (a.y - b.y) ** 2);
  }

  updateAllPaths() {
    this.zombies.forEach(zombie => this.updateZombiePath(zombie));
  }

  updateZombiePath(zombie) {
    // Create a new connector for pathfinding
    const srcEnd = new ConnEnd(new Point(zombie.x, zombie.y));
    const dstEnd = new ConnEnd(new Point(this.player.x, this.player.y));

    const conn = ConnRef.createWithEndpoints(this.router, srcEnd, dstEnd);
    conn.setRoutingType(ZOMBIE_ROUTING);
    this.router.addConnector(conn);

    this.router.processTransaction();

    const route = this.router.getConnectorRoute(conn.id());

    zombie.path = [];
    if (route && route.size() > 0) {
      for (let i = 0; i < route.size(); i++) {
        const pt = route.at(i);
        if (pt) {
          zombie.path.push({ x: pt.x, y: pt.y });
        }
      }
    }

    // Reset zombie progress along path
    zombie.pathIndex = 0;
    zombie.pathProgress = 0;

    // Delete connector after use
    this.router.deleteConnector(conn);
    this.router.processTransaction();

    this.drawZombiePath(zombie);
  }

  gameLoop(currentTime) {
    if (!this.gameRunning) return;

    const deltaTime = (currentTime - this.lastFrameTime) / 1000;
    this.lastFrameTime = currentTime;

    // Update timer
    this.timeRemaining -= deltaTime;
    if (this.timeRemaining <= 0) {
      this.winRound();
      return;
    }

    // Update player
    this.updatePlayer(deltaTime);

    // Update all zombies
    for (const zombie of this.zombies) {
      const zombieMoved = this.updateZombie(zombie, deltaTime);

      // Update path if zombie reached end of current path
      if (zombieMoved && zombie.pathIndex >= zombie.path.length - 1) {
        this.updateZombiePath(zombie);
      }

      // Check collision - take damage if touching zombie
      if (this.distance(this.player, zombie) < CATCH_DISTANCE) {
        // Take damage every DAMAGE_INTERVAL_MS while touching
        if (currentTime - this.lastDamageTime >= DAMAGE_INTERVAL_MS) {
          this.health--;
          this.lastDamageTime = currentTime;

          if (this.health <= 0) {
            this.lose();
            return;
          }
        }
      }
    }

    // Check apple collection
    this.checkAppleCollection();

    // Update UI
    this.updateHUD();
    this.drawEntities();

    requestAnimationFrame((t) => this.gameLoop(t));
  }

  checkAppleCollection() {
    for (let i = this.apples.length - 1; i >= 0; i--) {
      const apple = this.apples[i];
      if (this.distance(this.player, apple) < APPLE_COLLECT_DISTANCE) {
        // Collect the apple
        if (this.health < MAX_HEALTH) {
          this.health++;
        }
        // Remove apple from array and SVG
        if (apple.element) {
          apple.element.remove();
        }
        this.apples.splice(i, 1);
      }
    }
  }

  updatePlayer(deltaTime) {
    let dx = 0;
    let dy = 0;

    if (this.keysPressed.has('w')) dy -= 1;
    if (this.keysPressed.has('s')) dy += 1;
    if (this.keysPressed.has('a')) dx -= 1;
    if (this.keysPressed.has('d')) dx += 1;

    // Normalize diagonal movement
    if (dx !== 0 && dy !== 0) {
      const len = Math.sqrt(dx * dx + dy * dy);
      dx /= len;
      dy /= len;
    }

    const newX = this.player.x + dx * PLAYER_SPEED * deltaTime;
    const newY = this.player.y + dy * PLAYER_SPEED * deltaTime;

    // Check bounds and obstacles
    const moved = this.tryMove(this.player, newX, newY);

    if (moved) {
      this.updateAllPaths();
    }
  }

  tryMove(entity, newX, newY) {
    // Clamp to bounds
    newX = Math.max(ENTITY_SIZE / 2, Math.min(CANVAS_WIDTH - ENTITY_SIZE / 2, newX));
    newY = Math.max(ENTITY_SIZE / 2, Math.min(CANVAS_HEIGHT - ENTITY_SIZE / 2, newY));

    // Check if new position is valid
    if (!this.isInsideObstacle(newX, newY)) {
      entity.x = newX;
      entity.y = newY;
      return true;
    }

    // Try sliding along walls
    if (!this.isInsideObstacle(newX, entity.y)) {
      entity.x = newX;
      return true;
    }
    if (!this.isInsideObstacle(entity.x, newY)) {
      entity.y = newY;
      return true;
    }

    return false;
  }

  updateZombie(zombie, deltaTime) {
    // Update wobble time for stumbling effect
    zombie.wobbleTime += deltaTime;

    if (zombie.path.length < 2) return false;

    const moveDistance = zombie.speed * deltaTime;
    let remaining = moveDistance;
    let moved = false;

    while (remaining > 0 && zombie.pathIndex < zombie.path.length - 1) {
      const current = zombie.path[zombie.pathIndex];
      const next = zombie.path[zombie.pathIndex + 1];
      const segmentLength = this.distance(current, next);

      if (segmentLength === 0) {
        zombie.pathIndex++;
        continue;
      }

      const remainingInSegment = segmentLength - zombie.pathProgress;
      moved = true;

      if (remaining >= remainingInSegment) {
        // Move to next waypoint
        remaining -= remainingInSegment;
        zombie.pathIndex++;
        zombie.pathProgress = 0;
        zombie.x = next.x;
        zombie.y = next.y;
      } else {
        // Move along current segment
        zombie.pathProgress += remaining;
        const t = zombie.pathProgress / segmentLength;
        zombie.x = current.x + (next.x - current.x) * t;
        zombie.y = current.y + (next.y - current.y) * t;
        remaining = 0;
      }
    }

    return moved;
  }

  clearSvg() {
    while (this.svg.firstChild) {
      this.svg.removeChild(this.svg.firstChild);
    }
  }

  drawObstacles() {
    for (const obs of this.obstacles) {
      const rect = document.createElementNS(SVG_NS, 'rect');
      rect.setAttribute('x', obs.x);
      rect.setAttribute('y', obs.y);
      rect.setAttribute('width', obs.width);
      rect.setAttribute('height', obs.height);
      rect.setAttribute('fill', '#2d4059');
      rect.setAttribute('stroke', '#3d5a80');
      rect.setAttribute('stroke-width', '2');
      rect.setAttribute('rx', '4');
      this.svg.appendChild(rect);
    }
  }

  createEntityElements() {
    // Create path elements for each zombie (drawn first, behind entities)
    for (const zombie of this.zombies) {
      const pathEl = document.createElementNS(SVG_NS, 'path');
      pathEl.setAttribute('fill', 'none');
      pathEl.setAttribute('stroke', zombie.color);
      pathEl.setAttribute('stroke-width', '2');
      pathEl.setAttribute('stroke-dasharray', '8,4');
      pathEl.setAttribute('opacity', '0.6');
      pathEl.style.display = this.showPaths ? 'block' : 'none';
      this.svg.appendChild(pathEl);
      zombie.pathElement = pathEl;
    }

    // Create zombie elements
    for (const zombie of this.zombies) {
      const el = document.createElementNS(SVG_NS, 'text');
      el.setAttribute('font-size', ENTITY_SIZE);
      el.setAttribute('text-anchor', 'middle');
      el.setAttribute('dominant-baseline', 'central');
      el.textContent = zombie.emoji;
      this.svg.appendChild(el);
      zombie.element = el;
    }

    // Create apple elements
    for (const apple of this.apples) {
      const el = document.createElementNS(SVG_NS, 'text');
      el.setAttribute('font-size', APPLE_SIZE);
      el.setAttribute('text-anchor', 'middle');
      el.setAttribute('dominant-baseline', 'central');
      el.setAttribute('x', apple.x);
      el.setAttribute('y', apple.y);
      el.textContent = EMOJI_APPLE;
      this.svg.appendChild(el);
      apple.element = el;
    }

    // Player (drawn last, on top)
    this.playerElement = document.createElementNS(SVG_NS, 'text');
    this.playerElement.setAttribute('font-size', ENTITY_SIZE);
    this.playerElement.setAttribute('text-anchor', 'middle');
    this.playerElement.setAttribute('dominant-baseline', 'central');
    this.playerElement.textContent = '\u{1F642}';
    this.svg.appendChild(this.playerElement);
  }

  drawZombiePath(zombie) {
    if (!zombie.pathElement) return;

    if (zombie.path.length < 2) {
      zombie.pathElement.setAttribute('d', '');
      return;
    }

    let d = `M ${zombie.path[0].x} ${zombie.path[0].y}`;
    for (let i = 1; i < zombie.path.length; i++) {
      d += ` L ${zombie.path[i].x} ${zombie.path[i].y}`;
    }
    zombie.pathElement.setAttribute('d', d);
  }

  drawEntities() {
    this.playerElement.setAttribute('x', this.player.x);
    this.playerElement.setAttribute('y', this.player.y);

    // Update player emoji based on nearest zombie distance
    let minDist = Infinity;
    for (const zombie of this.zombies) {
      const d = this.distance(this.player, zombie);
      if (d < minDist) minDist = d;
    }

    if (minDist < SCARED_DISTANCE) {
      this.playerElement.textContent = EMOJI_SCARED;
    } else {
      this.playerElement.textContent = EMOJI_NORMAL;
    }

    for (const zombie of this.zombies) {
      if (zombie.element) {
        // Calculate wobble offset for stumbling zombie effect
        const wobbleX = Math.sin(zombie.wobbleTime * ZOMBIE_WOBBLE_SPEED + zombie.wobblePhase) * ZOMBIE_WOBBLE_AMOUNT;
        const wobbleY = Math.cos(zombie.wobbleTime * ZOMBIE_WOBBLE_SPEED * 0.7 + zombie.wobblePhase) * ZOMBIE_WOBBLE_AMOUNT * 0.5;

        zombie.element.setAttribute('x', zombie.x + wobbleX);
        zombie.element.setAttribute('y', zombie.y + wobbleY);
      }
    }
  }

  updateHUD() {
    document.getElementById('time-value').textContent = this.timeRemaining.toFixed(1);
    document.getElementById('round-value').textContent = this.round;
    document.getElementById('zombies-value').textContent = this.zombies.length;

    // Find closest zombie
    let minDist = Infinity;
    for (const zombie of this.zombies) {
      const d = this.distance(this.player, zombie);
      if (d < minDist) minDist = d;
    }
    document.getElementById('distance-value').textContent = Math.round(minDist);

    // Update health bar
    const healthBar = document.getElementById('health-bar');
    if (healthBar) {
      let healthDisplay = '';
      for (let i = 0; i < MAX_HEALTH; i++) {
        if (i < this.health) {
          healthDisplay += EMOJI_APPLE;
        } else {
          healthDisplay += '\u{1F5A4}'; // 🖤 (black heart) for empty slots
        }
      }
      healthBar.textContent = healthDisplay;
    }
  }

  winRound() {
    this.gameRunning = false;
    const overlay = document.getElementById('game-over');
    overlay.className = 'win';
    document.getElementById('result-text').textContent = `ROUND ${this.round} COMPLETE!`;

    const nextZombieSpeed = Math.round(BASE_ZOMBIE_SPEED * Math.pow(ZOMBIE_SPEED_MULTIPLIER, this.round) * 10) / 10;
    const countdownEl = document.getElementById('result-message');
    const continueBtn = document.getElementById('continue-btn');

    continueBtn.style.display = 'inline-block';
    document.getElementById('restart-btn').textContent = 'Restart from Round 1';
    overlay.style.display = 'block';

    // Auto-advance countdown
    let countdown = 5;
    const updateCountdown = () => {
      countdownEl.textContent = `Next round: ${this.round + 1} zombies at ${nextZombieSpeed} px/s. Starting in ${countdown}s...`;
    };
    updateCountdown();

    this.countdownTimer = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        clearInterval(this.countdownTimer);
        this.countdownTimer = null;
        this.nextRound();
      } else {
        updateCountdown();
      }
    }, 1000);
  }

  lose() {
    this.gameRunning = false;

    // Change player emoji to dead
    this.playerElement.textContent = EMOJI_DEAD;

    const overlay = document.getElementById('game-over');
    overlay.className = 'lose';
    document.getElementById('result-text').textContent = 'GAME OVER';
    document.getElementById('result-message').textContent =
      `Caught on round ${this.round} with ${this.timeRemaining.toFixed(1)}s remaining!`;
    document.getElementById('continue-btn').style.display = 'none';
    document.getElementById('restart-btn').textContent = 'Play Again';
    overlay.style.display = 'block';
  }
}

// Initialize game
const game = new ZombieGame();
window.game = game;
game.init().catch(err => {
  document.getElementById('loading').textContent = 'Error loading game: ' + err.message;
  console.error(err);
});
