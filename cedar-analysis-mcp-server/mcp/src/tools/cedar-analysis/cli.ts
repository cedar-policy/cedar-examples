// src/tools/cedar-analysis/client.ts
import { exec } from "child_process";
import { promisify } from "util";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

const execAsync = promisify(exec);

/**
 * Path to the Cedar CLI executable
 * This can be overridden by setting the CEDAR_CLI_PATH environment variable
 */
export const CEDAR_CLI_PATH = process.env.CEDAR_CLI_PATH || "cedar-lean-cli";

/**
 * Create a temporary file with the given content
 * @param content - The content to write to the file
 * @param extension - The file extension to use
 * @returns The path to the created temporary file
 */
export async function createTempFile(
  content: string,
  extension: string
): Promise<string> {
  const tempDir = os.tmpdir();
  const tempFilePath = path.join(tempDir, `cedar-${Date.now()}${extension}`);
  await fs.promises.writeFile(tempFilePath, content);
  return tempFilePath;
}

/**
 * Execute the Cedar CLI with the given arguments
 * @param args - The arguments to pass to the Cedar CLI
 * @returns The stdout output from the command
 */
export async function executeCedarCli(args: string[]): Promise<string> {
  try {
    const command = `${CEDAR_CLI_PATH} ${args.join(" ")}`;
    const { stdout, stderr } = await execAsync(command);

    if (stderr && stderr.trim() !== "") {
      console.warn(`CLI warning: ${stderr}`);
    }

    return stdout;
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Failed to execute Cedar CLI: ${error.message}`);
    }
    throw error;
  }
}
