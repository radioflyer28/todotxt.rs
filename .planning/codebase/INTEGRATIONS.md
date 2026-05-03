# External Integrations

**Analysis Date:** 2025-01-30

## APIs & External Services

**Update Checking:**
- GitHub Raw Content — Fetches `Updates.xml` to check for new versions
  - URL: `https://raw.github.com/benrhughes/todotxt.net/master/Updates.xml`
  - Implementation: `Client\UpdateChecker.cs`
  - Mechanism: `BackgroundWorker` + `XmlDocument.Load(XmlTextReader)` HTTP GET
  - No API key required; public unauthenticated HTTP read
  - Result: calls `MainWindow.ToggleUpdateMenu(version)` on completion

- App/Download Website — Directs users to download page on update
  - URL: `http://benrhughes.com/todotxt.net`
  - Constant: `UpdateChecker.updateClientUrl` in `Client\UpdateChecker.cs`
  - Usage: linked from UI update notification; opened via `Process.Start`

**URL Handling:**
- System browser — Opens URLs found inside task text
  - Implementation: `Client\UrlService.cs`
  - Mechanism: `Process.Start(uri)` launches default browser for http/https/ftp/www links
  - No external SDK; uses regex to detect URLs in WPF `TextBlock` content

## Data Storage

**Databases:**
- None — No database used

**File Storage (Primary Persistence):**
- Local flat file (`todo.txt` format)
  - Path: user-configured, stored in settings (`FilePath` setting)
  - Implementation: `ToDoLib\TaskList.cs` — reads/writes plain text file directly
  - Format: todo.txt open standard (one task per line, priority `(A)`, dates, `+projects`, `@contexts`)
  - Concurrent write protection: `System.Threading.Mutex` on the file path (see `ToDoLib\Log.cs`)

**Archive File:**
- Local flat file (`done.txt` format)
  - Path: user-configured (`ArchiveFilePath` setting), or auto-selected alongside `todo.txt`
  - Triggered by: manual archive action or `AutoArchive` setting

**Settings Storage:**
- XML file stored alongside executable (portable mode via `Client\PortableSettingsProvider.cs`)
  - File: `<exe-dir>\todotxt.settings`
  - Falls back to: standard `%LocalAppData%` user settings if portable provider not active

**Log File:**
- Plain text, append-only
  - Path: `%AppData%\Hughesoft\todotxt.exe\log.txt`
  - Implementation: `ToDoLib\Log.cs`

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- None — No authentication or user identity system

## Monitoring & Observability

**Error Tracking:**
- None (no third-party service)

**Logs:**
- Custom file logger in `ToDoLib\Log.cs`
  - Levels: `Error`, `Debug`
  - Controlled by: `DebugLoggingOn` user setting
  - Output: `%AppData%\Hughesoft\todotxt.exe\log.txt`
  - Thread-safe via `Mutex`

## CI/CD & Deployment

**Hosting:**
- Desktop application — distributed as Windows installer `.exe`
- Installer: Inno Setup script at `Installer\Installer.iss`
  - Output: `todotxt-setup-{version}.exe`
  - Publisher: Hughesoft / `http://www.todotxt.net`

**CI Pipeline:**
- None detected (no GitHub Actions, AppVeyor, or other CI config files present)
- Build script: `Build.proj` (MSBuild Community Tasks) for producing release builds

## Environment Configuration

**Required env vars:**
- None — application is fully self-contained with no required environment variables

**Secrets location:**
- None — no API keys, tokens, or secrets in use

## Webhooks & Callbacks

**Incoming:**
- None — desktop application, no web server

**Outgoing:**
- None — update check is a one-way HTTP GET (no webhook pattern)

## File System Watching

**File Change Observer:**
- `Client\FileChangeObserver.cs` — monitors the `todo.txt` file for external changes
- Mechanism: `System.IO.FileSystemWatcher`
- Trigger: reloads task list when file is modified by another application (e.g., Dropbox sync, manual edit)
- Enabled by: `AutoRefresh` user setting

---

*Integration audit: 2025-01-30*
