# -*- coding: utf-8 -*-
"""REACTOR Live Link — Empaquetador del Addon para Blender.

Empaqueta la extensión de Blender a un .zip listo para instalar
desde Edit → Preferences → Add-ons → Install…

Uso:
    python python/scripts/empaquetar_addon.py
"""

import os
import sys
import zipfile


def empaquetar():
    current_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.abspath(os.path.join(current_dir, "..", ".."))

    addon_dir = os.path.join(project_root, "blender_addon")
    zip_path = os.path.join(current_dir, "..", "reactor_live_link.zip")

    if not os.path.exists(addon_dir):
        print(f"[ERROR] No se encontró 'blender_addon' en: {addon_dir}")
        sys.exit(1)

    count = 0
    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for root, _dirs, files in os.walk(addon_dir):
            if "__pycache__" in root:
                continue
            for file in files:
                if file.endswith((".pyc", ".pyo")):
                    continue
                src = os.path.join(root, file)
                rel = os.path.relpath(src, addon_dir)
                arcname = os.path.join("reactor_live_link", rel)
                zf.write(src, arcname)
                print(f" [+] {arcname}")
                count += 1

    print(f"\n[OK] {count} archivos empaquetados en {zip_path}")


if __name__ == "__main__":
    empaquetar()
