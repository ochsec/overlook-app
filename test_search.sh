#!/bin/bash

# Test script for web search functionality

echo "Testing web search integration..."
echo ""

# Check if environment variables are set
if [ -z "$SEARXNG_URL" ]; then
    echo "❌ SEARXNG_URL is not set"
    echo "Please add to .env file: SEARXNG_URL=https://search.replmade.com"
    exit 1
fi

if [ -z "$SEARXNG_API_TOKEN" ]; then
    echo "❌ SEARXNG_API_TOKEN is not set"
    echo "Please add to .env file: SEARXNG_API_TOKEN=your-token"
    exit 1
fi

echo "✅ SEARXNG_URL: $SEARXNG_URL"
echo "✅ SEARXNG_API_TOKEN is set"
echo ""

# Test the SearXNG API directly
echo "Testing SearXNG API directly..."
curl -s -H "X-API-Token: $SEARXNG_API_TOKEN" "${SEARXNG_URL}/search?q=rust+programming&format=json" | jq '.results[0] | {title, url}' 2>/dev/null

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ SearXNG API is working!"
    echo ""
    echo "Search keywords that will trigger web search:"
    echo "  - latest, recent, current, now, today"
    echo "  - this week, this month, this year"
    echo "  - news, update, what's new, breaking"
    echo "  - 2024, 2025"
    echo ""
    echo "Example queries:"
    echo "  - 'What's the latest news about Rust?'"
    echo "  - 'What are the current AI trends in 2025?'"
    echo "  - 'Recent developments in quantum computing'"
else
    echo "❌ SearXNG API test failed. Check your API token and URL."
    exit 1
fi

echo "Now run: cargo run --bin overlook"
