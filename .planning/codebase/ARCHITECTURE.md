# Architecture

**Analysis Date:** 2025-01-31

## Pattern Overview

**Overall:** Hybrid MVVM + Code-Behind (WPF)

**Key Characteristics:**
- `MainWindowViewModel` implements `INotifyPropertyChanged` and holds all business logic
- The View (`MainWindow`) retains a typed back-reference to the ViewModel (`_window`), so the ViewModel can directly access named UI controls (`_window.lbTasks`, `_window.taskText`, etc.)
- `MainWindow.xaml.cs` acts purely as a command router — every handler is a one-liner forwarding to the ViewModel
- Domain logic lives in a separate library (`ToDoLib`) with no WPF dependency
- Settings persistence uses `ApplicationSettings` (`User.Default`) as a global state bag throughout both the Client and ViewModel

## Layers

**Domain Library:**
- Purpose: Parse, represent, and persist todo.txt tasks — no UI dependency whatsoever
- Location: `ToDoLib/`
- Contains: `Task` (model + parser), `TaskList` (file I/O), `Log` (file logger), `TaskException`
- Depends on: `CommonExtensions`
- Used by: `Client` project

**Presentation / Application Layer:**
- Purpose: WPF application — windows, dialogs, commands, settings, and orchestration
- Location: `Client/`
- Contains: `MainWindowViewModel`, `MainWindow` (and all dialogs), converters, `FileChangeObserver`, hot-key/tray helpers
- Depends on: `ToDoLib`, `CommonExtensions`, `ColorFont`

**Supporting Libraries:**
- `CommonExtensions/` — `StringExtensions` (date comparison helpers, case-insensitive `Contains`) and `IEnumerableExtensions`; used by both `ToDoLib` and `Client`
- `ColorFont/` — standalone WPF color/font picker dialog (MVVM-within-a-control via `ColorPickerViewModel`); used only by `Client` options dialog

## Data Flow

**Opening / Loading a File:**

1. `App.Application_Startup` → creates `MainWindow`, calls `mainWindow.Show()`
2. `MainWindow.Window_Loaded` → creates `MainWindowViewModel(this)`; ViewModel constructor calls `LoadTasks(User.Default.FilePath)` if a file was previously saved
3. `LoadTasks` → creates `new TaskList(filePath)` → `TaskList.ReloadTasks()` reads all lines from disk, constructs a `Task` per line, populates `Tasks` list and metadata (Projects, Contexts, Priorities)
4. `TaskList.Modified` event fires → ViewModel subscribes → updates `TotalTasks` count
5. `ViewModel.UpdateDisplayedTasks()` → `FilterList` → `SortList` → binds result to `_window.lbTasks.ItemsSource`

**Adding a Task:**

1. User types in `taskText` (`IntellisenseTextBox`) — `@` / `+` / `(` triggers autocomplete popup sourced from `TaskList.Contexts` / `.Projects`
2. User presses Enter (or Ctrl+Enter per option) → `MainWindow.taskText_PreviewKeyUp` → `ViewModel.TaskTextPreviewKeyUp`
3. `AddTaskFromTextbox()` → optionally prepends creation date → `new Task(rawString)` → `TaskList.Add(task)` → appends to file on disk, adds to in-memory `Tasks`
4. `UpdateDisplayedTasks()` re-filters and re-sorts the list box

**Modifying a Task (Generic Pattern):**

1. Menu/command handler in `MainWindow.xaml.cs` calls a ViewModel method (e.g., `ViewModel.ToggleCompletion()`)
2. ViewModel calls `ModifySelectedTasks(Func<Task,dynamic,Task> modificationFunction, param)` — a single generic mutation dispatcher
3. `ModifySelectedTasks` disables `FileChangeObserver`, calls `TaskList.ReloadTasks()` (fresh read), applies the transform to each selected task, calls `TaskList.Update(old, new)` (full file rewrite), re-enables observer
4. `UpdateDisplayedTasks()` rebinds the list box

**External File Change:**

1. `FileChangeObserver` wraps `FileSystemWatcher` watching the active file
2. `OnFileChanged` event fires (after 1-second sleep to allow writer to release lock)
3. Dispatched to UI thread via `_window.Dispatcher.BeginInvoke` → `ViewModel.ReloadFile()` → `TaskList.ReloadTasks()` → `UpdateDisplayedTasks()`

**State Management:**
- Persistent state (file path, sort type, filter presets, font, window size): `User.Default` (`ApplicationSettings`) — saved with `User.Default.Save()` after every change
- Portable mode: `PortableSettingsProvider` (`Client/PortableSettingsProvider.cs`) overrides `ApplicationSettings` to write XML beside the `.exe` instead of `%APPDATA%`
- In-memory transient state: `_selectedTasks` list in ViewModel, `_updating` field (the task being edited), `_sortType`, `ActiveFilterNumber`

## Key Abstractions

**`Task` (ToDoLib/Task.cs):**
- Purpose: Represents one todo.txt line — parses raw text via regex into structured fields, and serializes back preserving raw format
- Pattern: Regex-driven parsing in constructor; `Raw` string is the canonical representation; `ToString()` prefers `Raw` with minimal mutations
- Properties: `Priority`, `Projects` (list), `Contexts` (list), `DueDate`, `ThresholdDate`, `CreationDate`, `CompletedDate`, `Completed`, `Body`
- Relative date resolution (`today`, `tomorrow`, weekday names) happens at parse time

**`TaskList` (ToDoLib/TaskList.cs):**
- Purpose: Thin file I/O abstraction — treats itself as a direct mirror of the todo.txt file; every mutation writes through to disk immediately
- Pattern: Each public method (Add, Delete, Update) reads/writes the file autonomously to minimise concurrent conflict risk
- Event: `Modified` fires after every mutation; ViewModel subscribes to update `TotalTasks`
- Metadata: `Projects`, `Contexts`, `Priorities` are derived via `UpdateTaskListMetaData()` after every reload/mutation

**`MainWindowViewModel` (Client/MainWindowViewModel.cs):**
- Purpose: Central orchestrator — owns `TaskList`, drives all task operations, filter/sort state, dialog coordination, and status bar values
- Pattern: Exposes public methods named after UI commands; `ModifySelectedTasks` is the generic task-mutation dispatcher using `Func<Task,dynamic,Task>`
- Implements `INotifyPropertyChanged` for status bar bindings (`TotalTasks`, `FilteredTasks`, `IncompleteTasks`, `TasksDueToday`, `TasksOverdue`, `SortType`, `ActiveFilterNumber`)

**`IntellisenseTextBox` (Client/Controls/IntellisenseTextBox.cs):**
- Purpose: Custom `TextBox` subclass with a `Popup`-based autocomplete list triggered by `+`, `@`, `(` characters
- Sources completions from the bound `TaskList.Projects`, `.Contexts`, or a static priority list (A–Z)
- Registered as a WPF dependency property `TaskListProperty` so it can receive the `TaskList` via XAML binding

**`RoutedUICommand` (Client/Controls/MainWindow.xaml):**
- All application commands are declared as `RoutedUICommand` resources in XAML
- `CanExecute` guards are defined in `MainWindow.xaml.cs` (`WhenTasksSelectedCanExecute`, `WhenSingleTaskSelectedCanExecute`, etc.)
- Execute handlers in `MainWindow.xaml.cs` each contain a single ViewModel call — no logic lives in code-behind

## Entry Points

**Application Bootstrap:**
- Location: `Client/Controls/App.xaml.cs`
- Triggers: WPF application startup
- Responsibilities: Detects portable mode (compile flag `PORTABLE`), migrates settings from previous version (`User.Default.Upgrade()`), creates and shows `MainWindow`

**Main Window:**
- Location: `Client/Controls/MainWindow.xaml` + `Client/Controls/MainWindow.xaml.cs`
- Triggers: `App.Application_Startup`
- Responsibilities: Declares all `RoutedUICommand` resources and key bindings; wires up command `Executed`/`CanExecute` handlers; sets up tray icon and global hotkey; creates `MainWindowViewModel` on `Window_Loaded`

**MainWindowViewModel Constructor:**
- Location: `Client/MainWindowViewModel.cs` line 174
- Triggers: Called from `MainWindow.Window_Loaded`
- Responsibilities: Initialises sort type from settings; opens the last-used file if path is stored; sets up the `FileChangeObserver`; subscribes to `TaskList.Modified`

## Error Handling

**Strategy:** Catch-log-show. Exceptions are caught at the ViewModel boundary, logged via `ToDoLib.Log`, and shown to the user via `MessageBox`.

**Patterns:**
- `ExceptionExtensions.Handle(string message)` extension method (`Client/ExceptionExtensions.cs`) — single call logs + shows `MessageBox`
- `TaskException` wraps `IOException` from `TaskList` methods with user-friendly messages
- `TaskList` methods all use `try/catch/finally` — `finally` always calls `UpdateTaskListMetaData()` + `RaiseModifiedEvent()` even on failure
- Global unhandled exception handler registered in `MainWindow` constructor via `Application.Current.DispatcherUnhandledException`

## Cross-Cutting Concerns

**Logging:** `ToDoLib.Log` static class writes timestamped entries to a text file (`%APPDATA%\Hughesoft\todotxt.exe\log.txt` or `log.txt` beside exe in portable mode). Mutex-protected for multi-instance safety. `LogLevel` (Error or Debug) controlled by user setting.

**Validation:** Input validation is inline in ViewModel methods (null/empty checks, regex matches). No validation framework.

**Settings:** `User.Default` (`ApplicationSettings`) is the universal settings store. Read directly at usage site throughout ViewModel and dialogs; saved with `User.Default.Save()` immediately after changes. Portable alternative is `PortableSettingsProvider` (XML file beside exe).

**File Locking:** `TaskList` deliberately performs short-lived, autonomous file operations (read + write per operation) to minimise lock duration and reduce conflict with external editors. `FileChangeObserver` is disabled/re-enabled around bulk multi-task operations.

---

*Architecture analysis: 2025-01-31*
