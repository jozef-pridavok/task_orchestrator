# Task Orchestrator

A Rust-based service for orchestrating and executing multi-step, long-running background tasks.

## Overview

This project implements a task orchestrator that can execute instances of a hardcoded task blueprint concurrently. Each task follows a sequence of steps: making an HTTP request, waiting for a delay, and emitting an event.

## Task Blueprint

Each task executes the following steps:
1. **fetch_data**: Makes an HTTP GET request to `https://httpbin.org/get`
2. **long_delay**: Pauses execution for 5 seconds
3. **emit_event**: Prints a confirmation message to stderr

## Usage

```bash
cargo run -- tasks.csv > results.csv
```

**Automatic Scaling:**
- **≤1000 tasks**: Uses channel-based execution (bounded memory)
- **>1000 tasks**: Uses streaming execution (unlimited scale)

## Input Format (tasks.csv)

The input is a CSV file with a header and two columns:
- `task_id`: A u64 that uniquely identifies a task instance
- `task_type`: A string (currently ignored, assumed to be the blueprint above)

Example:
```csv
task_id,task_type
101,process_data
102,process_data
101,process_data
103,process_data
```

## Output Format

The program writes results to stdout as CSV with three columns:
- `task_id`: The unique u64 task identifier
- `final_status`: Either "Completed" or "Failed"
- `error_info`: Empty for completed tasks, contains error message for failed tasks

Example output:
```csv
task_id,final_status,error_info
101,Completed,
102,Completed,
103,Failed,Network timeout
```

## Architecture

The project is organized into several modules:

### Task Types (`src/task.rs`)
- `TaskInput`: Represents input tasks from CSV
- `TaskOutput`: Final output format for CSV
- `TaskResult`: Internal result representation
- `TaskStatus`: Enumeration of task states

### Task Blueprint (`src/task_blueprint.rs`)
- `TaskBlueprint`: Implements the hardcoded task sequence
- Each step is implemented as a separate async function

### Orchestrator (`src/orchestrator.rs`)
- `TaskOrchestrator`: Stateless manager for concurrent task execution
- **Two execution modes:**
  - `execute_tasks()`: Uses Tokio channels (good for moderate task counts)
  - `execute_tasks_streaming()`: Uses FuturesUnordered (better for large scale)
- Returns all task results without deduplication (handled in CSV output)

### Main (`src/main.rs`)
- CLI interface and application entry point
- Coordinates CSV reading, task execution, and result output

## Concurrency

The orchestrator executes tasks concurrently using Tokio's async runtime. Task results are collected through channels to avoid race conditions. Tasks with the same `task_id` are deduplicated during CSV output generation, keeping only the latest result.

## Performance

- Stateless orchestrator design eliminates memory leaks
- **Automatic scaling**: Chooses optimal execution method based on task count
  - **Channel-based**: ≤1000 tasks (bounded memory, capacity 1000)
  - **Streaming**: >1000 tasks (FuturesUnordered with backpressure)
- Thread-safe result collection

## Error Handling

- Network failures in the HTTP request are captured and reported
- File I/O errors are handled gracefully
- The program exits with error codes on failure

## Testing

The project includes comprehensive unit tests for all modules:

```bash
cargo test
```

Tests cover:
- CSV parsing and writing
- Task blueprint execution
- Orchestrator behavior with various scenarios
- Integration testing of the full pipeline

## Dependencies

- `tokio`: Async runtime for concurrent execution
- `reqwest`: HTTP client for the fetch_data step
- `serde`: Serialization/deserialization for CSV handling
- `csv`: CSV parsing and writing
- `anyhow`: Error handling

## Assumptions and Trade-offs

### Assumptions:
1. Network connectivity is available for HTTP requests to httpbin.org
2. The task blueprint is hardcoded and doesn't need to be configurable
3. All tasks use the same blueprint regardless of task_type value
4. Duplicate task_ids should be handled by keeping the latest execution result

### Trade-offs:
1. **Simplicity vs Flexibility**: Hardcoded blueprint makes the implementation simpler but less flexible
2. **Memory vs Performance**: Results are stored in memory for deduplication, which may not scale to millions of tasks
3. **Error Handling**: Individual task failures don't stop the entire execution, which is good for resilience but may mask systematic issues

## Recent Improvements

### Code Quality Fixes (2024-11-11)
- **Fixed race condition**: Removed concurrent HashMap updates in orchestrator
- **Eliminated memory leak**: Changed to stateless orchestrator design
- **Improved thread safety**: All result collection now uses Tokio channels
- **Simplified architecture**: Deduplication moved to single location in CSV output
- **Added HTTP timeout**: 10-second timeout for HTTP requests to prevent hanging
- **Mocked network calls**: Tests no longer depend on internet connectivity
- **Cleaned up tests**: Removed 6 unnecessary tests, kept only meaningful ones
- **Added streaming variant**: FuturesUnordered implementation for large-scale processing
- **Automatic scaling**: Dynamic execution mode selection based on task count

## AI Assistance

This project was developed with assistance from AI tools to accelerate development and ensure comprehensive test coverage.
