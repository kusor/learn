// import { v4 } from "https://deno.land/std/uuid/mod.ts"
// --Generate v4 UUID
// const uuid=v4.generate();
// Output: b39d93bf-740f-4190-9364-0bc8a365de9a
// Better with:
// crypto.randomUUID();

enum State {
  Pending = 0,
  Scheduled = 1,
  Completed = 2,
  Running = 3,
  Failed = 4,
}

type Task = {
	ID: string; // uuid.UUID
	Name: string;
	State: State;
	Image: string;
	Memory: number;
	Disk: number;
	// ExposedPorts  nat.PortSet
  // TODO:
	// PortBindings: map[string]string;
	RestartPolicy: string;
}

type TaskEvent = {
	ID: string; // uuid.UUID
	State: State;
	Timestamp: number; // time.Time
	Task: Task;
}

/*
let t: Task = {
    ID: crypto.randomUUID(),
    Name: 'First Task',
    State: State.Pending,
    Image: 'k8s-image-name',
    Memory: 2048,
    Disk: 2048,
    RestartPolicy: 'Always'
}
*/
