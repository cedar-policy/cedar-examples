# Cedar Analysis MCP Server

Model Context Protocol (MCP) server for Cedar Policy Analysis

This MCP server provides tools for analyzing Cedar authorization policies, helping developers ensure their policy changes maintain intended authorization behavior. Additionally this MCP server has a prompt walking you through adding a new policy to an existing policy set and using analysis to understand the impact of adding the policy. 

## Features

- **policy comparison tool**: given an original policy set and a modified policy set, show the impact of the policy changes on a per action signature basis. 
- **policy analysis tool**: analyze a single policy set and present a set of findings about each policy within the policyset indicating potential logical inconsistencies or unintented behavior.
- **add and verify new policy prompt** a workflow to analyze the impact of adding new Cedar policies to your existing policy set. Shows permission changes, and identifies policy issues.

## Prerequisites

Docker installed on your system.

## Installation

Build the Docker image:
```bash
docker build -t cedar-cli .
```

## Configuration

Configure the server in your MCP configuration file e.g. for Amazon Q Developer CLI MCP, edit the following file `~/.aws/amazonq/mcp.json`:

```json
{
  "mcpServers": {
    "cedar-cli": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "cedar-cli"],
      "env": {},
      "disabled": false,
      "autoApprove": []
    }
  }
}
```
