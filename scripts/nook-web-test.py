#!/usr/bin/env python3
"""
nook-web-test.py — Outil de test web pour Nook (similaire Chrome DevTools MCP)
Usage: python3 scripts/nook-web-test.py [--url URL] [--cookie COOKIE] [--pages PAGES] [--viewport VIEWPORT]
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime

def main():
    parser = argparse.ArgumentParser(description="Test Nook web UI across viewports")
    parser.add_argument("--url", default="http://192.168.1.192:6300", help="Nook URL")
    parser.add_argument("--cookie", default="", help="auth_token cookie value")
    parser.add_argument("--pages", default="chat,calendar,chess,polls,settings", help="Comma-separated pages")
    parser.add_argument("--viewport", default="desktop,tablet,mobile", help="Comma-separated viewports")
    parser.add_argument("--output", default="/tmp/nook-test", help="Output directory")
    parser.add_argument("--check-errors", action="store_true", help="Check for JS errors")
    parser.add_argument("--check-network", action="store_true", help="Check network requests")
    args = parser.parse_args()

    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("Installing Playwright...")
        os.system("pip install --break-system-packages playwright && playwright install chromium")
        from playwright.sync_api import sync_playwright

    os.makedirs(args.output, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    viewports = {
        "desktop": (1440, 900),
        "tablet": (768, 1024),
        "mobile": (375, 812),
    }

    pages = args.pages.split(",")
    selected_viewports = args.viewport.split(",")

    print(f"=== Nook Web Test — {timestamp} ===")
    print(f"URL: {args.url}")
    print(f"Pages: {pages}")
    print(f"Viewports: {selected_viewports}")
    print()

    results = []

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)

        for page_name in pages:
            for vp_name in selected_viewports:
                if vp_name not in viewports:
                    continue

                width, height = viewports[vp_name]
                context = browser.new_context(
                    viewport={"width": width, "height": height},
                    device_scale_factor=2 if vp_name == "mobile" else 1,
                )

                if args.cookie:
                    cookie_value = args.cookie.split("=")[1] if "=" in args.cookie else args.cookie
                    context.add_cookies([{
                        "name": "auth_token",
                        "value": cookie_value,
                        "domain": "192.168.1.192",
                        "path": "/",
                    }])

                page = context.new_page()

                # Collect errors
                errors = []
                if args.check_errors:
                    page.on("console", lambda msg: errors.append(f"{msg.type}: {msg.text}") if msg.type in ["error", "warning"] else None)

                # Collect network
                network = []
                if args.check_network:
                    page.on("request", lambda req: network.append(req.url) if req.url.endswith((".js", ".css", ".wasm")) else None)

                try:
                    page.goto(f"{args.url}/{page_name}", wait_until="networkidle", timeout=15000)
                    page.wait_for_timeout(3000)

                    # Take screenshot
                    filename = f"{page_name}_{vp_name}_{width}x{height}.png"
                    filepath = os.path.join(args.output, filename)
                    page.screenshot(path=filepath)

                    # Collect DOM info
                    dom_info = page.evaluate("""() => {
                        return {
                            inputs: document.querySelectorAll('input').length,
                            buttons: document.querySelectorAll('button').length,
                            forms: document.querySelectorAll('form').length,
                            links: document.querySelectorAll('a').length,
                            images: document.querySelectorAll('img').length,
                        };
                    }""")

                    result = {
                        "page": page_name,
                        "viewport": vp_name,
                        "screenshot": filepath,
                        "errors": len(errors),
                        "error_messages": errors[:5],
                        "network_requests": len(network),
                        "dom": dom_info,
                        "success": True,
                    }

                    status = "✅" if len(errors) == 0 else f"⚠️ {len(errors)} errors"
                    print(f"  {page_name:12} {vp_name:8} {status}")

                except Exception as e:
                    result = {
                        "page": page_name,
                        "viewport": vp_name,
                        "error": str(e),
                        "success": False,
                    }
                    print(f"  {page_name:12} {vp_name:8} ❌ {str(e)[:50]}")

                results.append(result)
                context.close()

        browser.close()

    # Save results
    results_file = os.path.join(args.output, f"results_{timestamp}.json")
    with open(results_file, "w") as f:
        json.dump(results, f, indent=2)

    # Summary
    total = len(results)
    success = sum(1 for r in results if r.get("success"))
    total_errors = sum(r.get("errors", 0) for r in results)

    print(f"
=== Summary ===")
    print(f"  Tests: {success}/{total} passed")
    print(f"  Errors: {total_errors}")
    print(f"  Results: {results_file}")
    print(f"  Screenshots: {args.output}/")

if __name__ == "__main__":
    main()
