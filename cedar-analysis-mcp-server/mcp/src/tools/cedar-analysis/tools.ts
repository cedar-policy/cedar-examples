// src/tools/cedar-analysis/tools.ts
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { createTempFile, executeCedarCli } from "./cli.js";
import { formatError, formatResponse } from "./utils.js";
import * as fs from 'fs';

/**
 * Register Cedar analysis tools with the MCP server
 * @param server - The MCP server instance
 */
export function registerCedarAnalysisTools(server: McpServer): void {
    // Analyze Policies Tool
    server.tool(
        "analyze-policies",
        "Analyze Cedar policies against a schema to validate policy structure and identify potential issues.",
        {
            policy_set: z.string().describe("Cedar policy set content as a string (in Cedar policy syntax). Provide the actual policy content, not a file path."),
            schema: z.string().describe("Cedar schema content as a string (in Cedar schema syntax) - defines entity types and their attributes. Provide the actual schema content, not a file path."),
        },
        async ({ policy_set, schema }) => {
            try {
                // Create temporary files for policies and schema
                const policiesPath = await createTempFile(policy_set, '.cedar');
                const schemaPath = await createTempFile(schema, '.cedarschema');
                
                try {
                    // Build the CLI command arguments
                    const args = [
                        'analyze',
                        'policies',
                        policiesPath,
                        schemaPath,
                        '--json-output'
                    ];
                    
                    // Execute the CLI command
                    const output = await executeCedarCli(args);
                    
                    // Parse the output and return results
                    return formatResponse({
                        output,
                        summary: "Policy analysis completed successfully"
                    });
                } finally {
                    // Clean up temporary files
                    await Promise.all([
                        fs.promises.unlink(policiesPath),
                        fs.promises.unlink(schemaPath)
                    ]).catch(err => console.error('Error cleaning up temp files:', err));
                }
            } catch (error) {
                return formatError(error);
            }
        }
    );

    // Compare Policy Sets Tool
    server.tool(
        "compare-policy-sets",
        "Compare two Cedar policy sets to verify that policy changes maintain intended authorization behavior. This tool identifies if changes make policies more or less permissive, helping developers ensure security is maintained when updating policies.",
        {
            policy_set1: z.string().describe("Original/baseline Cedar policy set content as a string (in Cedar policy syntax). Provide the actual policy content, not a file path."),
            policy_set2: z.string().describe("New/modified Cedar policy set content as a string (in Cedar policy syntax) - typically containing your policy changes. Provide the actual policy content, not a file path."),
            schema: z.string().describe("Cedar schema content as a string (in Cedar schema syntax) - defines entity types and their attributes. Provide the actual schema content, not a file path."),
        },
        async ({ policy_set1, policy_set2, schema }) => {
            try {
                // Create temporary files for policies and schema
                const policies1Path = await createTempFile(policy_set1, '.cedar');
                const policies2Path = await createTempFile(policy_set2, '.cedar');
                const schemaPath = await createTempFile(schema, '.cedarschema');
                
                try {
                    // Build the CLI command arguments
                    const args = [
                        'analyze',
                        'compare',
                        policies2Path,
                        policies1Path,
                        schemaPath,
                        '--json-output'
                    ];
                    
                    // Execute the CLI command
                    const output = await executeCedarCli(args);
                    
                    // Parse the output and return results
                    return formatResponse({
                        output,
                        summary: "Policy comparison completed successfully"
                    });
                } finally {
                    // Clean up temporary files
                    await Promise.all([
                        fs.promises.unlink(policies1Path),
                        fs.promises.unlink(policies2Path),
                        fs.promises.unlink(schemaPath)
                    ]).catch(err => console.error('Error cleaning up temp files:', err));
                }
            } catch (error) {
                return formatError(error);
            }
        }
    );
    
}
