# Testing Patterns

**Analysis Date:** 2025-01-31

## Test Framework

**Runner:**
- NUnit 2.6.4
- Config: `ToDoTests/ToDoTests.csproj` (references `packages\NUnit.2.6.4\lib\nunit.framework.dll`)
- Test runner: NUnit GUI or NUnit console (no MSTest or xUnit)

**Assertion Library:**
- NUnit built-in: `Assert.AreEqual`, `Assert.IsTrue`, `Assert.Fail`, `CollectionAssert.AreEquivalent`

**Run Commands:**
```bash
# Via NUnit console (from solution root)
nunit-console.exe ToDoTests\bin\Debug\ToDoTests.dll

# Via Visual Studio Test Explorer (NUnit adapter required)
# Build the solution first, then run tests from Test Explorer
```

## Test File Organization

**Location:**
- Separate `ToDoTests/` project, not co-located with source
- `ToDoTests/` references both `ToDoLib` and `Client` project assemblies

**Naming:**
- Test class files named `{Subject}Tests.cs`: `TaskTests.cs`, `TaskListTests.cs`
- Test class name matches file name exactly

**Structure:**
```
ToDoTests/
├── TaskTests.cs          # Tests for ToDoLib/Task.cs
├── TaskListTests.cs      # Tests for ToDoLib/TaskList.cs
├── Data.resx             # Resource file holding test data path
├── Data.Designer.cs      # Generated resource accessor (auto-generated)
├── Properties/
│   └── AssemblyInfo.cs
├── ToDoTests.csproj
└── packages.config
```

## Test Structure

**Suite Organization:**
```csharp
[TestFixture]
public class TaskTests
{
    // Shared test data declared as fields
    List<string> _projects = new List<string>() { "+test" };
    List<string> _contexts = new List<string>() { "@work" };

    #region Create
    [Test]
    public void Create_Priority_Body_Project_Context()
    {
        var task = new Task("(A) This is a test task +test @work");
        var expectedTask = new Task("(A)", _projects, _contexts, "This is a test task");
        AssertEquivalence(expectedTask, task);
    }
    #endregion
}
```

**Test Method Naming:**
- `{MethodOrConcept}_{Condition}` pattern: `Create_Priority_Body_Project_Context`, `Add_ToCollection`, `Delete_InFile`
- Underscores separate the subject and scenario
- No "Should" prefix used
- Descriptive enough to understand the intent without reading the body

**Patterns:**
- `[TestFixture]` on all test classes
- `[Test]` on every test method
- `[TestFixtureSetUp]` / `[TestFixtureTearDown]` for class-level setup/teardown (NUnit 2.x style)
- No `[SetUp]` / `[TearDown]` per-test attributes observed — setup is inlined in each test
- `#region` directives group tests by method/feature being tested

## Setup and Teardown

**Fixture-Level Setup:**
```csharp
[TestFixtureSetUp]
public void TFSetup()
{
    if (!File.Exists(Data.TestDataPath))
        File.WriteAllText(Data.TestDataPath, "");
}

[TestFixtureTearDown]
public void TearDown()
{
    if (File.Exists(Data.TestDataPath))
        File.Delete(Data.TestDataPath);
}
```
- Located in `ToDoTests/TaskListTests.cs`
- Creates and deletes a real file on disk (`testtasks.txt`) for integration-style tests
- `Data.TestDataPath` is resolved from `Data.resx` resource (`"testtasks.txt"`)

**Per-Test Setup:**
- No `[SetUp]` method; each test creates its own objects inline
- `TaskTests.cs` shares field-level test data (`_projects`, `_contexts`) initialized at field declaration

## Mocking

**Framework:** None — no mocking framework is used (no Moq, NSubstitute, FakeItEasy, etc.)

**Patterns:**
- Integration-style: tests operate against real file system via `TaskList` with a temporary file
- No interfaces extracted for `TaskList` or `Task` to enable mocking
- Real `Task` objects constructed directly for expected values

**What is tested:**
- Domain parsing logic (pure/deterministic): `Task` constructor with raw strings
- File I/O operations: `TaskList.Add`, `TaskList.Delete`, `TaskList.Update`, `TaskList.ReloadTasks`
- Concurrency scenario: file lock behavior during reload (`Read_when_file_is_open_in_another_process`)

**What is NOT tested:**
- `Client/` layer (no tests for `MainWindowViewModel` or any WPF code-behind)
- `Log` class behavior
- Extension methods in `CommonExtensions/`
- Sort and filter logic in `MainWindowViewModel.FilterList` / `MainWindowViewModel.SortList`

## Fixtures and Factories

**Test Data:**
```csharp
// Raw task string — most common approach
var task = new Task("(A) This is a test task +test @work");

// Constructor with parameters for expected values
var expectedTask = new Task("(A)", _projects, _contexts, "This is a test task");

// Helper list factory inside TaskListTests
private List<Task> getTestList()
{
    var tl = new List<Task>();
    tl.Add(new Task("(c) 3test +test2 due:2000-01-03"));
    tl.Add(new Task("(d) 1test +test1 @test1 due:2000-01-01"));
    // ...
    return tl;
}
```

**Location:**
- No dedicated fixtures directory; test data is inlined in test methods
- `Data.resx` holds the test file path string (`testtasks.txt`)
- `getTestList()` in `TaskListTests.cs` provides a local task collection helper (note: lowercase `g` — non-standard naming)

## Coverage

**Requirements:** None enforced — no coverage configuration found

**View Coverage:**
```bash
# No coverage tooling configured; would need OpenCover + ReportGenerator or VS Coverage
# dotnet-coverage or NCover could be added manually
```

**Observed gaps:**
- No tests for `Client/` layer code at all
- No tests for `CommonExtensions/` extension methods
- No negative-path/exception tests (e.g., missing file, corrupt data)
- `getTestList()` helper in `TaskListTests.cs` is defined but never called in any test method

## Test Types

**Unit Tests (`TaskTests.cs`):**
- Test the `Task` class constructor parsing in isolation — no file I/O, no dependencies
- Cover: priority, body, projects, contexts, due dates, threshold dates, completion status
- Use custom `AssertEquivalence` helper instead of object equality

**Integration Tests (`TaskListTests.cs`):**
- Test `TaskList` CRUD operations against a real file on disk
- Require file system access; create/delete `testtasks.txt` in fixture setup/teardown
- Include a concurrency test using `Thread` to simulate concurrent file access

**E2E Tests:** Not used

## Common Patterns

**Custom Assertion Helper:**
```csharp
// In TaskTests.cs — reusable multi-property comparison
void AssertEquivalence(Task t1, Task t2)
{
    Assert.AreEqual(t1.Priority, t2.Priority);
    CollectionAssert.AreEquivalent(t1.Projects, t2.Projects);
    CollectionAssert.AreEquivalent(t1.Contexts, t2.Contexts);
    Assert.AreEqual(t1.DueDate, t2.DueDate);
    Assert.AreEqual(t1.Completed, t2.Completed);
    Assert.AreEqual(t1.Body, t2.Body);
}
```

**Async/Threading Testing:**
```csharp
// Concurrency test uses Thread directly — no async/await
var thread = new Thread(x => {
    var f = File.Open(Data.TestDataPath, FileMode.Open, FileAccess.ReadWrite);
    // ...
    Thread.Sleep(500);
});
thread.Start();
Thread.Sleep(100);
try
{
    t.ReloadTasks();
}
catch (Exception ex)
{
    Assert.Fail(ex.Message);
}
finally
{
    thread.Join();
}
```

**Date-Dependent Tests:**
```csharp
// Tests involving "today", "tomorrow", day-of-week calculate expected values at runtime
string due = DateTime.Now.ToString("yyyy-MM-dd");
var expectedTask = new Task("(A)", _projects, ..., due, false);
```

**Incomplete Test Bodies:**
- `Completed_adds_x_to_begining` in `TaskTests.cs` has no assertion — the test body sets `t.Completed = true` but does not assert anything. This test always passes vacuously.

## Adding New Tests

**New domain/parsing tests:**
- Add `[Test]` methods to `ToDoTests/TaskTests.cs` within the appropriate `#region`
- Follow `{Concept}_{Scenario}` naming convention
- Construct `Task` from raw string; compare with `AssertEquivalence()` or direct `Assert.AreEqual`

**New TaskList integration tests:**
- Add `[Test]` methods to `ToDoTests/TaskListTests.cs`
- Use `Data.TestDataPath` for the temp file path — fixture setup/teardown handles file lifecycle
- Do not depend on test ordering; each test should leave the file in a neutral state or set it up explicitly

**New library tests:**
- Create a new `*Tests.cs` file in `ToDoTests/`, add `[TestFixture]` class, reference `NUnit.Framework`
- Register the new file in `ToDoTests.csproj` under `<Compile Include="..." />`

---

*Testing analysis: 2025-01-31*
