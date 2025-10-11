#!/bin/bash

# Simple test to check if search detection works

cat > /tmp/test_search_detection.rs << 'EOF'
fn needs_search(query: &str) -> bool {
    let lower = query.to_lowercase();

    let current_keywords = [
        "latest", "recent", "current", "now", "today", "this week", "this month",
        "this year", "news", "update", "what's new", "breaking", "just",
        "currently", "as of", "2024", "2025",
    ];

    if current_keywords.iter().any(|kw| lower.contains(kw)) {
        return true;
    }

    let comparison_patterns = [
        "compare", "comparison", "versus", "vs", "vs.", "ranked", "ranking",
        "top ", "best ", "worst ", "better than", "worse than",
        "how does", "how do", "differ", "difference between",
    ];

    if comparison_patterns.iter().any(|pattern| lower.contains(pattern)) {
        return true;
    }

    let data_patterns = [
        "how many", "how much", "statistics", "stats", "data", "rate",
        "percentage", "number of", "amount of", "total", "average",
    ];

    if data_patterns.iter().any(|pattern| lower.contains(pattern)) {
        return true;
    }

    let question_patterns = [
        "who is the", "what is the current", "what's the latest",
        "when did", "when was", "what happened", "where is",
        "which city", "which country", "which state",
    ];

    if question_patterns.iter().any(|pattern| lower.contains(pattern)) {
        return true;
    }

    if lower.contains("city") || lower.contains("cities") ||
       lower.contains("country") || lower.contains("countries") ||
       lower.contains("state") || lower.contains("states") {
        if lower.contains("?") || lower.starts_with("how") ||
           lower.starts_with("what") || lower.starts_with("where") ||
           lower.starts_with("which") || lower.starts_with("is") {
            return true;
        }
    }

    false
}

fn main() {
    let test_queries = vec![
        "How does Philadelphia's road traffic compare with other U.S. cities?",
        "What's the latest news about Rust?",
        "Tell me about the history of Rome",
        "What are the current AI trends?",
        "How many people live in Tokyo?",
    ];

    println!("Testing search detection:\n");
    for query in test_queries {
        let should_search = needs_search(query);
        let icon = if should_search { "🔍" } else { "❌" };
        println!("{} {} -> {}", icon, query, should_search);
    }
}
EOF

cd /tmp && rustc test_search_detection.rs && ./test_search_detection
rm -f test_search_detection test_search_detection.rs
