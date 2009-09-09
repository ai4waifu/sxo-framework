declare module '../lib/sxo_lite.js' {
    export default function init(input?: { module_or_path?: string | URL | BufferSource }): Promise<unknown>;
    export class Expression {
        constructor(input: string, dialect?: string | null);
        d(varName: string): Expression;
        simplify(): Expression;
        toString(): string;
        toDSL(): string;
        toWolfram(): string;
        isEqual(other: Expression): boolean;
    }
    export function version(): string;
    export function expression(input: string, dialect?: string | null): Expression;
    export function d(input: string, varName: string, dialect?: string | null): Expression;
    export function simplify(input: string, dialect?: string | null): Expression;
}
