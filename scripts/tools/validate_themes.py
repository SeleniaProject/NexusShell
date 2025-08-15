#!/usr/bin/env python3
"""
NexusShell テーマファイル基本バリデータ
依存関係なしでJSONとして妥当性を検証
"""

import json
import os
import re
import sys
from pathlib import Path

def validate_hex_color(color_str):
    """Validate hex color code"""
    if not isinstance(color_str, str):
        return False
    return re.match(r'^#[0-9a-fA-F]{6}$', color_str) is not None

def validate_semver(version):
    """Validate semantic version"""
    if not isinstance(version, str):
        return False
    return re.match(r'^\d+\.\d+\.\d+$', version) is not None

def validate_theme_file(theme_path):
    errors = []
    warnings = []
    try:
        with open(theme_path, 'r', encoding='utf-8') as f:
            theme_data = json.load(f)
    except json.JSONDecodeError as e:
        return [f"JSON解析エラー: {e}"], []
    except Exception as e:
        return [f"ファイル読み込みエラー: {e}"], []

    required_fields = ['name', 'version', 'author', 'colors']
    for field in required_fields:
        if field not in theme_data:
            errors.append(f"必須フィールド '{field}' が見つかりません")

    if 'name' in theme_data:
        name = theme_data['name']
        if not isinstance(name, str) or not name.strip():
            errors.append("nameフィールドは空でない文字列である必要があります")
        elif not re.match(r'^[a-zA-Z0-9_-]+$', name):
            warnings.append(f"nameフィールドに推奨されない文字が含まれています: {name}")

    if 'version' in theme_data:
        version = theme_data['version']
        if not validate_semver(version):
            errors.append(f"無効なバージョン形式: {version} (x.y.z形式が必要)")

    if 'author' in theme_data:
        author = theme_data['author']
        if not isinstance(author, str) or not author.strip():
            errors.append("authorフィールドは空でない文字列である必要があります")

    if 'colors' in theme_data:
        colors = theme_data['colors']
        if not isinstance(colors, dict):
            errors.append("colorsフィールドはオブジェクトである必要があります")
        else:
            basic_colors = ['primary', 'background', 'foreground']
            for color in basic_colors:
                if color not in colors:
                    warnings.append(f"推奨色 '{color}' が見つかりません")
                elif not validate_hex_color(colors[color]):
                    errors.append(f"無効な色形式 '{color}': {colors[color]}")
            for color_name, color_value in colors.items():
                if not validate_hex_color(color_value):
                    errors.append(f"無効な色形式 '{color_name}': {color_value}")

    return errors, warnings

def main():
    print("🎨 NexusShell テーマバリデータ (Python版)")
    print("=========================================")

    themes_dir = Path("assets/themes")
    if not themes_dir.exists():
        print(f"❌ themesディレクトリが見つかりません: {themes_dir}")
        return 1

    theme_files = [p for p in themes_dir.glob("*.json") if p.name != "theme-schema.json"]
    theme_files.sort()

    print(f"検証中のテーマ数: {len(theme_files)}\n")

    total_themes = len(theme_files)
    valid_themes = 0
    total_warnings = 0
    total_errors = 0

    for theme_file in theme_files:
        theme_name = theme_file.stem
        print(f"📄 {theme_name} ... ", end="", flush=True)
        errors, warnings = validate_theme_file(theme_file)
        if errors:
            print(f"❌ 無効 ({len(errors)}個のエラー)")
            for error in errors:
                print(f"    ❌ {error}")
            total_errors += len(errors)
        elif warnings:
            print(f"⚠️  有効 ({len(warnings)}個の警告)")
            for warning in warnings:
                print(f"    ⚠️  {warning}")
            valid_themes += 1
            total_warnings += len(warnings)
        else:
            print("✅ 完全に有効")
            valid_themes += 1

    print("\n=== 検証結果サマリー ===")
    print(f"総テーマ数: {total_themes}")
    print(f"有効テーマ数: {valid_themes}")
    print(f"無効テーマ数: {total_themes - valid_themes}")
    print(f"総警告数: {total_warnings}")
    print(f"総エラー数: {total_errors}")

    success_rate = (valid_themes / total_themes * 100) if total_themes > 0 else 0
    print(f"成功率: {success_rate:.1f}%")

    if valid_themes == total_themes:
        print("🎉 すべてのテーマが検証に合格しました！")
        return 0
    elif success_rate >= 80.0:
        print("✅ 多くのテーマが検証に合格しました")
        return 0
    else:
        print("⚠️  いくつかのテーマに問題があります")
        return 1

if __name__ == "__main__":
    sys.exit(main())


