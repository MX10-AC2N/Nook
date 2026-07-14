/**
 * Nook UI Audit Script
 * Audits visual bugs, missing pages, UX issues, accessibility, and console errors
 */
import playwright from 'playwright';

const BASE = 'http://192.168.1.192:6300';
const USER = { username: 'hermes-bot', password: 'Hermes2026!' };

const findings = [];
function issue(severity, area, title, detail, fix = null) {
  findings.push({ severity, area, title, detail, fix });
}

async function run() {
  const browser = await playwright.chromium.launch({
    executablePath: '/opt/data/Nook/frontend/.playwright-browsers/chromium-1228/chrome-linux64/chrome',
    headless: true,
  });
  const ctx = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
  });
  const page = await ctx.newPage();

  // Capture console messages
  const consoleErrors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      consoleErrors.push({ text: msg.text(), url: page.url() });
    }
  });
  page.on('pageerror', err => {
    consoleErrors.push({ text: err.message, url: page.url(), stack: err.stack });
  });

  async function navigate(url, description) {
    console.log(`\n--- ${description} ---`);
    await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(2000);
    const title = await page.title();
    console.log(`Page: ${title} (${page.url()})`);
    return { title, url: page.url() };
  }

  async function login() {
    console.log('\n=== 1. LOGIN ===');
    await page.goto(`${BASE}/login`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    
    // Check for errors before login
    const preErrors = [...consoleErrors];
    consoleErrors.length = 0;
    
    // Fill login form
    const usernameInput = page.locator('input[name="username"], input[id="username"], input[type="text"]').first();
    const passwordInput = page.locator('input[type="password"]').first();
    const submitBtn = page.locator('button[type="submit"], button:has-text("Se connecter"), button:has-text("Connexion")').first();
    
    await usernameInput.fill(USER.username);
    await passwordInput.fill(USER.password);
    await submitBtn.click();
    
    await page.waitForTimeout(3000);
    await page.waitForLoadState('networkidle');
    
    return consoleErrors;
  }

  async function screenshot(name) {
    const path = `/opt/data/Nook/audit-screenshots/${name}.png`;
    await page.screenshot({ path, fullPage: false });
    return path;
  }

  async function checkConsole() {
    const errs = [...consoleErrors];
    const filtered = errs.filter(e => {
      // Filter out benign errors
      if (e.text?.includes('favicon')) return false;
      if (e.text?.includes('404')) return false;
      if (e.text?.includes('Failed to load resource')) return false;
      return true;
    });
    console.log(`Console errors: ${filtered.length} non-trivial`);
    if (filtered.length > 0) {
      filtered.forEach(e => console.log(`  ${e.url}: ${e.text}`));
    }
    return filtered;
  }

  async function checkImages(msgContent) {
    // Check that images in message content respect max-width:100% and height:auto
    const issues = [];
    if (msgContent && msgContent.includes('<img')) {
      // Check uploaded images have proper classes
      const hasUploadedClass = msgContent.includes('class="uploaded-image"');
      const hasGifClass = msgContent.includes('class="chat-gif"');
      if (!hasUploadedClass && !hasGifClass) {
        issues.push('Images in message content missing uploaded-image or chat-gif class');
      }
    }
    return issues;
  }

  // ──────────────── SCREENSHOTS DIRECTORY ────────────────
  const fs = await import('fs');
  try { fs.mkdirSync('/opt/data/Nook/audit-screenshots', { recursive: true }); } catch {}

  try {
    // ── LOGIN ──
    console.log('\n=== LOGIN ===');
    await navigate(`${BASE}/login`, 'Login page');
    const loginConsoleErrors = await login();
    
    if (loginConsoleErrors.length > 0) {
      issue('🔴', 'Console', 'Login page console errors', 
        `Found ${loginConsoleErrors.length} errors during login: ${loginConsoleErrors.map(e => e.text).join('; ')}`,
        'Investigate console errors from login flow');
    }

    // Take screenshot of main page after login
    await page.waitForTimeout(1500);
    await screenshot('01-main-chat');
    console.log('Screenshot: 01-main-chat');

    // Check main page console
    const mainConsole = await checkConsole();
    if (mainConsole.length > 0) {
      issue('🔴', 'Console', 'Main page console errors',
        mainConsole.map(e => e.text).join('; '),
        'Fix JS errors on main page');
    }

    // ── CHECK CHAT PAGE IMAGE RENDERING ──
    console.log('\n=== CHAT PAGE ===');
    await page.goto(`${BASE}/chat`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    await screenshot('02-chat-page');

    // Check the CSS for uploaded images
    const imageCSS = await page.evaluate(() => {
      const styles = document.querySelectorAll('style');
      let cssText = '';
      styles.forEach(s => cssText += s.textContent || '');
      
      // Check for image-related CSS rules
      const hasUploadedImageRule = cssText.includes('.uploaded-image');
      const hasChatGifRule = cssText.includes('.chat-gif');
      const hasMaxWidth100 = cssText.includes('max-width: 100%');
      const hasHeightAuto = cssText.includes('height: auto');
      const hasMessageContent = cssText.includes('.message-content');
      
      return {
        hasUploadedImageRule,
        hasChatGifRule,
        hasMaxWidth100,
        hasHeightAuto,
        hasMessageContent,
        cssSample: cssText.substring(0, 500),
      };
    });
    console.log('Image CSS checks:', imageCSS);

    if (imageCSS.hasUploadedImageRule && imageCSS.hasMaxWidth100 && imageCSS.hasHeightAuto) {
      console.log('✅ Image CSS correctly includes max-width: 100% and height: auto');
    } else {
      issue('🟡', 'Images', 'Image CSS may be missing responsive rules',
        `uploaded-image: ${imageCSS.hasUploadedImageRule}, max-width:100%: ${imageCSS.hasMaxWidth100}, height:auto: ${imageCSS.hasHeightAuto}`,
        'Verify CSS at lines 2167-2169 of chat/+page.svelte');
    }

    // Check actual images on the page
    const images = await page.evaluate(() => {
      const imgs = document.querySelectorAll('img');
      const issues = [];
      imgs.forEach(img => {
        const style = window.getComputedStyle(img);
        if (img.offsetWidth > 0 && img.offsetWidth > (img.parentElement?.offsetWidth || 9999)) {
          issues.push(`Image ${img.src?.substring(0,80)} overflows parent (${img.offsetWidth}px > ${img.parentElement?.offsetWidth}px)`);
        }
      });
      return { count: imgs.length, issues };
    });
    if (images.issues?.length > 0) {
      issue('🔴', 'Images', 'Images overflowing their container',
        images.issues.join('\n'),
        'Add max-width: 100% and height: auto to all message images');
    }

    // ── CHECK CONVERSATION SIDEBAR ──
    const sidebarVisible = await page.evaluate(() => {
      const sidebar = document.querySelector('.sidebar, [class*="sidebar"], [class*="conv-list"], .conversation-list');
      if (!sidebar) return 'No sidebar found';
      const style = window.getComputedStyle(sidebar);
      return `sidebar found: display=${style.display}, width=${sidebar.offsetWidth}`;
    });
    console.log('Sidebar:', sidebarVisible);

    // ── NAVIGATION: Check all nav links ──
    console.log('\n=== NAVIGATION ===');
    
    // Check nav items from layout
    const navItems = await page.evaluate(() => {
      const nav = document.querySelector('nav, [role="navigation"], .nav, .sidebar-nav');
      if (!nav) return { found: false, links: [] };
      const links = nav.querySelectorAll('a, button[role="link"], .nav-item');
      const items = [];
      links.forEach(l => {
        const text = l.textContent?.trim() || '';
        const href = l.getAttribute('href') || l.getAttribute('data-href') || '';
        if (text && text.length > 0 && text.length < 50) {
          items.push({ text, href, tag: l.tagName, visible: l.offsetParent !== null });
        }
      });
      return { found: true, links: items };
    });
    console.log('Nav items:', JSON.stringify(navItems, null, 2));
    
    if (navItems.found) {
      // Check if /events or "Événements" is in nav
      const hasEvents = navItems.links.some(l => l.href?.includes('events') || l.text?.toLowerCase().includes('évén') || l.text?.toLowerCase().includes('event'));
      if (!hasEvents) {
        issue('🟡', 'Navigation', 'Events page has no navigation link',
          'The /events page exists and is functional but has NO link in the sidebar navigation.',
          'Add an "Événements" nav item to the navItems array in +layout.svelte pointing to /events');
      }

      // Check all expected nav items
      const expectedItems = ['Chat', 'Échecs', 'Calendrier', 'Sondages', 'Administration', 'Paramètres', 'Aide'];
      const missingItems = expectedItems.filter(item => {
        return !navItems.links.some(l => l.text?.toLowerCase().includes(item.toLowerCase()) || l.text?.toLowerCase().includes(item.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase()));
      });
      if (missingItems.length > 0) {
        issue('🟡', 'Navigation', `Missing nav items in sidebar: ${missingItems.join(', ')}`,
          `Could not find: ${missingItems.join(', ')} in sidebar navigation. Also missing: Événements (Events).`,
          'Add missing nav links to navItems array in +layout.svelte');
      }
    } else {
      issue('🔴', 'Navigation', 'Navigation sidebar not found on page',
        'Could not locate nav sidebar on the page. Layout may not be rendering properly.',
        'Check +layout.svelte for sidebar rendering issues');
    }

    // ── MOBILE NAV DRAWER ──
    console.log('\n=== MOBILE NAV ===');
    // Set viewport to mobile size
    await page.setViewportSize({ width: 375, height: 812 });
    await page.waitForTimeout(1000);
    
    const mobileNav = await page.evaluate(() => {
      // Look for hamburger/mobile menu toggle
      const toggles = document.querySelectorAll('button:has(☰), button:has-text("☰"), [class*="menu-toggle"], [class*="hamburger"], .mobile-toggle, [class*="burger"]');
      return { toggleCount: toggles.length };
    });
    console.log('Mobile nav toggles:', mobileNav);
    
    // Try to find the mobile menu toggle by class
    const mobileToggle = page.locator('[class*="menu-toggle"], [class*="hamburger"], .mobile-toggle, [class*="burger"], button:has-text("☰")').first();
    if (await mobileToggle.isVisible().catch(() => false)) {
      await mobileToggle.click();
      await page.waitForTimeout(1000);
      await screenshot('03-mobile-nav-open');
      console.log('Screenshot: 03-mobile-nav-open (mobile nav open)');
      
      // Check what's visible in the mobile drawer
      const mobileDrawerItems = await page.evaluate(() => {
        const drawer = document.querySelector('[class*="drawer"], [class*="mobile-nav"], [class*="sidebar-overlay"], [class*="sidebar"]');
        if (!drawer) return 'No drawer/mobile nav found';
        const links = drawer.querySelectorAll('a, button');
        return Array.from(links).map(l => ({ text: l.textContent?.trim(), href: l.getAttribute('href'), visible: l.offsetParent !== null }));
      });
      console.log('Mobile drawer items:', JSON.stringify(mobileDrawerItems, null, 2));
      
      // Close mobile nav
      await page.keyboard.press('Escape');
      await page.waitForTimeout(500);
    } else {
      issue('🟡', 'Navigation/Mobile', 'Mobile menu toggle not found',
        'Could not find a hamburger/mobile menu toggle button at 375px viewport width.',
        'Check +layout.svelte for mobile navigation implementation');
    }

    // Reset viewport
    await page.setViewportSize({ width: 1440, height: 900 });

    // ── PAGE BY PAGE AUDIT ──
    const pagesToAudit = [
      { url: '/chat', label: 'Chat' },
      { url: '/chess', label: 'Chess' },
      { url: '/calendar', label: 'Calendar' },
      { url: '/events', label: 'Events' },
      { url: '/polls', label: 'Polls' },
      { url: '/admin', label: 'Admin' },
      { url: '/settings', label: 'Settings' },
      { url: '/help', label: 'Help' },
    ];

    for (const p of pagesToAudit) {
      console.log(`\n--- ${p.label} (${p.url}) ---`);
      const consoleBefore = consoleErrors.length;
      
      try {
        await page.goto(`${BASE}${p.url}`, { waitUntil: 'networkidle', timeout: 15000 });
        await page.waitForTimeout(2000);
      } catch(e) {
        issue('🔴', 'Page Load', `${p.label} page failed to load`,
          `Navigation to ${p.url} failed: ${e.message}`,
          'Check server and route');
        continue;
      }
      
      await screenshot(`04-${p.label.toLowerCase()}`);
      
      // Check console errors
      const newErrors = consoleErrors.slice(consoleBefore);
      if (newErrors.length > 0) {
        issue('🔴', 'Console', `${p.label} page has console errors`,
          newErrors.map(e => `${e.url}: ${e.text}`).join('\n'),
          'Investigate and fix JS errors');
      }
      
      // Check for placeholder / coming soon content
      const pageText = await page.evaluate(() => document.body?.innerText?.substring(0, 200));
      if (pageText?.toLowerCase().includes('coming soon') || pageText?.toLowerCase().includes('bientôt') || pageText?.toLowerCase().includes('en construction')) {
        issue('🟡', 'Content', `${p.label} page shows placeholder/coming-soon content`,
          `Page says: "${pageText.substring(0, 150)}"`,
          'Implement the full page if placeholder content remains');
      }
      
      // Check for broken buttons
      const brokenButtons = await page.evaluate(() => {
        const buttons = document.querySelectorAll('button, a[role="button"], [onclick]');
        const issues = [];
        buttons.forEach(b => {
          const text = b.textContent?.trim() || '';
          const href = b.getAttribute('href');
          if (href === '' || href === '#') {
            if (!b.hasAttribute('onclick') && !b.getAttribute('href')?.startsWith('#')) {
              issues.push(`Button "${text.substring(0,30)}" has empty/invalid href`);
            }
          }
          if (b.disabled && !b.hasAttribute('data-enabled-on')) {
            // This is normal for some buttons, just note it
          }
        });
        return issues;
      });
      if (brokenButtons.length > 0) {
        // Only report truly broken (non-functional) buttons
        const nonFunctional = brokenButtons.filter(b => !b.includes('just a placeholder'));
        if (nonFunctional.length > 0) {
          issue('🟡', 'UI', `${p.label} has potentially broken buttons`,
            nonFunctional.join('\n'),
            'Fix empty hrefs or add proper click handlers');
        }
      }
      
      // Check accessibility - visible text labels on buttons
      const a11yIssues = await page.evaluate(() => {
        const issues = [];
        const buttons = document.querySelectorAll('button');
        buttons.forEach(b => {
          const text = b.textContent?.trim() || '';
          const ariaLabel = b.getAttribute('aria-label');
          // Button with icon only
          if (text.length < 2 && !ariaLabel && !b.querySelector('img[alt]')) {
            issues.push(`Button with no text/aria-label: ${b.outerHTML?.substring(0,80)}`);
          }
        });
        const links = document.querySelectorAll('a');
        links.forEach(l => {
          if (!l.textContent?.trim() && !l.getAttribute('aria-label') && !l.querySelector('img[alt]')) {
            issues.push(`Link with no text/aria-label: ${l.outerHTML?.substring(0,80)}`);
          }
        });
        return issues;
      });
      if (a11yIssues.length > 0) {
        issue('🟢', 'Accessibility', `${p.label} has ${a11yIssues.length} unlabeled interactive elements`,
          a11yIssues.join('\n'),
          'Add aria-label to icon-only buttons/links');
      }
    }

    // ── THEME CHECK ──
    console.log('\n=== THEME CHECK ===');
    const themes = [
      { id: 'jardin-secret', name: 'Jardin Secret' },
      { id: 'space-hub', name: 'Space Hub' },
      { id: 'maison-chaleureuse', name: 'Maison Chaleureuse' },
      { id: 'nuit-douce', name: 'Nuit Douce' },
    ];

    // Go to settings to check theme options
    await page.goto(`${BASE}/settings`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    await screenshot('05-settings');

    // Check if 'nuit-douce' theme appears in settings page
    const settingsText = await page.evaluate(() => document.body?.innerText || '');
    if (!settingsText.includes('Nuit Douce') && !settingsText.includes('nuit-douce')) {
      issue('🟡', 'Theme', 'Nuit Douce theme not listed in settings Appearance tab',
        'The ThemeStore has 4 themes but settings page only lists 3 (jardin-secret, space-hub, maison-chaleureuse). nuit-douce is missing from the settings themes array.',
        'Add nuit-douce to the themes array in settings/+page.svelte');
    }

    // Switch to appearance tab and cycle themes
    const appearanceTab = page.locator('button:has-text("Apparence")').first();
    if (await appearanceTab.isVisible().catch(() => false)) {
      await appearanceTab.click();
      await page.waitForTimeout(500);
      await screenshot('06-settings-appearance');
      
      // Apply each theme and take screenshot
      for (const theme of themes) {
        const themeCard = page.locator(`button:has-text("${theme.name}")`).first();
        if (await themeCard.isVisible().catch(() => false)) {
          await themeCard.click();
          await page.waitForTimeout(500);
          
          // Check the theme was applied
          const bodyClass = await page.evaluate(() => document.body.className);
          if (bodyClass.includes(`theme-${theme.id}`)) {
            console.log(`✅ Theme "${theme.name}" applied: ${bodyClass}`);
          } else {
            issue('🟡', 'Theme', `Theme "${theme.name}" may not have applied properly`,
              `body class: ${bodyClass}, expected theme-${theme.id}`,
              'Check theme application logic in ThemeStore');
          }
          
          // Check if dark mode toggle works
          const darkToggle = page.locator('#darkModeToggle, [for="darkModeToggle"]').first();
          if (await darkToggle.isVisible().catch(() => false)) {
            await darkToggle.click();
            await page.waitForTimeout(500);
            // Toggle back
            await darkToggle.click();
            await page.waitForTimeout(500);
          }
        } else {
          issue('🟡', 'Theme', `Theme "${theme.name}" card not found in settings`,
            `Could not find theme card for ${theme.name} in Appearance tab`,
            `Add ${theme.name} to the themes array in settings/+page.svelte`);
        }
      }
    }

    // ── EVENTS PAGE (already navigated, but double-check content) ──
    console.log('\n=== EVENTS PAGE DETAIL ===');
    await page.goto(`${BASE}/events`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    
    const eventsContent = await page.evaluate(() => {
      const h1 = document.querySelector('h1');
      const buttons = Array.from(document.querySelectorAll('button')).map(b => b.textContent?.trim()).filter(Boolean);
      return { heading: h1?.textContent, buttons, text: document.body?.innerText?.substring(0, 300) };
    });
    console.log('Events page content:', JSON.stringify(eventsContent, null, 2));

    // ── CHECK MISSING ROUTES / BROKEN LINKS ──
    console.log('\n=== LINK CHECK ===');
    const links = await page.evaluate(() => {
      return Array.from(document.querySelectorAll('a[href]')).map(a => ({
        href: a.getAttribute('href'),
        text: a.textContent?.trim()?.substring(0, 40),
      })).filter(l => l.href && !l.href.startsWith('#') && !l.href.startsWith('javascript'));
    });
    
    // Check a sample of internal links
    for (const link of links.slice(0, 10)) {
      if (link.href.startsWith('/') || link.href.startsWith(BASE)) {
        const url = link.href.startsWith('http') ? link.href : `${BASE}${link.href}`;
        try {
          const resp = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 10000 });
          if (resp?.status() >= 400) {
            issue('🟡', 'Navigation', `Broken link: "${link.text}" → ${url}`,
              `HTTP ${resp?.status()} for ${url}`,
              'Fix the broken link');
          }
          // Go back
          await page.goBack();
          await page.waitForTimeout(1000);
        } catch(e) {
          // Skip external links
        }
      }
    }

    // ── RESPONSIVE CHECKS ──
    console.log('\n=== RESPONSIVE CHECKS ===');
    const viewports = [
      { width: 1440, height: 900, label: 'Desktop' },
      { width: 1024, height: 768, label: 'Tablet' },
      { width: 375, height: 812, label: 'Mobile' },
    ];

    for (const vp of viewports) {
      await page.setViewportSize({ width: vp.width, height: vp.height });
      await page.goto(`${BASE}/chat`, { waitUntil: 'networkidle' });
      await page.waitForTimeout(1500);
      await screenshot(`07-${vp.label.toLowerCase()}-chat`);
      
      // Check if content is overflowing or clipped
      const overflow = await page.evaluate(() => {
        const body = document.body;
        const docWidth = document.documentElement.scrollWidth;
        const viewportWidth = window.innerWidth;
        const hasHScroll = docWidth > viewportWidth + 5;
        const hasVScroll = document.documentElement.scrollHeight > window.innerHeight + 50;
        return { docWidth, viewportWidth, hasHorizontalScroll: hasHScroll, hasVerticalScroll: hasVScroll };
      });
      if (overflow.hasHorizontalScroll) {
        issue('🟡', 'Responsive', `Horizontal scroll on ${vp.label} (${vp.width}px)`,
          `Document width ${overflow.docWidth}px > viewport ${overflow.viewportWidth}px. Content overflows.`,
          'Add overflow-x: hidden to body or fix wide elements');
      }
    }

    // ── FINAL CONSOLE ERROR SUMMARY ──
    console.log('\n=== FINAL CONSOLE ERROR SUMMARY ===');
    const distinctErrors = [...new Set(consoleErrors.map(e => e.text))];
    if (distinctErrors.length > 0) {
      issue('🔴', 'Console', `Total: ${distinctErrors.length} distinct console errors found`,
        distinctErrors.join('\n---\n'),
        'Fix all JavaScript console errors');
    }

  } catch (err) {
    console.error('FATAL:', err);
    issue('🔴', 'Audit', 'Audit script encountered a fatal error',
      err.message,
      'Review script and fix');
  } finally {
    await browser.close();
  }

  // ── REPORT ──
  console.log('\n\n========================================');
  console.log('         NOOK UI AUDIT REPORT');
  console.log('========================================\n');

  console.log(`Total issues found: ${findings.length}\n`);

  for (const f of findings) {
    console.log(`${f.severity} [${f.area}] ${f.title}`);
    console.log(`  Detail: ${f.detail}`);
    if (f.fix) console.log(`  Fix: ${f.fix}`);
    console.log();
  }

  // Write report to file
  let report = '# Nook UI Audit Report\n\n';
  report += `**Date:** ${new Date().toISOString()}\n`;
  report += `**Target:** ${BASE}\n`;
  report += `**Total issues:** ${findings.length}\n\n`;
  report += '## Issues by Severity\n\n';

  const bySeverity = { '🔴': [], '🟡': [], '🟢': [] };
  findings.forEach(f => bySeverity[f.severity]?.push(f));

  for (const [sev, items] of Object.entries(bySeverity)) {
    if (items.length > 0) {
      report += `### ${sev} ${sev === '🔴' ? 'High' : sev === '🟡' ? 'Medium' : 'Low'} Priority\n\n`;
      items.forEach(f => {
        report += `- **${f.title}** ([${f.area}])\n`;
        report += `  - Detail: ${f.detail}\n`;
        if (f.fix) report += `  - Fix: ${f.fix}\n`;
        report += '\n';
      });
    }
  }

  require('fs').writeFileSync('/opt/data/Nook/audit-report.md', report);
  console.log('\nReport saved to: /opt/data/Nook/audit-report.md');
}

run().catch(console.error);
