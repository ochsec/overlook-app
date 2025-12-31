# OpenCode-Style Chat View Redesign

## Overview

Redesign the Overlook chat view to match OpenCode's visual style, implementing session turns with auto-generated titles and turn navigation.

**Note:** Sticky headers are not currently supported by GPUI (the UI framework), so turn headers are rendered as regular headers at the top of each turn. If GPUI adds sticky positioning support in the future, this can be updated.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Sticky header background | Seamless (same as main background) |
| Assistant model header | Keep `--- model-name (Provider) ---` |
| Turn separation | Gap only (vertical spacing) |
| Welcome message | Keep current style |
| Input style | Keep terminal style with `>` prefix |
| User message style | Keep terminal style with `>` prefix |

## What's Changing

1. **Session turns with headers** - Auto-generated titles for each user message + response pair
2. **Visual grouping** - Messages grouped into "turns" (user message + assistant response)
3. **Gap-based separation** - Vertical spacing between turns
4. **Turn navigation** - Navigation bar to paginate through turns with [<] and [>] buttons

## What's Staying the Same

- Terminal-style input with `>` prefix
- Terminal-style user messages with `>` prefix
- Model header on assistant messages (`--- model-name (Provider) ---`)
- Welcome message when no chat exists
- Streaming behavior and status indicators
- Edit functionality for user messages
- Search results display (collapsible)
- Attachment chips

## Files to Modify

### 1. `src/types.rs` (Minor)
- Add optional `title: Option<String>` field to `Message` struct
- This allows caching auto-generated titles

### 2. `src/views/chat_view.rs` (Major)
- Add `Turn` struct to group user message + responses
- Add `generate_turn_title()` helper function
- Add `group_messages_into_turns()` function
- Add `render_turn()` function with sticky header
- Modify `render_messages_area()` to use turn-based rendering
- Keep existing edit functionality, search results, and streaming behavior

### 3. `src/components/message_bubble.rs` (No changes)
- Keep as-is since we're keeping the model header on assistant messages

### 4. `src/components/chat_input.rs` (No changes)
- Already supports multi-line input with `auto_grow(1, 10)`

## Implementation Details

### Turn Data Structure

```rust
/// A "turn" groups a user message with its associated responses
struct Turn<'a> {
    user_message: &'a Message,
    user_message_index: usize,
    search_message: Option<&'a Message>,  // System message with search results
    assistant_messages: Vec<(&'a Message, usize)>,  // (message, index)
}
```

### Title Generation

```rust
/// Generate a short title from user message content
fn generate_turn_title(content: &str) -> String {
    let content = content.trim();
    if content.is_empty() {
        return "New message".to_string();
    }
    
    // Find first sentence end or take first 60 chars
    let sentence_end = content.find(|c| c == '.' || c == '!' || c == '?')
        .map(|i| i + 1);
    
    let end = match sentence_end {
        Some(i) if i <= 60 => i,
        _ => content.chars().take(60).collect::<String>().len(),
    };
    
    let title: String = content.chars().take(end).collect();
    
    if content.len() > end {
        format!("{}...", title.trim_end())
    } else {
        title
    }
}
```

### Message Grouping Logic

```rust
fn group_messages_into_turns<'a>(messages: &'a [Message]) -> Vec<Turn<'a>> {
    let mut turns: Vec<Turn<'a>> = Vec::new();
    let mut current_turn: Option<Turn<'a>> = None;
    
    for (idx, msg) in messages.iter().enumerate() {
        match msg.role.as_str() {
            "user" => {
                // Save previous turn if exists
                if let Some(turn) = current_turn.take() {
                    turns.push(turn);
                }
                // Start new turn
                current_turn = Some(Turn {
                    user_message: msg,
                    user_message_index: idx,
                    search_message: None,
                    assistant_messages: Vec::new(),
                });
            }
            "system" if msg.search_results.is_some() => {
                // Add search results to current turn
                if let Some(ref mut turn) = current_turn {
                    turn.search_message = Some(msg);
                }
            }
            "assistant" => {
                // Add assistant message to current turn
                if let Some(ref mut turn) = current_turn {
                    turn.assistant_messages.push((msg, idx));
                }
            }
            _ => {}
        }
    }
    
    // Don't forget the last turn
    if let Some(turn) = current_turn {
        turns.push(turn);
    }
    
    turns
}
```

## Visual Result

```
+-------------------------------------------------------------+
| ChatHeader                                                  |
+-------------------------------------------------------------+
|                                                             |
| What is the weather like in Tokyo?              <- header   |
| ----------------------------------------------------------- |
| > What is the weather like in Tokyo?        [edit] [copy]   |
|                                                             |
| # Web Search (3 results) [+]                                |
|                                                             |
| --- gpt-4 (OpenRouter) ---                                  |
| Based on the search results...                              |
|                                                             |
|                          ^ gap ^                            |
|                                                             |
| Tell me more about Shibuya                      <- header   |
| ----------------------------------------------------------- |
| > Tell me more about Shibuya                [edit] [copy]   |
|                                                             |
| --- gpt-4 (OpenRouter) ---                                  |
| Shibuya is one of Tokyo's most famous districts...          |
|                                                             |
+-------------------------------------------------------------+
| [<] 2/2 [>]              Tell me more about Shibuya         | <- Turn navigation
+-------------------------------------------------------------+
| > [input area]                          [attach] [send]     |
+-------------------------------------------------------------+
```
+-------------------------------------------------------------+
| ChatHeader (provider/model picker)                          |
+-------------------------------------------------------------+
|                                                             |
| What is the weather like in Tokyo?              <- sticky   |
| ----------------------------------------------------------- |
| > What is the weather like in Tokyo?        [edit] [copy]   |
|                                                             |
| # Web Search (3 results) [+]                                |
|                                                             |
| --- gpt-4 (OpenRouter) ---                                  |
| Based on the search results, Tokyo is currently experiencing|
| mild weather with temperatures around 18C...                |
|                                                             |
|                          ^ gap_4 v                          |
|                                                             |
| Tell me more about Shibuya                      <- sticky   |
| ----------------------------------------------------------- |
| > Tell me more about Shibuya                [edit] [copy]   |
|                                                             |
| --- gpt-4 (OpenRouter) ---                                  |
| Shibuya is one of Tokyo's most famous districts...          |
|                                                             |
| # streaming...                                              |
+-------------------------------------------------------------+
| > [input area]                          [attach] [send]     |
+-------------------------------------------------------------+
```

## Implementation Order

1. Add `title` field to Message (`types.rs`)
2. Create Turn struct and title generator (`chat_view.rs`)
3. Implement `group_messages_into_turns()` function
4. Implement `render_turn()` with sticky headers
5. Update `render_messages_area()` to use turn-based rendering
6. Test and verify sticky headers work correctly

## Estimated Time

~1.5-2 hours
