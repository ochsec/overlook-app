# Context Isolation Between Projects - Implementation Plan

## Current State Analysis

- Projects exist but only organize chats visually
- All chats share the same global context (no project-specific settings)
- System prompts are supported in API calls but not stored/managed
- No mechanism to prevent cross-project information leakage

## What "Context Isolation" Means

1. **Project-specific system prompts** - Each project can have its own instructions/persona
2. **Project-specific settings** - Default model, provider, temperature per project
3. **Conversation boundaries** - Chats in different projects don't cross-reference
4. **Visual separation** - Clear UI indicators showing project context

---

## Phase 1: Add Project-Level Settings

### 1.1 Extend Project Type (src/types.rs)
- Add `system_prompt: Option<String>` to Project struct
- Add `default_provider: Option<ProviderKind>`
- Add `default_model: Option<String>`
- Add `settings: Option<ProjectSettings>` for future expansion

### 1.2 Create ProjectSettings Type
```rust
pub struct ProjectSettings {
    pub system_prompt: Option<String>,
    pub default_provider: Option<ProviderKind>,
    pub default_model: Option<String>,
    pub temperature: Option<f32>,
}
```

## Phase 2: System Prompt Management

### 2.1 Update Chat Type (src/types.rs)
- Add `system_prompt: Option<String>` to Chat struct
- Inherit from project when chat is created

### 2.2 Modify send_message (src/app.rs)
- When creating user message, check if current project has system_prompt
- Inject system message at beginning of conversation if present
- Ensure system prompt is sent with every API request

### 2.3 Update API Layer (src/api.rs)
- Already supports system prompts in anthropic_request_body
- Extend openai_request_body and openrouter_request_body to support system messages

## Phase 3: Project Context Enforcement

### 3.1 Chat Creation (src/app.rs:create_new_chat)
- Automatically apply project's system_prompt to new chat
- Set chat's default_provider/model from project settings
- Visual indicator in UI showing project context is active

### 3.2 Project Switching
- When switching projects, update current_provider/current_model from project defaults
- Clear token usage when switching projects
- Option to warn user if switching mid-conversation

## Phase 4: UI/UX Enhancements

### 4.1 Project Settings Modal (src/ui.rs)
- Add "Project Settings" button next to each project
- Form to edit: system prompt, default provider, default model
- Preview showing how system prompt affects conversations

### 4.2 Visual Indicators
- Show active project's system prompt in a collapsible header
- Badge on chat showing which project context it belongs to
- Different accent colors per project (optional)

### 4.3 Settings View
- New tab for "Project Settings" in settings view
- List all projects with their configurations
- Template library for common system prompts

## Phase 5: Data Migration & Storage

### 5.1 Storage Updates (src/storage.rs)
- Save/load project settings with project data
- Save chat's system_prompt with chat data
- Migration for existing projects (add default empty settings)

### 5.2 Backward Compatibility
- Use `#[serde(default)]` for new Project fields
- Existing chats without system_prompt work normally
- Graceful fallback if project deleted but chat references it

---

## Implementation Order (Recommended)

### Week 1: Foundation
1. Update types (Project, Chat, ProjectSettings)
2. Update storage layer to handle new fields
3. Test data migration with existing projects

### Week 2: Core Functionality
4. Implement system prompt injection in send_message
5. Update API layer to send system messages
6. Add project default settings (provider/model)

### Week 3: UI/UX
7. Create project settings modal
8. Add visual indicators for active context
9. Add system prompt preview/editor

### Week 4: Polish
10. Template library for system prompts
11. Context switching warnings
12. Documentation and testing

---

## Key Design Decisions

### 1. System Prompt Storage
- Store at Project level (default for all chats in project)
- Store at Chat level (override/cache for specific chat)
- Benefits: Flexibility + consistency

### 2. Context Switching
- Switching projects changes current defaults
- Existing chat keeps its original context
- New chats inherit current project's context

### 3. Migration Strategy
- All new fields are `Option<T>` with `#[serde(default)]`
- No breaking changes to existing data
- Gradual adoption: works with and without context isolation

### 4. Future Extensions
- Add per-project API keys (useful for team/billing separation)
- Add per-project knowledge bases (for future RAG feature)
- Add project templates for quick setup

---

## Example: How It Works

### Before Context Isolation
```
Project: "Personal"
  - Chat 1: Uses GPT-4, no system prompt
  - Chat 2: Uses Claude, no system prompt

Project: "Work"
  - Chat 3: Uses GPT-4, no system prompt
  - Chat 4: Uses Claude, no system prompt
```

### After Context Isolation
```
Project: "Personal"
  Settings:
    - System Prompt: "You are a friendly assistant"
    - Default Model: GPT-4
    - Default Provider: OpenAI
  Chats:
    - Chat 1: Inherits settings, casual conversations
    - Chat 2: Inherits settings, personal planning

Project: "Work - Code Reviews"
  Settings:
    - System Prompt: "You are an expert code reviewer. Focus on security, performance, and best practices."
    - Default Model: Claude Opus
    - Default Provider: Anthropic
  Chats:
    - Chat 3: Code review for PR #123
    - Chat 4: Code review for PR #124

Project: "Work - Documentation"
  Settings:
    - System Prompt: "You are a technical writer. Create clear, concise documentation."
    - Default Model: GPT-4
    - Default Provider: OpenAI
  Chats:
    - Chat 5: API documentation
    - Chat 6: User guide
```

## Benefits

1. **Better Organization** - Projects become functional workspaces, not just folders
2. **Consistent Behavior** - All chats in a project follow the same rules
3. **Context Preservation** - Work discussions stay professional, personal stays casual
4. **Reduced Cognitive Load** - Don't need to repeat instructions in every chat
5. **Template Reusability** - Set up once, use for all related conversations
6. **Data Separation** - Future-proofs for multi-user or team features
