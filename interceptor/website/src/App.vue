<script setup>
import { ref } from 'vue'

const features = [
  {
    title: 'Traffic Interception',
    desc: 'Real-time capture and analysis of HTTP/HTTPS traffic with advanced filtering.',
    icon: '🔍'
  },
  {
    title: 'Intruder Fuzzer',
    desc: 'Automated attacks with Sniper, Battering Ram, Pitchfork, and Cluster Bomb modes.',
    icon: '⚡'
  },
  {
    title: 'Repeater',
    desc: 'Manually modify and replay requests to test edge cases and validate fixes.',
    icon: '🔄'
  },
  {
    title: 'WebSocket Support',
    desc: 'Full bidirectional frame capture and inspection for modern real-time apps.',
    icon: '🔌'
  }
]
</script>

<template>
  <div class="app-container">
    <div class="hex-bg"></div>
    
    <!-- Navigation -->
    <nav class="navbar container">
      <div class="logo">INT3RCEPTOR</div>
      <div class="links">
        <a href="https://github.com/S1b-Team/int3rceptor#readme" target="_blank">Documentation</a>
        <a href="https://github.com/S1b-Team/int3rceptor" target="_blank">GitHub</a>
        <a href="https://github.com/S1b-Team/int3rceptor/releases" class="nav-btn">Download v1.0</a>
      </div>
    </nav>

    <!-- Hero Section -->
    <header class="hero container">
      <div class="hero-content">
        <h1>The Future of <br>Web Interception</h1>
        <p class="tagline">
          A next-generation HTTP proxy designed for performance, automation, and the modern web. 
          Built in Rust for speed, designed for hackers.
        </p>
        <div class="cta-group">
          <a href="https://github.com/S1b-Team/int3rceptor/releases" class="btn btn-primary">Get Started</a>
          <a href="https://github.com/S1b-Team/int3rceptor" class="btn btn-secondary">View Source</a>
        </div>
      </div>
      <div class="hero-image">
        <div class="image-glow"></div>
        <img src="/assets/screenshots/dashboard.png" alt="Int3rceptor Dashboard" class="dashboard-preview" />
      </div>
    </header>

    <!-- Stats Banner -->
    <section class="stats-banner">
      <div class="container stat-grid">
        <div class="stat-item">
          <span class="stat-value">15k+</span>
          <span class="stat-label">Req/Sec</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">12ms</span>
          <span class="stat-label">Latency</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">45MB</span>
          <span class="stat-label">Memory</span>
        </div>
        <div class="stat-item highlight">
          <span class="stat-value">7x</span>
          <span class="stat-label">Faster than Burp</span>
        </div>
      </div>
    </section>

    <!-- Features Grid -->
    <section class="features container">
      <h2>Power Tools for Pentesters</h2>
      <div class="feature-grid">
        <div v-for="feature in features" :key="feature.title" class="feature-card">
          <div class="feature-icon">{{ feature.icon }}</div>
          <h3>{{ feature.title }}</h3>
          <p>{{ feature.desc }}</p>
        </div>
      </div>
    </section>

    <!-- Ecosystem Preview -->
    <section class="ecosystem container">
      <h2>Built for the Future</h2>
      <p class="section-desc">An extensible platform ready for the next generation of security tools.</p>
      <div class="ecosystem-showcase">
        <img src="/assets/screenshots/ecosystem-preview.png" alt="Ecosystem Preview" />
      </div>
    </section>

    <!-- Footer -->
    <footer class="footer">
      <div class="container">
        <p>&copy; 2025 S1bGr0up. Open Source Security.</p>
        <div class="socials">
          <a href="#">Matrix</a>
          <a href="#">GitHub</a>
          <a href="#">Twitter</a>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  min-height: 100vh;
  position: relative;
}

/* Navbar */
.navbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 2rem;
  padding-bottom: 2rem;
}

.logo {
  font-family: 'Orbitron', sans-serif;
  font-size: 1.5rem;
  font-weight: bold;
  letter-spacing: 2px;
  color: var(--accent-cyan);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.5);
}

.links a {
  margin-left: 2rem;
  font-weight: 500;
}

.nav-btn {
  color: var(--accent-orange) !important;
  border: 1px solid var(--accent-orange);
  padding: 0.5rem 1rem;
  border-radius: 4px;
}

/* Hero */
.hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4rem 2rem;
  gap: 4rem;
}

.hero-content {
  flex: 1;
}

.tagline {
  font-size: 1.2rem;
  color: var(--text-secondary);
  margin-bottom: 2.5rem;
  max-width: 500px;
}

.hero-image {
  flex: 1.2;
  position: relative;
  perspective: 1000px;
}

.dashboard-preview {
  width: 100%;
  border-radius: 8px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(0, 212, 255, 0.2);
  transform: rotateY(-5deg) rotateX(5deg);
  transition: transform 0.5s ease;
}

.hero-image:hover .dashboard-preview {
  transform: rotateY(0) rotateX(0);
}

.image-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 80%;
  height: 80%;
  background: var(--accent-cyan);
  filter: blur(100px);
  opacity: 0.2;
  z-index: -1;
}

/* Stats */
.stats-banner {
  background: var(--bg-secondary);
  padding: 3rem 0;
  margin: 4rem 0;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 2.5rem;
  font-weight: 800;
  color: white;
}

.stat-label {
  color: var(--text-secondary);
  text-transform: uppercase;
  font-size: 0.9rem;
  letter-spacing: 1px;
}

.highlight .stat-value {
  color: var(--accent-magenta);
  text-shadow: 0 0 15px rgba(255, 0, 110, 0.5);
}

/* Features */
.features {
  padding: 4rem 2rem;
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 2rem;
}

.feature-card {
  background: var(--bg-secondary);
  padding: 2rem;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: transform 0.3s;
}

.feature-card:hover {
  transform: translateY(-5px);
  border-color: var(--accent-cyan);
}

.feature-icon {
  font-size: 2rem;
  margin-bottom: 1rem;
}

.feature-card h3 {
  margin-bottom: 1rem;
  color: var(--accent-cyan);
}

.feature-card p {
  color: var(--text-secondary);
  font-size: 0.95rem;
}

/* Ecosystem */
.ecosystem {
  text-align: center;
  padding: 4rem 2rem;
}

.section-desc {
  color: var(--text-secondary);
  margin-bottom: 3rem;
}

.ecosystem-showcase img {
  max-width: 100%;
  border-radius: 8px;
  border: 1px solid rgba(255, 0, 110, 0.2);
  box-shadow: 0 0 30px rgba(255, 0, 110, 0.1);
}

/* Footer */
.footer {
  padding: 3rem 0;
  background: var(--bg-secondary);
  margin-top: 4rem;
  text-align: center;
  color: var(--text-secondary);
}

.socials {
  margin-top: 1rem;
}

.socials a {
  margin: 0 1rem;
  color: var(--text-secondary);
}

.socials a:hover {
  color: white;
}

@media (max-width: 768px) {
  .hero {
    flex-direction: column;
    text-align: center;
    padding: 2rem 1rem;
    gap: 2rem;
  }
  
  .hero-content {
    max-width: 100%;
  }
  
  .tagline {
    max-width: 100%;
    font-size: 1.1rem;
  }
  
  .cta-group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
  }
  
  .stat-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 2rem;
  }
  
  .stat-value {
    font-size: 2rem;
  }
  
  .navbar {
    flex-direction: column;
    gap: 1rem;
    padding-top: 1.5rem;
    padding-bottom: 1.5rem;
  }
  
  .links {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 1rem;
  }
  
  .links a {
    margin: 0;
  }
  
  .feature-grid {
    grid-template-columns: 1fr;
  }
  
  .dashboard-preview {
    transform: none;
  }
}

@media (max-width: 480px) {
  .hero {
    padding: 1.5rem 0.5rem;
  }
  
  .tagline {
    font-size: 1rem;
    margin-bottom: 1.5rem;
  }
  
  .stat-grid {
    grid-template-columns: 1fr;
    gap: 1.5rem;
  }
  
  .stat-value {
    font-size: 1.8rem;
  }
  
  .stat-label {
    font-size: 0.8rem;
  }
  
  .logo {
    font-size: 1.2rem;
  }
  
  .links a {
    font-size: 0.9rem;
  }
  
  .nav-btn {
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
  }
  
  .features,
  .ecosystem {
    padding: 2rem 1rem;
  }
  
  .feature-card {
    padding: 1.5rem;
  }
  
  .footer {
    padding: 2rem 0;
  }
  
  .socials a {
    margin: 0 0.5rem;
    font-size: 0.9rem;
  }
}
</style>
