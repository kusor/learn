// import { v4 } from "https://deno.land/std/uuid/mod.ts"
// --Generate v4 UUID
// const uuid=v4.generate();
// Output: b39d93bf-740f-4190-9364-0bc8a365de9a
// Better with:
// crypto.randomUUID();

enum State = [
  Pending
  Scheduled
  Completed
  Running
  Failed
]

interface Task {
	// ID            uuid.UUID
	Name          string
	State         State
	Image         string
	Memory        number
	Disk          number
	// ExposedPorts  nat.PortSet
	PortBindings  map[string]string
	RestartPolicy string
}

interface TaskEvent {
	// ID        uuid.UUID
	State     State
	Timestamp time.Time
	Task      Task
}
