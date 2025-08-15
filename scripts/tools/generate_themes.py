#!/usr/bin/env python3
"""
Generate NexusShell themes with diverse color schemes into assets/themes
"""
import json
from pathlib import Path

THEMES = [
    {"name": "nxsh-solarized-dark", "description": "Dark variant of Solarized", "colors": {"primary": "#268bd2","secondary":"#657b83","accent":"#cb4b16","background":"#002b36","foreground":"#839496","error":"#dc322f","warning":"#b58900","success":"#859900","info":"#268bd2","muted":"#657b83","highlight":"#d33682","border":"#073642"}},
]

def create_theme(theme_data):
    return {
        "name": theme_data["name"],
        "version": "1.0.0",
        "author": "NexusShell Team",
        "description": theme_data["description"],
        "colors": theme_data["colors"],
    }

def main():
    theme_dir = Path("assets/themes")
    theme_dir.mkdir(parents=True, exist_ok=True)
    for theme_data in THEMES:
        theme = create_theme(theme_data)
        filepath = theme_dir / f"{theme_data['name']}.json"
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(theme, f, indent=2, ensure_ascii=False)
        print(f"Created theme: {filepath.name}")
    print("Done.")

if __name__ == "__main__":
    main()


