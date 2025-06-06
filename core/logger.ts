export function log(message: string, ...args: unknown[]) {
  console.log(`[CodeOrbit] ${message}`, ...args);
}
