// Markdown Renderer - Converts markdown to GPUI elements
use gpui::*;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind, Options};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style, Color as SyntectColor};
use syntect::easy::HighlightLines;
use crate::gpui_theme::GpuiTheme;

/// Render markdown content to GPUI elements
pub fn render_markdown(content: &str, theme: &GpuiTheme, font_size: f32) -> AnyElement {
    let renderer = MarkdownRenderer::new(theme.clone(), font_size);
    renderer.render(content)
}

struct MarkdownRenderer {
    theme: GpuiTheme,
    font_size: f32,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl MarkdownRenderer {
    fn new(theme: GpuiTheme, font_size: f32) -> Self {
        Self {
            theme,
            font_size,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    fn render(&self, content: &str) -> AnyElement {
        // Enable tables in markdown parsing
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(content, options);
        let events: Vec<Event> = parser.collect();

        let mut elements: Vec<AnyElement> = Vec::new();
        let mut i = 0;

        while i < events.len() {
            let (element, consumed) = self.process_event(&events[i..]);
            if let Some(el) = element {
                elements.push(el);
            }
            i += consumed.max(1);
        }

        div()
            .flex()
            .flex_col()
            .gap_1()
            .children(elements)
            .into_any_element()
    }

    fn process_event(&self, events: &[Event]) -> (Option<AnyElement>, usize) {
        if events.is_empty() {
            return (None, 0);
        }

        match &events[0] {
            Event::Start(tag) => self.process_tag(tag, &events[1..]),
            Event::Text(text) => {
                (Some(self.render_text(text.to_string())), 1)
            }
            Event::Code(code) => {
                (Some(self.render_inline_code(code.to_string())), 1)
            }
            Event::SoftBreak | Event::HardBreak => {
                (None, 1)
            }
            Event::Rule => {
                (Some(self.render_horizontal_rule()), 1)
            }
            _ => (None, 1),
        }
    }

    fn process_tag(&self, tag: &Tag, remaining: &[Event]) -> (Option<AnyElement>, usize) {
        match tag {
            Tag::Paragraph => {
                let (content, consumed) = self.collect_until_end(remaining, TagEnd::Paragraph);
                (Some(self.render_paragraph(content)), consumed + 1)
            }
            Tag::Heading { level, .. } => {
                let end_tag = match level {
                    pulldown_cmark::HeadingLevel::H1 => TagEnd::Heading(pulldown_cmark::HeadingLevel::H1),
                    pulldown_cmark::HeadingLevel::H2 => TagEnd::Heading(pulldown_cmark::HeadingLevel::H2),
                    pulldown_cmark::HeadingLevel::H3 => TagEnd::Heading(pulldown_cmark::HeadingLevel::H3),
                    pulldown_cmark::HeadingLevel::H4 => TagEnd::Heading(pulldown_cmark::HeadingLevel::H4),
                    pulldown_cmark::HeadingLevel::H5 => TagEnd::Heading(pulldown_cmark::HeadingLevel::H5),
                    pulldown_cmark::HeadingLevel::H6 => TagEnd::Heading(pulldown_cmark::HeadingLevel::H6),
                };
                let (content, consumed) = self.collect_until_end(remaining, end_tag);
                (Some(self.render_heading(*level, content)), consumed + 1)
            }
            Tag::BlockQuote(_) => {
                let (content, consumed) = self.collect_until_end(remaining, TagEnd::BlockQuote(None));
                (Some(self.render_blockquote(content)), consumed + 1)
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    CodeBlockKind::Indented => None,
                };
                let (code_content, consumed) = self.collect_code_block(remaining);
                (Some(self.render_code_block(&code_content, lang.as_deref())), consumed + 1)
            }
            Tag::List(start_num) => {
                let (items, consumed) = self.collect_list_items(remaining, start_num.is_some());
                (Some(self.render_list(items, *start_num)), consumed + 1)
            }
            Tag::Item => {
                // Items are handled by collect_list_items
                (None, 1)
            }
            Tag::Emphasis => {
                let (content, consumed) = self.collect_until_end(remaining, TagEnd::Emphasis);
                (Some(self.render_emphasis(content)), consumed + 1)
            }
            Tag::Strong => {
                let (content, consumed) = self.collect_until_end(remaining, TagEnd::Strong);
                (Some(self.render_strong(content)), consumed + 1)
            }
            Tag::Link { dest_url, .. } => {
                let (content, consumed) = self.collect_until_end(remaining, TagEnd::Link);
                (Some(self.render_link(content, dest_url.to_string())), consumed + 1)
            }
            Tag::Table(_alignments) => {
                let (table_data, consumed) = self.collect_table(remaining);
                (Some(self.render_table(table_data)), consumed + 1)
            }
            Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                // These are handled by collect_table
                (None, 1)
            }
            Tag::Strikethrough => {
                let (content, consumed) = self.collect_until_end(remaining, TagEnd::Strikethrough);
                (Some(self.render_strikethrough(content)), consumed + 1)
            }
            _ => (None, 1),
        }
    }

    fn collect_until_end(&self, events: &[Event], end_tag: TagEnd) -> (String, usize) {
        let mut content = String::new();
        let mut consumed = 0;

        for event in events {
            consumed += 1;
            match event {
                Event::End(tag) if *tag == end_tag => break,
                Event::Text(text) => content.push_str(text),
                Event::Code(code) => {
                    content.push('`');
                    content.push_str(code);
                    content.push('`');
                }
                Event::SoftBreak | Event::HardBreak => content.push(' '),
                _ => {}
            }
        }

        (content, consumed)
    }

    fn collect_code_block(&self, events: &[Event]) -> (String, usize) {
        let mut content = String::new();
        let mut consumed = 0;

        for event in events {
            consumed += 1;
            match event {
                Event::End(TagEnd::CodeBlock) => break,
                Event::Text(text) => content.push_str(text),
                _ => {}
            }
        }

        (content, consumed)
    }

    fn collect_list_items(&self, events: &[Event], _ordered: bool) -> (Vec<String>, usize) {
        let mut items = Vec::new();
        let mut current_item = String::new();
        let mut consumed = 0;
        let mut depth = 0;

        for event in events {
            consumed += 1;
            match event {
                Event::Start(Tag::Item) => {
                    depth += 1;
                    if depth == 1 {
                        current_item.clear();
                    }
                }
                Event::End(TagEnd::Item) => {
                    depth -= 1;
                    if depth == 0 {
                        items.push(current_item.trim().to_string());
                        current_item.clear();
                    }
                }
                Event::End(TagEnd::List(_)) => break,
                Event::Text(text) if depth > 0 => current_item.push_str(text),
                Event::Code(code) if depth > 0 => {
                    current_item.push('`');
                    current_item.push_str(code);
                    current_item.push('`');
                }
                Event::SoftBreak | Event::HardBreak if depth > 0 => current_item.push(' '),
                _ => {}
            }
        }

        (items, consumed)
    }

    /// Collect table data as rows of cells
    fn collect_table(&self, events: &[Event]) -> (Vec<Vec<String>>, usize) {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        let mut consumed = 0;
        let mut in_cell = false;

        for event in events {
            consumed += 1;
            match event {
                Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                    current_row.clear();
                }
                Event::Start(Tag::TableCell) => {
                    current_cell.clear();
                    in_cell = true;
                }
                Event::End(TagEnd::TableCell) => {
                    current_row.push(current_cell.trim().to_string());
                    current_cell.clear();
                    in_cell = false;
                }
                Event::End(TagEnd::TableHead) | Event::End(TagEnd::TableRow) => {
                    if !current_row.is_empty() {
                        rows.push(current_row.clone());
                    }
                    current_row.clear();
                }
                Event::End(TagEnd::Table) => break,
                Event::Text(text) if in_cell => current_cell.push_str(text),
                Event::Code(code) if in_cell => {
                    current_cell.push('`');
                    current_cell.push_str(code);
                    current_cell.push('`');
                }
                Event::SoftBreak | Event::HardBreak if in_cell => current_cell.push(' '),
                _ => {}
            }
        }

        (rows, consumed)
    }

    // Rendering functions

    fn render_text(&self, text: String) -> AnyElement {
        div()
            .text_size(px(self.font_size))
            .text_color(self.theme.text_secondary)
            .child(text)
            .into_any_element()
    }

    fn render_paragraph(&self, content: String) -> AnyElement {
        // Check if content has inline code
        if content.contains('`') {
            self.render_mixed_content(&content)
        } else {
            div()
                .text_size(px(self.font_size))
                .text_color(self.theme.text_secondary)
                .child(content)
                .into_any_element()
        }
    }

    fn render_mixed_content(&self, content: &str) -> AnyElement {
        // Split by inline code markers and render appropriately
        let mut elements: Vec<AnyElement> = Vec::new();
        let mut in_code = false;
        let mut current = String::new();

        for c in content.chars() {
            if c == '`' {
                if !current.is_empty() {
                    if in_code {
                        elements.push(self.render_inline_code(current.clone()));
                    } else {
                        elements.push(
                            div()
                                .text_size(px(self.font_size))
                                .text_color(self.theme.text_secondary)
                                .child(current.clone())
                                .into_any_element()
                        );
                    }
                    current.clear();
                }
                in_code = !in_code;
            } else {
                current.push(c);
            }
        }

        // Handle remaining content
        if !current.is_empty() {
            if in_code {
                elements.push(self.render_inline_code(current));
            } else {
                elements.push(
                    div()
                        .text_size(px(self.font_size))
                        .text_color(self.theme.text_secondary)
                        .child(current)
                        .into_any_element()
                );
            }
        }

        div()
            .flex()
            .flex_wrap()
            .gap_0()
            .children(elements)
            .into_any_element()
    }

    fn render_heading(&self, level: pulldown_cmark::HeadingLevel, content: String) -> AnyElement {
        let (size_offset, weight) = match level {
            pulldown_cmark::HeadingLevel::H1 => (6.0, FontWeight::BOLD),
            pulldown_cmark::HeadingLevel::H2 => (4.0, FontWeight::BOLD),
            pulldown_cmark::HeadingLevel::H3 => (2.0, FontWeight::SEMIBOLD),
            pulldown_cmark::HeadingLevel::H4 => (1.0, FontWeight::SEMIBOLD),
            pulldown_cmark::HeadingLevel::H5 => (0.0, FontWeight::MEDIUM),
            pulldown_cmark::HeadingLevel::H6 => (-1.0, FontWeight::MEDIUM),
        };

        div()
            .text_size(px(self.font_size + size_offset))
            .font_weight(weight)
            .text_color(self.theme.text_primary)
            .py_1()
            .child(content)
            .into_any_element()
    }

    fn render_inline_code(&self, code: String) -> AnyElement {
        div()
            .px_1()
            .rounded_sm()
            .bg(self.theme.surface)
            .text_size(px(self.font_size - 1.0))
            .font_family("monospace")
            .text_color(self.theme.accent)
            .child(code)
            .into_any_element()
    }

    fn render_code_block(&self, code: &str, lang: Option<&str>) -> AnyElement {
        // Try to get syntax highlighting
        let highlighted = self.highlight_code(code, lang);

        div()
            .w_full()
            .p_3()
            .my_2()
            .rounded_md()
            .bg(self.theme.surface)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .font_family("monospace")
                    .text_size(px(self.font_size - 1.0))
                    .children(highlighted)
            )
            .into_any_element()
    }

    fn highlight_code(&self, code: &str, lang: Option<&str>) -> Vec<AnyElement> {
        // Determine syntax
        let syntax = lang
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Choose theme based on app theme
        let theme_name = if self.is_dark_theme() {
            "base16-ocean.dark"
        } else {
            "base16-ocean.light"
        };

        let theme = self.theme_set.themes.get(theme_name)
            .unwrap_or_else(|| self.theme_set.themes.values().next().unwrap());

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut lines: Vec<AnyElement> = Vec::new();

        for line in code.lines() {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    let line_elements: Vec<AnyElement> = ranges.iter().map(|(style, text)| {
                        self.styled_text(text, style)
                    }).collect();

                    lines.push(
                        div()
                            .flex()
                            .children(line_elements)
                            .into_any_element()
                    );
                }
                Err(_) => {
                    // Fallback to plain text
                    lines.push(
                        div()
                            .text_color(self.theme.text_secondary)
                            .child(line.to_string())
                            .into_any_element()
                    );
                }
            }
        }

        lines
    }

    fn styled_text(&self, text: &str, style: &Style) -> AnyElement {
        let fg = syntect_to_gpui_color(style.foreground);

        div()
            .text_color(fg)
            .child(text.to_string())
            .into_any_element()
    }

    fn is_dark_theme(&self) -> bool {
        // Check if background is dark by looking at lightness
        let l = self.theme.background.l;
        l < 0.5
    }

    fn render_blockquote(&self, content: String) -> AnyElement {
        div()
            .flex()
            .pl_3()
            .border_l_2()
            .border_color(self.theme.accent)
            .child(
                div()
                    .text_size(px(self.font_size))
                    .text_color(self.theme.text_muted)
                    .italic()
                    .child(content)
            )
            .into_any_element()
    }

    fn render_list(&self, items: Vec<String>, start_num: Option<u64>) -> AnyElement {
        let elements: Vec<AnyElement> = items.iter().enumerate().map(|(i, item)| {
            let marker = if let Some(start) = start_num {
                format!("{}. ", start + i as u64)
            } else {
                "• ".to_string()
            };

            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .text_size(px(self.font_size))
                        .text_color(self.theme.text_muted)
                        .child(marker)
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(self.font_size))
                        .text_color(self.theme.text_secondary)
                        .child(item.clone())
                )
                .into_any_element()
        }).collect();

        div()
            .flex()
            .flex_col()
            .pl_2()
            .gap_1()
            .children(elements)
            .into_any_element()
    }

    fn render_emphasis(&self, content: String) -> AnyElement {
        div()
            .text_size(px(self.font_size))
            .text_color(self.theme.text_secondary)
            .italic()
            .child(content)
            .into_any_element()
    }

    fn render_strong(&self, content: String) -> AnyElement {
        div()
            .text_size(px(self.font_size))
            .text_color(self.theme.text_secondary)
            .font_weight(FontWeight::BOLD)
            .child(content)
            .into_any_element()
    }

    fn render_link(&self, content: String, url: String) -> AnyElement {
        let url_clone = url.clone();
        div()
            .id(SharedString::from(format!("link_{}", url)))
            .text_size(px(self.font_size))
            .text_color(self.theme.accent)
            .cursor_pointer()
            .hover(|s| s.underline())
            .child(content)
            .on_click(move |_, _, _| {
                let _ = open::that(&url_clone);
            })
            .into_any_element()
    }

    fn render_horizontal_rule(&self) -> AnyElement {
        div()
            .w_full()
            .h(px(1.0))
            .my_2()
            .bg(self.theme.border)
            .into_any_element()
    }

    fn render_table(&self, rows: Vec<Vec<String>>) -> AnyElement {
        if rows.is_empty() {
            return div().into_any_element();
        }

        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        // Calculate column widths based on content (approximate chars * font size factor)
        let char_width = self.font_size * 0.6; // Approximate character width
        let mut col_widths: Vec<f32> = vec![0.0; num_cols];

        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    let cell_width = cell.chars().count() as f32 * char_width + 16.0; // +16 for padding
                    col_widths[i] = col_widths[i].max(cell_width).max(60.0); // min 60px
                }
            }
        }

        // Build table rows as elements
        let mut row_elements: Vec<AnyElement> = Vec::new();

        for (row_idx, row) in rows.iter().enumerate() {
            let is_header = row_idx == 0;
            let col_widths = col_widths.clone();

            // Build cells for this row
            let cell_elements: Vec<AnyElement> = (0..num_cols).map(|i| {
                let cell_content = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let width = col_widths.get(i).copied().unwrap_or(100.0);

                let mut cell = div()
                    .min_w(px(width))
                    .max_w(px(width.max(300.0))) // Cap max width
                    .px_2()
                    .py_1()
                    .border_r_1()
                    .border_color(self.theme.border)
                    .text_size(px(self.font_size - 1.0))
                    .text_color(self.theme.text_secondary)
                    .overflow_hidden()
                    .child(cell_content.to_string());

                if is_header {
                    cell = cell.font_weight(FontWeight::SEMIBOLD);
                }

                cell.into_any_element()
            }).collect();

            let mut row_div = div()
                .flex()
                .border_b_1()
                .border_color(self.theme.border)
                .children(cell_elements);

            // Header row gets a thicker bottom border
            if is_header {
                row_div = row_div.border_b_2();
            }

            row_elements.push(row_div.into_any_element());
        }

        div()
            .my_2()
            .border_1()
            .border_color(self.theme.border)
            .rounded_md()
            .overflow_hidden()
            .children(row_elements)
            .into_any_element()
    }

    fn render_strikethrough(&self, content: String) -> AnyElement {
        div()
            .text_size(px(self.font_size))
            .text_color(self.theme.text_muted)
            .line_through()
            .child(content)
            .into_any_element()
    }
}

fn syntect_to_gpui_color(color: SyntectColor) -> Hsla {
    // Convert syntect RGBA to GPUI Hsla
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;
    let a = color.a as f32 / 255.0;

    // Convert RGB to HSL
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        // Achromatic
        hsla(0.0, 0.0, l, a)
    } else {
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        hsla(h / 6.0, s, l, a)
    }
}
