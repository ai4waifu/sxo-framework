import { loadNative, MATLAB_DIALECT, type NativeExpression } from './native.js';

/** Values accepted wherever a MATLAB expression is expected. */
export type ExprInput = string | Expression;

/**
 * Opaque expression produced by a {@link Matlab} frontend.
 *
 * Methods are thin conveniences; engine IR stays in Rust `SxoFrontend`.
 */
export class Expression {
    readonly #inner: NativeExpression;
    readonly #frontend: Matlab;

    private constructor(frontend: Matlab, inner: NativeExpression) {
        this.#frontend = frontend;
        this.#inner = inner;
    }

    /** @internal */
    static fromNative(frontend: Matlab, inner: NativeExpression): Expression {
        return new Expression(frontend, inner);
    }

    /** Frontend that produced this expression. */
    get frontend(): Matlab {
        return this.#frontend;
    }

    /** Differentiate w.r.t. `varName` via the owning frontend. */
    d(varName: string): Expression {
        return this.#frontend.d(this, varName);
    }

    /** Simplify via the owning frontend. */
    simplify(): Expression {
        return this.#frontend.simplify(this);
    }

    /** Same as {@link toMatlab}. */
    toString(): string {
        return this.toMatlab();
    }

    /** Render as MATLAB text. */
    toMatlab(): string {
        return this.#inner.toMatlab();
    }

    /** Structural equality. */
    isEqual(other: Expression): boolean {
        return this.#inner.isEqual(other.#inner);
    }

    /** @internal */
    get native(): NativeExpression {
        return this.#inner;
    }
}

export type MatlabOptions = {
    /**
     * When true (default), {@link Matlab.evaluate} runs simplify after evaluate.
     * {@link Matlab.parse} always returns the structural parse result.
     */
    autoSimplify?: boolean;
};

/**
 * MATLAB **frontend** — the public entry of `@sxo/matlab`.
 *
 * Feed MATLAB text (or expressions); the sole CAS kernel is Rust `SxoFrontend`
 * (engine IR `Term`). Parsing goes through upstream `oak-matlab` (no handmade dialect parser).
 */
export class Matlab {
    readonly #autoSimplify: boolean;

    private constructor(options: MatlabOptions = {}) {
        this.#autoSimplify = options.autoSimplify ?? true;
    }

    /** Create a MATLAB frontend instance. */
    static create(options?: MatlabOptions): Matlab {
        return new Matlab(options);
    }

    /** Native / package version. */
    version(): string {
        return loadNative().version();
    }

    /** Render `input` as MATLAB text. */
    toMatlab(input: ExprInput): string {
        if (input instanceof Expression) return input.toMatlab();
        return input;
    }

    /**
     * Parse MATLAB text (no evaluate).
     */
    parse(input: ExprInput): Expression {
        if (input instanceof Expression) return input;
        return Expression.fromNative(this, loadNative().expression(input, MATLAB_DIALECT));
    }

    /**
     * Parse, evaluate through the kernel, then optionally simplify
     * (`autoSimplify`, default true).
     */
    evaluate(input: ExprInput): Expression {
        const parsed = this.parse(input);
        const evaluated = Expression.fromNative(this, parsed.native.evaluate());
        return this.#autoSimplify ? this.simplify(evaluated) : evaluated;
    }

    /**
     * Tagged template → {@link evaluate}.
     */
    matlab(strings: TemplateStringsArray, ...values: ExprInput[]): Expression {
        let out = '';
        for (let i = 0; i < strings.length; i++) {
            out += strings[i];
            if (i < values.length) out += this.toMatlab(values[i]!);
        }
        return this.evaluate(out);
    }

    /** Differentiate (MATLAB `diff` / engine `D`). */
    d(expr: ExprInput, varName: string): Expression {
        if (expr instanceof Expression) {
            return Expression.fromNative(this, expr.native.d(varName));
        }
        return Expression.fromNative(this, loadNative().d(expr, varName, MATLAB_DIALECT));
    }

    /** Algebraic simplify. */
    simplify(expr: ExprInput): Expression {
        if (expr instanceof Expression) {
            return Expression.fromNative(this, expr.native.simplify());
        }
        return Expression.fromNative(this, loadNative().simplify(expr, MATLAB_DIALECT));
    }
}

/** Shared default MATLAB frontend (stateless S0). */
export const matlab = Matlab.create();

/** @deprecated Prefer {@link Matlab.create} / {@link matlab}. */
export function version(): string {
    return matlab.version();
}

/** Alias kept for older placeholder naming. */
export { Matlab as MatlabFrontend };
