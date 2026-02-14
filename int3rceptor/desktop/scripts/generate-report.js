#!/usr/bin/env node

/**
 * Consolidated Test Report Generator for INT3RCEPTOR UI
 *
 * This script parses test results from all test suites and generates:
 * - A consolidated HTML report with all findings
 * - A findings.json file with structured data
 *
 * Usage:
 *   node scripts/generate-report.js
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.dirname(__dirname);

// Colors for console output
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m'
};

function logInfo(message) {
  console.log(`${colors.blue}[INFO]${colors.reset} ${message}`);
}

function logSuccess(message) {
  console.log(`${colors.green}[SUCCESS]${colors.reset} ${message}`);
}

function logWarning(message) {
  console.log(`${colors.yellow}[WARNING]${colors.reset} ${message}`);
}

function logError(message) {
  console.log(`${colors.red}[ERROR]${colors.reset} ${message}`);
}

// Test suite results structure
const testResults = {
  crossBrowser: {
    status: 'unknown',
    passed: 0,
    failed: 0,
    duration: 0,
    browsers: {},
    failures: []
  },
  accessibility: {
    status: 'unknown',
    passed: 0,
    failed: 0,
    duration: 0,
    violations: [],
    passes: []
  },
  visual: {
    status: 'unknown',
    passed: 0,
    failed: 0,
    duration: 0,
    diffs: []
  },
  performance: {
    status: 'unknown',
    passed: 0,
    failed: 0,
    duration: 0,
    metrics: {}
  }
};

// Parse Playwright JSON results
function parsePlaywrightResults(filePath) {
  try {
    if (!fs.existsSync(filePath)) {
      logWarning(`Playwright results file not found: ${filePath}`);
      return null;
    }
    const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    return data;
  } catch (error) {
    logWarning(`Failed to parse Playwright results: ${error.message}`);
    return null;
  }
}

// Parse test log files
function parseTestLog(filePath) {
  try {
    if (!fs.existsSync(filePath)) {
      logWarning(`Test log file not found: ${filePath}`);
      return null;
    }
    return fs.readFileSync(filePath, 'utf8');
  } catch (error) {
    logWarning(`Failed to read test log: ${error.message}`);
    return null;
  }
}

// Parse cross-browser test results
function parseCrossBrowserResults() {
  logInfo('Parsing cross-browser test results...');

  const resultsPath = path.join(PROJECT_ROOT, 'test-results/results.json');
  const logPath = path.join(PROJECT_ROOT, 'test-results/cross-browser/test-output.log');

  const playwrightResults = parsePlaywrightResults(resultsPath);
  const logContent = parseTestLog(logPath);

  if (playwrightResults) {
    testResults.crossBrowser.status = playwrightResults.status || 'unknown';
    testResults.crossBrowser.passed = playwrightResults.stats?.expected || 0;
    testResults.crossBrowser.failed = playwrightResults.stats?.unexpected || 0;

    // Extract browser-specific results
    if (playwrightResults.suites) {
      playwrightResults.suites.forEach(suite => {
        const browserName = suite.title || 'unknown';
        testResults.crossBrowser.browsers[browserName] = {
          passed: suite.specs?.filter(s => s.tests?.every(t => t.results?.[0]?.status === 'passed')).length || 0,
          failed: suite.specs?.filter(s => s.tests?.some(t => t.results?.[0]?.status !== 'passed')).length || 0
        };
      });
    }

    // Extract failures
    if (playwrightResults.suites) {
      playwrightResults.suites.forEach(suite => {
        suite.specs?.forEach(spec => {
          spec.tests?.forEach(test => {
            const result = test.results?.[0];
            if (result?.status !== 'passed') {
              testResults.crossBrowser.failures.push({
                browser: suite.title,
                test: spec.title,
                error: result?.error?.message || 'Unknown error',
                location: result?.error?.location || ''
              });
            }
          });
        });
      });
    }
  }

  // Try to extract duration from log
  if (logContent) {
    const durationMatch = logContent.match(/Duration: (\d+)s/);
    if (durationMatch) {
      testResults.crossBrowser.duration = parseInt(durationMatch[1]);
    }
  }
}

// Parse accessibility test results
function parseAccessibilityResults() {
  logInfo('Parsing accessibility test results...');

  const resultsPath = path.join(PROJECT_ROOT, 'test-results/results.json');
  const logPath = path.join(PROJECT_ROOT, 'test-results/accessibility/test-output.log');

  const playwrightResults = parsePlaywrightResults(resultsPath);
  const logContent = parseTestLog(logPath);

  if (playwrightResults) {
    testResults.accessibility.status = playwrightResults.status || 'unknown';
    testResults.accessibility.passed = playwrightResults.stats?.expected || 0;
    testResults.accessibility.failed = playwrightResults.stats?.unexpected || 0;

    // Extract violations from test results
    if (playwrightResults.suites) {
      playwrightResults.suites.forEach(suite => {
        suite.specs?.forEach(spec => {
          spec.tests?.forEach(test => {
            const result = test.results?.[0];
            if (result?.status !== 'passed') {
              // Try to parse axe violations from error message
              const error = result?.error?.message || '';
              const violationsMatch = error.match(/Found (\d+) violations/);
              if (violationsMatch) {
                const violationCount = parseInt(violationsMatch[1]);
                for (let i = 0; i < violationCount; i++) {
                  testResults.accessibility.violations.push({
                    test: spec.title,
                    impact: 'medium', // Default impact
                    description: 'Accessibility violation detected',
                    element: 'Unknown element'
                  });
                }
              } else {
                testResults.accessibility.violations.push({
                  test: spec.title,
                  impact: 'medium',
                  description: error || 'Accessibility issue detected',
                  element: 'Unknown element'
                });
              }
            } else {
              testResults.accessibility.passes.push({
                test: spec.title,
                description: 'No accessibility violations found'
              });
            }
          });
        });
      });
    }
  }

  // Try to extract duration from log
  if (logContent) {
    const durationMatch = logContent.match(/Duration: (\d+)s/);
    if (durationMatch) {
      testResults.accessibility.duration = parseInt(durationMatch[1]);
    }
  }
}

// Parse visual regression test results
function parseVisualResults() {
  logInfo('Parsing visual regression test results...');

  const resultsPath = path.join(PROJECT_ROOT, 'test-results/results.json');
  const logPath = path.join(PROJECT_ROOT, 'test-results/visual/test-output.log');

  const playwrightResults = parsePlaywrightResults(resultsPath);
  const logContent = parseTestLog(logPath);

  if (playwrightResults) {
    testResults.visual.status = playwrightResults.status || 'unknown';
    testResults.visual.passed = playwrightResults.stats?.expected || 0;
    testResults.visual.failed = playwrightResults.stats?.unexpected || 0;

    // Extract visual diffs
    if (playwrightResults.suites) {
      playwrightResults.suites.forEach(suite => {
        suite.specs?.forEach(spec => {
          spec.tests?.forEach(test => {
            const result = test.results?.[0];
            if (result?.status !== 'passed') {
              testResults.visual.diffs.push({
                test: spec.title,
                screenshot: result?.attachments?.[0]?.path || '',
                diff: result?.attachments?.[1]?.path || '',
                error: result?.error?.message || 'Visual difference detected'
              });
            }
          });
        });
      });
    }
  }

  // Try to extract duration from log
  if (logContent) {
    const durationMatch = logContent.match(/Duration: (\d+)s/);
    if (durationMatch) {
      testResults.visual.duration = parseInt(durationMatch[1]);
    }
  }
}

// Parse performance test results
function parsePerformanceResults() {
  logInfo('Parsing performance test results...');

  const logPath = path.join(PROJECT_ROOT, 'test-results/performance/test-output.log');
  const reportsDir = path.join(PROJECT_ROOT, 'tests/performance/reports');

  const logContent = parseTestLog(logPath);

  // Try to find the latest performance report JSON
  try {
    if (fs.existsSync(reportsDir)) {
      const files = fs.readdirSync(reportsDir).filter(f => f.endsWith('.json'));
      if (files.length > 0) {
        const latestFile = files.sort().pop();
        const reportPath = path.join(reportsDir, latestFile);
        const reportData = JSON.parse(fs.readFileSync(reportPath, 'utf8'));

        if (reportData.metrics) {
          testResults.performance.metrics = reportData.metrics;
        }
      }
    }
  } catch (error) {
    logWarning(`Failed to parse performance report: ${error.message}`);
  }

  // Try to extract duration from log
  if (logContent) {
    const durationMatch = logContent.match(/Duration: (\d+)s/);
    if (durationMatch) {
      testResults.performance.duration = parseInt(durationMatch[1]);
    }

    // Try to extract pass/fail from log
    const passedMatch = logContent.match(/(\d+) tests passed/);
    const failedMatch = logContent.match(/(\d+) tests failed/);

    if (passedMatch) {
      testResults.performance.passed = parseInt(passedMatch[1]);
    }
    if (failedMatch) {
      testResults.performance.failed = parseInt(failedMatch[1]);
    }

    testResults.performance.status = testResults.performance.failed === 0 ? 'passed' : 'failed';
  }
}

// Generate findings.json
function generateFindingsJson() {
  logInfo('Generating findings.json...');

  const findings = {
    timestamp: new Date().toISOString(),
    summary: {
      totalSuites: 4,
      passedSuites: 0,
      failedSuites: 0,
      totalDuration: 0
    },
    suites: {
      crossBrowser: {
        name: 'Cross-Browser Compatibility',
        status: testResults.crossBrowser.status,
        passed: testResults.crossBrowser.passed,
        failed: testResults.crossBrowser.failed,
        duration: testResults.crossBrowser.duration,
        issues: testResults.crossBrowser.failures.map(f => ({
          type: 'compatibility',
          severity: 'high',
          description: `${f.browser}: ${f.test} - ${f.error}`,
          recommendation: 'Test and fix the failing functionality across all target browsers',
          effort: 'medium'
        }))
      },
      accessibility: {
        name: 'Accessibility',
        status: testResults.accessibility.status,
        passed: testResults.accessibility.passed,
        failed: testResults.accessibility.failed,
        duration: testResults.accessibility.duration,
        issues: testResults.accessibility.violations.map(v => ({
          type: 'accessibility',
          severity: v.impact === 'critical' ? 'high' : v.impact === 'serious' ? 'medium' : 'low',
          description: `${v.test}: ${v.description}`,
          recommendation: 'Fix accessibility violations to ensure WCAG compliance',
          effort: v.impact === 'critical' ? 'high' : 'low'
        }))
      },
      visual: {
        name: 'Visual Regression',
        status: testResults.visual.status,
        passed: testResults.visual.passed,
        failed: testResults.visual.failed,
        duration: testResults.visual.duration,
        issues: testResults.visual.diffs.map(d => ({
          type: 'visual',
          severity: 'medium',
          description: `${d.test}: ${d.error}`,
          recommendation: 'Review visual differences and update baselines if needed',
          effort: 'low'
        }))
      },
      performance: {
        name: 'Performance',
        status: testResults.performance.status,
        passed: testResults.performance.passed,
        failed: testResults.performance.failed,
        duration: testResults.performance.duration,
        metrics: testResults.performance.metrics,
        issues: testResults.performance.failed > 0 ? [{
          type: 'performance',
          severity: 'medium',
          description: `${testResults.performance.failed} performance test(s) failed`,
          recommendation: 'Optimize performance metrics to meet baseline requirements',
          effort: 'medium'
        }] : []
      }
    },
    recommendations: []
  };

  // Calculate summary
  for (const suite of Object.values(findings.suites)) {
    if (suite.status === 'passed' || suite.failed === 0) {
      findings.summary.passedSuites++;
    } else {
      findings.summary.failedSuites++;
    }
    findings.summary.totalDuration += suite.duration;
  }

  // Add overall recommendations
  if (findings.summary.failedSuites > 0) {
    findings.recommendations.push({
      priority: 'high',
      description: `${findings.summary.failedSuites} test suite(s) have failures`,
      action: 'Review and fix failing tests before proceeding to production'
    });
  }

  if (testResults.accessibility.violations.length > 0) {
    findings.recommendations.push({
      priority: 'high',
      description: `${testResults.accessibility.violations.length} accessibility violation(s) found`,
      action: 'Address accessibility issues to ensure compliance with WCAG 2.1 AA standards'
    });
  }

  if (testResults.visual.diffs.length > 0) {
    findings.recommendations.push({
      priority: 'medium',
      description: `${testResults.visual.diffs.length} visual difference(s) detected`,
      action: 'Review visual differences and update baselines if changes are intentional'
    });
  }

  if (testResults.crossBrowser.failures.length > 0) {
    findings.recommendations.push({
      priority: 'high',
      description: `${testResults.crossBrowser.failures.length} cross-browser compatibility issue(s) found`,
      action: 'Test and fix compatibility issues across all supported browsers'
    });
  }

  // Write findings.json
  const findingsPath = path.join(PROJECT_ROOT, 'test-results/findings.json');
  fs.writeFileSync(findingsPath, JSON.stringify(findings, null, 2));

  logSuccess(`Findings JSON generated: ${findingsPath}`);

  return findings;
}

// Generate HTML report
function generateHtmlReport(findings) {
  logInfo('Generating HTML report...');

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Consolidated Test Report - INT3RCEPTOR UI</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #1e3a5f 0%, #2d5a87 50%, #1e3a5f 100%);
            min-height: 100vh;
            padding: 20px;
            color: #333;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 16px;
            box-shadow: 0 25px 80px rgba(0, 0, 0, 0.4);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #1e3a5f 0%, #2d5a87 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }
        .header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
            font-weight: 700;
        }
        .header .subtitle {
            opacity: 0.9;
            font-size: 1.1em;
            margin-bottom: 20px;
        }
        .header .timestamp {
            opacity: 0.8;
            font-size: 0.9em;
            background: rgba(255, 255, 255, 0.1);
            padding: 8px 16px;
            border-radius: 20px;
            display: inline-block;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8fafc;
            border-bottom: 2px solid #e2e8f0;
        }
        .summary-card {
            background: white;
            padding: 25px;
            border-radius: 12px;
            text-align: center;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
            transition: transform 0.2s;
        }
        .summary-card:hover {
            transform: translateY(-4px);
        }
        .summary-card h3 {
            font-size: 2.5em;
            margin-bottom: 8px;
            font-weight: 700;
        }
        .summary-card.passed h3 { color: #10b981; }
        .summary-card.failed h3 { color: #ef4444; }
        .summary-card.duration h3 { color: #3b82f6; }
        .summary-card p {
            color: #64748b;
            font-size: 0.85em;
            text-transform: uppercase;
            letter-spacing: 1px;
            font-weight: 600;
        }
        .content {
            padding: 30px;
        }
        .section {
            margin-bottom: 40px;
        }
        .section h2 {
            color: #1e3a5f;
            margin-bottom: 20px;
            padding-bottom: 12px;
            border-bottom: 3px solid #3b82f6;
            font-size: 1.5em;
            font-weight: 600;
        }
        .suite-card {
            background: #f8fafc;
            border-radius: 12px;
            padding: 25px;
            margin-bottom: 20px;
            border-left: 4px solid #3b82f6;
        }
        .suite-card.passed { border-left-color: #10b981; }
        .suite-card.failed { border-left-color: #ef4444; }
        .suite-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 15px;
        }
        .suite-title {
            font-size: 1.3em;
            font-weight: 600;
            color: #1e3a5f;
        }
        .suite-stats {
            display: flex;
            gap: 15px;
        }
        .stat-badge {
            padding: 6px 14px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: 600;
        }
        .stat-badge.pass { background: #d1fae5; color: #065f46; }
        .stat-badge.fail { background: #fee2e2; color: #991b1b; }
        .stat-badge.duration { background: #dbeafe; color: #1e40af; }
        .issue-list {
            margin-top: 15px;
        }
        .issue-item {
            background: white;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 10px;
            border-left: 3px solid #fbbf24;
        }
        .issue-item.high { border-left-color: #ef4444; }
        .issue-item.medium { border-left-color: #f59e0b; }
        .issue-item.low { border-left-color: #10b981; }
        .issue-header {
            display: flex;
            justify-content: space-between;
            margin-bottom: 8px;
        }
        .issue-type {
            font-weight: 600;
            color: #1e3a5f;
        }
        .issue-severity {
            font-size: 0.8em;
            padding: 2px 8px;
            border-radius: 10px;
            font-weight: 600;
            text-transform: uppercase;
        }
        .issue-severity.high { background: #fee2e2; color: #991b1b; }
        .issue-severity.medium { background: #fef3c7; color: #92400e; }
        .issue-severity.low { background: #d1fae5; color: #065f46; }
        .issue-description {
            color: #64748b;
            margin-bottom: 8px;
        }
        .issue-recommendation {
            background: #f0f9ff;
            padding: 10px;
            border-radius: 6px;
            font-size: 0.9em;
            color: #0369a1;
        }
        .issue-effort {
            display: inline-block;
            margin-top: 8px;
            font-size: 0.85em;
            color: #64748b;
        }
        .browser-matrix {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }
        .browser-item {
            background: white;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
        }
        .browser-name {
            font-weight: 600;
            color: #1e3a5f;
            margin-bottom: 8px;
        }
        .browser-status {
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 0.85em;
            font-weight: 600;
        }
        .browser-status.pass { background: #d1fae5; color: #065f46; }
        .browser-status.fail { background: #fee2e2; color: #991b1b; }
        .metrics-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 15px;
        }
        .metrics-table th,
        .metrics-table td {
            padding: 12px 15px;
            text-align: left;
            border-bottom: 1px solid #e2e8f0;
        }
        .metrics-table th {
            background: #f1f5f9;
            font-weight: 600;
            color: #1e3a5f;
        }
        .metrics-table tr:hover {
            background: #f8fafc;
        }
        .diff-gallery {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-top: 15px;
        }
        .diff-item {
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }
        .diff-header {
            background: #f1f5f9;
            padding: 12px 15px;
            font-weight: 600;
            color: #1e3a5f;
        }
        .diff-content {
            padding: 15px;
        }
        .diff-image {
            width: 100%;
            border-radius: 6px;
            margin-bottom: 10px;
        }
        .recommendations {
            background: #fef3c7;
            border-radius: 12px;
            padding: 20px;
            margin-top: 20px;
        }
        .recommendations h3 {
            color: #92400e;
            margin-bottom: 15px;
            font-size: 1.2em;
        }
        .recommendation-item {
            background: white;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 10px;
            border-left: 4px solid #f59e0b;
        }
        .recommendation-priority {
            display: inline-block;
            padding: 2px 8px;
            border-radius: 10px;
            font-size: 0.8em;
            font-weight: 600;
            text-transform: uppercase;
            margin-right: 10px;
        }
        .recommendation-priority.high { background: #fee2e2; color: #991b1b; }
        .recommendation-priority.medium { background: #fef3c7; color: #92400e; }
        .recommendation-priority.low { background: #d1fae5; color: #065f46; }
        .footer {
            text-align: center;
            padding: 25px;
            color: #64748b;
            font-size: 0.9em;
            background: #f8fafc;
            border-top: 1px solid #e2e8f0;
        }
        .no-issues {
            text-align: center;
            padding: 30px;
            color: #10b981;
            font-size: 1.1em;
            font-weight: 600;
        }
        .no-issues::before {
            content: "✓ ";
            font-size: 1.3em;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Consolidated Test Report</h1>
            <div class="subtitle">INT3RCEPTOR UI - Complete Test Suite Results</div>
            <div class="timestamp">Generated: ${findings.timestamp}</div>
        </div>

        <div class="summary">
            <div class="summary-card passed">
                <h3>${findings.summary.passedSuites}</h3>
                <p>Suites Passed</p>
            </div>
            <div class="summary-card failed">
                <h3>${findings.summary.failedSuites}</h3>
                <p>Suites Failed</p>
            </div>
            <div class="summary-card duration">
                <h3>${findings.summary.totalDuration}s</h3>
                <p>Total Duration</p>
            </div>
            <div class="summary-card ${findings.summary.failedSuites === 0 ? 'passed' : 'failed'}">
                <h3>${findings.summary.failedSuites === 0 ? 'PASSED' : 'FAILED'}</h3>
                <p>Overall Status</p>
            </div>
        </div>

        <div class="content">
            <!-- Cross-Browser Section -->
            <div class="section">
                <h2>Cross-Browser Compatibility</h2>
                <div class="suite-card ${findings.suites.crossBrowser.status === 'passed' ? 'passed' : 'failed'}">
                    <div class="suite-header">
                        <div class="suite-title">${findings.suites.crossBrowser.name}</div>
                        <div class="suite-stats">
                            <span class="stat-badge pass">${findings.suites.crossBrowser.passed} passed</span>
                            <span class="stat-badge fail">${findings.suites.crossBrowser.failed} failed</span>
                            <span class="stat-badge duration">${findings.suites.crossBrowser.duration}s</span>
                        </div>
                    </div>
                    ${Object.keys(testResults.crossBrowser.browsers).length > 0 ? `
                    <div class="browser-matrix">
                        ${Object.entries(testResults.crossBrowser.browsers).map(([browser, data]) => `
                        <div class="browser-item">
                            <div class="browser-name">${browser}</div>
                            <div class="browser-status ${data.failed === 0 ? 'pass' : 'fail'}">
                                ${data.failed === 0 ? '✓ Passed' : '✗ Failed'}
                            </div>
                            <div style="margin-top: 8px; font-size: 0.85em; color: #64748b;">
                                ${data.passed} passed, ${data.failed} failed
                            </div>
                        </div>
                        `).join('')}
                    </div>
                    ` : ''}
                    ${findings.suites.crossBrowser.issues.length > 0 ? `
                    <div class="issue-list">
                        ${findings.suites.crossBrowser.issues.map(issue => `
                        <div class="issue-item ${issue.severity}">
                            <div class="issue-header">
                                <span class="issue-type">${issue.type}</span>
                                <span class="issue-severity ${issue.severity}">${issue.severity}</span>
                            </div>
                            <div class="issue-description">${issue.description}</div>
                            <div class="issue-recommendation"><strong>Recommendation:</strong> ${issue.recommendation}</div>
                            <div class="issue-effort">Effort: ${issue.effort}</div>
                        </div>
                        `).join('')}
                    </div>
                    ` : '<div class="no-issues">No cross-browser compatibility issues found</div>'}
                </div>
            </div>

            <!-- Accessibility Section -->
            <div class="section">
                <h2>Accessibility Testing</h2>
                <div class="suite-card ${findings.suites.accessibility.status === 'passed' ? 'passed' : 'failed'}">
                    <div class="suite-header">
                        <div class="suite-title">${findings.suites.accessibility.name}</div>
                        <div class="suite-stats">
                            <span class="stat-badge pass">${findings.suites.accessibility.passed} passed</span>
                            <span class="stat-badge fail">${findings.suites.accessibility.failed} failed</span>
                            <span class="stat-badge duration">${findings.suites.accessibility.duration}s</span>
                        </div>
                    </div>
                    ${findings.suites.accessibility.issues.length > 0 ? `
                    <div class="issue-list">
                        ${findings.suites.accessibility.issues.map(issue => `
                        <div class="issue-item ${issue.severity}">
                            <div class="issue-header">
                                <span class="issue-type">${issue.type}</span>
                                <span class="issue-severity ${issue.severity}">${issue.severity}</span>
                            </div>
                            <div class="issue-description">${issue.description}</div>
                            <div class="issue-recommendation"><strong>Recommendation:</strong> ${issue.recommendation}</div>
                            <div class="issue-effort">Effort: ${issue.effort}</div>
                        </div>
                        `).join('')}
                    </div>
                    ` : '<div class="no-issues">No accessibility violations found</div>'}
                </div>
            </div>

            <!-- Visual Regression Section -->
            <div class="section">
                <h2>Visual Regression Testing</h2>
                <div class="suite-card ${findings.suites.visual.status === 'passed' ? 'passed' : 'failed'}">
                    <div class="suite-header">
                        <div class="suite-title">${findings.suites.visual.name}</div>
                        <div class="suite-stats">
                            <span class="stat-badge pass">${findings.suites.visual.passed} passed</span>
                            <span class="stat-badge fail">${findings.suites.visual.failed} failed</span>
                            <span class="stat-badge duration">${findings.suites.visual.duration}s</span>
                        </div>
                    </div>
                    ${testResults.visual.diffs.length > 0 ? `
                    <div class="diff-gallery">
                        ${testResults.visual.diffs.map(diff => `
                        <div class="diff-item">
                            <div class="diff-header">${diff.test}</div>
                            <div class="diff-content">
                                <p style="margin-bottom: 10px; color: #64748b;">${diff.error}</p>
                                ${diff.diff ? `<img src="../${diff.diff}" alt="Diff" class="diff-image">` : ''}
                            </div>
                        </div>
                        `).join('')}
                    </div>
                    ` : '<div class="no-issues">No visual differences detected</div>'}
                    ${findings.suites.visual.issues.length > 0 ? `
                    <div class="issue-list" style="margin-top: 20px;">
                        ${findings.suites.visual.issues.map(issue => `
                        <div class="issue-item ${issue.severity}">
                            <div class="issue-header">
                                <span class="issue-type">${issue.type}</span>
                                <span class="issue-severity ${issue.severity}">${issue.severity}</span>
                            </div>
                            <div class="issue-description">${issue.description}</div>
                            <div class="issue-recommendation"><strong>Recommendation:</strong> ${issue.recommendation}</div>
                            <div class="issue-effort">Effort: ${issue.effort}</div>
                        </div>
                        `).join('')}
                    </div>
                    ` : ''}
                </div>
            </div>

            <!-- Performance Section -->
            <div class="section">
                <h2>Performance Testing</h2>
                <div class="suite-card ${findings.suites.performance.status === 'passed' ? 'passed' : 'failed'}">
                    <div class="suite-header">
                        <div class="suite-title">${findings.suites.performance.name}</div>
                        <div class="suite-stats">
                            <span class="stat-badge pass">${findings.suites.performance.passed} passed</span>
                            <span class="stat-badge fail">${findings.suites.performance.failed} failed</span>
                            <span class="stat-badge duration">${findings.suites.performance.duration}s</span>
                        </div>
                    </div>
                    ${Object.keys(testResults.performance.metrics).length > 0 ? `
                    <table class="metrics-table">
                        <thead>
                            <tr>
                                <th>Metric</th>
                                <th>Value</th>
                                <th>Status</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${Object.entries(testResults.performance.metrics).map(([metric, value]) => `
                            <tr>
                                <td>${metric}</td>
                                <td>${typeof value === 'number' ? value.toFixed(2) : value}</td>
                                <td><span class="stat-badge pass">OK</span></td>
                            </tr>
                            `).join('')}
                        </tbody>
                    </table>
                    ` : '<p style="color: #64748b; margin-top: 15px;">No performance metrics available</p>'}
                    ${findings.suites.performance.issues.length > 0 ? `
                    <div class="issue-list" style="margin-top: 20px;">
                        ${findings.suites.performance.issues.map(issue => `
                        <div class="issue-item ${issue.severity}">
                            <div class="issue-header">
                                <span class="issue-type">${issue.type}</span>
                                <span class="issue-severity ${issue.severity}">${issue.severity}</span>
                            </div>
                            <div class="issue-description">${issue.description}</div>
                            <div class="issue-recommendation"><strong>Recommendation:</strong> ${issue.recommendation}</div>
                            <div class="issue-effort">Effort: ${issue.effort}</div>
                        </div>
                        `).join('')}
                    </div>
                    ` : ''}
                </div>
            </div>

            <!-- Recommendations Section -->
            ${findings.recommendations.length > 0 ? `
            <div class="section">
                <h2>Recommendations</h2>
                <div class="recommendations">
                    ${findings.recommendations.map(rec => `
                    <div class="recommendation-item">
                        <span class="recommendation-priority ${rec.priority}">${rec.priority}</span>
                        <strong>${rec.description}</strong>
                        <p style="margin-top: 8px; color: #64748b;">${rec.action}</p>
                    </div>
                    `).join('')}
                </div>
            </div>
            ` : ''}
        </div>

        <div class="footer">
            INT3RCEPTOR UI - Consolidated Test Report<br>
            Generated on ${new Date().toLocaleString()}
        </div>
    </div>
</body>
</html>`;

  // Write HTML report
  const htmlPath = path.join(PROJECT_ROOT, 'test-results/consolidated-report.html');
  fs.writeFileSync(htmlPath, html);

  logSuccess(`HTML report generated: ${htmlPath}`);
}

// Main execution
function main() {
  console.log('');
  console.log('==========================================');
  console.log('  Consolidated Report Generator');
  console.log('  INT3RCEPTOR UI');
  console.log('==========================================');
  console.log('');

  // Ensure test-results directory exists
  const testResultsDir = path.join(PROJECT_ROOT, 'test-results');
  if (!fs.existsSync(testResultsDir)) {
    fs.mkdirSync(testResultsDir, { recursive: true });
  }

  // Parse all test results
  parseCrossBrowserResults();
  parseAccessibilityResults();
  parseVisualResults();
  parsePerformanceResults();

  // Generate findings JSON
  const findings = generateFindingsJson();

  // Generate HTML report
  generateHtmlReport(findings);

  console.log('');
  logSuccess('Report generation completed!');
  console.log('');
  console.log('Generated files:');
  console.log(`  - HTML: ${path.join(PROJECT_ROOT, 'test-results/consolidated-report.html')}`);
  console.log(`  - JSON: ${path.join(PROJECT_ROOT, 'test-results/findings.json')}`);
  console.log('');
}

// Run main function
main();
