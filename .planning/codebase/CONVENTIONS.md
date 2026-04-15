# Coding Conventions

**Analysis Date:** 2025-01-31

## Naming Patterns

**Files:**
- PascalCase for all `.cs` source files: `TaskList.cs`, `MainWindowViewModel.cs`, `ExceptionExtensions.cs`
- XAML files use PascalCase: `MainWindow.xaml`, `FilterDialog.xaml`, `AppendTextDialog.xaml`
- XAML code-behind matches XAML name: `MainWindow.xaml.cs`, `FilterDialog.xaml.cs`
- Auto-generated designer files use `.Designer.cs` suffix: `Data.Designer.cs`, `Resource.Designer.cs`
- One class per file; file name matches class name

**Classes and Interfaces:**
- PascalCase: `TaskList`, `MainWindowViewModel`, `ExceptionExtensions`, `IEnumerableExtensions`
- Extension class names end in `Extensions`: `StringExtensions`, `IEnumerableExtensions`, `ExceptionExtensions`, `EnumExtensions`
- Exception classes end in `Exception`: `TaskException`
- ViewModel classes end in `ViewModel`: `MainWindowViewModel`
- Dialog classes end in `Dialog`: `FilterDialog`, `AppendTextDialog`, `DeleteConfirmationDialog`

**Methods:**
- PascalCase for all public and private methods: `ReloadTasks()`, `UpdateTaskListMetaData()`, `RaiseModifiedEvent()`
- Boolean-returning methods use `Is`/`Are`/`Has` prefix: `IsTaskSelected()`, `AreTasksSelected()`, `IsNullOrEmpty()`, `HasValues()`
- Event handlers named `{Noun}_{Event}`: `TaskList_Modified`
- Private helpers use descriptive verb phrases: `GetSelectedTasks()`, `SetSelectedTasks()`, `WriteAllTasksToFile()`

**Variables and Fields:**
- Private instance fields prefixed with `_`: `_filePath`, `_sortType`, `_window`, `_updating`, `_selectedTasks`
- Local variables use `camelCase`: `sortedTaskList`, `fileContents`, `uniqueProjects`
- Constants use `PascalCase` (not UPPER_SNAKE): `CompletedPattern`, `PriorityPattern`, `DueDatePattern`
- Loop counter variables use short names: `i`, `j`, `c`
- Inconsistency: some local variable names in `TaskList.cs` use PascalCase (`UniqueProjects`, `UniqueContexts`) — this is non-standard and should be avoided

**Properties:**
- PascalCase: `Tasks`, `Projects`, `Contexts`, `DueDate`, `Completed`, `PreserveWhiteSpace`
- Boolean properties use adjective or `Is` prefix: `Completed`, `PreserveWhiteSpace`
- INotifyPropertyChanged backing fields use `camelCase` without underscore prefix in `MainWindowViewModel.cs`: `totalTasks`, `filteredTasks`, `incompleteTasks`, `tasksOverdue`

**Enums:**
- PascalCase enum name: `Due`, `SortType`, `LogLevel`
- PascalCase enum values: `NotDue`, `Today`, `Overdue`, `Alphabetical`, `DueDate`
- Description attributes used for display-friendly names: `[Description("Due Date")]`, `[Description("Order in file")]`

**Events:**
- PascalCase: `Modified`, `PropertyChanged`
- Event arguments follow `EventArgs` convention

## Code Style

**Formatting:**
- No EditorConfig or .editorconfig detected
- No Prettier, StyleCop, or similar automated formatter configuration detected
- Indentation is inconsistent: `Client/` uses 4-space indentation; `ToDoLib/` uses tab-based indentation
- Braces: `{` always on the same line for `if/else`; inconsistently placed for method bodies in some files
- Single-line `if` statements without braces appear in some places (e.g., `if (m.WaitOne(10000)) File.AppendAllText(...)`)

**Linting:**
- No explicit lint rules file detected
- Standard .NET compiler warnings at level 4 (per `.csproj` files)

## Import Organization

**Order:**
1. System namespaces (`using System;`, `using System.Collections.Generic;`, etc.)
2. Third-party namespaces
3. Project-internal namespaces (e.g., `using ToDoLib;`, `using CommonExtensions;`, `using ColorFont;`)

**Path Aliases:**
- None used; projects reference each other via assembly references in `.csproj`

**Namespace Structure:**
- `ToDoLib` — domain model library
- `Client` — WPF presentation layer
- `Client.Utilities` — utility helpers within client (e.g., `EnumExtensions`)
- `CommonExtensions` — shared extension methods across projects
- `ColorFont` — color/font helper library
- `ToDoTests` — test project

## Error Handling

**ToDoLib layer pattern:**
- All public I/O methods wrap operations in `try/catch(IOException ex)` then `catch(Exception ex)`
- `IOException` is caught, wrapped in a human-readable message, then re-thrown as `TaskException`
- Generic `Exception` is logged with `Log.Error(ex.ToString())` then re-thrown with bare `throw`
- `finally` blocks always call `UpdateTaskListMetaData()` and `RaiseModifiedEvent()`
- Example from `ToDoLib/TaskList.cs`:
  ```csharp
  catch (IOException ex)
  {
      var msg = "There was a problem trying to read from your todo.txt file.";
      Log.Error(msg, ex);
      throw new TaskException(msg, ex);
  }
  catch (Exception ex)
  {
      Log.Error(ex.ToString());
      throw;
  }
  finally
  {
      UpdateTaskListMetaData();
      RaiseModifiedEvent();
  }
  ```

**Client layer pattern:**
- `ExceptionExtensions.Handle()` provides a unified handler: logs via `Log.Error`, shows `MessageBox`
- Catches `Exception ex` and calls `ex.Handle("descriptive message")`
- `TaskException` sometimes caught separately and displayed directly without `Handle()`
- Unhandled exceptions caught globally at `Application.Current.DispatcherUnhandledException`
- Example from `Client/MainWindowViewModel.cs`:
  ```csharp
  catch (Exception ex)
  {
      ex.Handle("An error occurred while opening " + filePath);
  }
  ```
- Bare `catch` (swallowing all exceptions) used in one case inside `SelectTaskByIndex` for UI focus failures — avoid this pattern

## Logging

**Framework:** Custom static `Log` class in `ToDoLib/Log.cs`

**Log Levels:** `LogLevel.Debug` and `LogLevel.Error`; configured via `User.Default.DebugLoggingOn`

**Patterns:**
- Debug log at method entry and exit for significant operations: `Log.Debug("Loading tasks from {0}.", _filePath)`
- Error log before re-throwing: `Log.Error(msg, ex)`
- Log format uses `{0}` positional placeholders, not interpolation
- Log file location: `%APPDATA%\Hughesoft\todotxt.exe\log.txt`
- Log writes protected by `Mutex` for multi-process safety

## Comments

**When to Comment:**
- Class-level `/// <summary>` XML doc comments used for public APIs (e.g., `TaskList`, `TaskList.Update`)
- Method-level `/// <summary>` XML doc used for non-obvious public methods in `MainWindowViewModel`
- Inline `//` comments explain non-obvious logic, regex patterns, or workarounds
- TODO comments exist for known incomplete work (see `ToDoLib/Task.cs` lines 90 and 190)
- Attribution comments reference sources (e.g., StackOverflow links in `EnumExtensions.cs`)

**Examples:**
```csharp
/// <summary>
/// This method updates one task in the file. It works by replacing the "current task" with the "new task".
/// </summary>

// NB, you need asciiShift +1 to go from A to B, even though that's a 'decrease' in priority
```

## Function Design

**Size:** Methods are generally focused; `MainWindowViewModel.cs` contains a small number of longer methods (e.g., `FilterList`, `UpdateDisplayedTasks`) but most are under 30 lines

**Parameters:** Prefer named parameters for optional booleans: `bool preserveWhitespace = false`, `bool writeTasks = true`

**Return Values:** Methods return `void` or the relevant domain object; `null` returned on failure from some methods (e.g., `AddTaskFromTextBox` returns `null` on exception — avoid this implicit contract)

## Module Design

**Region Directives:**
- `#region` / `#endregion` used to organize large files into logical sections
- Standard regions used: `#region Properties`, `#region Constructor`, `#region Events`, `#region Filter Methods`
- Present in both `ToDoLib/TaskList.cs` and `Client/MainWindowViewModel.cs`

**Exports:**
- All public types are directly exported; no internal visibility used except auto-generated designer files
- Extension methods pattern used throughout for cross-cutting utilities

**Barrel Files:**
- Not used; no `index.cs` equivalent — each namespace is a separate assembly

## WPF-Specific Patterns

**MVVM:**
- `MainWindowViewModel` implements `INotifyPropertyChanged`
- `RaiseProperyChanged(nameof(Property))` used for all property setters (note: typo "Propery" vs "Property" in method name)
- ViewModel holds direct reference to `_window` (code-behind) — not pure MVVM, but consistent throughout

**Settings:**
- All user preferences accessed via `User.Default.*` (generated settings class in `Client/User.settings`)
- Settings saved with `User.Default.Save()` after mutations

---

*Convention analysis: 2025-01-31*
