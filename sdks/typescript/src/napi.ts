/**
 * Universal bindings loader
 * 
 * This module handles loading the appropriate native module:
 * - NAPI for Node.js
 * - WASM for browsers
 */

import { T2ZError } from './types';

let nativeModule: any = null;
let moduleType: 'napi' | 'wasm' | null = null;

/**
 * Detects the current environment
 */
function detectEnvironment(): 'node' | 'browser' {
  // Check if we're in a browser environment
  if (typeof window !== 'undefined' && typeof window.document !== 'undefined') {
    return 'browser';
  }
  
  // Check if we're in Node.js
  if (typeof process !== 'undefined' && process.versions != null && process.versions.node != null) {
    return 'node';
  }
  
  // Default to browser for other environments (e.g., workers)
  return 'browser';
}

/**
 * Loads the NAPI module for Node.js
 */
function loadNapiModule(): any {
  try {
    // Try to load from the build output
    const module = require('../../../wrapper/target/release/pczt_wrapper.node');
    moduleType = 'napi';
    return module;
  } catch (error: any) {
    throw new T2ZError(
      `Failed to load NAPI module for Node.js. Make sure to build it first with: npm run build:napi\nError: ${error.message}`,
      'NAPI_LOAD_ERROR'
    );
  }
}

/**
 * Loads the WASM module for browsers
 */
async function loadWasmModule(): Promise<any> {
  try {
    // Dynamic import for WASM module
    const wasmModule = await import('../../../wrapper/pkg/pczt_wrapper');
    await wasmModule.default(); // Initialize WASM
    moduleType = 'wasm';
    return wasmModule;
  } catch (error: any) {
    throw new T2ZError(
      `Failed to load WASM module for browser. Make sure to build it first with: npm run build:wasm\nError: ${error.message}`,
      'WASM_LOAD_ERROR'
    );
  }
}

/**
 * Loads the appropriate native module based on environment
 * 
 * @throws {T2ZError} If the native module cannot be loaded
 */
export async function loadNativeModule(): Promise<any> {
  if (nativeModule) {
    return nativeModule;
  }

  const env = detectEnvironment();

  if (env === 'node') {
    nativeModule = loadNapiModule();
  } else {
    nativeModule = await loadWasmModule();
  }

  return nativeModule;
}

/**
 * Gets the native module, loading it if necessary
 */
export async function getNativeModule(): Promise<any> {
  return await loadNativeModule();
}

/**
 * Gets the current module type
 */
export function getModuleType(): 'napi' | 'wasm' | null {
  return moduleType;
}

/**
 * Checks if the module is loaded
 */
export function isModuleLoaded(): boolean {
  return nativeModule !== null;
}

