import {
  type OpenDialogOptions,
  open,
  type SaveDialogOptions,
  save,
} from '@tauri-apps/plugin-dialog';

const LAST_DIR_KEY = 'anafis_last_directory';

/**
 * Gets the last used directory path from local storage
 */
function getLastDirectory(): string | undefined {
  return localStorage.getItem(LAST_DIR_KEY) || undefined;
}

/**
 * Saves a directory path to local storage
 */
function setLastDirectory(path: string | null | undefined): void {
  if (!path) return;

  try {
    // Basic extraction of the directory from a full file path
    // This works on both Windows (\) and Unix (/) paths
    const isWindows = path.includes('\\');
    const separator = isWindows ? '\\' : '/';

    let dirPath = path;
    if (path.includes(separator)) {
      const parts = path.split(separator);
      parts.pop(); // Remove filename
      dirPath = parts.join(separator);
    }

    localStorage.setItem(LAST_DIR_KEY, dirPath);
  } catch (e) {
    console.error('Failed to save last directory:', e);
  }
}

/**
 * Wrapper for Tauri's open dialog that remembers the last used directory.
 */
export async function openWithMemory(
  options: OpenDialogOptions = {}
): Promise<string | string[] | null> {
  const defaultPath = options.defaultPath || getLastDirectory();

  const openOptions: OpenDialogOptions = { ...options };
  if (defaultPath) {
    openOptions.defaultPath = defaultPath;
  }

  const result = await open(openOptions);

  if (result) {
    // If multiple files were selected, use the directory of the first one
    const firstPath = Array.isArray(result) ? result[0] : result;
    setLastDirectory(firstPath);
  }

  return result;
}

/**
 * Wrapper for Tauri's save dialog that remembers the last used directory.
 */
export async function saveWithMemory(
  options: SaveDialogOptions = {}
): Promise<string | null> {
  let defaultPath = options.defaultPath;

  // If a default path is provided (like "export.csv"), we append it to the last directory
  // so the dialog opens in the right folder but keeps the suggested filename
  const lastDir = getLastDirectory();
  if (lastDir) {
    if (
      defaultPath &&
      !defaultPath.includes('/') &&
      !defaultPath.includes('\\')
    ) {
      // It's just a filename, append it to the last directory
      const separator = lastDir.includes('\\') ? '\\' : '/';
      defaultPath = `${lastDir}${separator}${defaultPath}`;
    } else if (!defaultPath) {
      defaultPath = lastDir;
    }
  }

  const saveOptions: SaveDialogOptions = { ...options };
  if (defaultPath) {
    saveOptions.defaultPath = defaultPath;
  }

  const result = await save(saveOptions);

  if (result) {
    setLastDirectory(result);
  }

  return result;
}
