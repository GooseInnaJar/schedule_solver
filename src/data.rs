use serde::{Deserialize, Serialize};
use std::fmt;

// Type aliases for clarity
pub type RoomId = u32;
pub type CourseId = u32;
pub type InstructorId = u32;
pub type Timeslot = u32;

/// Represents a physical room with a given capacity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Room {
    pub id: RoomId,
    pub capacity: u32,
}

/// Represents a course to be scheduled.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: CourseId,
    pub instructor_id: InstructorId,
    pub duration_slots: u32,
    pub required_capacity: u32,
}

/// Represents an instructor with their scheduling constraints.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instructor {
    pub id: InstructorId,
    pub unavailable_slots: Vec<Timeslot>,
}

/// The complete input for the scheduling problem.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingInput {
    pub rooms: Vec<Room>,
    pub courses: Vec<Course>,
    pub instructors: Vec<Instructor>,
    pub total_timeslots: u32,
}

/// Represents a single, scheduled course assignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
    pub course_id: CourseId,
    pub room_id: RoomId,
    pub start_slot: Timeslot,
}

/// Describes a soft constraint that was not met in the final schedule.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmetSoftConstraint {
    pub constraint_type: String,
    pub description: String,
}

impl fmt::Display for UnmetSoftConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.constraint_type, self.description)
    }
}


/// The final output of the solver.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingOutput {
    pub assignments: Vec<Assignment>,
    pub score: i32,
    pub unmet_soft_constraints: Vec<UnmetSoftConstraint>,
}