import { createSession, type NativeBinding, type NativeExpression } from '@sxo/core';

export type { NativeBinding, NativeExpression };

/** Load the shared native host binding via `@sxo/core` Session. */
export function loadNative(): NativeBinding {
    return createSession().binding;
}

export const MATH_DIALECT = 'mathematica';
