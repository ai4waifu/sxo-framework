/** Wire-facing diagnostic severity (locale text lives in product packages). */
export type Severity = 'error' | 'warning' | 'info' | 'hint';

/** Neutral diagnostic shape for host / dialect adapters. */
export type Diagnostic = {
    code: string;
    message: string;
    severity?: Severity;
};
