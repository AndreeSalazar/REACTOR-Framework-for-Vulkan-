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
import sys

def safe_print(text, color_code=None):
    """Imprime texto de forma segura controlando errores de codificación en Windows."""
    if color_code and sys.stdout.isatty():
        formatted = f"\033[{color_code}m{text}\033[0m"
    else:
        formatted = text
        
    try:
        print(formatted)
    except UnicodeEncodeError:
        # Reemplazar caracteres no imprimibles en la codificación actual
        fallback = text.encode(sys.stdout.encoding, errors='replace').decode(sys.stdout.encoding)
        print(fallback)

def empaquetar():
    # Obtener rutas absolutas
    current_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.abspath(os.path.join(current_dir, ".."))
    
    addon_dir = os.path.join(project_root, "reactor-blender-bridge", "blender_addon")
    zip_path = os.path.join(current_dir, "reactor_live_link.zip")
    
    safe_print("\n" + r"""
   REACTOR   REACTOR   REACTOR   REACTOR   REACTOR   REACTOR   REACTOR  
   _  _ ____ ____ ____ ___ ____ ____    ___  ____ _ ___  ____ ____ 
   |\/| |___ [__  [__   |  |___ |__/    |__] |__/ | |  \ | __ |___ 
   |  | |___ ___] ___]  |  |___ |  \    |__] |  \ | |__/ |__] |___ 
    """, "91")
    
    safe_print("=================================================================")
    safe_print("          REACTOR Addon Packaging Tool for Blender               ")
    safe_print("=================================================================")
    safe_print(f" -> Buscando archivos en: {addon_dir}")
    safe_print(f" -> Guardando archivo ZIP en: {zip_path}")
    
    if not os.path.exists(addon_dir):
        safe_print("\n[ERROR] No se encontró la carpeta 'blender_addon' en la ruta especificada.", "91")
        safe_print(f"Ruta intentada: {addon_dir}")
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
                safe_print(f" [+] Añadido: {arcname}")
                archivos_agregados += 1
                
    safe_print("\n[ÉXITO] Addon empaquetado de forma espectacular.", "92")
    safe_print(f" -> Se empaquetaron {archivos_agregados} archivos.")
    safe_print(f" -> Archivo generado: {zip_path}")
    safe_print("\nYa puedes abrir Blender y seleccionar este archivo ZIP para instalar el Addon.\n")

if __name__ == "__main__":
    empaquetar()
