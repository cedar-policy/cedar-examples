import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

const ADD_AND_VERIFY_POLICY_WORKFLOW_PROMPT = `# Add and Verify New Policy Workflow

## Overview

Analyzes the impact of adding new Cedar policies to your existing policy set. Shows permission changes, provides authorization examples, and identifies policy issues.

## Parameters

This prompt takes no parameters. You will be asked to provide:
- Path to Cedar schema file
- Path to current Cedar policies file  
- Path to new Cedar policy file

## Steps

### 1. Verify Dependencies

**Constraints:**
- You MUST verify these tools are available: cedar_analysis___comparepolicysets, cedar_analysis___analyzepolicies, fs_read
- You MUST inform user of any missing tools and ask if they want to proceed

### 2. Load Files

**Constraints:**
- You MUST read all three files using fs_read tool
- You MUST validate files exist and contain valid Cedar content
- You MUST combine current policies with new policy for analysis

### 3. Compare Policy Sets

**Constraints:**
- You MUST use cedar_analysis___comparepolicysets with current vs combined policy sets
- You MUST present results in a simple table showing entity, action, and change type
- You MUST focus on meaningful changes (more/less permissive)

### 4. Show Authorization Examples

**Constraints:**
- You MUST provide one concrete example showing the most significant permission change
- You MUST format the example in JSON with complete principal, action, and resource details
- You MUST include before/after authorization decisions with reasoning
- You MUST use generic identifiers (userA, userB, CompanyX, resource1, example.jpg) to make examples easily adaptable
- You MUST use the following JSON structure:
\`\`\`json
{
  "principal": {
    "id": "userA",
    "type": "EntityType",
    "attributes": { "key": "value" }
  },
  "action": "Action::\"name\"",
  "resource": {
    "id": "resource1", 
    "type": "EntityType",
    "attributes": { "key": "value" }
  },
  "authorization_change": {
    "before": "Allow/Deny",
    "after": "Allow/Deny", 
    "reason": "explanation"
  }
}
\`\`\`

### 5. Analyze Policy Issues

**Constraints:**
- You MUST use cedar_analysis___analyzepolicies on combined policy set
- You MUST check for: shadowed permits, impossible conditions, forbid overrides, complete denials
- You MUST present findings in a table with Issue Type, Description, Impact Level

### 6. Summary

**Constraints:**
- You MUST provide a concise summary with key findings and recommendations
- You MUST highlight any security concerns
- You MUST indicate if it's safe to deploy the new policy

## Examples

### Input
\`\`\`
Schema: sample/photo_sharing.cedarschema
Current policies: sample/current_policies.cedar  
New policy: sample/company_based_policy.cedar
\`\`\`

### Output Format

**Permission Changes:**
| Entity | Action | Change | Impact |
|--------|--------|--------|---------|
| User | view | More Permissive | Same-company users can view private photos |

**Authorization Examples:**
\`\`\`json
{
  "principal": {
    "id": "userA",
    "type": "PhotoSharing::User",
    "attributes": {
      "company": "CompanyX",
      "email": "userA@companyx.com",
      "isAdmin": false
    }
  },
  "action": "PhotoSharing::Action::\"view\"",
  "resource": {
    "id": "resource1",
    "type": "PhotoSharing::Photo",
    "attributes": {
      "filename": "example.jpg",
      "owner": {
        "id": "userB",
        "type": "PhotoSharing::User",
        "attributes": {
          "company": "CompanyX"
        }
      },
      "private": true
    }
  },
  "authorization_change": {
    "before": "Deny",
    "after": "Allow",
    "reason": "New policy allows same-company users to access private resources"
  }
}
\`\`\`

**Policy Issues:**
| Issue Type | Description | Impact |
|------------|-------------|---------|
| None Found | All policies are clean | ✅ Safe to deploy |

**Summary:** New policy safely adds same-company private photo access without conflicts.

## Troubleshooting

### File Issues
- Verify file paths are correct
- Check Cedar syntax if analysis fails
- Provide content directly if file access fails

### Analysis Issues  
- Review error messages for syntax problems
- Ensure schema matches policy entities
- Check that all required entities/actions are defined`;

/**
 * Register all prompts with the MCP server
 * 
 * This function registers prompts with the server using inline content.
 * 
 * @param server - The MCP server instance to register prompts with
 */
export function registerAllPrompts(server: McpServer): void {
  // Register the add-and-verify-new-policy-workflow prompt
  server.prompt(
    "add-and-verify-new-policy-workflow",
    "Analyzes the impact of adding new Cedar policies to your existing policy set. Shows permission changes, provides authorization examples, and identifies policy issues.",
    {}, // No arguments for this prompt
    async () => {
      return {
        messages: [
          {
            role: "assistant",
            content: {
              type: "text",
              text: ADD_AND_VERIFY_POLICY_WORKFLOW_PROMPT
            }
          }
        ]
      };
    }
  );
}
