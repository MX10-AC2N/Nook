#!/usr/bin/env python3
"""
Nook Visual Tester — Prend des screenshots de Nook à différentes résolutions.
Usage: python3 scripts/nook-screenshot.py [--url URL] [--cookie COOKIE]
"""

import argparse
import os
from datetime import datetime

def main():
    parser = argparse.ArgumentParser(description="Screenshot Nook at different viewports")
    parser.add_argument("--url", default="http://192.168.1.192:6300", help="Nook URL")
    parser.add_argument("--cookie", default="", help="auth_token cookie")
    parser.add_argument("--output", default="/tmp/nook-screenshots", help="Output directory")
    args = parser.parse_args()

    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("Installing Playwright...")
        os.system("pip install playwright && playwright install chromium")
        from playwright.sync_api import sync_playwright

    os.makedirs(args.output, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    viewports = [
        ("desktop", 1440, 900),
        ("tablet", 768, 1024),
        ("mobile", 375, 812),
    ]

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        
        for name, width, height in viewports:
            context = browser.new_context(
                viewport={"width": width, "height": height},
                device_scale_factor=2 if name == "mobile" else 1,
            )
            
            if args.cookie:
                context.add_cookies([{
                    "name": "auth_token",
                    "value": args.cookie.split("=")[1] if "=" in args.cookie else args.cookie,
                    "domain": "192.168.1.192",
                    "path": "/",
                }])
            
            page = context.new_page()
            page.goto(args.url, wait_until="networkidle", timeout=15000)
            page.wait_for_timeout(2000)
            
            path = f"{args.output}/{timestamp}_{name}_{width}x{height}.png"
            page.screenshot(path=path, full_page=False)
            print(f"  {name}: {path}")
            
            context.close()
        
        browser.close()
    
    print(f"\nScreenshots saved to {args.output}/")

if __name__ == "__main__":
    main()
