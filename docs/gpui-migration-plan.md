# GPUI Migration Plan for Overlook

**Version:** 1.0
**Date:** 2025-11-15
**Status:** Planning Phase

---

## Executive Summary

This document outlines a comprehensive plan to migrate the Overlook chat application from **egui** to **GPUI** with the **gpui-component** library. This migration will provide access to 60+ professional UI components, improved performance, and a modern component architecture.

### Key Metrics

- **Timeline:** 2-4 weeks (full-time development)
- **Effort Estimate:** ~80-120 hours
- **Risk Level:** Medium-High (complete framework change)
- **Expected Improvement:** 40-60% code reduction in UI layer, 2-3x performance improvement for large datasets

### Strategic Rationale

1. **Component Library:** Access to 60+ production-ready components vs. current custom implementations
2. **Performance:** Virtualization and optimization for large chat histories
3. **Modern Architecture:** Declarative component API with method chaining
4. **Professional Theming:** 20+ built-in themes vs. 2 custom themes
5. **Advanced Features:** LSP support, rich text editing, context menus, tooltips
6. **Long-term Maintainability:** Active development by Zed team, growing ecosystem

### Success Criteria

- ✓ All current functionality preserved
- ✓ Performance improvements in large chat histories (1000+ messages)
- ✓ Code reduction of 40%+ in UI components
- ✓ No regressions in user experience
- ✓ Successful compilation on macOS, Linux, Windows

---

## Table of Contents

1. [Current State Analysis](#current-state-analysis)
2. [Target Architecture](#target-architecture)
3. [Migration Phases](#migration-phases)
4. [Component Mapping](#component-mapping)
5. [Code Migration Patterns](#code-migration-patterns)
6. [Dependency Changes](#dependency-changes)
7. [Risk Assessment & Mitigation](#risk-assessment--mitigation)
8. [Timeline & Milestones](#timeline--milestones)
9. [Testing Strategy](#testing-strategy)
10. [Rollback Plan](#rollback-plan)
11. [Post-Migration Benefits](#post-migration-benefits)

---

## Current State Analysis

### Technology Stack (Before Migration)

```toml
[dependencies]
eframe = "0.31.0"           # egui framework wrapper
egui = "0.31.0"              # Immediate mode GUI
egui_extras = { version = "0.31.0", features = ["image"] }
egui-twemoji = "0.7.1"       # Emoji rendering
```

### Application Architecture

#### File Structure
```
src/
├── main.rs                  # Application entry point, window setup
├── app.rs                   # Main application struct, view routing
├── theme.rs                 # Custom theme system (One Light/Dark)
├── markdown_renderer.rs     # Custom markdown rendering
├── chat.rs                  # Chat view implementation
├── settings.rs              # Settings view
├── export.rs                # Export view
├── organize.rs              # Project management view
├── model_picker.rs          # Model selection modal
├── providers/               # API provider implementations
└── database.rs              # SQLite data layer
```

#### Current UI Component Inventory

| Component | Current Implementation | Lines of Code | Complexity |
|-----------|----------------------|---------------|------------|
| **Theme System** | Custom color structs, manual styling | ~200 | Medium |
| **Markdown Renderer** | Custom with pulldown-cmark | ~350 | High |
| **Chat Interface** | Manual layout with ScrollArea | ~800 | Very High |
| **Message Bubbles** | Custom drawing with RichText | ~150 | Medium |
| **Model Picker** | egui::Window with custom layout | ~400 | High |
| **Settings Form** | Manual form layout | ~300 | Medium |
| **Sidebar** | egui::SidePanel + custom list | ~200 | Medium |
| **Menu Bar** | egui::menu::bar | ~100 | Low |
| **Input Area** | egui::TextEdit::multiline | ~150 | Medium |
| **Buttons** | egui::Button with manual styling | ~50 | Low |
| **File Attachments** | Custom preview rendering | ~200 | Medium |
| **Search Results** | Custom collapsible groups | ~150 | Medium |
| **Modals/Dialogs** | egui::Window | ~100 | Low |
| **Token Usage Display** | Custom label formatting | ~50 | Low |

**Total UI Code:** ~3,200 lines
**Estimated Post-Migration:** ~1,500-2,000 lines (40-50% reduction)

### Current Pain Points

1. **Manual Layout Calculations:** Excessive width/spacing/positioning code
2. **Repetitive Styling:** Color and font specifications repeated throughout
3. **Limited Component Reuse:** Most UI elements built from primitives each time
4. **State Management:** Manual modal/popup state tracking
5. **No Virtualization:** Full re-render of entire chat history every frame
6. **Basic Tooltips:** Simple text-only `.on_hover_text()`
7. **No Context Menus:** All actions require button clicks
8. **Limited Keyboard Navigation:** Basic tab support only
9. **Performance:** Noticeable lag with 500+ message conversations
10. **Theming:** Only 2 themes, difficult to add more

---

## Target Architecture

### New Technology Stack (After Migration)

```toml
[dependencies]
gpui = "0.1"                           # GPUI framework
gpui-component = "0.1"                 # Component library
pulldown-cmark = "0.12"                # Keep for markdown parsing
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
anyhow = "1.0"
chrono = "0.4"
base64 = "0.22"
image = "0.25"

# Remove these:
# eframe = "0.31.0"
# egui = "0.31.0"
# egui_extras = "0.31.0"
# egui-twemoji = "0.7.1"
```

### New File Structure

```
src/
├── main.rs                  # GPUI app initialization
├── app.rs                   # Root component, app state
├── theme.rs                 # Theme adapter for gpui-component themes
├── components/              # Reusable UI components (NEW)
│   ├── mod.rs
│   ├── message_bubble.rs    # Chat message component
│   ├── chat_input.rs        # Message input component
│   ├── model_badge.rs       # Model indicator component
│   └── attachment_preview.rs
├── views/                   # Main application views (NEW)
│   ├── mod.rs
│   ├── chat_view.rs
│   ├── settings_view.rs
│   ├── export_view.rs
│   └── organize_view.rs
├── markdown/                # Markdown rendering (REFACTORED)
│   ├── mod.rs
│   └── renderer.rs
├── providers/               # Keep as-is
├── database.rs              # Keep as-is
└── state.rs                 # Global app state management (NEW)
```

### Architecture Principles

1. **Component-Based:** Every UI element is a reusable component
2. **Declarative:** UI declared as function of state, not imperative commands
3. **Single Source of Truth:** Global state managed in one place
4. **Separation of Concerns:** Views, components, state, data clearly separated
5. **Type Safety:** Leverage Rust's type system for UI correctness

---

## Migration Phases

### Phase 0: Preparation & Setup (Days 1-2)

**Objective:** Prepare environment and create migration branch

**Tasks:**
- [ ] Create `gpui-migration` branch from `main`
- [ ] Document current functionality (screenshots, feature list)
- [ ] Set up GPUI dependencies in new branch
- [ ] Create basic GPUI "Hello World" window
- [ ] Verify GPUI builds on target platforms
- [ ] Study gpui-component documentation and examples
- [ ] Create backup of current working `main` branch

**Deliverables:**
- Working GPUI window that launches
- Documentation of current features
- Migration branch setup

**Success Criteria:**
- GPUI app opens with blank window
- No compilation errors
- All team members have environment set up

**Estimated Time:** 8-12 hours

---

### Phase 1: Infrastructure & Window Management (Days 2-3)

**Objective:** Set up core GPUI infrastructure

**Tasks:**
- [ ] Implement GPUI `App` struct replacing egui `eframe::App`
- [ ] Set up window creation and management
- [ ] Implement basic navigation/routing between views
- [ ] Port theme system to GPUI (colors, fonts)
- [ ] Set up global app state management
- [ ] Implement async task handling for API calls
- [ ] Create basic layout structure (menu, sidebar, content area)

**Code Changes:**

**Before (egui):**
```rust
// main.rs
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Overlook",
        options,
        Box::new(|cc| Box::new(OverlookApp::new(cc))),
    )
}
```

**After (GPUI):**
```rust
// main.rs
use gpui::*;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|cx| OverlookApp::new(cx))
        });
    });
}
```

**Deliverables:**
- GPUI app with window management
- Theme system ported
- Basic view routing working
- State management foundation

**Success Criteria:**
- Window opens with correct size and title
- Can switch between empty view placeholders
- Theme colors applied correctly
- No crashes or panics

**Estimated Time:** 10-14 hours

---

### Phase 2: Core Components (Days 3-5)

**Objective:** Migrate fundamental UI components

**Tasks:**
- [ ] Replace all `egui::Button` with `gpui_component::Button`
- [ ] Replace `egui::TextEdit` with `gpui_component::Input`
- [ ] Replace `egui::Window` modals with `gpui_component::Dialog`
- [ ] Replace `egui::ComboBox` with `gpui_component::Select`
- [ ] Implement custom `MessageBubble` component using gpui-component primitives
- [ ] Create `ChatInput` component with attachment support
- [ ] Create `ModelBadge` component
- [ ] Port file attachment preview rendering

**Component Mapping:**

| egui Component | gpui-component Replacement | Status |
|----------------|---------------------------|---------|
| `egui::Button` | `Button::new().primary()` | ⏳ Pending |
| `egui::TextEdit::singleline()` | `Input::new()` | ⏳ Pending |
| `egui::TextEdit::multiline()` | `Input::new().multiline()` | ⏳ Pending |
| `egui::ComboBox` | `Select::new()` | ⏳ Pending |
| `egui::Window` | `Dialog::new()` | ⏳ Pending |
| `egui::SidePanel` | `Sidebar::new()` | ⏳ Pending |
| Custom collapsible | `Collapsible::new()` | ⏳ Pending |

**Example Migration:**

**Before (egui):**
```rust
// Button with manual styling
if ui.add(
    egui::Button::new("Send")
        .fill(theme.primary_color)
        .min_size(egui::vec2(80.0, 32.0))
).clicked() {
    self.send_message();
}
```

**After (GPUI + gpui-component):**
```rust
// Button with component API
Button::new("send")
    .label("Send")
    .primary()
    .on_click(cx, |this, cx| this.send_message(cx))
    .render(cx)
```

**Deliverables:**
- All core components ported
- Component library established
- Consistent styling across components

**Success Criteria:**
- Buttons render and respond to clicks
- Text inputs accept keyboard input
- Modals open and close correctly
- All components match design system

**Estimated Time:** 16-20 hours

---

### Phase 3: Settings View Migration (Days 5-7)

**Objective:** Migrate simplest view as proof of concept

**Tasks:**
- [ ] Create `SettingsView` component structure
- [ ] Port API key input sections
- [ ] Port theme selector dropdown
- [ ] Port font size slider
- [ ] Port streaming toggle
- [ ] Implement settings save/load
- [ ] Add validation and error handling
- [ ] Test all settings persistence

**Why Settings First:**
- Simplest view (mostly forms)
- Clear inputs/outputs
- Less state management complexity
- Good test case for component library

**Deliverables:**
- Fully functional Settings view
- All settings save and load correctly
- Validation working

**Success Criteria:**
- Can enter and save API keys
- Theme changes apply immediately
- Font size adjustments work
- Streaming toggle persists

**Estimated Time:** 10-12 hours

---

### Phase 4: Chat Interface Migration (Days 7-12)

**Objective:** Migrate most complex view

**Tasks:**
- [ ] Create `ChatView` component structure
- [ ] Implement message list with `VirtualList` for performance
- [ ] Port `MessageBubble` rendering
- [ ] Implement markdown rendering in messages
- [ ] Port message input area with attachments
- [ ] Implement streaming message animation
- [ ] Port model/provider selector
- [ ] Implement search results display
- [ ] Add token usage display
- [ ] Port sidebar with chat/project lists
- [ ] Implement chat creation/deletion
- [ ] Add right-click context menus (NEW FEATURE)
- [ ] Add tooltips for UI elements (NEW FEATURE)

**Message List Optimization:**

**Before (egui):**
```rust
// Full render of all messages every frame
egui::ScrollArea::vertical().show(ui, |ui| {
    for message in &self.messages {
        render_message(ui, message);
    }
});
```

**After (GPUI + VirtualList):**
```rust
// Only render visible messages
VirtualList::new(
    cx,
    self.messages.len(),
    |index, cx| {
        MessageBubble::new(&self.messages[index])
            .render(cx)
    }
)
```

**Deliverables:**
- Fully functional chat interface
- Message rendering with markdown
- Streaming messages working
- Chat management (create/delete/rename)
- Performance optimized with virtualization

**Success Criteria:**
- Can send and receive messages
- Markdown renders correctly
- Streaming animation smooth
- No lag with 1000+ messages
- All chat operations work

**Estimated Time:** 30-40 hours

---

### Phase 5: Remaining Views & Polish (Days 12-14)

**Objective:** Migrate Export and Organize views, add polish

**Tasks:**
- [ ] Migrate Export view
- [ ] Migrate Organize view (project management)
- [ ] Port Model Picker modal with `Table` component
- [ ] Add keyboard shortcuts
- [ ] Implement notification toasts for actions
- [ ] Add rich tooltips with component info
- [ ] Implement context menus for chats/messages
- [ ] Add progress indicators for long operations
- [ ] Polish animations and transitions
- [ ] Accessibility improvements

**New Features (Enabled by gpui-component):**

1. **Context Menus:**
   - Right-click on chat → Rename, Delete, Export, Pin
   - Right-click on message → Copy, Regenerate, Edit, Delete
   - Right-click on attachment → Download, Remove

2. **Rich Tooltips:**
   - Model info (context length, pricing, capabilities)
   - Feature explanations
   - Keyboard shortcuts

3. **Toast Notifications:**
   - "Chat exported successfully"
   - "API key saved"
   - "Model switched to Claude 3.5"
   - Error messages

4. **Progress Indicators:**
   - File upload progress
   - Export progress
   - Model loading

**Deliverables:**
- All views migrated
- New UX features implemented
- Polish and animations complete

**Success Criteria:**
- Export works correctly
- Project management functional
- Context menus work
- Notifications appear correctly
- Keyboard shortcuts operational

**Estimated Time:** 16-20 hours

---

### Phase 6: Testing & Optimization (Days 14-16)

**Objective:** Comprehensive testing and performance optimization

**Tasks:**
- [ ] Test all features systematically
- [ ] Performance testing with large datasets
  - 1000+ message conversations
  - 100+ chats in sidebar
  - Large file attachments
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] Memory leak detection
- [ ] UI/UX testing for regressions
- [ ] Accessibility testing
- [ ] Load testing with multiple providers
- [ ] Bug fixing
- [ ] Code cleanup and documentation
- [ ] Update README with new architecture

**Performance Benchmarks:**

| Scenario | egui Baseline | GPUI Target | Improvement |
|----------|---------------|-------------|-------------|
| Render 1000 messages | ~50ms/frame | <16ms/frame | 3x faster |
| Scroll large chat | Stutters | Smooth 60fps | Smooth |
| Memory (1000 msgs) | ~200MB | <150MB | 25% less |
| Startup time | ~500ms | <400ms | 20% faster |

**Deliverables:**
- Test report with all scenarios passed
- Performance benchmarks documented
- All critical bugs fixed
- Documentation updated

**Success Criteria:**
- No feature regressions
- Performance targets met
- Builds on all platforms
- Zero critical bugs

**Estimated Time:** 12-16 hours

---

## Component Mapping

### Complete Component Mapping Table

| Current Implementation | egui API | gpui-component | Priority | Complexity | Effort |
|----------------------|----------|----------------|----------|------------|---------|
| **Buttons** | `egui::Button` | `Button` | High | Low | 2h |
| **Text Input (single)** | `TextEdit::singleline()` | `Input` | High | Low | 2h |
| **Text Input (multi)** | `TextEdit::multiline()` | `Input::multiline()` | High | Medium | 3h |
| **Checkboxes** | `egui::Checkbox` | `Checkbox` | Medium | Low | 1h |
| **Radio Buttons** | `egui::RadioButton` | `Radio` | Low | Low | 1h |
| **Dropdowns** | `egui::ComboBox` | `Select` | High | Medium | 4h |
| **Sliders** | `egui::Slider` | `Slider` | Medium | Low | 2h |
| **Modals** | `egui::Window` | `Dialog` | High | Medium | 4h |
| **Sidebar** | `egui::SidePanel` | `Sidebar` | High | Medium | 4h |
| **Menu Bar** | `egui::menu::bar` | `Menu` | Medium | Low | 2h |
| **Tooltips** | `.on_hover_text()` | `Tooltip` | Medium | Low | 2h |
| **Progress Bars** | Custom labels | `Progress` | Medium | Low | 2h |
| **Spinners** | Custom animation | `Spinner` | Low | Low | 1h |
| **Badges** | Custom labels | `Badge` | Medium | Low | 2h |
| **Tables** | `egui_extras::TableBuilder` | `Table` | High | High | 6h |
| **Collapsible Groups** | `egui::CollapsingHeader` | `Collapsible` / `Accordion` | Medium | Low | 3h |
| **Scrollable Areas** | `egui::ScrollArea` | `Scrollable` | High | Medium | 3h |
| **Message List** | Custom rendering | `VirtualList` | Critical | High | 8h |
| **Markdown Renderer** | Custom | Custom (keep) + styling | High | High | 10h |
| **Theme System** | Custom structs | gpui-component themes | High | Medium | 6h |
| **File Attachments** | Custom preview | Custom + `Image` | Medium | Medium | 4h |
| **Model Picker** | Custom modal | `Dialog` + `Table` | High | High | 6h |
| **Search Results** | Custom collapsible | `Collapsible` | Medium | Medium | 3h |
| **Token Display** | Custom labels | `Label` + `Badge` | Low | Low | 1h |
| **Context Menus** | None | `Menu` | High | Medium | 4h |
| **Notifications** | None | `Notification` | Medium | Medium | 3h |
| **Keyboard Shortcuts** | Basic | Enhanced | Medium | Medium | 4h |

**Total Estimated Effort:** 84 hours

---

## Code Migration Patterns

### Pattern 1: Button Migration

**Before (egui):**
```rust
ui.horizontal(|ui| {
    if ui.add_sized(
        [100.0, 36.0],
        egui::Button::new("New Chat")
            .fill(theme.primary_color)
    ).clicked() {
        self.create_new_chat();
    }

    if ui.add_sized(
        [80.0, 36.0],
        egui::Button::new("Export")
            .fill(theme.secondary_color)
    ).clicked() {
        self.export_chat();
    }
});
```

**After (GPUI):**
```rust
h_flex()
    .gap_2()
    .child(
        Button::new("new_chat")
            .label("New Chat")
            .primary()
            .on_click(cx, |this, cx| this.create_new_chat(cx))
    )
    .child(
        Button::new("export")
            .label("Export")
            .secondary()
            .on_click(cx, |this, cx| this.export_chat(cx))
    )
```

**Improvements:**
- 40% less code
- Cleaner API with method chaining
- No manual sizing/styling
- Consistent button variants

---

### Pattern 2: Form Input Migration

**Before (egui):**
```rust
ui.vertical(|ui| {
    ui.label("API Key:");
    let response = ui.add(
        egui::TextEdit::singleline(&mut self.api_key)
            .password(self.show_key)
            .desired_width(300.0)
            .hint_text("Enter your API key...")
    );

    if response.changed() {
        self.validate_api_key();
    }

    ui.checkbox(&mut self.show_key, "Show key");
});
```

**After (GPUI):**
```rust
v_flex()
    .gap_2()
    .child(Label::new("API Key:"))
    .child(
        Input::new("api_key")
            .value(&self.api_key)
            .password(!self.show_key)
            .placeholder("Enter your API key...")
            .on_change(cx, |this, value, cx| {
                this.api_key = value;
                this.validate_api_key(cx);
            })
    )
    .child(
        Checkbox::new("show_key")
            .label("Show key")
            .checked(self.show_key)
            .on_change(cx, |this, checked, cx| {
                this.show_key = checked;
            })
    )
```

**Improvements:**
- Declarative value binding
- Built-in change callbacks
- No manual width calculations
- Better semantic structure

---

### Pattern 3: Modal Dialog Migration

**Before (egui):**
```rust
if self.show_model_picker {
    egui::Window::new("Select Model")
        .default_width(600.0)
        .default_height(400.0)
        .collapsible(false)
        .show(ctx, |ui| {
            // Model picker content
            ui.vertical(|ui| {
                for model in &self.models {
                    if ui.button(&model.name).clicked() {
                        self.select_model(model);
                        self.show_model_picker = false;
                    }
                }
            });

            if ui.button("Cancel").clicked() {
                self.show_model_picker = false;
            }
        });
}
```

**After (GPUI):**
```rust
if self.show_model_picker {
    Dialog::new("model_picker")
        .title("Select Model")
        .width(px(600.0))
        .height(px(400.0))
        .child(
            VirtualList::new(self.models.len(), |index, cx| {
                let model = &self.models[index];
                Button::new(&model.id)
                    .label(&model.name)
                    .on_click(cx, |this, cx| {
                        this.select_model(model.clone(), cx);
                        this.show_model_picker = false;
                    })
            })
        )
        .actions(vec![
            Button::new("cancel")
                .label("Cancel")
                .on_click(cx, |this, cx| {
                    this.show_model_picker = false;
                })
        ])
        .render(cx)
}
```

**Improvements:**
- Structured dialog API
- Built-in action buttons area
- Virtual list for performance
- Better modal behavior

---

### Pattern 4: Message List Migration (Critical)

**Before (egui - Full Render):**
```rust
egui::ScrollArea::vertical()
    .id_source("chat_messages")
    .auto_shrink([false, false])
    .stick_to_bottom(true)
    .show(ui, |ui| {
        for (i, message) in self.messages.iter().enumerate() {
            ui.horizontal(|ui| {
                // Render avatar
                ui.label(if message.role == "user" { "👤" } else { "🤖" });

                // Render message content
                ui.vertical(|ui| {
                    ui.label(&message.content);
                    ui.label(format!("Tokens: {}", message.tokens));
                });
            });

            if i < self.messages.len() - 1 {
                ui.separator();
            }
        }
    });
```

**Performance Issue:** With 1000 messages, renders ~1000 elements every frame = 50ms

**After (GPUI - Virtualized):**
```rust
VirtualList::new(cx, self.messages.len(), |index, cx| {
    let message = &self.messages[index];

    MessageBubble::new(message)
        .show_avatar(true)
        .show_tokens(true)
        .render(cx)
})
.stick_to_bottom(true)
.render(cx)
```

**Performance Improvement:** Only renders ~20 visible messages = <5ms per frame

**Improvements:**
- 90% render time reduction for large chats
- Smooth scrolling
- Lower memory usage
- Component reuse

---

### Pattern 5: State Management

**Before (egui - Local State):**
```rust
struct OverlookApp {
    // State scattered throughout
    current_view: ViewMode,
    messages: Vec<Message>,
    api_key: String,
    theme: Theme,
    // ... 30+ fields
}

impl OverlookApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Manual state updates everywhere
        if self.send_button_clicked {
            self.messages.push(new_message);
            self.send_button_clicked = false;
        }
    }
}
```

**After (GPUI - Structured State):**
```rust
// Global app state
struct AppState {
    current_view: Model<ViewMode>,
    chats: Model<ChatManager>,
    settings: Model<Settings>,
    theme: Model<Theme>,
}

// View components
struct ChatView {
    state: Model<ChatState>,
}

impl ChatView {
    fn send_message(&mut self, cx: &mut ViewContext<Self>) {
        self.state.update(cx, |state, cx| {
            state.messages.push(new_message);
            cx.notify(); // Trigger re-render
        });
    }
}
```

**Improvements:**
- Structured state management
- Reactive updates
- Better separation of concerns
- Easier testing

---

## Dependency Changes

### Cargo.toml Changes

**Remove:**
```toml
[dependencies]
eframe = "0.31.0"
egui = "0.31.0"
egui_extras = { version = "0.31.0", features = ["image"] }
egui-twemoji = "0.7.1"
```

**Add:**
```toml
[dependencies]
gpui = "0.1"
gpui-component = { git = "https://github.com/longbridge/gpui-component" }

# For emoji support (find GPUI-compatible solution or render as Unicode)
# May need custom emoji rendering or use Unicode directly
```

**Keep (Compatible):**
```toml
[dependencies]
pulldown-cmark = "0.12"          # Markdown parsing
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
anyhow = "1.0"
chrono = "0.4"
base64 = "0.22"
image = "0.25"
directories = "5.0"
futures = "0.3"
```

### Platform-Specific Dependencies

**macOS:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
# GPUI handles macOS-specific requirements
```

**Linux:**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
# May need X11/Wayland support (check GPUI docs)
```

**Windows:**
```toml
[target.'cfg(target_os = "windows")'.dependencies]
# GPUI handles Windows-specific requirements
```

### Build Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.dev]
opt-level = 1  # Faster dev builds
```

---

## Risk Assessment & Mitigation

### Risk Matrix

| Risk | Likelihood | Impact | Severity | Mitigation Strategy |
|------|-----------|---------|----------|---------------------|
| **Framework incompatibilities** | Medium | High | 🔴 High | Test each component early; maintain egui branch |
| **Performance regressions** | Low | High | 🟡 Medium | Benchmark continuously; optimize early |
| **Data migration issues** | Low | Medium | 🟡 Medium | Extensive testing; maintain DB compatibility |
| **Timeline overrun** | High | Medium | 🟡 Medium | Buffer time; prioritize critical features |
| **API breaking changes** | Medium | Medium | 🟡 Medium | Pin dependency versions; monitor gpui-component releases |
| **Platform-specific bugs** | Medium | Medium | 🟡 Medium | Test on all platforms early; use CI/CD |
| **Loss of functionality** | Low | High | 🔴 High | Comprehensive feature checklist; testing matrix |
| **User experience regression** | Medium | High | 🔴 High | UX testing; collect feedback early |
| **Team unfamiliarity with GPUI** | High | Medium | 🟡 Medium | Training; documentation; pair programming |

### Mitigation Strategies

#### 1. Framework Incompatibilities

**Risk:** GPUI or gpui-component may have limitations not present in egui

**Mitigation:**
- Create proof-of-concept for each major feature before full migration
- Test complex components (markdown rendering, streaming) early
- Maintain `main` branch with working egui version
- Document all workarounds and limitations
- Engage with GPUI/gpui-component community for support

**Rollback Plan:**
- If critical blocker found, abandon migration
- Revert to egui on `main` branch
- Document findings for future attempt

#### 2. Performance Regressions

**Risk:** New framework may perform worse than expected

**Mitigation:**
- Establish performance baselines before migration
- Continuous benchmarking during development
- Use profiling tools (flamegraph, perf)
- Implement virtualization early
- Monitor memory usage

**Benchmarks:**
```bash
# Run benchmarks
cargo bench

# Profile memory
cargo instruments --template "Time Profiler"

# Check binary size
ls -lh target/release/overlook
```

#### 3. Timeline Overrun

**Risk:** Migration takes longer than 2-4 weeks

**Mitigation:**
- Build in 20% buffer time (add 3-5 days)
- Focus on MVP first, polish later
- Daily progress tracking
- Weekly checkpoint reviews
- Parallel development where possible (multiple devs)

**Contingency:**
- Reduce scope: Skip advanced features (context menus, rich tooltips)
- Keep minimal viable feature set
- Schedule Part 2 migration for remaining features

#### 4. Data Migration Issues

**Risk:** Database schema changes required

**Mitigation:**
- **No database schema changes planned** - migration is UI-only
- Test database read/write early
- Verify chat history, settings, projects all load correctly
- Backup database before testing

#### 5. Loss of Functionality

**Risk:** Features work differently or are missing in GPUI

**Mitigation:**
- Create comprehensive feature checklist (see Testing section)
- Test each feature as it's migrated
- Document any feature changes
- Get user feedback on UX changes

**Feature Checklist:** (Track during migration)
- [ ] Send/receive messages
- [ ] Streaming responses
- [ ] Markdown rendering (all elements)
- [ ] Code syntax highlighting
- [ ] Image attachments
- [ ] PDF attachments
- [ ] Text file attachments
- [ ] Model selection
- [ ] Provider selection
- [ ] API key management
- [ ] Theme switching
- [ ] Font size adjustment
- [ ] Chat creation/deletion/rename
- [ ] Project management
- [ ] Export to markdown
- [ ] Search functionality
- [ ] Token usage display
- [ ] Web search integration
- [ ] Settings persistence

---

## Timeline & Milestones

### Week 1: Foundation

| Day | Phase | Tasks | Hours | Milestone |
|-----|-------|-------|-------|-----------|
| **Mon** | Phase 0 | Setup, branching, GPUI hello world | 8h | ✓ GPUI builds |
| **Tue** | Phase 1 | Window management, app structure | 8h | ✓ Window opens |
| **Wed** | Phase 1-2 | Theme system, basic components | 8h | ✓ Buttons work |
| **Thu** | Phase 2 | Input, Dialog, Select components | 8h | ✓ Forms work |
| **Fri** | Phase 3 | Settings view migration | 8h | ✓ Settings complete |

**Week 1 Goal:** Settings view fully migrated and functional

---

### Week 2: Core Features

| Day | Phase | Tasks | Hours | Milestone |
|-----|-------|-------|-------|-----------|
| **Mon** | Phase 4 | Chat view structure, message list | 8h | ✓ Messages render |
| **Tue** | Phase 4 | Markdown rendering, message bubbles | 8h | ✓ Markdown works |
| **Wed** | Phase 4 | Input area, attachments, streaming | 8h | ✓ Can send messages |
| **Thu** | Phase 4 | Sidebar, chat management | 8h | ✓ Chat CRUD works |
| **Fri** | Phase 4 | Model picker, provider selection | 8h | ✓ Model selection works |

**Week 2 Goal:** Chat interface fully functional

---

### Week 3: Polish & Remaining Views

| Day | Phase | Tasks | Hours | Milestone |
|-----|-------|-------|-------|-----------|
| **Mon** | Phase 5 | Export view migration | 6h | ✓ Export works |
| **Tue** | Phase 5 | Organize view, project management | 6h | ✓ Projects work |
| **Wed** | Phase 5 | Context menus, tooltips, notifications | 8h | ✓ UX enhancements |
| **Thu** | Phase 5-6 | Keyboard shortcuts, accessibility | 6h | ✓ Keyboard nav |
| **Fri** | Phase 6 | Testing, bug fixing | 8h | ✓ Tests passing |

**Week 3 Goal:** All features migrated, UX polished

---

### Week 4: Testing & Release

| Day | Phase | Tasks | Hours | Milestone |
|-----|-------|-------|-------|-----------|
| **Mon** | Phase 6 | Performance testing, optimization | 8h | ✓ Performance targets |
| **Tue** | Phase 6 | Cross-platform testing | 8h | ✓ All platforms work |
| **Wed** | Phase 6 | Bug fixing, edge cases | 8h | ✓ Critical bugs fixed |
| **Thu** | Phase 6 | Documentation, cleanup | 6h | ✓ Docs updated |
| **Fri** | Release | Final testing, merge to main | 6h | ✓ **MIGRATION COMPLETE** |

**Week 4 Goal:** Production-ready release

---

### Milestone Checklist

- [ ] **Milestone 1:** GPUI builds and window opens (Day 1)
- [ ] **Milestone 2:** Settings view complete (Day 5)
- [ ] **Milestone 3:** Can send/receive messages (Day 10)
- [ ] **Milestone 4:** All views migrated (Day 16)
- [ ] **Milestone 5:** Performance targets met (Day 19)
- [ ] **Milestone 6:** Ready for production (Day 20)

### Critical Path

```
Setup (1d) → Infrastructure (1.5d) → Core Components (2d) →
Settings (1d) → Chat Interface (5d) → Remaining Views (2d) →
Testing & Optimization (3d) → Release (1d)
```

**Total:** 16.5 days (3.3 weeks) + buffer = **4 weeks**

---

## Testing Strategy

### Testing Phases

#### 1. Unit Testing (Ongoing)

**Component Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_bubble_renders() {
        let message = Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tokens: 10,
        };

        let bubble = MessageBubble::new(&message);
        // Assert rendering properties
    }
}
```

**Test Coverage Goals:**
- Core components: 80%+
- State management: 90%+
- Data layer: 90%+

#### 2. Integration Testing (After Each Phase)

**Feature Tests:**
- [ ] Send message → Receive response
- [ ] Switch providers → Settings persist
- [ ] Create chat → Appears in sidebar
- [ ] Export chat → File created correctly
- [ ] Upload image → Preview renders
- [ ] Stream response → Animates correctly
- [ ] Change theme → UI updates immediately

#### 3. Performance Testing (Phase 6)

**Benchmarks:**

| Test Case | Load | Target | Measurement |
|-----------|------|--------|-------------|
| Render 1000 messages | 1000 msgs | <16ms/frame | Frame time |
| Scroll large chat | 1000 msgs | 60fps | Frame rate |
| Load conversation | 1000 msgs | <500ms | Load time |
| Send message | 1 msg | <100ms | Response time |
| Switch chat | 2 chats | <200ms | Transition time |
| Startup time | Cold start | <1s | Launch time |
| Memory usage | 10 chats, 1000 msgs each | <300MB | RSS |

**Tools:**
```bash
# Frame time measurement
cargo run --release -- --benchmark

# Memory profiling
cargo instruments --template "Allocations"

# CPU profiling
cargo flamegraph
```

#### 4. Cross-Platform Testing (Phase 6)

**Platforms:**
- [ ] macOS 13+ (Ventura, Sonoma)
- [ ] Ubuntu 22.04 LTS
- [ ] Windows 10/11

**Test Matrix:**

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| Launch app | ⏳ | ⏳ | ⏳ |
| Send message | ⏳ | ⏳ | ⏳ |
| Render markdown | ⏳ | ⏳ | ⏳ |
| Upload files | ⏳ | ⏳ | ⏳ |
| Export chat | ⏳ | ⏳ | ⏳ |
| Theme switching | ⏳ | ⏳ | ⏳ |
| Keyboard shortcuts | ⏳ | ⏳ | ⏳ |

#### 5. User Acceptance Testing (Final Week)

**Test Scenarios:**
1. **New User Flow:**
   - Install app
   - Enter API key
   - Send first message
   - Try different models
   - Export conversation

2. **Power User Flow:**
   - Create multiple projects
   - Manage 50+ chats
   - Handle long conversations (500+ messages)
   - Use keyboard shortcuts extensively
   - Switch themes frequently

3. **Edge Cases:**
   - Very long messages (10,000+ characters)
   - Messages with complex markdown
   - Large file attachments (10MB+)
   - Rapid message sending
   - Network errors/timeouts

### Bug Tracking

**Severity Levels:**
- **P0 - Critical:** App crashes, data loss → Fix immediately
- **P1 - High:** Feature broken, major UX issue → Fix before release
- **P2 - Medium:** Minor bug, workaround exists → Fix if time permits
- **P3 - Low:** Polish, nice-to-have → Defer to post-release

**Bug Template:**
```markdown
## Bug Title
**Severity:** P0/P1/P2/P3
**Platform:** macOS/Linux/Windows/All
**Reproducible:** Always/Sometimes/Rare

### Steps to Reproduce:
1.
2.
3.

### Expected Behavior:

### Actual Behavior:

### Screenshots/Logs:

### Fix:
```

---

## Rollback Plan

### When to Rollback

**Rollback Triggers:**
1. **Critical blocker found** that cannot be resolved in reasonable time
2. **Performance significantly worse** than egui baseline (>50% regression)
3. **Platform incompatibility** discovered (doesn't build on Windows/Linux)
4. **Timeline overrun >50%** (>6 weeks)
5. **Loss of critical feature** with no workaround

### Rollback Procedure

**Step 1: Stop Development**
- Freeze `gpui-migration` branch
- Document blockers/issues found

**Step 2: Assess Damage**
- Review work completed
- Identify salvageable components
- Document learnings

**Step 3: Return to egui**
- Merge any non-framework changes back to `main` (data layer improvements, bug fixes)
- Continue development on `main` branch
- Archive `gpui-migration` branch

**Step 4: Post-Mortem**
- Document what went wrong
- Identify what was learned
- Determine if future migration is viable
- Estimate timeline for retry (6-12 months?)

### Salvageable Work

Even if full migration fails, these improvements can be ported back to egui:

1. **Component abstractions** - Create egui wrapper library
2. **State management improvements** - Better structured state
3. **Performance optimizations** - Apply optimization techniques
4. **Code organization** - Better file structure
5. **Bug fixes** - Any bugs fixed during migration
6. **Documentation** - Improved architecture docs

### Communication Plan

**Internal:**
- Daily standups on progress
- Weekly milestone reviews
- Immediate escalation of blockers

**External (if applicable):**
- Blog post on migration experience
- Open source learnings
- Contribute back to gpui-component

---

## Post-Migration Benefits

### Quantitative Improvements

| Metric | Before (egui) | After (GPUI) | Improvement |
|--------|--------------|--------------|-------------|
| **UI Code Size** | ~3,200 lines | ~1,800 lines | 44% reduction |
| **Render Time (1000 msgs)** | ~50ms/frame | ~5ms/frame | 90% faster |
| **Scroll Performance** | Stutters | 60fps | Smooth |
| **Memory Usage** | ~200MB | ~140MB | 30% less |
| **Startup Time** | ~500ms | ~350ms | 30% faster |
| **Binary Size** | ~15MB | ~12MB | 20% smaller |
| **Available Themes** | 2 | 20+ | 10x more |
| **Component Count** | ~15 custom | 60+ library | 4x more |

### Qualitative Improvements

1. **Developer Experience:**
   - Cleaner component API
   - Less boilerplate code
   - Better code organization
   - Easier to add new features
   - Type-safe UI construction

2. **User Experience:**
   - Smoother animations
   - Better performance in large chats
   - More professional appearance
   - Rich context menus
   - Toast notifications
   - Better keyboard navigation
   - Accessibility improvements

3. **Maintainability:**
   - Component library vs. custom code
   - Active upstream development (Zed team)
   - Community support
   - Better documentation
   - Easier onboarding for new developers

4. **Future Features Enabled:**
   - Advanced code editor (LSP support)
   - Split-screen chat comparison
   - Drag-and-drop chat organization
   - Rich tooltips with images/formatting
   - Better table rendering for data
   - Calendar-based chat filtering
   - Chart/graph rendering

### New Capabilities

**Features Not Possible (or very difficult) in egui:**

1. **Context Menus:** Right-click anywhere for actions
2. **Rich Tooltips:** Formatted tooltips with icons and multiple lines
3. **Toast Notifications:** Non-blocking notifications
4. **Virtual Lists:** Performance with 10,000+ items
5. **Advanced Tables:** Sorting, filtering, virtualization
6. **LSP Integration:** Code completion in chat (future)
7. **Drag and Drop:** Reorder chats, move to projects
8. **Split Views:** Side-by-side chat comparison
9. **Advanced Theming:** 20+ themes, custom theme creation
10. **Accessibility:** Screen reader support, keyboard navigation

### ROI Analysis

**Investment:**
- 80-120 hours development time
- Learning curve for GPUI
- Migration risk

**Return:**
- 40% less UI code to maintain
- 90% faster rendering for large datasets
- Modern component architecture
- Access to 60+ components for future features
- Better user experience
- Easier to attract contributors (modern stack)

**Break-even:** After 2-3 major feature additions that use new components, migration pays for itself in development time saved.

---

## Appendix A: GPUI Resources

### Documentation

- **GPUI GitHub:** https://github.com/zed-industries/gpui
- **gpui-component GitHub:** https://github.com/longbridge/gpui-component
- **gpui-component Docs:** https://longbridge.github.io/gpui-component/
- **Zed Editor Source:** https://github.com/zed-industries/zed (reference implementation)

### Community

- **Discord:** Zed Discord server (#gpui channel)
- **GitHub Discussions:** gpui-component discussions
- **Examples:** https://github.com/longbridge/gpui-component/tree/main/examples

### Learning Resources

1. **Study Zed codebase** - Real-world GPUI usage
2. **gpui-component storybook** - Component examples
3. **GPUI architecture docs** - Understanding the framework
4. **Rust GUI comparisons** - GPUI vs egui vs iced

---

## Appendix B: Emergency Contacts

**GPUI/gpui-component Support:**
- GitHub Issues: Report bugs/questions
- Discord: Real-time help from community
- Email: [Maintainer contact if available]

**Internal Team:**
- Lead Developer: [Name]
- UI/UX Designer: [Name]
- QA Engineer: [Name]

**Escalation Path:**
- P0 bugs → Immediate team notification
- Blocker issues → Daily review with lead
- Timeline concerns → Weekly stakeholder update

---

## Appendix C: Success Metrics

### Definition of Done

Migration is complete when:

- [ ] All features from egui version working
- [ ] No P0 or P1 bugs
- [ ] Performance targets met or exceeded
- [ ] Builds successfully on macOS, Linux, Windows
- [ ] Documentation updated
- [ ] README reflects new architecture
- [ ] Tests passing (80%+ coverage)
- [ ] Code review completed
- [ ] UX review completed
- [ ] Merged to `main` branch
- [ ] Release published

### Success Criteria

**Must Have (MVP):**
- ✓ Feature parity with egui version
- ✓ No performance regressions
- ✓ Builds on all platforms
- ✓ All critical bugs fixed

**Should Have:**
- ✓ Performance improvements (40%+)
- ✓ Code reduction (30%+)
- ✓ Context menus added
- ✓ Toast notifications added

**Nice to Have:**
- ✓ Rich tooltips
- ✓ Advanced keyboard shortcuts
- ✓ Drag and drop
- ✓ Split views

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-15 | Migration Team | Initial migration plan |

---

## Conclusion

This migration represents a significant architectural improvement for the Overlook application. While it requires substantial effort (80-120 hours), the benefits in code quality, performance, user experience, and future extensibility make it a worthwhile investment.

**Key Success Factors:**
1. Thorough preparation and testing
2. Incremental migration approach (view by view)
3. Continuous benchmarking
4. Clear rollback criteria
5. Active community engagement

**Next Steps:**
1. Review and approve this plan
2. Create `gpui-migration` branch
3. Begin Phase 0: Preparation & Setup
4. Daily progress tracking
5. Weekly checkpoint reviews

**Timeline:** 2-4 weeks (estimated completion: Early December 2025)

**Go/No-Go Decision Point:** After Phase 1 (Day 3)
- If infrastructure works smoothly → Continue
- If major blockers found → Reassess or rollback

---

**Migration Status:** 🟡 Planning Phase
**Estimated Start Date:** [To be determined]
**Estimated Completion:** [Start + 4 weeks]

---

*This document is a living plan and will be updated as the migration progresses.*
