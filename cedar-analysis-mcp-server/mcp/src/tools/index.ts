import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerCedarTools } from "./cedar-analysis/index.js";

/**
 * Register all tools with the MCP server
 */
export function registerAllTools(server: McpServer): void {
  registerCedarTools(server);
}
