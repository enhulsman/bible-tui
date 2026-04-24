/* tslint:disable */
/* eslint-disable */

/**
 * Debug API exposed to browser console for terminal inspection.
 */
export class TerminalDebugApi {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns the full atlas glyph-to-symbol mapping as a JavaScript array.
     *
     * # Panics
     *
     * Panics if setting properties on the JavaScript objects fails.
     */
    getAtlasLookup(): Array<any>;
    /**
     * Returns the base glyph ID for a given symbol, or null if not found.
     */
    getBaseGlyphId(symbol: string): number | undefined;
    /**
     * Returns the canvas size in pixels as an object with `width` and `height` fields.
     *
     * # Panics
     *
     * Panics if setting properties on the JavaScript object fails.
     */
    getCanvasSize(): any;
    /**
     * Returns the cell size in pixels as an object with `width` and `height` fields.
     *
     * # Panics
     *
     * Panics if setting properties on the JavaScript object fails.
     */
    getCellSize(): any;
    /**
     * Returns the number of glyphs available in the font atlas.
     */
    getGlyphCount(): number;
    /**
     * Returns an array of glyphs that were requested but not found in the font atlas.
     */
    getMissingGlyphs(): Array<any>;
    /**
     * Returns the symbol for a given glyph ID, or null if not found.
     */
    getSymbol(glyph_id: number): string | undefined;
    /**
     * Returns the terminal size in cells as an object with `cols` and `rows` fields.
     *
     * # Panics
     *
     * Panics if setting properties on the JavaScript object fails.
     */
    getTerminalSize(): any;
}

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_terminaldebugapi_free: (a: number, b: number) => void;
    readonly terminaldebugapi_getAtlasLookup: (a: number) => any;
    readonly terminaldebugapi_getBaseGlyphId: (a: number, b: number, c: number) => number;
    readonly terminaldebugapi_getCanvasSize: (a: number) => any;
    readonly terminaldebugapi_getCellSize: (a: number) => any;
    readonly terminaldebugapi_getGlyphCount: (a: number) => number;
    readonly terminaldebugapi_getMissingGlyphs: (a: number) => any;
    readonly terminaldebugapi_getSymbol: (a: number, b: number) => [number, number];
    readonly terminaldebugapi_getTerminalSize: (a: number) => any;
    readonly main: () => void;
    readonly wasm_bindgen__closure__destroy__hc86fc32e45991951: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h2c28a16e26889b7e: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hefb32cd09a9e0f2e: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h1d783d8bb5975cab: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h0cc9aff781414eb4: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
