# media-collector

A modular media information collector with configurable rate limiting and API integrations.

## Setup

1. **Copy config-exemple.toml** in the project root (same directory as `Cargo.toml`):
```
   media-collector/
   ├── Cargo.toml
   ├── config.toml     <-- Create this file here
   ├── logs/           <-- Log files will be created here
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

### Logging Configuration

Configure where and how logs are written:
```toml
[app.logging]
log_to_file = true              # Write logs to files
log_directory = "./logs"        # Directory for log files
log_file_prefix = "media-collector"  # Prefix for log file names
log_rotation = "daily"          # Rotation: daily, hourly, never
log_to_console = true           # Also show logs in console
```

**Log Files**:
- Located in `./logs/` by default
- Named `media-collector.log` (current), with rotated files like `media-collector.log.2024-01-16`
- Rotation options:
  - `daily`: New file each day
  - `hourly`: New file each hour
  - `never`: Single file (grows indefinitely)
- Old log files are automatically kept with timestamps

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

### View Logs

**In files** (default):
```bash
tail -f logs/media-collector.log
```

**In console only** (set `log_to_file = false` and `log_to_console = true`):
```bash
cargo run
```

### Log Levels

Set in `config.toml`:
- `trace`: Very detailed (includes all rate limiter activity)
- `debug`: Detailed operational info
- `info`: Important events (default)
- `warn`: Warnings and retries
- `error`: Errors and failures

Override with environment variable:
```bash
RUST_LOG=debug cargo run
RUST_LOG=trace cargo run  # Very verbose
```

### Log Rotation

Logs are automatically rotated based on your configuration:
- **daily**: Creates a new file each day (e.g., `media-collector.log.2024-01-16`)
- **hourly**: Creates a new file each hour
- **never**: Single file that grows continuously

Old log files are kept with timestamps for historical reference.

## Troubleshooting

**"Failed to load configuration"**
- Ensure `config.toml` exists in the project root (same level as `Cargo.toml`)
- Check the file is valid TOML syntax

**"Child module cannot start due to configuration issue"**
- Check the log files in `./logs/` for which module and why
- Usually means missing API key or module is disabled
- Add the required API key to `config.toml`

**Module not starting**
- Check `enabled = true` in config for that module
- Verify API key is present if required
- Check logs for specific error messages

**Can't find logs**
- Check `log_directory` setting in config.toml
- Ensure the application has write permissions to the directory
- Logs folder is automatically created if it doesn't exist
```

## Summary

The application now:

1. **Writes logs to files**: All logs are saved in the `./logs/` directory by default
2. **Automatic log rotation**: Creates new files daily (or hourly/never based on config)
3. **Flexible output**: Can log to file only, console only, or both
4. **Structured log files**: Include timestamps, thread IDs, line numbers, and targets
5. **Persistent logging**: Log files are kept with timestamps for historical reference
6. **Configurable location**: Change log directory via config.toml

Log file structure:
```
logs/
├── media-collector.log           # Current log file
├── media-collector.log.2024-01-16 # Yesterday's logs
├── media-collector.log.2024-01-15 # Day before
└── ...