# media-collector

A modular media information collector with configurable rate limiting and API integrations.

## Setup

1. **Copy config-exemple.toml** in the project root (same directory as `Cargo.toml`):
```
   media-collector/
   ├── Cargo.toml
   ├── config.toml     <-- Create this file here
   ├── src/
   └── ...
```
2. **Configure your settings** in `config.toml`:
   - Set your API keys (replace `YOUR_MAL_API_KEY_HERE` with actual keys)
   - Enable/disable modules as needed
   - Adjust rate limits for your use case
   - Configure logging level

3. **Get API Keys**:
   - MyAnimeList: https://myanimelist.net/apiconfig
   - Other APIs: Check their respective documentation

## Configuration

### Required Configuration

Some modules require API keys to function:
- **MyAnimeList**: Requires API key (get from https://myanimelist.net/apiconfig)

If a module is enabled but missing required configuration, it will:
- Log a warning message explaining the issue
- Continue running without that module
- Not crash the application

## Running
```bash
# With Docker
docker compose up --build -d

# Or locally
cargo run
```

## Logging

View logs based on configured level. You can also override with environment variable:
```bash
RUST_LOG=debug cargo run
RUST_LOG=trace cargo run  # Very verbose
```

## Troubleshooting

**"Failed to load configuration"**
- Ensure `config.toml` exists in the project root (same level as `Cargo.toml`)
- Check the file is valid TOML syntax

**"Child module cannot start due to configuration issue"**
- Check the log message for which module and why
- Usually means missing API key or module is disabled
- Add the required API key to `config.toml`

**Module not starting**
- Check `enabled = true` in config for that module
- Verify API key is present if required
- Check logs for specific error messages
```

## Summary

The application now:

1. **Gracefully handles missing API keys**: Child modules won't crash the app, they just won't start
2. **Logs clear warnings**: You'll see exactly why a module didn't start
3. **Validates configuration**: Checks required settings before attempting to start modules
4. **Clear file location**: `config.toml` goes in the project root alongside `Cargo.toml`
5. **Helpful error messages**: If config is missing or invalid, you get clear guidance

Example log output when API key is missing:
```
WARN media_collector::global::config: Child module cannot start due to configuration issue module=my_anime_list error="missing required API key for module: my_anime_list"
WARN media_collector: MyAnimeList child module is not available (disabled or missing required configuration)