# Agent Management (`agent.proto`)

**Status:** ⏳ Work in Progress  
**Last Updated:** 2026-01-04

---

## Purpose

List available agents with metadata (name, description, mode, color).

---

## Source of Truth

**Note:** No dedicated JSON Schema yet for agent types.

**Future work:** Create JSON Schema for agent types to align with model/provider pattern.

---

## Messages

```protobuf
syntax = "proto3";
package opencode.agent;

message AgentInfo {
  string name = 1;
  optional string description = 2;
  optional string mode = 3;         // "subagent" or null
  bool built_in = 4;
  optional string color = 5;        // Hex color for UI
}

message AgentList {
  repeated AgentInfo agents = 1;
}

message Empty {}
```

---

## Service Definition

```protobuf
service AgentService {
  rpc ListAgents(Empty) returns (AgentList);
}
```

---

## Maps to OpenCode Server

- `AgentService.ListAgents` → `GET /agent`

---

## Design Notes

**Agent Types:**

- `built_in: true` - Core agents (code, title, summary, compaction)
- `built_in: false` - Plugin-defined agents

**Agent Mode:**

- `null` - Standard agent (direct interaction)
- `"subagent"` - Spawned by other agents (Task tool)

**Color:**

- Hex color for UI differentiation
- Used in agent selector, message attribution

---

## TODO

- [ ] Create `agentInfo.schema.json` for agent metadata
- [ ] Document built-in agents vs plugin agents
- [ ] Validate against actual agent API responses
