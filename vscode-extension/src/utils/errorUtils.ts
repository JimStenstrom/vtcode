/**
 * Centralized error utilities - consolidates error handling patterns
 * Eliminates 50+ instances of duplicate error message extraction
 */

/**
 * Extract error message from unknown error type
 * Replaces pattern: error instanceof Error ? error.message : String(error)
 */
export function getErrorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

/**
 * Extract error stack trace if available
 */
export function getErrorStack(error: unknown): string | undefined {
    return error instanceof Error ? error.stack : undefined;
}

/**
 * Check if error matches a pattern (for error type detection)
 */
export function errorMatches(error: unknown, ...patterns: string[]): boolean {
    const message = getErrorMessage(error);
    return patterns.some(pattern => message.includes(pattern));
}

/**
 * Check if error is a network error
 */
export function isNetworkError(error: unknown): boolean {
    return errorMatches(
        error,
        'ECONNREFUSED',
        'ETIMEDOUT',
        'ENOTFOUND',
        'network',
        'timeout',
        'DNS'
    );
}

/**
 * Check if error is a rate limit error
 */
export function isRateLimitError(error: unknown): boolean {
    return errorMatches(error, '429', 'rate limit');
}

/**
 * Check if error is an authentication error
 */
export function isAuthError(error: unknown): boolean {
    return errorMatches(error, '401', 'unauthorized', 'invalid key', 'api key');
}

/**
 * Check if error is a token/context limit error
 */
export function isTokenLimitError(error: unknown): boolean {
    return errorMatches(error, 'token', 'limit', 'context');
}

/**
 * Check if error is a file system error
 */
export function isFileSystemError(error: unknown): boolean {
    return errorMatches(error, 'ENOENT', 'EACCES', 'not found', 'permission denied');
}
