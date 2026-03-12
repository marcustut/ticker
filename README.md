# ticker

A lightweight cron job scheduler written in Rust that runs shell commands on a schedule. Supports hot-reloading of the config file — no restart required when you add, remove, or change jobs.

## Features

- **Cron scheduling** — standard 5-field cron expressions via [`croner`](https://github.com/hexagon/croner-rust)
- **Hot reload** — config changes are detected automatically and jobs are restarted
- **Timezone-aware** — per-config timezone support with IANA names; defaults to system timezone
- **Structured logging** — logs to both stdout and rolling daily log files under `./logs/`
- **Cross-platform** — runs commands via `sh -c` on Unix and `cmd /C` on Windows

## Installation

### From source

```sh
cargo install --path .
```

Requires Rust 1.85+ (edition 2024).

## Usage

```sh
ticker --config-file path/to/config.toml
```

### Options

| Flag | Short | Description |
|------|-------|-------------|
| `--config-file` | `-c` | Path to the TOML config file (required) |

### Logging

Logs are written to stdout and to `./logs/ticker.log.<date>` (rolling daily). Override the log level with the `RUST_LOG` environment variable:

```sh
RUST_LOG=debug ticker -c config.toml
```

## Configuration

The config file is TOML. Define a `[jobs]` table where each key is the job name.

```toml
timezone = "America/New_York"   # optional, defaults to system timezone

[jobs.say-hello]
trigger  = "* * * * *"          # every minute
command  = "echo hello"

[jobs.daily-backup]
trigger  = "0 2 * * *"          # every day at 02:00
command  = "/usr/local/bin/backup.sh"

[jobs.cleanup]
trigger  = "0 0 * * 0"          # every Sunday at midnight
command  = "find /tmp -mtime +7 -delete"
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timezone` | string | No | IANA timezone name (e.g. `"UTC"`, `"Asia/Tokyo"`). Defaults to system timezone. |
| `jobs.<name>.trigger` | string | Yes | Cron expression (`minute hour day month weekday`) |
| `jobs.<name>.command` | string | Yes | Shell command to run |

### Cron expression syntax

```
┌──────── minute (0-59)
│ ┌────── hour (0-23)
│ │ ┌──── day of month (1-31)
│ │ │ ┌── month (1-12)
│ │ │ │ ┌ day of week (0-7, 0 and 7 are Sunday)
│ │ │ │ │
* * * * *
```

Common examples:

| Expression | Meaning |
|------------|---------|
| `* * * * *` | Every minute |
| `0 * * * *` | Every hour |
| `0 9 * * 1-5` | 09:00 on weekdays |
| `0 0 1 * *` | Midnight on the 1st of every month |
| `*/15 * * * *` | Every 15 minutes |

## Hot reload

ticker watches the config file for changes using [`sentinel`](https://github.com/balaenaquant/sentinel). When a change is detected, all running jobs are stopped and restarted with the new config. This means you can add, remove, or reschedule jobs without restarting the process.

```
[INFO] Detected config change, stop 2 jobs and start 3 jobs
[INFO] Spawned job daily-backup: Job { trigger: "0 2 * * *", command: "/usr/local/bin/backup.sh" }
```

## License

MIT
