#!/usr/bin/env python3
"""
Simple OpenAI Chat Completion via TensorZero HTTP Gateway with Auth
"""

import os
from openai import OpenAI

# Check for TensorZero API key (for authentication)
if not os.getenv("OPENAI_API_KEY"):
    print("⚠️  Set OPENAI_API_KEY environment variable")
    print("   Format: sk-t0-<public_id>-<long_key>")
    exit(1)

# Create OpenAI client pointing to TensorZero gateway
# Use TensorZero API key for authentication
client = OpenAI(
    base_url="http://localhost:8181/app/v1",
    api_key=os.getenv("OPENAI_API_KEY"),  # TensorZero key for auth
)

print("🤖 Calling OpenAI via TensorZero (Auth Enabled)...")
print(f"🔐 Using TensorZero API key: {os.getenv('OPENAI_API_KEY')[:15]}...")
print()

# Simple chat completion
# Use the Gemini Flash Lite model defined in your tensorzero.toml
response = client.chat.completions.create(
    model="tensorzero::model_name::gemini_flash_lite",  # Fixed: removed 'gpt_' prefix
    messages=[
        {"role": "user", "content": "Write a software engineer joke"}
    ],
)

# Print result
print(response.choices[0].message.content)
print()
print(f"✅ Tokens: {response.usage.total_tokens}")
print(f"🔐 Auth: Enabled ✓")
