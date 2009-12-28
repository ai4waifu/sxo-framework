/**
 * Opaque expression handle brand.
 *
 * Callers must not inspect numeric arena identities or cast handles to JSON.
 */
export type ExpressionHandle = {
    readonly __sxoExpression: unique symbol;
};
