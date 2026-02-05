# Tauri API Documentation

## Overview
This document maps all frontend functions to their corresponding Tauri backend commands, including which pages call them and what payloads are sent.

---

## Table of Contents
1. [Tauri Commands](#tauri-commands)
2. [Hooks Usage](#hooks-usage)
3. [Pages Usage](#pages-usage)
4. [Complete Call Flow](#complete-call-flow)

---

## Tauri Commands

All Tauri commands are defined in `/src/lib/tauri.ts` and invoke Rust backend functions.

### 1. `create_task`
**Frontend Function:** `tauriCommands.createTask()`

**Payload:**
```typescript
{
  title: string,
  taskDate: string  // ISO date string (e.g., "2026-02-05T00:00:00.000Z")
}
```

**Returns:** `Task` object

**Backend Command:** `create_task`

---

### 2. `update_task`
**Frontend Function:** `tauriCommands.updateTask()`

**Payload:**
```typescript
{
  taskId: string,
  data: {
    title?: string,
    notes?: string,
    hasDeadline?: boolean,
    deadline?: string,  // ISO date string
    hasCalendarIntegration?: boolean,
    calendarEmail?: string,
    reminderFrequency?: 'none' | 'hourly' | 'every-3-hours' | 'daily'
  }
}
```

**Returns:** `Task` object

**Backend Command:** `update_task`

---

### 3. `delete_task`
**Frontend Function:** `tauriCommands.deleteTask()`

**Payload:**
```typescript
{
  taskId: string
}
```

**Returns:** `void`

**Backend Command:** `delete_task`

---

### 4. `start_task`
**Frontend Function:** `tauriCommands.startTask()`

**Payload:**
```typescript
{
  taskId: string
}
```

**Returns:** `Task` object with updated status and `startedAt` timestamp

**Backend Command:** `start_task`

---

### 5. `pause_task`
**Frontend Function:** `tauriCommands.pauseTask()`

**Payload:**
```typescript
{
  taskId: string
}
```

**Returns:** `Task` object with updated status and `pausedAt` timestamp

**Backend Command:** `pause_task`

---

### 6. `resume_task`
**Frontend Function:** `tauriCommands.resumeTask()`

**Payload:**
```typescript
{
  taskId: string
}
```

**Returns:** `Task` object with updated status

**Backend Command:** `resume_task`

---

### 7. `complete_task`
**Frontend Function:** `tauriCommands.completeTask()`

**Payload:**
```typescript
{
  taskId: string
}
```

**Returns:** `Task` object with updated status and `completedAt` timestamp

**Backend Command:** `complete_task`

---

### 8. `get_tasks_by_date`
**Frontend Function:** `tauriCommands.getTasksByDate()`

**Payload:**
```typescript
{
  date: string  // ISO date string (e.g., "2026-02-05T00:00:00.000Z")
}
```

**Returns:** `Task[]` array

**Backend Command:** `get_tasks_by_date`

---

### 9. `get_task_by_id`
**Frontend Function:** `tauriCommands.getTaskById()`

**Payload:**
```typescript
{
  taskId: string
}
```

**Returns:** `Task` object or `null`

**Backend Command:** `get_task_by_id`

---

### 10. `get_ongoing_task`
**Frontend Function:** `tauriCommands.getOngoingTask()`

**Payload:** None

**Returns:** `Task` object or `null`

**Backend Command:** `get_ongoing_task`

---

### 11. `search_tasks`
**Frontend Function:** `tauriCommands.searchTasks()`

**Payload:**
```typescript
{
  query: string
}
```

**Returns:** `Task[]` array

**Backend Command:** `search_tasks`

---

### 12. `get_date_sections`
**Frontend Function:** `tauriCommands.getDateSections()`

**Payload:** None

**Returns:** `DateSection[]` array
```typescript
{
  date: Date,
  label: string,
  isToday: boolean,
  isYesterday: boolean
}[]
```

**Backend Command:** `get_date_sections`

**Note:** Currently not used - date sections are generated client-side in `useDateSections` hook.

---

### 13. `get_all_dates_with_tasks`
**Frontend Function:** `tauriCommands.getAllDatesWithTasks()`

**Payload:** None

**Returns:** `Date[]` array (ISO date strings converted to Date objects)

**Backend Command:** `get_all_dates_with_tasks`

---

### 14. `get_completed_tasks`
**Frontend Function:** `tauriCommands.getCompletedTasks()`

**Payload:** None

**Returns:** `Task[]` array of completed tasks

**Backend Command:** `get_completed_tasks`

---

## Hooks Usage

### `useTasks()` Hook
**Location:** `/src/hooks/useTasks.ts`

Wraps task-related Tauri commands with loading/error state management.

**Methods:**
- `addTask(title: string, date?: Date)` → calls `create_task`
- `updateTask(id: string, data: Partial<TaskFormData>)` → calls `update_task`
- `deleteTask(id: string)` → calls `delete_task`
- `startTask(id: string)` → calls `start_task`
- `pauseTask(id: string)` → calls `pause_task`
- `resumeTask(id: string)` → calls `resume_task`
- `completeTask(id: string)` → calls `complete_task`
- `getTasksByDate(date: Date)` → calls `get_tasks_by_date`
- `getTaskById(id: string)` → calls `get_task_by_id`
- `getOngoingTask()` → calls `get_ongoing_task`
- `searchTasks(query: string)` → calls `search_tasks`
- `getCompletedTasks()` → calls `get_completed_tasks`

---

### `useOngoingTask()` Hook
**Location:** `/src/hooks/useOngoingTask.ts`

Auto-fetches the ongoing task on mount.

**Calls:**
- `get_ongoing_task` (via `tauriCommands.getOngoingTask()`)

**Returns:**
- `task`: Current ongoing task or null
- `isLoading`: Loading state
- `error`: Error message if any
- `refetch()`: Function to refetch

---

### `useTasksByDate()` Hook
**Location:** `/src/hooks/useTasksByDate.ts`

Auto-fetches tasks for a specific date.

**Parameters:** `date: Date`

**Calls:**
- `get_tasks_by_date` (via `tauriCommands.getTasksByDate()`)

**Returns:**
- `tasks`: Array of tasks for the date
- `isLoading`: Loading state
- `error`: Error message if any
- `refetch()`: Function to refetch

---

### `useDatesWithTasks()` Hook
**Location:** `/src/hooks/useDatesWithTasks.ts`

Auto-fetches all dates that have tasks.

**Calls:**
- `get_all_dates_with_tasks` (via `tauriCommands.getAllDatesWithTasks()`)

**Returns:**
- `dates`: Array of Date objects
- `isLoading`: Loading state
- `error`: Error message if any
- `refetch()`: Function to refetch

---

### `useDateSections()` Hook
**Location:** `/src/hooks/useDateSections.ts`

Generates date sections client-side (Today, Yesterday, and 2 days ago).

**Calls:** None (client-side only)

**Returns:**
- `sections`: Array of DateSection objects (generated client-side)
- `isLoading`: false
- `error`: null
- `refetch()`: No-op function

---

## Pages Usage

### 1. DailyPage
**Location:** `/src/pages/DailyPage.tsx`

**Uses Hooks:**
- `useTasks()` - For all task operations
- `useDateSections()` - For date sections (client-side)

**Direct Tauri Calls:**
- `tauriCommands.getTasksByDate(date)` - Fetches tasks for each section
- `tauriCommands.searchTasks(query)` - Search functionality
- `tauriCommands.getTaskById(id)` - When editing a task

**User Actions & Commands Flow:**

1. **Adding a Task**
   - User types in TaskInput component
   - Calls `handleAddTask(date)` → `addTask(title, date)` → `create_task`
   - Payload: `{ title: string, taskDate: ISO string }`

2. **Starting a Task**
   - User clicks "Start" button
   - Calls `handleStart(id)` → `startTask(id)` → `start_task`
   - Payload: `{ taskId: string }`

3. **Pausing a Task**
   - User clicks "Pause" button
   - Calls `handlePause(id)` → `pauseTask(id)` → `pause_task`
   - Payload: `{ taskId: string }`

4. **Resuming a Task**
   - User clicks "Resume" button
   - Calls `handleResume(id)` → `resumeTask(id)` → `resume_task`
   - Payload: `{ taskId: string }`

5. **Completing a Task**
   - User clicks "Complete" button
   - Calls `handleComplete(id)` → `completeTask(id)` → `complete_task`
   - Payload: `{ taskId: string }`

6. **Editing a Task**
   - User clicks "Edit" button
   - Calls `handleEdit(id)` → `tauriCommands.getTaskById(id)` → `get_task_by_id`
   - Opens modal, then on save:
   - Calls `handleSaveEdit(id, data)` → `updateTask(id, data)` → `update_task`
   - Payload: `{ taskId: string, data: Partial<TaskFormData> }`

7. **Deleting a Task**
   - User clicks "Delete" button
   - Calls `handleDelete(id)` → `deleteTask(id)` → `delete_task`
   - Payload: `{ taskId: string }`

8. **Searching Tasks**
   - User types in search bar
   - Calls `handleSearch(query)` → `tauriCommands.searchTasks(query)` → `search_tasks`
   - Payload: `{ query: string }`

9. **Loading Tasks for Sections**
   - Auto-fetches on mount and section changes
   - Calls `fetchSectionTasks()` → `tauriCommands.getTasksByDate(date)` → `get_tasks_by_date`
   - Payload: `{ date: ISO string }`

---

### 2. FocusPage
**Location:** `/src/pages/FocusPage.tsx`

**Uses Hooks:**
- `useTasks()` - For pause and complete operations
- `useOngoingTask()` - Auto-fetches ongoing task
- `useTimer()` - Client-side timer

**User Actions & Commands Flow:**

1. **Loading Page**
   - Auto-fetches ongoing task on mount
   - `useOngoingTask()` → `get_ongoing_task`
   - Payload: None

2. **Pausing Task**
   - User clicks "Pause" button
   - Calls `handlePause()` → `pauseTask(ongoingTask.id)` → `pause_task`
   - Payload: `{ taskId: string }`

3. **Completing Task**
   - User clicks "Complete" button
   - Calls `handleComplete()` → `completeTask(ongoingTask.id)` → `complete_task`
   - Payload: `{ taskId: string }`

---

### 3. CalendarPage
**Location:** `/src/pages/CalendarPage.tsx`

**Uses Hooks:**
- `useDatesWithTasks()` - Auto-fetches dates with tasks

**Direct Tauri Calls:**
- `tauriCommands.getTasksByDate(date)` - Fetches tasks for selected date

**User Actions & Commands Flow:**

1. **Loading Page**
   - Auto-fetches all dates that have tasks
   - `useDatesWithTasks()` → `get_all_dates_with_tasks`
   - Payload: None

2. **Selecting a Date**
   - User clicks a date on calendar
   - Calls `fetchTasksForDate(date)` → `tauriCommands.getTasksByDate(date)` → `get_tasks_by_date`
   - Payload: `{ date: ISO string }`

3. **Calendar Highlights**
   - Dates with tasks are highlighted using the dates from `useDatesWithTasks()`

---

## Complete Call Flow

### Task Creation Flow
```
User types in TaskInput
  ↓
DailyPage.handleAddTask(date)
  ↓
useTasks.addTask(title, date)
  ↓
tauriCommands.createTask(title, date)
  ↓
invoke('create_task', { title, taskDate: ISO string })
  ↓
Rust Backend (create_task command)
  ↓
Returns Task object
  ↓
Parsed with parseTask() helper
  ↓
UI refreshes with new task
```

### Task Status Change Flow
```
User clicks action button (Start/Pause/Resume/Complete)
  ↓
Page handler (handleStart/handlePause/etc.)
  ↓
useTasks method (startTask/pauseTask/etc.)
  ↓
tauriCommands method
  ↓
invoke('start_task'|'pause_task'|'resume_task'|'complete_task', { taskId })
  ↓
Rust Backend (command)
  ↓
Returns updated Task object
  ↓
Parsed with parseTask() helper
  ↓
UI refreshes with updated task
```

### Search Flow
```
User types in search bar
  ↓
DailyPage.handleSearch(query)
  ↓
tauriCommands.searchTasks(query)
  ↓
invoke('search_tasks', { query })
  ↓
Rust Backend (search_tasks command)
  ↓
Returns Task[] array
  ↓
Parsed with parseTask() helper for each task
  ↓
Search results displayed in UI
```

### Calendar Date Selection Flow
```
User clicks calendar date
  ↓
CalendarPage.setSelectedDate(date)
  ↓
useEffect triggers fetchTasksForDate(date)
  ↓
tauriCommands.getTasksByDate(date)
  ↓
invoke('get_tasks_by_date', { date: ISO string })
  ↓
Rust Backend (get_tasks_by_date command)
  ↓
Returns Task[] array for that date
  ↓
Parsed with parseTask() helper for each task
  ↓
Tasks displayed for selected date
```

---

## Data Type Definitions

### Task Interface
```typescript
interface Task {
  id: string;
  title: string;
  notes?: string;
  status: 'not-started' | 'ongoing' | 'paused' | 'completed';
  createdAt: Date;
  updatedAt: Date;
  deadline?: Date;
  hasCalendarIntegration: boolean;
  calendarEmail?: string;
  reminderFrequency: 'none' | 'hourly' | 'every-3-hours' | 'daily';
  completedAt?: Date;
  startedAt?: Date;
  pausedAt?: Date;
}
```

### TaskFormData Interface
```typescript
interface TaskFormData {
  title: string;
  notes?: string;
  hasDeadline: boolean;
  deadline?: Date;
  hasCalendarIntegration: boolean;
  calendarEmail?: string;
  reminderFrequency: ReminderFrequency;
}
```

### DateSection Interface
```typescript
interface DateSection {
  date: Date;
  label: string;
  isToday: boolean;
  isYesterday: boolean;
}
```

---

## Backend Commands Summary Table

| Command | Method | Payload | Returns | Used By |
|---------|--------|---------|---------|---------|
| `create_task` | POST | `{ title, taskDate }` | Task | DailyPage |
| `update_task` | PUT | `{ taskId, data }` | Task | DailyPage |
| `delete_task` | DELETE | `{ taskId }` | void | DailyPage |
| `start_task` | POST | `{ taskId }` | Task | DailyPage |
| `pause_task` | POST | `{ taskId }` | Task | DailyPage, FocusPage |
| `resume_task` | POST | `{ taskId }` | Task | DailyPage |
| `complete_task` | POST | `{ taskId }` | Task | DailyPage, FocusPage |
| `get_tasks_by_date` | GET | `{ date }` | Task[] | DailyPage, CalendarPage |
| `get_task_by_id` | GET | `{ taskId }` | Task \| null | DailyPage |
| `get_ongoing_task` | GET | None | Task \| null | FocusPage |
| `search_tasks` | GET | `{ query }` | Task[] | DailyPage |
| `get_date_sections` | GET | None | DateSection[] | Not used (client-side) |
| `get_all_dates_with_tasks` | GET | None | Date[] | CalendarPage |
| `get_completed_tasks` | GET | None | Task[] | Not used yet |

---

## Notes

1. **Date Sections**: Currently generated client-side in `useDateSections` hook. The backend command `get_date_sections` exists but is not called.

2. **Date Serialization**: All Date objects are converted to ISO strings before being sent to the backend and parsed back to Date objects when received.

3. **Error Handling**: All hooks and page handlers include try-catch blocks with toast notifications for errors.

4. **Auto-Refresh**: Pages automatically refresh data after mutations (add, update, delete, status changes).

5. **Direct Calls**: Some pages bypass hooks and call `tauriCommands` directly for specific operations like search and fetching tasks by date.

6. **Completed Tasks**: The `get_completed_tasks` command exists but is not currently used in any page.
