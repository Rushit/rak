import os
from pathlib import Path
from tensorzero import TensorZeroGateway

# Manually load .env file
env_file = Path(__file__).parent / ".env"
if env_file.exists():
    with open(env_file) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith('#') and '=' in line:
                key, value = line.split('=', 1)
                os.environ.setdefault(key.strip(), value.strip())

# Set GOOGLE_AI_STUDIO_API_KEY from GEMINI_API_KEY if not already set
if not os.getenv("GOOGLE_AI_STUDIO_API_KEY") and os.getenv("GEMINI_API_KEY"):
    os.environ["GOOGLE_AI_STUDIO_API_KEY"] = os.getenv("GEMINI_API_KEY")

if not os.getenv("GOOGLE_AI_STUDIO_API_KEY") and os.getenv("GEMINI_API_KEY"):
    os.environ["GOOGLE_AI_STUDIO_API_KEY"] = os.getenv("GEMINI_API_KEY")

# Build the client with the config file
with TensorZeroGateway.build_embedded(
    clickhouse_url="http://chusertoxi:chpasswordtoxi$43@localhost:8123/tensorzero",
    config_file="config/tensorzero.toml",
) as client:
    response = client.inference(
        function_name="generate_haiku",
        input={
            "messages": [
                {
                    "role": "user",
                    "content": "Write a haiku about artificial intelligence.",
                }
            ]
        },
    )
    print(response)
