import { CAPABILITIES, type Capabilities } from './capabilities.js';
import { Session, init } from './session.js';

export { init };

/** Create a WASM-backed Session (requires {@link init} first in the browser). */
export function createSession(): Session {
    return new Session();
}

/** Engine / package version from the WASM module. */
export function version(): string {
    return createSession().version();
}

/** Capability snapshot for this product face. */
export function capabilities(): Capabilities {
    return CAPABILITIES;
}
