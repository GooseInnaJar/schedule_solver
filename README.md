# Course Scheduling Solver API

This project is a high-performance, Rust-based web service that solves a simplified version of the [university course timetabling problem](https://ieeexplore.ieee.org/document/9499056). It accepts a set of rooms, courses, and instructors with various constraints, and returns an optimal, conflict-free schedule via a RESTful API.

The core of the application uses an **Integer Linear Programming (ILP)** solver to guarantee the best possible schedule based on a defined set of preferences (soft constraints).

## Features

-   **Hard Constraint Enforcement**: Guarantees no double-booking of rooms, no overlapping classes for instructors, and that room capacity is always sufficient.
-   **Soft Constraint Optimization**: Optimizes the schedule to prefer morning classes and avoid back-to-back classes for instructors.
-   **RESTful API**: Simple `POST` endpoint for easy integration with other services.
-   **Deterministic & Reproducible**: Given the same input, the solver will always produce the exact same schedule and score.
-   **Logging**: Outputs logs to console for observability and monitoring.
-   **Performant**: Uses the HiGHS solver to quickly find solutions to optimization problems.

## Getting Started

### Prerequisites

-   Rust toolchain (latest stable version recommended). You can install it from [rustup.rs](https://rustup.rs/).
-   C++ toolchain is needed to build the HiGHS solver.
-   Python3 is needed to run the included test script. 

### How to Run

1.  **Clone the repository:**
    ```bash
    git clone <your-repo-url>
    cd <your-repo-directory>
    ```

2.  **Build the project:**
    ```bash
    cargo build --release
    ```
    

3.  **Run the server:**
    The server's logging level can be configured with the `RUST_LOG` environment variable.
    ```bash
    RUST_LOG=info cargo run --release
    ```
    The server will start on `http://127.0.0.1:8080`. By default the logging level of the server is `trace`.

### Example API Call

You can send a request to the solver using `curl` or any API client.

```bash
curl -X POST [http://127.0.0.1:8080/v1/schedule/solve](http://127.0.0.1:8080/v1/schedule/solve) \
-H "Content-Type: application/json" \
-d '{
  "rooms": [
    { "id": 101, "capacity": 30 },
    { "id": 102, "capacity": 25 },
    { "id": 103, "capacity": 40 }
  ],
  "courses": [
    { "id": 1, "instructor_id": 1, "duration_slots": 2, "required_capacity": 28 },
    { "id": 2, "instructor_id": 3, "duration_slots": 3, "required_capacity": 25 },
    { "id": 3, "instructor_id": 2, "duration_slots": 2, "required_capacity": 20 },
    { "id": 4, "instructor_id": 2, "duration_slots": 2, "required_capacity": 35 },
    { "id": 5, "instructor_id": 3, "duration_slots": 1, "required_capacity": 20 },
    { "id": 6, "instructor_id": 4, "duration_slots": 2, "required_capacity": 28 },
    { "id": 7, "instructor_id": 1, "duration_slots": 2, "required_capacity": 22 }
  ],
  "instructors": [
    { "id": 1, "unavailable_slots": [4] },
    { "id": 2, "unavailable_slots": [] },
    { "id": 3, "unavailable_slots": [0, 1] },
    { "id": 4, "unavailable_slots": [8, 9] }
  ]
}'
```

Alternatively, the script in `./test_script/request.py` can be called like this:

```bash
~/repos/schedule_solver$ python3 ./test_script/request.py ./examples/input_1.json 
```

# Complexity Notes
The timetabling problem is NP-hard. The performance of the ILP solver is highly dependent on the problem structure and size (specifically, the number of potential assignments: courses * rooms * timeslots). A pre-filtering step is implemented to reduce the number of decision variables by eliminating impossible assignments early, significantly improving performance.

Memory usage is primarily driven by the ILP solver's need to store the constraint matrix. The size of this matrix grows polynomially with the number of variables and constraints. For very large-scale problems, this could become a limiting factor.

# Assumptions and Trade-offs

**Assumptions:**

- All timeslots are discrete and of equal duration.
- There are 12 timeslots; this value is hardcoded.
- Travel time between rooms for instructors or students is zero.
- The set of courses an instructor teaches is fixed in the input.
- The weights for soft constraints (e.g., morning preference) used in the objective function are hardcoded.
- All soft constraints are scored equally.
- The preference for mornings is universal, regardless of instructor availability.
  
**Trade-offs:**

- This implementationg of the university scheduling problem is simplified and is mainly intended to presented as a proof of concept.
- The ILP approach guarantees an optimal solution for the given objective function, whereas heuristic methods (like a greedy algorithm) would be faster but provide no guarantee of optimality. I chose optimality for this project's scope.
- The ILP approach involves a greater number of dependencies and involves some additional effort in setting up. However, `good_lp`, `HiGHS_sys`, and `HiGHS` are all well-documented and maintained, reducing the amount of effort that need be expended for future software maintenance.
- The /solve endpoint is stateless, which simplifies the design but means that large problems must be solved synchronously.

# Future Work
- Allow soft constraint preferences and weights to be passed in the API request body.
- Provide for a greater range of soft constraints/ preferences. For example, allow instructors to select preferred hours on an individual basis, allow instructors to *prefer* back-to-back courses, etc.
- Implement a job queue system. The API would immediately return a jobId, and the client could poll another endpoint for the result. This would help in the case that the supplied input is very large.
- Integrate a database (e.g., PostgreSQL) to store room/course data and scheduling results.
- Implement more robust logging. Current implementation assumes the use of external tools to capture stdout if desired.

### Project Schematic

