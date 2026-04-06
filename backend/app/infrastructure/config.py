"""Global application configuration.

Defines project and storage base paths, and operational constants
such as the maximum chunk size for file uploads.
"""

from pathlib import Path

# Project root directory (three levels above this file)
BASE_DIR = Path(__file__).resolve().parent.parent.parent

# Root directory for all file storage
STORAGE_DIR = BASE_DIR / "storage"

# Directory where uploaded original files are saved
INPUT_DIR = STORAGE_DIR / "input"

# Directory where generated compressed archives are saved
OUTPUT_DIR = STORAGE_DIR / "output"

# Maximum number of bytes read per chunk when streaming an uploaded file (1 MB)
MAX_UPLOAD_CHUNK_SIZE = 1024 * 1024
