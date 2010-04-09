import { createSession, Expression, version } from '@sxo/core';

export { Expression, version };

/** Explicit simple-math dialect tag for the native host. */
const SIMPLE_MATH_DIALECT = 'simple-math';

function binding() {
    return createSession().binding;
}

/** Parse simple-math source (no evaluate). */
export function expression(input: string | Expression): Expression {
    if (input instanceof Expression) return input;
    return Expression.fromNative(binding().expression(input, SIMPLE_MATH_DIALECT));
}

/** Differentiate simple-math source or an existing expression. */
export function d(input: string | Expression, varName: string): Expression {
    if (input instanceof Expression) return input.d(varName);
    return Expression.fromNative(binding().d(input, varName, SIMPLE_MATH_DIALECT));
}

/** Simplify simple-math source or an existing expression. */
export function simplify(input: string | Expression): Expression {
    if (input instanceof Expression) return input.simplify();
    return Expression.fromNative(binding().simplify(input, SIMPLE_MATH_DIALECT));
}
