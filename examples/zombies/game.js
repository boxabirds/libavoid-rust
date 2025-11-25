/**
 * Zombie Chase Game
 *
 * A simple game demonstrating libavoid-rust pathfinding.
 * Survive 30 seconds while a zombie chases you using optimal paths!
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
const ZOMBIE_SPEED = 30;  // pixels per second
const GAME_DURATION_SECONDS = 30;
const CATCH_DISTANCE = 20;
const ENTITY_SIZE = 24;
const SPAWN_MARGIN = 50;
const MIN_SPAWN_DISTANCE = 200; // Minimum distance between player and zombie spawn

// Routing constants
const POLY_LINE_ROUTING = 1;
const ORTHOGONAL_ROUTING = 2;

// Use polyline for more direct zombie paths
const ZOMBIE_ROUTING = POLY_LINE_ROUTING;

// SVG namespace
const SVG_NS = 'http://www.w3.org/2000/svg';

class ZombieGame {
  constructor() {
    this.router = null;
    this.obstacles = [];
    this.player = { x: 0, y: 0 };
    this.zombie = { x: 0, y: 0 };
    this.zombiePath = [];
    this.zombiePathIndex = 0;
    this.zombiePathProgress = 0;

    this.keysPressed = new Set();
    this.gameRunning = false;
    this.timeRemaining = GAME_DURATION_SECONDS;
    this.lastFrameTime = 0;

    this.svg = null;
    this.pathElement = null;
    this.playerElement = null;
    this.zombieElement = null;
  }

  async init() {
    await init();

    document.getElementById('loading').style.display = 'none';
    document.getElementById('game-container').style.display = 'block';
    document.getElementById('hud').style.display = 'flex';

    this.svg = document.getElementById('game-canvas');
    this.setupInputHandlers();
    this.start();
  }

  setupInputHandlers() {
    document.addEventListener('keydown', (e) => {
      const key = e.key.toLowerCase();
      if (['w', 'a', 's', 'd'].includes(key)) {
        e.preventDefault();
        this.keysPressed.add(key);
      }
    });

    document.addEventListener('keyup', (e) => {
      const key = e.key.toLowerCase();
      this.keysPressed.delete(key);
    });
  }

  start() {
    this.clearSvg();
    this.createRouter();
    this.generateObstacles();
    this.spawnEntities();
    this.drawObstacles();
    this.createEntityElements();
    this.updatePath();

    this.timeRemaining = GAME_DURATION_SECONDS;
    this.gameRunning = true;
    this.lastFrameTime = performance.now();

    document.getElementById('game-over').style.display = 'none';

    requestAnimationFrame((t) => this.gameLoop(t));
  }

  restart() {
    this.start();
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

    // Spawn zombie far from player
    let zombie;
    let attempts = 0;
    do {
      zombie = this.findSpawnPoint();
      attempts++;
    } while (this.distance(this.player, zombie) < MIN_SPAWN_DISTANCE && attempts < 100);

    this.zombie = zombie;
    this.zombiePath = [];
    this.zombiePathIndex = 0;
    this.zombiePathProgress = 0;
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

  updatePath() {
    // Create a new connector for pathfinding
    const srcEnd = new ConnEnd(new Point(this.zombie.x, this.zombie.y));
    const dstEnd = new ConnEnd(new Point(this.player.x, this.player.y));

    const conn = ConnRef.createWithEndpoints(this.router, srcEnd, dstEnd);
    conn.setRoutingType(ZOMBIE_ROUTING);
    this.router.addConnector(conn);

    this.router.processTransaction();

    const route = this.router.getConnectorRoute(conn.id());

    this.zombiePath = [];
    if (route && route.size() > 0) {
      for (let i = 0; i < route.size(); i++) {
        const pt = route.at(i);
        if (pt) {
          this.zombiePath.push({ x: pt.x, y: pt.y });
        }
      }
    }

    // Reset zombie progress along path
    this.zombiePathIndex = 0;
    this.zombiePathProgress = 0;

    // Delete connector after use
    this.router.deleteConnector(conn);
    this.router.processTransaction();

    this.drawPath();
  }

  gameLoop(currentTime) {
    if (!this.gameRunning) return;

    const deltaTime = (currentTime - this.lastFrameTime) / 1000;
    this.lastFrameTime = currentTime;

    // Update timer
    this.timeRemaining -= deltaTime;
    if (this.timeRemaining <= 0) {
      this.win();
      return;
    }

    // Update player
    this.updatePlayer(deltaTime);

    // Update zombie
    const zombieMoved = this.updateZombie(deltaTime);

    // Update path if zombie reached end of current path or periodically
    if (zombieMoved && this.zombiePathIndex >= this.zombiePath.length - 1) {
      this.updatePath();
    }

    // Check collision
    if (this.distance(this.player, this.zombie) < CATCH_DISTANCE) {
      this.lose();
      return;
    }

    // Update UI
    this.updateHUD();
    this.drawEntities();

    requestAnimationFrame((t) => this.gameLoop(t));
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
      this.updatePath();
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

  updateZombie(deltaTime) {
    if (this.zombiePath.length < 2) return false;

    const moveDistance = ZOMBIE_SPEED * deltaTime;
    let remaining = moveDistance;
    let moved = false;

    while (remaining > 0 && this.zombiePathIndex < this.zombiePath.length - 1) {
      const current = this.zombiePath[this.zombiePathIndex];
      const next = this.zombiePath[this.zombiePathIndex + 1];
      const segmentLength = this.distance(current, next);

      if (segmentLength === 0) {
        this.zombiePathIndex++;
        continue;
      }

      const remainingInSegment = segmentLength - this.zombiePathProgress;
      moved = true;

      if (remaining >= remainingInSegment) {
        // Move to next waypoint
        remaining -= remainingInSegment;
        this.zombiePathIndex++;
        this.zombiePathProgress = 0;
        this.zombie.x = next.x;
        this.zombie.y = next.y;
      } else {
        // Move along current segment
        this.zombiePathProgress += remaining;
        const t = this.zombiePathProgress / segmentLength;
        this.zombie.x = current.x + (next.x - current.x) * t;
        this.zombie.y = current.y + (next.y - current.y) * t;
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
    // Path element (drawn first, behind entities)
    this.pathElement = document.createElementNS(SVG_NS, 'path');
    this.pathElement.setAttribute('fill', 'none');
    this.pathElement.setAttribute('stroke', '#e94560');
    this.pathElement.setAttribute('stroke-width', '2');
    this.pathElement.setAttribute('stroke-dasharray', '8,4');
    this.pathElement.setAttribute('opacity', '0.6');
    this.svg.appendChild(this.pathElement);

    // Zombie
    this.zombieElement = document.createElementNS(SVG_NS, 'text');
    this.zombieElement.setAttribute('font-size', ENTITY_SIZE);
    this.zombieElement.setAttribute('text-anchor', 'middle');
    this.zombieElement.setAttribute('dominant-baseline', 'central');
    this.zombieElement.textContent = '\u{1F9DF}';
    this.svg.appendChild(this.zombieElement);

    // Player
    this.playerElement = document.createElementNS(SVG_NS, 'text');
    this.playerElement.setAttribute('font-size', ENTITY_SIZE);
    this.playerElement.setAttribute('text-anchor', 'middle');
    this.playerElement.setAttribute('dominant-baseline', 'central');
    this.playerElement.textContent = '\u{1F642}';
    this.svg.appendChild(this.playerElement);
  }

  drawPath() {
    if (this.zombiePath.length < 2) {
      this.pathElement.setAttribute('d', '');
      return;
    }

    let d = `M ${this.zombiePath[0].x} ${this.zombiePath[0].y}`;
    for (let i = 1; i < this.zombiePath.length; i++) {
      d += ` L ${this.zombiePath[i].x} ${this.zombiePath[i].y}`;
    }
    this.pathElement.setAttribute('d', d);
  }

  drawEntities() {
    this.playerElement.setAttribute('x', this.player.x);
    this.playerElement.setAttribute('y', this.player.y);

    this.zombieElement.setAttribute('x', this.zombie.x);
    this.zombieElement.setAttribute('y', this.zombie.y);
  }

  updateHUD() {
    document.getElementById('time-value').textContent = this.timeRemaining.toFixed(1);
    document.getElementById('distance-value').textContent =
      Math.round(this.distance(this.player, this.zombie));
  }

  win() {
    this.gameRunning = false;
    const overlay = document.getElementById('game-over');
    overlay.className = 'win';
    document.getElementById('result-text').textContent = 'YOU SURVIVED!';
    document.getElementById('result-message').textContent = 'The zombie couldn\'t catch you in time!';
    overlay.style.display = 'block';
  }

  lose() {
    this.gameRunning = false;
    const overlay = document.getElementById('game-over');
    overlay.className = 'lose';
    document.getElementById('result-text').textContent = 'GAME OVER';
    document.getElementById('result-message').textContent =
      `The zombie got you with ${this.timeRemaining.toFixed(1)}s remaining!`;
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
