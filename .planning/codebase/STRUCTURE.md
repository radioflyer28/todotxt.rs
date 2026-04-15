# Codebase Structure

**Analysis Date:** 2025-01-31

## Directory Layout

```
todotxt.net/
├── Client/                         # WPF application project (EXE)
│   ├── Controls/                   # XAML windows and dialogs (code-behind pairs)
│   ├── Converters/                 # WPF IValueConverter implementations
│   ├── Utilities/                  # Enum extension helpers
│   ├── images/                     # Application icons (PNG, ICO)
│   ├── Properties/                 # AssemblyInfo.cs, SharedAssemblyInfo.cs
│   ├── MainWindowViewModel.cs      # Central application orchestrator (~1900 lines)
│   ├── FileChangeObserver.cs       # FileSystemWatcher wrapper for auto-reload
│   ├── HotKey.cs                   # Win32 RegisterHotKey P/Invoke wrapper
│   ├── HotKeyMainWindows.cs        # Ctrl+Alt+M global hotkey wiring
│   ├── TrayMainWindows.cs          # System tray NotifyIcon
│   ├── UpdateChecker.cs            # Background version check via GitHub XML
│   ├── PortableSettingsProvider.cs # XML settings provider (portable mode)
│   ├── GroupConverter.cs           # IValueConverter for task grouping display
│   ├── ExceptionExtensions.cs      # Exception.Handle() log+dialog extension
│   ├── SortType.cs                 # SortType enum (Alphabetical/Context/etc.)
│   ├── WindowLocation.cs           # POCO for window size/position snapshot
│   ├── User.settings               # ApplicationSettings schema definition
│   ├── User.cs / User.Designer.cs  # Generated settings accessor
│   ├── Resource.resx               # Embedded resources (help text, CSS)
│   ├── app.config                  # Application configuration
│   └── packages.config             # NuGet package references
├── ToDoLib/                        # Domain library (DLL) — no WPF dependency
│   ├── Task.cs                     # Task model + todo.txt regex parser
│   ├── TaskList.cs                 # File I/O abstraction (CRUD on todo.txt)
│   ├── Log.cs                      # Static file logger with mutex protection
│   ├── TaskException.cs            # Domain exception wrapping IOException
│   └── Properties/AssemblyInfo.cs
├── ToDoTests/                      # Unit test project
│   ├── TaskTests.cs                # Tests for Task parsing and manipulation
│   ├── TaskListTests.cs            # Tests for TaskList CRUD operations
│   ├── Data.resx                   # Embedded test data (raw todo.txt strings)
│   └── Properties/AssemblyInfo.cs
├── CommonExtensions/               # Shared utility library (DLL)
│   ├── StringExtensions.cs         # Contains (case-insensitive), IsDateGreaterThan, IsDateLessThan
│   └── IEnumerableExtensions.cs    # IsNullOrEmpty for collections
├── ColorFont/                      # WPF color+font picker control library (DLL)
│   ├── ColorFontChooser.xaml(.cs)  # Composite chooser control
│   ├── ColorFontDialog.xaml(.cs)   # Modal dialog wrapping the chooser
│   ├── ColorPicker.xaml(.cs)       # Color-only picker control
│   ├── ColorPickerViewModel.cs     # ViewModel for color picker
│   ├── FontInfo.cs                 # FontInfo model (Family, Size, Style, etc.)
│   ├── FontColor.cs                # Color ↔ string helpers
│   └── AvailableColors.cs          # Named color list for the picker
├── Installer/                      # Deployment scripts
│   ├── Installer.iss               # Inno Setup script
│   ├── Create-Installer.bat        # Build + package script
│   └── Build.xml                   # MSBuild/NAnt build configuration
├── .build/                         # Build output / CI artifacts directory
├── .planning/                      # GSD planning documents
│   └── codebase/                   # Codebase map documents (this directory)
├── ToDo.Net.sln                    # Visual Studio solution file
├── Updates.xml                     # Version manifest (checked for auto-update)
└── README.md
```

## Directory Purposes

**`Client/Controls/`:**
- Purpose: All XAML user interface definitions and their code-behind files
- Contains: One `.xaml` + `.xaml.cs` pair per window/dialog
- Key files:
  - `MainWindow.xaml` — primary window; declares all `RoutedUICommand` resources and keyboard bindings
  - `MainWindow.xaml.cs` — command routing only; every handler is a one-liner to ViewModel
  - `Options.xaml(.cs)` — settings/preferences dialog
  - `FilterDialog.xaml(.cs)` — filter text entry with 9 presets
  - `AppendTextDialog.xaml(.cs)` — dialog for appending text to selected tasks
  - `DeleteConfirmationDialog.xaml(.cs)` — delete confirmation prompt
  - `SetPriorityDialog.xaml(.cs)` — priority letter entry
  - `SetDueDateDialog.xaml(.cs)` — date picker for due date
  - `postponedialog.xaml(.cs)` — postpone by N days input
  - `Help.xaml(.cs)` — about/help window (HTML content from `Resource.resx`)
  - `IntellisenseTextBox.cs` — code-only custom `TextBox` subclass (no XAML file)
  - `App.xaml(.cs)` — WPF application bootstrap; handles portable mode and settings migration

**`Client/Converters/`:**
- Purpose: WPF `IValueConverter` implementations used in XAML bindings
- Key files:
  - `ActiveFilterToStatusConvertor.cs` — converts filter number to status bar string ("Filter #: 2" or "Filter: None")
  - `SortTypeToDescriptionConvertor.cs` — converts `SortType` enum to display string using `[Description]` attribute

**`Client/Utilities/`:**
- Purpose: General utility helpers in the Client project
- Key files: `EnumExtensions.cs` — reads `[Description]` attribute from enum values

**`ToDoLib/`:**
- Purpose: Self-contained domain layer — pure C#, no WPF or UI dependency
- All persistence operations go through `TaskList`; task domain logic lives in `Task`

**`CommonExtensions/`:**
- Purpose: Cross-project utility extensions consumed by both `ToDoLib` and `Client`
- Key files:
  - `StringExtensions.cs` — case-insensitive `Contains`, date comparison helpers (`IsDateGreaterThan`, `IsDateLessThan`), `IsNullOrEmpty`
  - `IEnumerableExtensions.cs` — `IsNullOrEmpty` for collections

**`ColorFont/`:**
- Purpose: Reusable WPF color+font picker dialog. Self-contained with its own ViewModel.
- Used in `Client` only via `ShowOptionsDialog` when the user selects a task list font

## Key File Locations

**Entry Points:**
- `Client/Controls/App.xaml.cs`: Application startup — portable mode, settings migration, creates `MainWindow`
- `Client/Controls/MainWindow.xaml`: Declares all `RoutedUICommand` resources and bindings
- `Client/Controls/MainWindow.xaml.cs`: Command routing to ViewModel (no logic)

**Core Orchestration:**
- `Client/MainWindowViewModel.cs`: All application logic — task operations, filter/sort, dialog coordination, file load/save (~1900 lines)

**Domain Model:**
- `ToDoLib/Task.cs`: Task parsing, serialization, priority/date mutation methods
- `ToDoLib/TaskList.cs`: CRUD over the todo.txt flat file

**Configuration / Settings:**
- `Client/User.settings`: ApplicationSettings schema — defines all user preferences
- `Client/User.cs` / `Client/User.Designer.cs`: Auto-generated typed accessor for settings
- `Client/PortableSettingsProvider.cs`: Alternative settings provider for portable deployments (XML beside exe)

**Infrastructure:**
- `Client/FileChangeObserver.cs`: `FileSystemWatcher` wrapper — triggers reload when the file changes externally
- `Client/HotKey.cs`: P/Invoke wrapper for Win32 `RegisterHotKey` / `UnregisterHotKey`
- `Client/TrayMainWindows.cs`: System tray `NotifyIcon` with double-click restore and context menu
- `Client/UpdateChecker.cs`: Background thread fetch of `Updates.xml` from GitHub to detect new versions
- `ToDoLib/Log.cs`: Mutex-protected append-only file logger; log path is `%APPDATA%\Hughesoft\todotxt.exe\log.txt`

**Testing:**
- `ToDoTests/TaskTests.cs`: Unit tests for `Task` parsing logic
- `ToDoTests/TaskListTests.cs`: Unit tests for `TaskList` CRUD
- `ToDoTests/Data.resx`: Embedded resource file with raw todo.txt test strings

## Naming Conventions

**Files:**
- C# class files: PascalCase matching the class name (`MainWindowViewModel.cs`, `TaskList.cs`)
- XAML/code-behind pairs: share exact name (`FilterDialog.xaml` + `FilterDialog.xaml.cs`)
- One exception: `postponedialog.xaml` (all lowercase — inconsistency)
- Designer-generated files: `*.Designer.cs` suffix (`User.Designer.cs`, `Data.Designer.cs`)

**Classes:**
- PascalCase throughout (`TaskList`, `MainWindowViewModel`, `FileChangeObserver`)
- Dialog windows suffixed `Dialog` (`FilterDialog`, `SetPriorityDialog`, `AppendTextDialog`)
- Converters suffixed `Convertor` (note: project uses British-variant spelling — `Convertor` not `Converter`)

**Methods:**
- PascalCase for public/internal methods
- camelCase for private helpers with leading underscore on private fields (`_window`, `_sortType`, `_changefile`)
- ViewModel public command methods match the XAML command name: `ViewModel.ToggleCompletion()` for command `ToggleCompletion`
- Code-behind handlers follow pattern `[CommandName]Executed` (e.g., `ToggleCompletionExecuted`)

**Variables / Fields:**
- Private instance fields: leading underscore + camelCase (`_selectedTasks`, `_updating`)
- Public properties: PascalCase (`TaskList`, `SortType`, `HelpPage`)
- Local variables: camelCase

**XAML:**
- Named controls use camelCase (`lbTasks`, `taskText`, `filterMenu`, `sortMenu`, `webBrowser1`)
- Command keys in XAML resources match ViewModel method names exactly (e.g., `x:Key="ToggleCompletion"`)

## Where to Add New Code

**New Task Operation (e.g., a new way to modify a task):**
1. Add a `RoutedUICommand` resource in `Client/Controls/MainWindow.xaml` (under the appropriate menu section)
2. Add keyboard binding in the `<Window.InputBindings>` block in `MainWindow.xaml`
3. Add `CanExecute` check and `Executed` handler (one-liner) in `Client/Controls/MainWindow.xaml.cs`
4. Add the public method to `Client/MainWindowViewModel.cs` — use `ModifySelectedTasks(Func<Task,dynamic,Task>, param)` pattern for any operation that transforms selected tasks
5. Add corresponding private `*Task(Task, dynamic)` implementation method in the ViewModel

**New Dialog:**
1. Add `[Name]Dialog.xaml` + `[Name]Dialog.xaml.cs` to `Client/Controls/`
2. Instantiate from the ViewModel (not from code-behind): `var dialog = new [Name]Dialog(...); dialog.Owner = _window; dialog.ShowDialog()`

**New Domain Logic (task parsing/manipulation):**
- Add to `ToDoLib/Task.cs` or `ToDoLib/TaskList.cs` as appropriate
- Pure C# only — no WPF/System.Windows references allowed in `ToDoLib`

**New Application-Level Utility:**
- Shared between projects → `CommonExtensions/StringExtensions.cs` or `CommonExtensions/IEnumerableExtensions.cs`
- Client-only helpers → `Client/` root (for infrastructure) or `Client/Utilities/` (for small helpers)

**New Settings:**
- Add property to `Client/User.settings` (via Visual Studio Settings Designer or directly in XML)
- Access via `User.Default.[PropertyName]`; save with `User.Default.Save()`

**New Tests:**
- Task model tests → `ToDoTests/TaskTests.cs`
- TaskList file I/O tests → `ToDoTests/TaskListTests.cs`
- Test data strings → embed in `ToDoTests/Data.resx`

## Special Directories

**`.build/`:**
- Purpose: Build artifacts and CI output directory
- Generated: Yes
- Committed: No (typically)

**`.planning/codebase/`:**
- Purpose: GSD codebase map documents
- Generated: Yes (by GSD tooling)
- Committed: Yes

**`Installer/`:**
- Purpose: Inno Setup script and build bat to produce the Windows installer
- Generated: No (hand-maintained)
- Committed: Yes

---

*Structure analysis: 2025-01-31*
