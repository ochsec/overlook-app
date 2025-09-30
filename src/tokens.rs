use crate::types::{Message, ProviderKind};
use std::collections::HashMap;
use std::sync::Arc;
use tiktoken_rs::{cl100k_base, get_bpe_from_model, CoreBPE};

const TOKEN_MESSAGE_OVERHEAD: usize = 4;
const TOKEN_PROMPT_PRIMER: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub max_context_tokens: Option<usize>,
}

impl TokenUsage {
    pub fn percentage_of_max(&self) -> Option<f32> {
        self.max_context_tokens.map(|max| {
            if max == 0 {
                0.0
            } else {
                (self.total_tokens as f32 / max as f32) * 100.0
            }
        })
    }

    pub fn remaining_tokens(&self) -> Option<usize> {
        self.max_context_tokens
            .map(|max| max.saturating_sub(self.total_tokens))
    }
}

#[derive(Clone)]
enum Encoder {
    Bpe(Arc<CoreBPE>),
    Approx,
}

pub struct TokenCounter {
    cache: HashMap<String, Encoder>,
}

impl TokenCounter {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn count_prompt_tokens(
        &mut self,
        provider: ProviderKind,
        model: &str,
        messages: &[Message],
    ) -> usize {
        if messages.is_empty() {
            return 0;
        }

        let encoder = self.resolve_encoder(provider, model);
        let mut total = TOKEN_PROMPT_PRIMER;

        for message in messages {
            total += encode_text(&encoder, &format!("{}: {}", message.role, message.content));
            total += TOKEN_MESSAGE_OVERHEAD;
        }

        total
    }

    pub fn count_completion_tokens(
        &mut self,
        provider: ProviderKind,
        model: &str,
        text: &str,
    ) -> usize {
        if text.trim().is_empty() {
            return 0;
        }

        let encoder = self.resolve_encoder(provider, model);
        encode_text(&encoder, text)
    }

    fn resolve_encoder(&mut self, provider: ProviderKind, model: &str) -> Encoder {
        let key = format!("{:?}::{}", provider, model);
        if let Some(encoder) = self.cache.get(&key) {
            return encoder.clone();
        }

        let mut candidates = candidate_model_names(provider, model);
        let encoder = candidates
            .drain(..)
            .find_map(|candidate| match get_bpe_from_model(&candidate) {
                Ok(bpe) => Some(Encoder::Bpe(Arc::new(bpe))),
                Err(_) => None,
            })
            .unwrap_or_else(|| match cl100k_base() {
                Ok(bpe) => Encoder::Bpe(Arc::new(bpe)),
                Err(_) => Encoder::Approx,
            });

        self.cache.insert(key, encoder.clone());
        encoder
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

fn candidate_model_names(provider: ProviderKind, model: &str) -> Vec<String> {
    let mut names = Vec::new();
    names.push(model.to_string());

    if let Some(after_slash) = model.split('/').last() {
        names.push(after_slash.to_string());
    }

    if let Some(before_colon) = model.split(':').next() {
        if before_colon != model {
            names.push(before_colon.to_string());
        }
    }

    match provider {
        ProviderKind::OpenAI => {
            names.push("gpt-4o".to_string());
            names.push("gpt-3.5-turbo".to_string());
        }
        ProviderKind::Anthropic => {
            names.push("claude-3-opus-20240229".to_string());
        }
        ProviderKind::OpenRouter => {
            names.push("gpt-4o".to_string());
            names.push("claude-3-opus-20240229".to_string());
        }
    }

    names.sort();
    names.dedup();
    names
}

fn encode_text(encoder: &Encoder, text: &str) -> usize {
    match encoder {
        Encoder::Bpe(bpe) => bpe.encode_ordinary(text).len(),
        Encoder::Approx => approximate_token_count(text),
    }
}

fn approximate_token_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let char_based = (text.chars().count() + 3) / 4; // ~4 chars per token heuristic
    let word_based = text.split_whitespace().count();
    std::cmp::max(char_based, word_based).max(1)
}
