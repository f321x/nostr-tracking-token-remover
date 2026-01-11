FROM python:3.14-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libsecp256k1-dev \
    && rm -rf /var/lib/apt/lists/*

# Set environment variable to skip compiling libsecp256k1 for electrum-aionostr dependency
ENV ELECTRUM_ECC_DONT_COMPILE=1

ENV PIP_BREAK_SYSTEM_PACKAGES=1

# Install dependencies first for better caching
COPY pyproject.toml ./
RUN pip install --no-cache-dir .

# Copy application code
COPY nostr_tracking_token_remover/ ./nostr_tracking_token_remover/
COPY run_bot.py ./

# Run as non-root user
RUN useradd --create-home --shell /bin/bash appuser
USER appuser

CMD ["python3", "run_bot.py"]