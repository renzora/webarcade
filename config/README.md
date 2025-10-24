# Configuration Directory

This directory contains sensitive configuration files that are **not tracked by git**.

## Setup Instructions

### Twitch Configuration

1. Copy `twitch_config.toml.example` to `twitch_config.toml`
2. Fill in your Twitch API credentials:
   - `client_id` - Your Twitch application client ID
   - `client_secret` - Your Twitch application client secret (encrypted)
   - `bot_username` - Your Twitch bot username
   - `channels` - List of channels to join

### SSL Certificates (for HTTPS development)

The application expects SSL certificates in the root directory:
- `localhost+2-key.pem` - Private key
- `localhost+2.pem` - Certificate

You can generate these using [mkcert](https://github.com/FiloSottile/mkcert):

```bash
# Install mkcert
winget install FiloSottile.mkcert

# Install local CA
mkcert -install

# Generate certificates
mkcert localhost 127.0.0.1 ::1
```

## Security Notes

⚠️ **Never commit sensitive files to git:**
- `*.toml` (except example files)
- `*.pem`, `*.key` - SSL certificates and private keys
- `.twitch_key` - API keys
- Any files containing tokens, passwords, or credentials

These files are already excluded in `.gitignore`.
