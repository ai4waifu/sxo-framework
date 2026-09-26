declare module '../lib/sxo_lite.js' {
    export default function init(input?: { module_or_path?: string | URL | BufferSource }): Promise<unknown>;
    export class Expression {
        constructor(input: string, dialect?: string | null);
        d(varName: string): Expression;
        evaluate(strategy?: string | null): Expression;
        simplify(): Expression;
        toString(): string;
        toWolfram(): string;
        toMatlab(): string;
        isEqual(other: Expression): boolean;
        readonly status: string;
        readonly coverage: string;
    }
    export function version(): string;
    export function evaluate(input: string, dialect?: string | null, strategy?: string | null): Expression;
    export function expression(input: string, dialect?: string | null): Expression;
    export function d(input: string, varName: string, dialect?: string | null): Expression;
    export function simplify(input: string, dialect?: string | null): Expression;
}
