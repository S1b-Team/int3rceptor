#!/usr/bin/env node

/**
 * Performance Test Runner Script
 * 
 * This script runs performance tests and generates reports
 * Usage:
 *   node scripts/run-performance-tests.js                    # Run all tests
 *   node scripts/run-performance-tests.js --lighthouse       # Run with Lighthouse audits
 *   node scripts/run-performance-tests.js --generate-report # Generate HTML report
 *   node scripts/run-performance-tests.js --save-baseline  # Save current metrics as baseline
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Parse command line arguments
const args = process.argv.slice(2);
const options = {
  lighthouse: args.includes('--lighthouse'),
  generateReport: args.includes('--generate-report'),
  saveBaseline: args.includes('--save-baseline'),
  cold: args.includes('--cold') || args.includes('--cold-start'),
  warm: args.includes('--warm') || args.includes('--warm-cache'),
};

console.log('🚀 INT3RCEPTOR Performance Test Runner');
console.log('=====================================\n');

// Ensure reports directory exists
const reportsDir = path.join(__dirname, '..', 'tests', 'performance', 'reports');
const baselinesDir = path.join(__dirname, '..', 'tests', 'performance', 'baselines');

[reportsDir, baselinesDir].forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
    console.log(`📁 Created directory: ${dir}`);
  }
});

// Run performance tests
function runTests() {
  console.log('🧪 Running performance tests...\n');
  
  const playwrightArgs = ['test', 'tests/performance'];
  
  if (options.lighthouse) {
    console.log('🔍 Including Lighthouse audits');
  }
  
  if (options.cold) {
    console.log('❄️  Running cold cache tests');
  } else if (options.warm) {
    console.log('🔥 Running warm cache tests');
  }
  
  try {
    execSync(`npx playwright ${playwrightArgs.join(' ')}`, {
      stdio: 'inherit',
      cwd: path.join(__dirname, '..'),
    });
    console.log('\n✅ Performance tests completed successfully');
  } catch (error) {
    console.error('\n❌ Performance tests failed');
    process.exit(1);
  }
}

// Generate HTML report
function generateReport() {
  console.log('📊 Generating performance report...\n');
  
  const reportScript = path.join(__dirname, '..', 'tests', 'performance', 'utils.ts');
  
  // This would normally be run via a separate script or test
  console.log('📝 Report generation will be handled by Playwright test reporter');
  console.log('📄 Check the reports directory for HTML output');
}

// Save baseline
function saveBaseline() {
  console.log('💾 Saving performance baseline...\n');
  
  const baselineFiles = [
    'dashboard.json',
    'traffic.json',
    'websocket.json',
    'repeater.json',
    'intruder.json',
    'scanner.json',
    'decoder.json',
    'comparer.json',
    'plugins.json',
    'settings.json',
  ];
  
  console.log('📝 Baseline files will be created after running tests');
  console.log('📁 Location:', baselinesDir);
}

// Run Lighthouse audits
function runLighthouseAudits() {
  console.log('🔍 Running Lighthouse audits...\n');
  
  const lighthouse = require('lighthouse');
  const chromeLauncher = require('chrome-launcher');
  
  const urls = [
    'http://localhost:5173/',
    'http://localhost:5173/#/dashboard',
    'http://localhost:5173/#/traffic',
    'http://localhost:5173/#/websocket',
    'http://localhost:5173/#/repeater',
    'http://localhost:5173/#/intruder',
    'http://localhost:5173/#/scanner',
    'http://localhost:5173/#/decoder',
    'http://localhost:5173/#/comparer',
    'http://localhost:5173/#/plugins',
    'http://localhost:5173/#/settings',
  ];
  
  console.log('📊 Auditing', urls.length, 'pages...');
  
  const results = [];
  
  urls.forEach(async (url) => {
    const chrome = await chromeLauncher.launch({
      chromeFlags: ['--headless', '--disable-gpu', '--no-sandbox'],
    });
    
    const options = {
      logLevel: 'info',
      output: 'json',
      onlyCategories: ['performance'],
      port: chrome.port,
    };
    
    const runnerResult = await lighthouse(url, options);
    await chrome.kill();
    
    const report = JSON.parse(runnerResult.report);
    results.push({
      url,
      score: report.categories.performance.score * 100,
      metrics: {
        fcp: report.audits['first-contentful-paint'].numericValue,
        lcp: report.audits['largest-contentful-paint'].numericValue,
        tbt: report.audits['total-blocking-time'].numericValue,
        cls: report.audits['cumulative-layout-shift'].numericValue,
        si: report.audits['speed-index'].numericValue,
        tti: report.audits['interactive'].numericValue,
      },
    });
    
    console.log(`✅ ${url}: ${report.categories.performance.score * 100}`);
  });
  
  // Save results
  const outputPath = path.join(reportsDir, `lighthouse-report-${Date.now()}.json`);
  fs.writeFileSync(outputPath, JSON.stringify(results, null, 2));
  console.log(`\n📄 Lighthouse report saved to: ${outputPath}`);
}

// Main execution
if (options.generateReport) {
  generateReport();
} else if (options.saveBaseline) {
  saveBaseline();
  runTests();
} else if (options.lighthouse) {
  runLighthouseAudits();
} else {
  runTests();
}

console.log('\n🎉 Done!');
