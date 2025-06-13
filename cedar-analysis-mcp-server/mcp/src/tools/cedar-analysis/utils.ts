/**
 * Type definitions for MCP tool responses
 */

/**
 * Represents a text content item in a response
 * Includes index signature to satisfy MCP server requirements
 */
export interface TextContent {
    type: "text";
    text: string;
    [key: string]: unknown;
}

/**
 * Represents a successful response from a tool
 */
export interface SuccessResponse {
    content: TextContent[];
    isError?: false;
    [key: string]: unknown;
}

/**
 * Represents an error response from a tool
 */
export interface ErrorResponse {
    content: TextContent[];
    isError: true;
    [key: string]: unknown;
}

/**
 * Union type for all possible response types
 */
export type ToolResponse = SuccessResponse | ErrorResponse;

/**
 * Formats an error into a standardized error response
 * @param error - The error object to format
 * @returns A formatted error response
 */
export function formatError(error: unknown): ErrorResponse {
    // Handle different error types
    let errorText: string;
    
    if (error instanceof Error) {
        // Standard Error object
        errorText = JSON.stringify({
            message: error.message,
            name: error.name,
            stack: error.stack
        }, null, 2);
    } else if (typeof error === 'object' && error !== null) {
        // Object-like error
        errorText = JSON.stringify(error, null, 2);
    } else {
        // Primitive error or unknown type
        errorText = String(error);
    }
    
    return {
        content: [
            {
                type: "text",
                text: errorText
            }
        ],
        isError: true
    };
}

/**
 * Formats data into a standardized success response
 * @param data - The data to format
 * @returns A formatted success response
 */
export function formatResponse(data: unknown): SuccessResponse {
    return {
        content: [
            {
                type: "text",
                text: JSON.stringify(data, null, 2)
            }
        ]
    };
}
