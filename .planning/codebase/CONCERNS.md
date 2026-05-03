# Codebase Concerns

**Analysis Date:** 2025-01-24

---

## Tech Debt

**God Class: MainWindowViewModel**
- Issue: Single 2,004-line class handles sorting, filtering, file I/O, task CRUD, archive operations, print rendering, UI state management, options dialogs, keyboard input, and clipboard operations. Violates Single Responsibility Principle severely.
- Files: `Client\MainWindowViewModel.cs`
- Impact: Any change risks unintended side effects across unrelated features. Difficult to test in isolation. Methods are tightly coupled to `_window` (a `MainWindow` reference), preventing ViewModel reuse.
- Fix approach: Extract dedicated classes: `TaskFilterService`, `TaskSortService`, `ArchiveService`, `PrintService`, `TaskEditService`. Introduce interfaces so the ViewModel depends on abstractions, not on the concrete `MainWindow`.

**Task Identity Uses Raw String Equality**
- Issue: Task identity throughout the system is determined by comparing `Task.Raw` strings. Two tasks with identical text are indistinguishable. Update and Delete use `Tasks.First(t => t.Raw == task.Raw)`, which silently targets the wrong task when duplicates exist.
- Files: `ToDoLib\TaskList.cs` lines 175, 216, 221; `Client\MainWindowViewModel.cs` line 264
- Impact: If a user has two identical tasks, deleting or editing one will always affect the first match. Silent data corruption; no error is thrown.
- Fix approach: Assign a stable unique ID (e.g., `Guid`) to each `Task` at load time and use ID-based lookup. The `Raw` string can remain for serialization fidelity.

**Filter Preset Duplication (10-way Copy-Paste)**
- Issue: `ApplyFilterPreset0()` through `ApplyFilterPreset9()` are 10 identical one-line wrapper methods that each call `ApplyFilterPreset(n)`.
- Files: `Client\MainWindowViewModel.cs` lines 672–788
- Impact: Adding or changing behaviour requires updating 10 methods. Pure boilerplate.
- Fix approach: Bind commands with a parameter in XAML and route to a single `ApplyFilterPreset(int n)` method.

**`dynamic` Typed Parameters in Task Modification Pipeline**
- Issue: `ModifySelectedTasks(Func<Task, dynamic, Task>, dynamic)` uses `dynamic` to pass parameters to task-modification lambdas. This bypasses compile-time type checking.
- Files: `Client\MainWindowViewModel.cs` lines 934, 1214, 1224, 1234, 1241, 1368
- Impact: Runtime `RuntimeBinderException` possible if a wrong type is passed. Anonymous types (`new { Days = 1, DateType = "due" }`) are accessed via `daysToPostpone.Days` and `daysToPostpone.DateType` with no compile-time validation.
- Fix approach: Define a concrete `PostponeParameters` record/struct and replace `dynamic` with typed parameters.

**TODO Comments Left in Shipped Code**
- Issue: Two explicit `//TODO` comments mark unfinished specification work.
- Files:
  - `ToDoLib\Task.cs` line 90: `//TODO priority regex need to only recognice upper case single chars` — the priority pattern currently accepts lowercase priority characters despite the spec requiring uppercase only.
  - `ToDoLib\Task.cs` line 190: `//TODO other languages` — relative date parsing is English-only with no i18n path documented.
- Impact: Priority regex accepts `(a)` as a valid priority when the todo.txt spec requires `(A)`. Tasks with lowercase priorities may sort or filter incorrectly.
- Fix approach: Change `PriorityPattern` from `RegexOptions.IgnoreCase` to case-sensitive; add an uppercase-only character class `[A-Z]`.

**`ArchiveCompleted` Race Condition**
- Issue: `ArchiveCompleted()` calls `TaskList.ReloadTasks()` *after* capturing `completed` tasks from the old in-memory list. If the file was modified externally between capture and reload, the `completed` snapshot can be stale or inconsistent with the reloaded file.
- Files: `Client\MainWindowViewModel.cs` lines 1541–1555
- Impact: Completed tasks from the stale snapshot may fail to match during `TaskList.Delete()`, throwing `InvalidOperationException` from `Tasks.First()`.
- Fix approach: Reload before capturing the `completed` list, or reload once and derive both the archive and delete operations from the same snapshot.

---

## Known Bugs

**`Completed_adds_x_to_begining` Test Has No Assertions**
- Symptoms: Test passes trivially regardless of whether `Completed = true` correctly prepends "x" to the task string.
- Files: `ToDoTests\TaskTests.cs` lines 304–310
- Trigger: Test always passes, masking any regression in completion formatting.
- Workaround: None — the bug is silently ignored.

**Priority Pattern Accepts Lowercase (Spec Violation)**
- Symptoms: `new Task("(a) lowercase priority task")` parses `(a)` as a valid priority.
- Files: `ToDoLib\Task.cs` line 126 (`RegexOptions.IgnoreCase` on `PriorityPattern`), and the TODO comment on line 90.
- Trigger: Enter any task with a lowercase priority letter.
- Workaround: Always use uppercase priority letters.

**`DeleteTasks` Iterates `SelectedItems` While Modifying Collection**
- Symptoms: Deleting multiple selected tasks iterates `_window.lbTasks.SelectedItems` and calls `TaskList.Delete()` which calls `WriteAllTasksToFile()` each iteration.
- Files: `Client\MainWindowViewModel.cs` lines 1038–1041
- Trigger: Select and delete multiple tasks simultaneously.
- Impact: O(n²) file writes. Collection modification during iteration may cause skips. `ReloadTasks()` is called at the start but not between deletes, so `Tasks.First(t => t.Raw == ...)` may fail if a prior delete already removed a matching raw string.

**`ShowPostponeDialog` Returns 0 for "today" Without Setting Date**
- Symptoms: If the user types "today" in the postpone dialog, `return 0` exits before `PostponeTask` is called. `PostponeTask` skips the update when `Days == 0`. The `ModifySelectedTasks(SetTaskDueDate, DateTime.Today)` call on line 1438 is a side-effectful pre-call inside a dialog handler, but the 0 return means the outer `PostponeTask` never runs.
- Files: `Client\MainWindowViewModel.cs` lines 1438–1443
- Trigger: Enter "today" in the postpone dialog.

---

## Security Considerations

**`Process.Start` Called with Unvalidated External Data**
- Risk: `UrlService.cs` calls `Process.Start(link.NavigateUri.ToString())` directly. If a todo.txt file contains a malformed URL with a non-http scheme (e.g., `file://`, `cmd://`, `javascript:`), it may launch unintended processes.
- Files: `Client\UrlService.cs` line 86; `Client\Controls\Help.xaml.cs` line 35; `Client\MainWindowViewModel.cs` lines 1663, 1670
- Current mitigation: `Uri.IsWellFormedUriString` is checked before adding the hyperlink, but `Process.Start` still fires for any well-formed URI including `file://` paths.
- Recommendations: Whitelist URI schemes (`http`, `https`) before calling `Process.Start`. Use `UseShellExecute = true` with explicit scheme validation.

**Update Check Uses HTTP (Not HTTPS) for Download Redirect**
- Risk: `updateClientUrl = "http://benrhughes.com/todotxt.net"` is plain HTTP.
- Files: `Client\UpdateChecker.cs` line 12
- Current mitigation: The XML feed itself uses HTTPS (`raw.github.com`). The HTTP URL is only shown as a menu label/link.
- Recommendations: Change to HTTPS.

**Settings File Stored in Executable Directory**
- Risk: `PortableSettingsProvider` writes `{appname}.settings` to the directory containing the executable. On systems where the app is installed in `Program Files`, this will silently fail (permissions), losing all user settings. The failure is swallowed by an empty `catch (Exception)` block.
- Files: `Client\PortableSettingsProvider.cs` lines 20–27, 101–109
- Recommendations: Fall back to `%APPDATA%` when the executable directory is not writable; log the failure rather than silently swallowing it.

---

## Performance Bottlenecks

**Full File Rewrite on Every Single Task Operation**
- Problem: `Add`, `Delete`, and `Update` each call `WriteAllTasksToFile()`, which rewrites the entire file. `Delete` performs one file write per deleted task in a multi-select delete loop.
- Files: `ToDoLib\TaskList.cs` lines 137, 177, 228; `Client\MainWindowViewModel.cs` lines 1038–1041
- Cause: The design is intentional for conflict safety (per comment on line 14–17), but multi-task operations don't batch writes.
- Improvement path: Add a `DeleteRange(IEnumerable<Task>)` method that performs one reload + one write, instead of N writes.

**`GetPreferredFileLineEndingFromFile` Opens File Twice on Load**
- Problem: `ReloadTasks()` opens the file via `File.OpenRead`, then immediately calls `GetPreferredFileLineEndingFromFile()` which opens it again with `new StreamReader`.
- Files: `ToDoLib\TaskList.cs` lines 88, 103, 249–287
- Improvement path: Detect line endings during the initial `StreamReader` pass.

**`FileChangeObserver` Uses `Thread.Sleep(1000)` on File-Watcher Thread**
- Problem: Every external file change blocks the file-system watcher thread for 1 second.
- Files: `Client\FileChangeObserver.cs` line 63
- Improvement path: Use a debounce timer (`DispatcherTimer` or `System.Threading.Timer`) instead of a blocking sleep.

**`SortList` Calls `tasks.Count()` on `IEnumerable` for Logging**
- Problem: `tasks.Count()` on an `IEnumerable<Task>` enumerates the entire sequence just for a debug log message.
- Files: `Client\MainWindowViewModel.cs` line 803
- Improvement path: Pass a collection type or use `.ToList()` before sorting, then call `.Count` on the list.

---

## Fragile Areas

**Task Parsing is Order-Dependent and Non-Recoverable**
- Files: `ToDoLib\Task.cs` lines 91–182
- Why fragile: Parsing strips tokens sequentially by replacing regex matches on a mutating `raw` string. If any regex over-matches (e.g., `CreatedDatePattern` matching a date inside a URL), subsequent fields are silently corrupted with no error. The `PriorityPattern` uses `RegexOptions.IgnoreCase` against spec.
- Safe modification: Write unit tests covering edge cases (tasks with dates in URLs, multiple date fields, priority-like patterns inside body text) before changing any pattern constant.
- Test coverage: `ToDoTests\TaskTests.cs` covers happy paths; no tests for malformed/adversarial input.

**`SetSelectedTasks` Relies on `Raw` String Matching After Sort/Reload**
- Files: `Client\MainWindowViewModel.cs` lines 245–290
- Why fragile: After a reload or sort, `SetSelectedTasks` matches previously selected tasks by `.Raw` equality. If a round-trip through `Task.ToString()` changes whitespace or field ordering (which it can when `Raw` is null), the selection is silently lost and focus jumps to index 0.
- Test coverage: Not tested.

**`AddCalendarToTitle` Uses Magic Length Check `< 15`**
- Files: `Client\MainWindowViewModel.cs` line 1678
- Why fragile: Toggle logic checks `title.Length < 15` to decide if the calendar is already shown. The application title "todotxt.net" is 11 chars, giving 4 chars of margin. Any change to the window title format will break the toggle.
- Test coverage: Not tested.

**`PortableSettingsProvider._rootDocument` Silently Fails on XML Parse Error**
- Files: `Client\PortableSettingsProvider.cs` lines 56–64
- Why fragile: If the settings file is corrupted, the `catch (Exception)` block swallows the error, `_xmlDocument` is left null, and the next line `_xmlDocument.SelectSingleNode(...)` throws a `NullReferenceException`. This crashes the app on startup with no user-friendly message.
- Test coverage: Not tested.

**`EmitGroupHeader` Uses Mutable Shared State `_viewGroups`**
- Files: `Client\MainWindowViewModel.cs` lines 1970–1990
- Why fragile: `_viewGroups` is a field populated inside `GetPrintContents()` and consumed destructively via `RemoveAt(0)`. If `GetPrintContents()` throws mid-way, `_viewGroups` is left partially consumed, corrupting the next print operation.
- Test coverage: Not tested.

---

## Dependencies at Risk

**NUnit 2.6.4 (2013 — Three Major Versions Behind)**
- Risk: NUnit 2.6.4 predates the current NUnit 3.x/4.x API. `[TestFixtureSetUp]` and `[TestFixtureTearDown]` attributes used in tests are obsolete in NUnit 3+ (replaced by `[OneTimeSetUp]` / `[OneTimeTearDown]`). Upgrading NUnit will require test attribute changes.
- Files: `ToDoTests\packages.config`; `ToDoTests\TaskListTests.cs` lines 17, 24
- Impact: Tests will fail to compile if NUnit is upgraded without updating attributes.
- Migration plan: Update to NUnit 3.x or 4.x; replace obsolete attributes; consider migrating to `xUnit` or `MSTest` for better tooling integration.

**Target Framework: .NET 4.0 Client Profile (End-of-Life)**
- Risk: .NET Framework 4.0 reached end of support. The `Client Profile` subset is no longer a recognised target in modern SDK-style projects.
- Files: `Client\Client.csproj`, `ToDoLib\ToDoLib.csproj`, `ToDoTests\ToDoTests.csproj`
- Impact: No security patches. Modern NuGet packages no longer publish `net40` TFMs. Cannot use C# language features beyond 5.0.
- Migration plan: Migrate to .NET 4.8 (maintains WPF, minimal breaking changes) or .NET 8 with `net8.0-windows` TFM for long-term support.

**`Unofficial.Microsoft.mshtml` (Unofficial Package)**
- Risk: This is an unofficial NuGet re-packaging of the COM `mshtml` interop. It is used for the print-preview feature via `WebBrowser`/`IHTMLDocument2`. The `WebBrowser` control is a legacy IE11 wrapper, deprecated since IE is end-of-life.
- Files: `Client\packages.config`; `Client\Controls\MainWindow.xaml.cs` lines 241–258
- Impact: Print preview functionality depends on the IE rendering engine, which is removed in Windows 11 consumer builds.
- Migration plan: Replace print preview with a `FlowDocument`/`DocumentViewer` WPF approach, or use `WebView2` (Edge-based).

**`XmlTextReader` Deprecated**
- Risk: `XmlTextReader` is marked obsolete since .NET 2.0; Microsoft recommends `XmlReader.Create()`.
- Files: `Client\UpdateChecker.cs` line 30
- Impact: No immediate runtime failure, but suppresses IDE warnings; not a security risk in this context.
- Migration plan: Replace `new XmlTextReader(url)` with `XmlReader.Create(url)`.

---

## Missing Critical Features

**No Undo/Redo Support**
- Problem: Every task modification immediately writes to disk. There is no undo buffer.
- Blocks: Users who accidentally delete or modify tasks cannot recover without manually editing the file.

**No Backup Before Write**
- Problem: `WriteAllTasksToFile` overwrites the file with no backup.
- Files: `ToDoLib\TaskList.cs` lines 289–311
- Blocks: A crash mid-write (power loss, exception in `writer.WriteLine`) corrupts the todo.txt file with no recovery path.

---

## Test Coverage Gaps

**No Tests for `MainWindowViewModel`**
- What's not tested: All sorting logic, filter logic, task modification pipeline, archive logic, print content generation.
- Files: `Client\MainWindowViewModel.cs` (2,004 lines, 0 test coverage)
- Risk: Any refactoring of the ViewModel breaks silently.
- Priority: High

**No Tests for `PortableSettingsProvider`**
- What's not tested: Settings persistence, corruption recovery, write-failure fallback.
- Files: `Client\PortableSettingsProvider.cs`
- Risk: Settings corruption crashes the app on next launch with `NullReferenceException`.
- Priority: High

**No Tests for `FileChangeObserver`**
- What's not tested: Concurrent file access, watcher lifecycle (enable/disable), dispose behaviour.
- Files: `Client\FileChangeObserver.cs`
- Risk: File-watcher double-subscription or dangling references after settings change.
- Priority: Medium

**`Completed_adds_x_to_begining` Has No Assertions**
- What's not tested: The actual output string after setting `Completed = true`.
- Files: `ToDoTests\TaskTests.cs` lines 304–310
- Risk: Completion formatting can regress without the test failing.
- Priority: Medium

**No Integration Tests for Multi-Task Delete / Archive**
- What's not tested: Deleting multiple tasks simultaneously; archiving completed tasks when duplicates exist.
- Files: `Client\MainWindowViewModel.cs` lines 1009–1056, 1529–1562
- Risk: Duplicate-`Raw` silent mis-targeting; collection-modification bugs.
- Priority: High

---

*Concerns audit: 2025-01-24*
