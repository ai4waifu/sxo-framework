/* tslint:disable */
/* eslint-disable */

/**
 * Opaque expression handle backed by engine [`Term`].
 */
export class Expression {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Differentiate with respect to `var`.
     */
    d(_var: string): Expression;
    /**
     * Evaluate (canonical rewrite) this expression.
     */
    evaluate(): Expression;
    /**
     * Structural equality.
     */
    isEqual(other: Expression): boolean;
    /**
     * Parse `input` with optional dialect.
     */
    constructor(input: string, dialect?: string | null);
    /**
     * Simplify via `SxoFrontend`.
     */
    simplify(): Expression;
    /**
     * Render as MATLAB text.
     */
    toMatlab(): string;
    /**
     * Render as string in the expression's dialect.
     */
    toString(): string;
    /**
     * Render as Mathematica / Wolfram text.
     */
    toWolfram(): string;
}

/**
 * Top-level `d`.
 */
export function d(input: string, _var: string, dialect?: string | null): Expression;

/**
 * Top-level `expression` — parse only (no evaluate).
 */
export function expression(input: string, dialect?: string | null): Expression;

/**
 * Top-level `simplify`.
 */
export function simplify(input: string, dialect?: string | null): Expression;

/**
 * Return the SXO engine version string.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_expression_free: (a: number, b: number) => void;
    readonly d: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly expression: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly expression_d: (a: number, b: number, c: number) => number;
    readonly expression_evaluate: (a: number) => number;
    readonly expression_isEqual: (a: number, b: number) => number;
    readonly expression_new: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly expression_simplify: (a: number) => number;
    readonly expression_toMatlab: (a: number) => [number, number];
    readonly expression_toString: (a: number) => [number, number];
    readonly expression_toWolfram: (a: number) => [number, number];
    readonly simplify: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly version: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
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
