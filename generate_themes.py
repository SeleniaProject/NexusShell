#!/usr/bin/env python3
"""
Generate 20 NexusShell themes with diverse color schemes
"""
import json
import os
from pathlib import Path

# Theme definitions with colors and descriptions
THEMES = [
    {
        "name": "nxsh-solarized-dark",
        "description": "Dark variant of Solarized color scheme",
        "colors": {
            "primary": "#268bd2", "secondary": "#657b83", "accent": "#cb4b16",
            "background": "#002b36", "foreground": "#839496", "error": "#dc322f",
            "warning": "#b58900", "success": "#859900", "info": "#268bd2",
            "muted": "#657b83", "highlight": "#d33682", "border": "#073642"
        }
    },
    {
        "name": "nxsh-solarized-light", 
        "description": "Light variant of Solarized color scheme",
        "colors": {
            "primary": "#268bd2", "secondary": "#93a1a1", "accent": "#cb4b16",
            "background": "#fdf6e3", "foreground": "#657b83", "error": "#dc322f",
            "warning": "#b58900", "success": "#859900", "info": "#268bd2",
            "muted": "#93a1a1", "highlight": "#d33682", "border": "#eee8d5"
        }
    },
    {
        "name": "nxsh-gruvbox-dark",
        "description": "Warm and cozy dark theme inspired by Gruvbox",
        "colors": {
            "primary": "#83a598", "secondary": "#928374", "accent": "#fe8019",
            "background": "#282828", "foreground": "#ebdbb2", "error": "#fb4934",
            "warning": "#fabd2f", "success": "#b8bb26", "info": "#83a598",
            "muted": "#928374", "highlight": "#d3869b", "border": "#3c3836"
        }
    },
    {
        "name": "nxsh-gruvbox-light",
        "description": "Warm and cozy light theme inspired by Gruvbox",
        "colors": {
            "primary": "#076678", "secondary": "#7c6f64", "accent": "#af3a03",
            "background": "#f9f5d7", "foreground": "#3c3836", "error": "#cc241d",
            "warning": "#b57614", "success": "#79740e", "info": "#076678",
            "muted": "#7c6f64", "highlight": "#8f3f71", "border": "#ebdbb2"
        }
    },
    {
        "name": "nxsh-cyberpunk",
        "description": "Futuristic neon cyberpunk theme",
        "colors": {
            "primary": "#ff2a6d", "secondary": "#6930c3", "accent": "#01fbfe",
            "background": "#0c0c0c", "foreground": "#e6e6e6", "error": "#ff2a6d",
            "warning": "#ffaa00", "success": "#00ff9f", "info": "#01fbfe",
            "muted": "#6930c3", "highlight": "#ffc600", "border": "#1a1a1a"
        }
    },
    {
        "name": "nxsh-forest",
        "description": "Nature-inspired green theme",
        "colors": {
            "primary": "#228b22", "secondary": "#556b2f", "accent": "#daa520",
            "background": "#0f1419", "foreground": "#f0f0f0", "error": "#b22222",
            "warning": "#daa520", "success": "#32cd32", "info": "#228b22",
            "muted": "#556b2f", "highlight": "#9acd32", "border": "#2f4f4f"
        }
    },
    {
        "name": "nxsh-ocean",
        "description": "Deep blue oceanic theme",
        "colors": {
            "primary": "#1e90ff", "secondary": "#4682b4", "accent": "#40e0d0",
            "background": "#001122", "foreground": "#f0f8ff", "error": "#dc143c",
            "warning": "#ffa500", "success": "#32cd32", "info": "#1e90ff",
            "muted": "#4682b4", "highlight": "#40e0d0", "border": "#2f4f4f"
        }
    },
    {
        "name": "nxsh-sunset",
        "description": "Warm sunset colors",
        "colors": {
            "primary": "#ff6b35", "secondary": "#7209b7", "accent": "#ffd23f",
            "background": "#2d1b69", "foreground": "#f0f0f0", "error": "#e74c3c",
            "warning": "#f39c12", "success": "#2ecc71", "info": "#3498db",
            "muted": "#7209b7", "highlight": "#ffd23f", "border": "#553c9a"
        }
    },
    {
        "name": "nxsh-minimalist",
        "description": "Clean minimalist monochrome theme",
        "colors": {
            "primary": "#333333", "secondary": "#666666", "accent": "#999999",
            "background": "#ffffff", "foreground": "#1a1a1a", "error": "#cc0000",
            "warning": "#ff8800", "success": "#008800", "info": "#0066cc",
            "muted": "#999999", "highlight": "#cccccc", "border": "#e0e0e0"
        }
    },
    {
        "name": "nxsh-high-contrast",
        "description": "High contrast theme for accessibility",
        "colors": {
            "primary": "#ffffff", "secondary": "#cccccc", "accent": "#ffff00",
            "background": "#000000", "foreground": "#ffffff", "error": "#ff0000",
            "warning": "#ffff00", "success": "#00ff00", "info": "#00ffff",
            "muted": "#cccccc", "highlight": "#ffff00", "border": "#ffffff"
        }
    },
    {
        "name": "nxsh-pastel",
        "description": "Soft pastel colors for gentle viewing",
        "colors": {
            "primary": "#dda0dd", "secondary": "#d8bfd8", "accent": "#ffd1dc",
            "background": "#f5f5f5", "foreground": "#2f2f2f", "error": "#ff6b6b",
            "warning": "#ffa500", "success": "#98fb98", "info": "#87ceeb",
            "muted": "#d8bfd8", "highlight": "#ffd1dc", "border": "#e0e0e0"
        }
    },
    {
        "name": "nxsh-matrix",
        "description": "Green-on-black matrix style",
        "colors": {
            "primary": "#00ff41", "secondary": "#008f11", "accent": "#008000",
            "background": "#000000", "foreground": "#00ff41", "error": "#ff0000",
            "warning": "#ffff00", "success": "#00ff00", "info": "#00ff41",
            "muted": "#008f11", "highlight": "#39ff14", "border": "#003300"
        }
    },
    {
        "name": "nxsh-retro",
        "description": "80s retro computer terminal colors",
        "colors": {
            "primary": "#00ffff", "secondary": "#ff00ff", "accent": "#ffff00",
            "background": "#000080", "foreground": "#00ffff", "error": "#ff0000",
            "warning": "#ffff00", "success": "#00ff00", "info": "#00ffff",
            "muted": "#ff00ff", "highlight": "#ffff00", "border": "#800080"
        }
    },
    {
        "name": "nxsh-autumn",
        "description": "Warm autumn colors",
        "colors": {
            "primary": "#d2691e", "secondary": "#8b4513", "accent": "#ffd700",
            "background": "#2f1b14", "foreground": "#f5deb3", "error": "#b22222",
            "warning": "#daa520", "success": "#9acd32", "info": "#d2691e",
            "muted": "#8b4513", "highlight": "#ffd700", "border": "#654321"
        }
    },
    {
        "name": "nxsh-winter",
        "description": "Cool winter theme with icy blues",
        "colors": {
            "primary": "#4169e1", "secondary": "#6495ed", "accent": "#87ceeb",
            "background": "#0f1419", "foreground": "#f0f8ff", "error": "#b22222",
            "warning": "#daa520", "success": "#32cd32", "info": "#4169e1",
            "muted": "#6495ed", "highlight": "#87ceeb", "border": "#2f4f4f"
        }
    }
]

def create_theme(theme_data):
    """Create a theme JSON object"""
    return {
        "name": theme_data["name"],
        "version": "1.0.0",
        "author": "NexusShell Team",
        "description": theme_data["description"],
        "colors": theme_data["colors"],
        "styles": {
            "prompt": {"foreground": "Blue", "bold": True},
            "command": {"foreground": "White"},
            "error": {"foreground": "Red", "bold": True},
            "warning": {"foreground": "Yellow"},
            "success": {"foreground": "Green", "bold": True}
        }
    }

def main():
    theme_dir = Path("c:/Users/Aqua/Programming/SeleniaProject/NexusShell/assets/themes")
    
    for theme_data in THEMES:
        theme = create_theme(theme_data)
        filename = f"{theme_data['name']}.json"
        filepath = theme_dir / filename
        
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(theme, f, indent=2, ensure_ascii=False)
        
        print(f"Created theme: {filename}")
    
    print(f"\\nTotal themes created: {len(THEMES)}")
    print("All theme files have been generated in assets/themes/")

if __name__ == "__main__":
    main()
