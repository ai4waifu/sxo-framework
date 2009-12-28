import { Expression } from './expression.js';
import { loadNative, type NativeBinding } from './native.js';

/**
 * Native-backed Session facade.
 *
 * Owns host binding lifetime for related handles. Does not parse dialect source
 * strings — dialect packages adapt this session (or load the same native host).
 */
export class Session {
    readonly #binding: NativeBinding;

    constructor(binding: NativeBinding = loadNative()) {
        this.#binding = binding;
    }

    /** Engine / package version from the native addon. */
    version(): string {
        return this.#binding.version();
    }

    /**
     * @internal Host binding for dialect adapters that still call N-API directly.
     * Prefer dialect packages migrating onto Session-owned methods over time.
     */
    get binding(): NativeBinding {
        return this.#binding;
    }

    /** Wrap an already-produced native expression handle. */
    expressionFromNative(inner: Parameters<typeof Expression.fromNative>[0]): Expression {
        return Expression.fromNative(inner);
    }
}
