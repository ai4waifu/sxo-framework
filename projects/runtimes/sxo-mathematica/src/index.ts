import { loadNative, MATH_DIALECT, type NativeExpression } from './native.js';

/** Values accepted wherever a Mathematica expression is expected. */
export type ExprInput = string | Expression;

/**
 * Opaque expression produced by a {@link Mathematica} frontend.
 *
 * Methods are thin conveniences; the package contract is frontend-centric
 * (`Mathematica.parse` / `evaluate` / `d` / `simplify`). Engine IR stays in Rust.
 */
export class Expression {
    readonly #inner: NativeExpression;
    readonly #frontend: Mathematica;

    private constructor(frontend: Mathematica, inner: NativeExpression) {
        this.#frontend = frontend;
        this.#inner = inner;
    }

    /** @internal */
    static fromNative(frontend: Mathematica, inner: NativeExpression): Expression {
        return new Expression(frontend, inner);
    }

    /** Frontend that produced this expression. */
    get frontend(): Mathematica {
        return this.#frontend;
    }

    /** @deprecated Prefer {@link frontend}. */
    get engine(): Mathematica {
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

    /** Same as {@link toWolfram}. */
    toString(): string {
        return this.toWolfram();
    }

    /** Render as Mathematica / Wolfram text. */
    toWolfram(): string {
        return this.#inner.toWolfram();
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

export type MathematicaOptions = {
    /**
     * When true (default), {@link Mathematica.evaluate} runs simplify after evaluate.
     * {@link Mathematica.parse} always returns the structural parse result.
     */
    autoSimplify?: boolean;
};

/**
 * Mathematica / Wolfram **frontend** — the public entry of `@sxo/mathematica`.
 *
 * Feed Wolfram text (or expressions); the sole CAS kernel is Rust `SxoFrontend`
 * (engine IR `Term`). This package must **not** be described as the kernel.
 *
 * @example
 * const mma = Mathematica.create();
 * mma.evaluate('Simplify[Sin[x]^2 + Cos[x]^2]').toWolfram(); // "1"
 * mma.d('x^3', 'x').toWolfram();
 */
export class Mathematica {
    readonly #autoSimplify: boolean;

    private constructor(options: MathematicaOptions = {}) {
        this.#autoSimplify = options.autoSimplify ?? true;
    }

    /** Create a Mathematica frontend instance. */
    static create(options?: MathematicaOptions): Mathematica {
        return new Mathematica(options);
    }

    /** Native / package version. */
    version(): string {
        return loadNative().version();
    }

    /** Render `input` as Wolfram text. */
    toWolfram(input: ExprInput): string {
        if (input instanceof Expression) return input.toWolfram();
        return input;
    }

    /**
     * Parse Wolfram / Mathematica text (no evaluate).
     */
    parse(input: ExprInput): Expression {
        if (input instanceof Expression) return input;
        return Expression.fromNative(this, loadNative().expression(input, MATH_DIALECT));
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
     *
     * @example
     * mma.wolfram`D[${mma.parse('x')}^3, x]`
     */
    wolfram(strings: TemplateStringsArray, ...values: ExprInput[]): Expression {
        let out = '';
        for (let i = 0; i < strings.length; i++) {
            out += strings[i];
            if (i < values.length) out += this.toWolfram(values[i]!);
        }
        return this.evaluate(out);
    }

    /** Differentiate (Mathematica `D`). */
    d(expr: ExprInput, varName: string): Expression {
        if (expr instanceof Expression) {
            return Expression.fromNative(this, expr.native.d(varName));
        }
        return Expression.fromNative(this, loadNative().d(expr, varName, MATH_DIALECT));
    }

    /** Algebraic simplify (Mathematica `Simplify`). */
    simplify(expr: ExprInput): Expression {
        if (expr instanceof Expression) {
            return Expression.fromNative(this, expr.native.simplify());
        }
        return Expression.fromNative(this, loadNative().simplify(expr, MATH_DIALECT));
    }
}

/** Shared default Mathematica frontend (stateless S0). */
export const mathematica = Mathematica.create();

/** @deprecated Prefer {@link Mathematica.create} / {@link mathematica}. */
export function version(): string {
    return mathematica.version();
}
