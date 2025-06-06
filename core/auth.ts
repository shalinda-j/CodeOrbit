/** Basic authentication/authorization stub for CodeOrbit */
export function checkFileAccess(path: string): boolean {
  // Disallow absolute paths and parent directory traversal
  if (path.startsWith('..') || path.startsWith('/')) {
    return false;
  }
  return true;
}
