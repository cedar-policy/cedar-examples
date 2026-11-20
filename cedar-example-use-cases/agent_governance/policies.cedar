// 1. allow-workspace-read: agents at ring 2+ can read files in /workspace
permit (
  principal is Agent::Principal,
  action == Agent::Action::"open",
  resource is Agent::File
)
when {
  principal.ring >= 2 &&
  resource.path like "/workspace/*" &&
  context.mode == "read"
};

// 2. allow-metadata-tool-ring-1: ring 1+ agents can invoke read-only tools
permit (
  principal is Agent::Principal,
  action == Agent::Action::"request_tool",
  resource is Agent::Tool
)
when {
  principal.ring >= 1 &&
  (resource.name == "Read" || resource.name == "Glob" || resource.name == "Grep")
};

// 3. deny-cloud-metadata: block cloud metadata endpoints absolutely
forbid (
  principal,
  action == Agent::Action::"connect",
  resource is Agent::Endpoint
)
when {
  resource.host == "169.254.169.254" ||
  resource.host == "metadata.google.internal" ||
  resource.host == "metadata.azure.com"
};

// 4. deny-credential-files: block reads in credential-bearing directories
forbid (
  principal,
  action == Agent::Action::"open",
  resource is Agent::File
)
when {
  resource.path like "*/.ssh/*" ||
  resource.path like "*/.aws/credentials" ||
  resource.path like "*/.aws/config" ||
  resource.path like "*/.kube/config"
};

// 5. require-trust-for-exec: exec gated on ring + trust_score
permit (
  principal is Agent::Principal,
  action == Agent::Action::"exec",
  resource is Agent::Executable
)
when {
  principal.ring <= 1 &&
  principal.trust_score > decimal("0.7") &&
  context.cwd like "/workspace/*"
};
