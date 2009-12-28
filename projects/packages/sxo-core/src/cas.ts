import { CAPABILITIES, type Capabilities } from './capabilities.js';
import { loadNative } from './native.js';
import { Session } from './session.js';

/** Create a native-backed Session. */
export function createSession(): Session {
    return new Session(loadNative());
}

/** Engine / package version from the native addon. */
export function version(): string {
    return loadNative().version();
}

/** Capability snapshot for this product face. */
export function capabilities(): Capabilities {
    return CAPABILITIES;
}
