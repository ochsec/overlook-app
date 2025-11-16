use egui::{Color32, FontId, RichText, Ui};
use egui_twemoji::EmojiLabel;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Renders markdown content in an egui UI
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render markdown content into the given UI
    pub fn show(&self, ui: &mut Ui, markdown: &str) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown, options);
        let mut renderer = EventRenderer::new(ui);
        renderer.render(parser);
    }
}

/// Internal event renderer that walks the markdown AST
struct EventRenderer<'a> {
    ui: &'a mut Ui,
    current_text: String,
    in_heading: Option<u8>,
    in_paragraph: bool,
    in_emphasis: bool,
    in_strong: bool,
    in_strikethrough: bool,
    in_code: bool,
    code_block_text: String,
    code_block_language: Option<String>,
    in_list: bool,
    list_item_number: Option<usize>,
    list_item_start: bool, // Track if we're at the start of a list item
    in_blockquote: bool,
    blockquote_text: Vec<String>,
    link_url: Option<String>,
    table_state: TableState,
}

#[derive(Default)]
struct TableState {
    in_table: bool,
    in_table_head: bool,
    headers: Vec<String>,
    current_row: Vec<String>,
    rows: Vec<Vec<String>>,
    current_cell: String,
}

impl<'a> EventRenderer<'a> {
    fn new(ui: &'a mut Ui) -> Self {
        Self {
            ui,
            current_text: String::new(),
            in_heading: None,
            in_paragraph: false,
            in_emphasis: false,
            in_strong: false,
            in_strikethrough: false,
            in_code: false,
            code_block_text: String::new(),
            code_block_language: None,
            in_list: false,
            list_item_number: None,
            list_item_start: false,
            in_blockquote: false,
            blockquote_text: Vec::new(),
            link_url: None,
            table_state: TableState::default(),
        }
    }

    fn render(&mut self, parser: Parser) {
        for event in parser {
            self.handle_event(event);
        }
        // Flush any remaining content
        self.flush_text();
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag),
            Event::End(tag_end) => self.handle_end_tag(tag_end),
            Event::Text(text) => {
                if self.table_state.in_table {
                    self.table_state.current_cell.push_str(&text);
                } else if self.in_blockquote {
                    self.current_text.push_str(&text);
                } else if !self.code_block_text.is_empty() {
                    self.code_block_text.push_str(&text);
                } else {
                    self.current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                self.flush_text();
                self.render_inline_code(&code);
            }
            Event::SoftBreak => {
                if !self.code_block_text.is_empty() {
                    self.code_block_text.push('\n');
                } else {
                    self.current_text.push(' ');
                }
            }
            Event::HardBreak => {
                self.flush_text();
                if !self.code_block_text.is_empty() {
                    self.code_block_text.push('\n');
                }
            }
            Event::Rule => {
                self.flush_text();
                self.ui.separator();
            }
            _ => {}
        }
    }

    fn handle_start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush_text();
                self.in_heading = Some(level as u8);
            }
            Tag::Paragraph => {
                self.flush_text();
                self.in_paragraph = true;
            }
            Tag::Emphasis => {
                self.in_emphasis = true;
            }
            Tag::Strong => {
                self.in_strong = true;
            }
            Tag::Strikethrough => {
                self.in_strikethrough = true;
            }
            Tag::Link { dest_url, .. } => {
                self.link_url = Some(dest_url.to_string());
            }
            Tag::List(start) => {
                self.flush_text();
                self.in_list = true;
                self.list_item_number = start.map(|n| n as usize);
            }
            Tag::Item => {
                self.list_item_start = true; // Mark that we're starting a new list item
            }
            Tag::BlockQuote(..) => {
                self.flush_text();
                self.in_blockquote = true;
            }
            Tag::CodeBlock(kind) => {
                self.flush_text();
                if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                    self.code_block_language = Some(lang.to_string());
                }
            }
            Tag::Table(_) => {
                self.flush_text();
                self.table_state.in_table = true;
            }
            Tag::TableHead => {
                self.table_state.in_table_head = true;
            }
            Tag::TableRow => {
                self.table_state.current_row.clear();
            }
            Tag::TableCell => {
                self.table_state.current_cell.clear();
            }
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Heading(_) => {
                self.flush_text();
                self.in_heading = None;
            }
            TagEnd::Paragraph => {
                self.flush_text();
                self.in_paragraph = false;
                self.ui.add_space(4.0);
            }
            TagEnd::Emphasis => {
                self.in_emphasis = false;
            }
            TagEnd::Strong => {
                self.in_strong = false;
            }
            TagEnd::Strikethrough => {
                self.in_strikethrough = false;
            }
            TagEnd::Link => {
                if let Some(url) = self.link_url.take() {
                    let text = if self.current_text.is_empty() {
                        url.clone()
                    } else {
                        self.current_text.clone()
                    };
                    self.current_text.clear();
                    self.render_link(&text, &url);
                }
            }
            TagEnd::List(_) => {
                self.flush_text();
                self.in_list = false;
                self.list_item_number = None;
                self.ui.add_space(4.0);
            }
            TagEnd::Item => {
                if let Some(ref mut num) = self.list_item_number {
                    *num += 1;
                }
            }
            TagEnd::BlockQuote(_) => {
                self.flush_text();
                self.render_blockquote();
                self.in_blockquote = false;
                self.blockquote_text.clear();
            }
            TagEnd::CodeBlock => {
                self.render_code_block();
                self.code_block_text.clear();
                self.code_block_language = None;
            }
            TagEnd::Table => {
                self.render_table();
                self.table_state = TableState::default();
            }
            TagEnd::TableHead => {
                self.table_state.in_table_head = false;
            }
            TagEnd::TableRow => {
                let row = self.table_state.current_row.clone();
                if self.table_state.in_table_head {
                    self.table_state.headers = row;
                } else {
                    self.table_state.rows.push(row);
                }
            }
            TagEnd::TableCell => {
                self.table_state
                    .current_row
                    .push(self.table_state.current_cell.clone());
            }
            _ => {}
        }
    }

    fn flush_text(&mut self) {
        if self.current_text.is_empty() {
            return;
        }

        let text = self.current_text.clone();
        self.current_text.clear();

        // Check if text contains emoji and render accordingly
        if self.contains_emoji(&text) {
            self.render_text_with_emoji(&text);
        } else {
            self.render_plain_text(&text);
        }
    }

    fn contains_emoji(&self, text: &str) -> bool {
        text.chars().any(|c| is_emoji_char(c))
    }

    fn render_text_with_emoji(&mut self, text: &str) {
        // Split text into emoji and non-emoji segments for proper rendering
        let segments = split_emoji_segments(text);

        if self.in_list {
            // Render list bullet only at the start of the item
            if self.list_item_start {
                if let Some(num) = self.list_item_number {
                    self.ui.label(format!("{}. ", num));
                } else {
                    self.ui.label("• ");
                }
                self.list_item_start = false; // Only render bullet once per item
            }

            // Render text segments inline
            for (segment, is_emoji) in segments {
                if is_emoji {
                    EmojiLabel::new(segment).show(self.ui);
                } else {
                    let mut rich_text = RichText::new(segment);
                    if self.in_strong {
                        rich_text = rich_text.strong();
                    }
                    if self.in_emphasis {
                        rich_text = rich_text.italics();
                    }
                    if self.in_strikethrough {
                        rich_text = rich_text.strikethrough();
                    }
                    self.ui.label(rich_text);
                }
            }
        } else {
            // Use horizontal_wrapped to allow inline flow without forced line breaks
            self.ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0; // No spacing between segments
                for (segment, is_emoji) in segments {
                    if is_emoji {
                        EmojiLabel::new(segment).show(ui);
                    } else {
                        let mut rich_text = RichText::new(segment);

                        // Apply heading styles
                        if let Some(level) = self.in_heading {
                            let size = match level {
                                1 => 28.0,
                                2 => 24.0,
                                3 => 20.0,
                                4 => 18.0,
                                5 => 16.0,
                                _ => 14.0,
                            };
                            rich_text = rich_text.size(size).strong();
                        }

                        // Apply text styling
                        if self.in_strong {
                            rich_text = rich_text.strong();
                        }
                        if self.in_emphasis {
                            rich_text = rich_text.italics();
                        }
                        if self.in_strikethrough {
                            rich_text = rich_text.strikethrough();
                        }

                        ui.label(rich_text);
                    }
                }
            });
        }

        // Add spacing after headings
        if self.in_heading.is_some() {
            self.ui.add_space(8.0);
        }
    }

    fn render_plain_text(&mut self, text: &str) {
        let mut rich_text = RichText::new(text);

        // Apply heading styles
        if let Some(level) = self.in_heading {
            let size = match level {
                1 => 28.0,
                2 => 24.0,
                3 => 20.0,
                4 => 18.0,
                5 => 16.0,
                _ => 14.0,
            };
            rich_text = rich_text.size(size).strong();
        }

        // Apply text styling
        if self.in_strong {
            rich_text = rich_text.strong();
        }
        if self.in_emphasis {
            rich_text = rich_text.italics();
        }
        if self.in_strikethrough {
            rich_text = rich_text.strikethrough();
        }

        // Handle list items
        if self.in_list {
            // Render list bullet only at the start of the item
            if self.list_item_start {
                if let Some(num) = self.list_item_number {
                    self.ui.label(format!("{}. ", num));
                } else {
                    self.ui.label("• ");
                }
                self.list_item_start = false; // Only render bullet once per item
            }
            self.ui.label(rich_text);
        } else {
            self.ui.label(rich_text);
        }

        // Add spacing after headings
        if self.in_heading.is_some() {
            self.ui.add_space(8.0);
        }
    }

    fn render_inline_code(&mut self, code: &str) {
        // Render list bullet if this is the start of a list item
        if self.in_list && self.list_item_start {
            if let Some(num) = self.list_item_number {
                self.ui.label(format!("{}. ", num));
            } else {
                self.ui.label("• ");
            }
            self.list_item_start = false;
        }

        let code_text = RichText::new(code)
            .font(FontId::monospace(12.0))
            .background_color(Color32::from_rgb(40, 40, 40))
            .color(Color32::from_rgb(220, 220, 220));

        self.ui.label(code_text);
    }

    fn render_link(&mut self, text: &str, url: &str) {
        // Render link with emoji support
        let has_emoji = text.chars().any(|c| is_emoji_char(c));

        if has_emoji {
            self.ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (segment, is_emoji) in split_emoji_segments(text) {
                    if is_emoji {
                        // Emoji in links - just show the emoji (clicking the emoji won't open link)
                        EmojiLabel::new(&segment).show(ui);
                    } else {
                        // Regular text as hyperlink
                        ui.hyperlink_to(&segment, url).on_hover_text(url);
                    }
                }
            });
        } else {
            self.ui.hyperlink_to(text, url).on_hover_text(url);
        }
    }

    fn render_code_block(&mut self) {
        if self.code_block_text.is_empty() {
            return;
        }

        let language = self.code_block_language.as_deref().unwrap_or("");

        egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 30))
            .inner_margin(8.0)
            .show(self.ui, |ui| {
                ui.horizontal(|ui| {
                    if !language.is_empty() {
                        ui.label(
                            RichText::new(language)
                                .small()
                                .color(Color32::from_rgb(150, 150, 150)),
                        );
                    }
                });

                ui.separator();

                let code_text = RichText::new(&self.code_block_text)
                    .font(FontId::monospace(12.0))
                    .color(Color32::from_rgb(220, 220, 220));

                ui.label(code_text);
            });

        self.ui.add_space(8.0);
    }

    fn render_blockquote(&mut self) {
        if self.current_text.is_empty() {
            return;
        }

        let text = &self.current_text;
        let has_emoji = text.chars().any(|c| is_emoji_char(c));

        egui::Frame::new()
            .fill(Color32::from_rgba_unmultiplied(100, 100, 100, 20))
            .inner_margin(8.0)
            .stroke(egui::Stroke::new(
                2.0,
                Color32::from_rgb(100, 100, 100),
            ))
            .show(self.ui, |ui| {
                if has_emoji {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (segment, is_emoji) in split_emoji_segments(text) {
                            if is_emoji {
                                EmojiLabel::new(&segment).show(ui);
                            } else {
                                ui.label(
                                    RichText::new(&segment)
                                        .italics()
                                        .color(Color32::from_rgb(180, 180, 180)),
                                );
                            }
                        }
                    });
                } else {
                    ui.label(
                        RichText::new(text)
                            .italics()
                            .color(Color32::from_rgb(180, 180, 180)),
                    );
                }
            });

        self.ui.add_space(8.0);
    }

    fn render_table(&mut self) {
        use egui_extras::{Column, TableBuilder};

        if self.table_state.headers.is_empty() && self.table_state.rows.is_empty() {
            return;
        }

        let num_columns = if !self.table_state.headers.is_empty() {
            self.table_state.headers.len()
        } else if !self.table_state.rows.is_empty() {
            self.table_state.rows[0].len()
        } else {
            return;
        };

        // Get available width for responsive column sizing
        let available_width = self.ui.available_width();
        let column_width = (available_width / num_columns as f32).max(80.0); // Min 80px per column

        // Clone the data to avoid borrow checker issues
        let headers = self.table_state.headers.clone();
        let rows = self.table_state.rows.clone();

        let mut table = TableBuilder::new(self.ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        // Add columns with responsive width
        for _ in 0..num_columns {
            table = table.column(Column::initial(column_width).at_least(60.0).resizable(true));
        }

        table
            .header(20.0, |mut header| {
                for header_text in &headers {
                    header.col(|ui| {
                        // Render header with emoji support
                        let has_emoji = header_text.chars().any(|c| is_emoji_char(c));
                        if has_emoji {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                for (segment, is_emoji) in split_emoji_segments(header_text) {
                                    if is_emoji {
                                        EmojiLabel::new(segment).show(ui);
                                    } else {
                                        ui.strong(&segment);
                                    }
                                }
                            });
                        } else {
                            ui.strong(header_text);
                        }
                    });
                }
            })
            .body(|mut body| {
                for row in &rows {
                    body.row(18.0, |mut table_row| {
                        for cell in row {
                            table_row.col(|ui| {
                                // Render cell content with emoji support
                                let has_emoji = cell.chars().any(|c| is_emoji_char(c));
                                if has_emoji {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 0.0;
                                        for (segment, is_emoji) in split_emoji_segments(cell) {
                                            if is_emoji {
                                                EmojiLabel::new(&segment).show(ui);
                                            } else {
                                                ui.label(&segment);
                                            }
                                        }
                                    });
                                } else {
                                    ui.label(cell);
                                }
                            });
                        }
                    });
                }
            });

        self.ui.add_space(8.0);
    }
}

/// Split text into alternating segments of emoji and non-emoji text
fn split_emoji_segments(text: &str) -> Vec<(String, bool)> {
    let mut segments = Vec::new();
    let mut current_segment = String::new();
    let mut is_emoji_segment = false;
    let mut first_char = true;

    for ch in text.chars() {
        let is_emoji = is_emoji_char(ch);

        if first_char {
            is_emoji_segment = is_emoji;
            first_char = false;
        } else if is_emoji != is_emoji_segment {
            if !current_segment.is_empty() {
                segments.push((current_segment.clone(), is_emoji_segment));
                current_segment.clear();
            }
            is_emoji_segment = is_emoji;
        }

        current_segment.push(ch);
    }

    if !current_segment.is_empty() {
        segments.push((current_segment, is_emoji_segment));
    }

    segments
}

/// Check if a character is an emoji using comprehensive Unicode ranges
fn is_emoji_char(ch: char) -> bool {
    matches!(ch as u32,
        // Common emoji ranges
        0x1F300..=0x1F9FF |   // Emoticons, pictographs, transport, symbols (covers most emoji)
        0x2600..=0x26FF |     // Miscellaneous Symbols (☀️ ⛄️ ⚡️ etc)
        0x2700..=0x27BF |     // Dingbats (✂️ ✏️ ✉️ etc)
        0x231A..=0x231B |     // Watch ⌚
        0x23E9..=0x23F3 |     // Media controls and clocks (⏰ ⏱ etc)
        0x23F8..=0x23FA |     // Media controls
        0x25AA..=0x25AB |     // Squares
        0x25B6 | 0x25C0 |     // Triangles
        0x25FB..=0x25FE |     // Squares
        0x2614..=0x2615 |     // Umbrella, coffee
        0x2648..=0x2653 |     // Zodiac signs
        0x267F | 0x2693 |     // Wheelchair, anchor
        0x26A1 | 0x26AA..=0x26AB | 0x26BD..=0x26BE | 0x26C4..=0x26C5 |
        0x26CE | 0x26D4 | 0x26EA | 0x26F2..=0x26F3 | 0x26F5 | 0x26FA | 0x26FD |
        0x2705 | 0x270A..=0x270B | 0x2728 | 0x274C | 0x274E |
        0x2753..=0x2755 | 0x2757 |
        0x2795..=0x2797 | 0x27B0 | 0x27BF |
        0x2B1B..=0x2B1C | 0x2B50 | 0x2B55 |
        0x1F004 | 0x1F0CF |   // Mahjong, playing card
        0x1F170..=0x1F171 |   // A, B blood types
        0x1F17E..=0x1F17F |   // O blood type, parking
        0x1F18E |             // AB blood type
        0x1F191..=0x1F19A |   // Squared symbols
        0x1F1E6..=0x1F1FF |   // Regional indicators (flags)
        0x1F201..=0x1F202 |   // Japanese symbols
        0x1F21A | 0x1F22F |
        0x1F232..=0x1F23A |
        0x1F250..=0x1F251 |
        0x1F600..=0x1F64F |   // Emoticons
        0x1F680..=0x1F6FF |   // Transport and Map
        0x1F910..=0x1F96B |   // Supplemental emoticons
        0x1F980..=0x1F9E0 |   // Supplemental symbols
        0xFE00..=0xFE0F       // Variation selectors
    )
}
