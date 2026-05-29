# -*- coding: utf-8 -*-
"""
REACTOR ⇄ Blender Live Link — Empaquetador de Addon
=============================================================================
Este script empaqueta de forma automática los archivos de la extensión de Blender
a un archivo .zip listo para ser instalado desde el menú de preferencias de Blender.

Uso:
    python empaquetar_addon.py
=============================================================================
"""

import os
import zipfile

def empaquetar():
    # Obtener rutas absolutas
    current_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.abspath(os.path.join(current_dir, ".."))
    
    addon_dir = os.path.join(project_root, "reactor-blender-bridge", "blender_addon")
    zip_path = os.path.join(current_dir, "reactor_live_link.zip")
    
    print("\n\033[91m" + r"""
   ▄████████    ▄████████    ▄████████  ▄████████     ▄████████   ▄██████▄   ▄██████▄  
  ███    ███   ███    ███   ███    ███ ███    ███    ███    ███  ███    ███ ███    ███ 
  ███    █▀    ███    █▀    ███    █▀  ███    █▀     ███    █▀   ███    ███ ███    ███ 
 ▄███▄▄▄      ▄███▄▄▄       ███        ███           ███         ███    ███ ███    ███ 
▀▀███▀▀▀     ▀▀███▀▀▀     ▀███████████ ███         ▀███████████  ███    ███ ███    ███ 
  ███    █▄    ███    █▄           ███ ███    █▄            ███  ███    ███ ███    ███ 
  ███    ███   ███    ███    ▄█    ███ ███    ███     ▄█    ███  ███    ███ ███    ███ 
  ████████▀    ██████████   ▄████████▀ ▀████████▀   ▄████████▀    ▀██████▀   ▀██████▀  
    """ + "\033[0m")
    
    print("╔═══════════════════════════════════════════════════════════════╗")
    print("║          REACTOR Addon Packaging Tool for Blender             ║")
    print("╚═══════════════════════════════════════════════════════════════╝")
    print(f" -> Buscando archivos en: {addon_dir}")
    print(f" -> Guardando archivo ZIP en: {zip_path}")
    
    if not os.path.exists(addon_dir):
        print("\n\033[91m[ERROR]\033[0m No se encontró la carpeta 'blender_addon' en la ruta especificada.")
        print(f"Ruta intentada: {addon_dir}")
        return

    # Crear archivo zip
    archivos_agregados = 0
    with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(addon_dir):
            # Omitir directorios innecesarios (caches)
            if '__pycache__' in root or '.git' in root:
                continue
                
            for file in files:
                if file.endswith('.pyc') or file.endswith('.pyo'):
                    continue
                    
                file_path = os.path.join(root, file)
                # Queremos que la carpeta raíz dentro del ZIP sea 'reactor_live_link'
                rel_path = os.path.relpath(file_path, addon_dir)
                arcname = os.path.join("reactor_live_link", rel_path)
                
                zipf.write(file_path, arcname)
                print(f" [+] Añadido: {arcname}")
                archivos_agregados += 1
                
    print("\n\033[92m[ÉXITO]\033[0m Addon empaquetado de forma espectacular.")
    print(f" -> Se empaquetaron {archivos_agregados} archivos.")
    print(f" -> Archivo generado: {zip_path}")
    print("\nYa puedes abrir Blender y seleccionar este archivo ZIP para instalar el Addon.\n")

if __name__ == "__main__":
    empaquetar()
