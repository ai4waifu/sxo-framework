import type {
    CaseKind,
    FeatureBackend,
    FeatureCase,
    FeatureEffect,
    FeatureEntry,
    FeatureHost,
    FeatureMatrix,
    FeatureStatus,
} from './types.js';

/** Optional fields shared by case constructors. */
export type FeatureCaseOptions = {
    notes?: string;
    host?: FeatureHost;
    backend?: FeatureBackend;
    device?: string;
};

export type GapCaseOptions = FeatureCaseOptions & {
    expected?: string;
};

export type NegativeCaseOptions = FeatureCaseOptions & {
    forbidden?: string;
    expected?: string;
};

export type PlotCaseOptions = FeatureCaseOptions & {
    expected?: string;
};

function caseBase(
    id: string,
    kind: CaseKind,
    input: string,
    extra: {
        expected?: string;
        forbidden?: string;
        notes?: string;
        host?: FeatureHost;
        backend?: FeatureBackend;
        device?: string;
    } = {},
): FeatureCase {
    const out: FeatureCase = { id, kind, input };
    if (extra.expected !== undefined) out.expected = extra.expected;
    if (extra.forbidden !== undefined) out.forbidden = extra.forbidden;
    if (extra.notes !== undefined) out.notes = extra.notes;
    if (extra.host !== undefined) out.host = extra.host;
    if (extra.backend !== undefined) out.backend = extra.backend;
    if (extra.device !== undefined) out.device = extra.device;
    return out;
}

/** `eval` case: evaluate → render equals `expected`. */
export function evalCase(id: string, input: string, expected: string, opts: FeatureCaseOptions = {}): FeatureCase {
    return caseBase(id, 'eval', input, { expected, ...opts });
}

/** `parse` case: parse → render contains `expected`. */
export function parseCase(id: string, input: string, expected: string, opts: FeatureCaseOptions = {}): FeatureCase {
    return caseBase(id, 'parse', input, { expected, ...opts });
}

/** `roundtrip` case: parse/render roundtrip. */
export function roundtripCase(id: string, input: string, expected: string, opts: FeatureCaseOptions = {}): FeatureCase {
    return caseBase(id, 'roundtrip', input, { expected, ...opts });
}

/** `plot` case: structural SVG checks (+ optional substring). */
export function plotCase(id: string, input: string, opts: PlotCaseOptions = {}): FeatureCase {
    const { expected, ...rest } = opts;
    return caseBase(id, 'plot', input, { expected, ...rest });
}

/** `negative` case: must not silently yield a wrong success value. */
export function negativeCase(id: string, input: string, opts: NegativeCaseOptions = {}): FeatureCase {
    return caseBase(id, 'negative', input, opts);
}

/** `gap` case: contract hole (`it.todo`). */
export function gapCase(id: string, input: string, opts: GapCaseOptions = {}): FeatureCase {
    return caseBase(id, 'gap', input, opts);
}

type EntryEnv = {
    notes?: string;
    host?: FeatureHost;
    backend?: FeatureBackend;
    device?: string;
};

/**
 * Fluent Feature Matrix entry builder.
 *
 * Prefer this over raw object literals so status / effect / case kinds stay typed.
 */
export class FeatureEntryBuilder {
    private readonly _name: string;
    private readonly _category: string;
    private _status: FeatureStatus | undefined;
    private _effect: FeatureEffect | undefined;
    private _notes: string | undefined;
    private _host: FeatureHost | undefined;
    private _backend: FeatureBackend | undefined;
    private _device: string | undefined;
    private readonly _cases: FeatureCase[] = [];

    constructor(name: string, category: string) {
        this._name = name;
        this._category = category;
    }

    supported(): this {
        this._status = 'supported';
        return this;
    }

    partial(notes?: string): this {
        this._status = 'partial';
        if (notes !== undefined) this._notes = notes;
        return this;
    }

    unsupported(notes?: string): this {
        this._status = 'unsupported';
        if (notes !== undefined) this._notes = notes;
        return this;
    }

    planned(notes?: string): this {
        this._status = 'planned';
        if (notes !== undefined) this._notes = notes;
        return this;
    }

    pure(): this {
        this._effect = 'pure';
        return this;
    }

    stateful(): this {
        this._effect = 'stateful';
        return this;
    }

    effectful(): this {
        this._effect = 'effectful';
        return this;
    }

    unevaluated(): this {
        this._effect = 'unevaluated';
        return this;
    }

    notes(text: string): this {
        this._notes = text;
        return this;
    }

    host(value: FeatureHost): this {
        this._host = value;
        return this;
    }

    backend(value: FeatureBackend): this {
        this._backend = value;
        return this;
    }

    device(value: string): this {
        this._device = value;
        return this;
    }

    /** Append pre-built cases (from `evalCase` / `gapCase` / …). */
    cases(...cases: FeatureCase[]): this {
        this._cases.push(...cases);
        return this;
    }

    eval(id: string, input: string, expected: string, opts?: FeatureCaseOptions): this {
        return this.cases(evalCase(id, input, expected, opts));
    }

    parse(id: string, input: string, expected: string, opts?: FeatureCaseOptions): this {
        return this.cases(parseCase(id, input, expected, opts));
    }

    roundtrip(id: string, input: string, expected: string, opts?: FeatureCaseOptions): this {
        return this.cases(roundtripCase(id, input, expected, opts));
    }

    plot(id: string, input: string, opts?: PlotCaseOptions): this {
        return this.cases(plotCase(id, input, opts));
    }

    negative(id: string, input: string, opts?: NegativeCaseOptions): this {
        return this.cases(negativeCase(id, input, opts));
    }

    gap(id: string, input: string, opts?: GapCaseOptions): this {
        return this.cases(gapCase(id, input, opts));
    }

    /** Freeze into a `FeatureEntry`. Throws if status / effect missing. */
    done(): FeatureEntry {
        if (this._status === undefined) {
            throw new Error(`Feature \`${this._name}\` is missing status (call supported/partial/unsupported/planned)`);
        }
        if (this._effect === undefined) {
            throw new Error(`Feature \`${this._name}\` is missing effect (call pure/stateful/effectful/unevaluated)`);
        }
        const entry: FeatureEntry = {
            name: this._name,
            category: this._category,
            status: this._status,
            effect: this._effect,
            cases: this._cases.slice(),
        };
        if (this._notes !== undefined) entry.notes = this._notes;
        if (this._host !== undefined) entry.host = this._host;
        if (this._backend !== undefined) entry.backend = this._backend;
        if (this._device !== undefined) entry.device = this._device;
        return entry;
    }
}

/** Start a typed feature entry for `name` in `category`. */
export function feature(name: string, category: string): FeatureEntryBuilder {
    return new FeatureEntryBuilder(name, category);
}

/**
 * Non-fluent entry helper when status / effect are already known.
 * Still prefers case constructors over nested object literals.
 */
export function entry(
    name: string,
    category: string,
    status: FeatureStatus,
    effect: FeatureEffect,
    cases: readonly FeatureCase[],
    env: EntryEnv = {},
): FeatureEntry {
    const out: FeatureEntry = { name, category, status, effect, cases };
    if (env.notes !== undefined) out.notes = env.notes;
    if (env.host !== undefined) out.host = env.host;
    if (env.backend !== undefined) out.backend = env.backend;
    if (env.device !== undefined) out.device = env.device;
    return out;
}

/** Assemble a matrix from finished entries. */
export function matrix(...entries: FeatureEntry[]): FeatureMatrix {
    return entries;
}
