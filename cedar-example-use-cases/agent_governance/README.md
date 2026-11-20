# Agent Governance

An example showing Cedar as the policy layer for an AI agent host (for instance: Claude Code, Cursor, a custom MCP agent, or an autonomous coding agent). The agent performs tool calls (shell commands, file reads and writes, network connections, MCP tool invocations); Cedar decides which are permitted based on the agent's identity, trust level, and the action's context.

This example uses the four canonical agent action verbs (`exec`, `open`, `connect`, `request_tool`) as the vocabulary for agent-host policy authoring. The full canonical schema lives in the community-maintained [cedar-agent-schemas](https://github.com/VeritasActa/cedar-agent-schemas) repository; this example imports a trimmed version for self-contained evaluation.

## Use case

An agent platform runs AI agents inside a sandbox. Each agent has:

- An `agent_id` (unique per-agent identity)
- A `trust_score` (typically between 0.0 and 1.0, derived from validation or user signal)
- A `ring` (0 = most privileged, 3+ = least privileged; a convention borrowed from OS isolation rings)
- An optional `session_id`

Each tool call the agent wants to make is an authorization request against Cedar. The policy set decides.

## Entities

### `Agent::Principal`

The calling agent. Attributes:

- `agent_id: String`
- `trust_score: Decimal` (between 0 and 1)
- `ring: Long` (0, 1, 2, 3)
- `session_id: String?` (optional)

### `Agent::File`

A file the agent wants to read or write. Attributes:

- `path: String`
- `owner_uid: Long?`

### `Agent::Endpoint`

A network endpoint the agent wants to connect to. Attributes:

- `host: String`
- `port: Long`
- `protocol: String`

### `Agent::Tool`

An MCP tool or agent-SDK tool the agent wants to invoke. Attributes:

- `name: String`
- `server: String?`

### `Agent::Executable`

A process the agent wants to spawn. Attributes:

- `path: String`
- `trusted: Boolean?`

## Actions

Four canonical verbs cover the relevant authorization boundaries:

- `exec`: spawn a process
- `open`: read or write a file
- `connect`: open a network connection
- `request_tool`: invoke an MCP tool or agent-SDK tool

## Policies

This example ships five policies that exercise the main patterns:

1. **allow-workspace-read**: agents at ring 2+ can read files inside `/workspace`
2. **allow-metadata-tool-ring-1**: agents at ring 1 can invoke safe read-only MCP tools (`Read`, `Glob`, `Grep`)
3. **deny-cloud-metadata**: forbid all network connections to cloud instance metadata endpoints regardless of trust
4. **deny-credential-files**: forbid all file reads in `.ssh`, `.aws`, `.kube` directories
5. **require-trust-for-exec**: `exec` is permitted only for agents at ring 0-1 AND trust_score > 0.7

Cedar's `forbid` takes precedence over `permit`, so the deny policies are absolute regardless of whether another policy allows the action.

## Running

```bash
cd cedar-example-use-cases
./run.sh agent_governance
```

Expected output: all requests in `ALLOW/` permitted, all requests in `DENY/` denied.

## Extending this example

This is a small, self-contained starting point. Production agent governance systems typically extend it with:

- Per-tool context attributes (beyond `args_hash` in `request_tool`)
- Session-scoped attributes (`prior_tool_count`, `prior_deny_count`, `session_age`) for rate limiting
- Resource-owner relationships (files owned by specific agents or users)
- Time-scoped policies (allowed during business hours; denied during maintenance windows)
- Integration with signed-receipt emitters that commit each decision to an audit log

See [cedar-agent-schemas](https://github.com/VeritasActa/cedar-agent-schemas) for the full community-maintained schema and for links to downstream systems (Microsoft Agent Governance Toolkit, protect-mcp, sb-runtime, APS, Signet, nono) that import and extend it.

## Related

- [cedar-agent-schemas](https://github.com/VeritasActa/cedar-agent-schemas): canonical community schema for agent action verbs
- [OWASP Agentic Top 10](https://owasp.org/www-project-agentic-top-10/): risk catalog that maps to the four action verbs used here
- [cedar-for-agents](https://github.com/cedar-policy/cedar-for-agents): Cedar WASM bindings for JS/TS agent hosts
