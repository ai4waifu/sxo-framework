import { d as coreD, expression as coreExpression, simplify as coreSimplify, Expression, version } from '@sxo/core';

export { Expression, version };

export function expression(input: string | Expression): Expression {
    return coreExpression(input);
}

export function d(input: string | Expression, varName: string): Expression {
    return coreD(input, varName);
}

export function simplify(input: string | Expression): Expression {
    return coreSimplify(input);
}
