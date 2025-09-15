use crate::data::{
    Assignment, Course, CourseId, Instructor, InstructorId, Room, RoomId, SchedulingInput,
    SchedulingOutput, Timeslot, UnmetSoftConstraint,
};
use good_lp::variable;
use good_lp::{
    Expression, ProblemVariables, Solution, SolverModel, Variable, constraint, default_solver,
};
use itertools::Itertools;
use log::{info, trace};
use std::collections::{HashMap, HashSet};
use std::time::Instant;


/// solves the scheduling problem using the HiGHs ILP solver.

pub fn solve(input: &SchedulingInput) -> Result<SchedulingOutput, String> {
    let start_time = Instant::now();
    // lookups
    let course_map: HashMap<CourseId, &Course> = input.courses.iter().map(|c| (c.id, c)).collect();
    let instructor_map: HashMap<InstructorId, &Instructor> =
        input.instructors.iter().map(|i| (i.id, i)).collect();
    let instructor_courses: HashMap<InstructorId, Vec<CourseId>> = input
        .courses
        .iter()
        .map(|c| (c.instructor_id, c.id))
        .into_group_map();

    //model setup
    info!(
        "Setting up ILP model with {} courses, {} rooms, and {} timeslots...",
        input.courses.len(),
        input.rooms.len(),
        input.total_timeslots
    );
    let mut problem = ProblemVariables::new();
    let mut all_possible_assignments = Vec::new();

    // x_crt =  1 if course c is in room r at time t
    //          0 otherwise

    // pre-filter for performance; implicitly handle some hard constraints
    for course in &input.courses {
        for room in &input.rooms {
            for start_slot in 0..input.total_timeslots {
                if is_assignment_possible(course, room, start_slot, input, &instructor_map) {
                    all_possible_assignments.push((course.id, room.id, start_slot));
                }
            }
        }
    }
    trace!(
        "Generated {} potential assignment variables out of a theoretical maximum of {}.",
        all_possible_assignments.len(),
        input.courses.len() * input.rooms.len() * input.total_timeslots as usize
    );

    if all_possible_assignments.is_empty() {
        return Err("No possible assignments found after pre-filtering. The problem might be too constrained.".to_string());
    }

    // decision map
    let mut assignment_vars_map: HashMap<(CourseId, RoomId, Timeslot), Variable> = HashMap::new();
    let assignment_vars_vec =
        problem.add_vector(variable().binary(), all_possible_assignments.len());

    for (i, (course_id, room_id, start_slot)) in all_possible_assignments.iter().enumerate() {
        assignment_vars_map.insert((*course_id, *room_id, *start_slot), assignment_vars_vec[i]);
    }

    //begin soft constraints
    let mut back_to_back_links = Vec::new();
    if input.total_timeslots > 1 {
        for instructor_id in instructor_courses.keys() {
            let courses = instructor_courses.get(instructor_id).unwrap();
            for k in 0..(input.total_timeslots - 1) {
                let starts_at_k_plus_1: Expression = assignment_vars_map
                    .iter()
                    .filter(|((c_id, _, start_slot), _)| {
                        courses.contains(c_id) && *start_slot == k + 1
                    })
                    .map(|(_, var)| *var)
                    .sum();
                let ends_at_k: Expression = assignment_vars_map
                    .iter()
                    .filter(|((c_id, _, start_slot), _)| {
                        let course = course_map.get(c_id).unwrap();
                        courses.contains(c_id) && (*start_slot + course.duration_slots - 1) == k
                    })
                    .map(|(_, var)| *var)
                    .sum();

                let penalty_var = problem.add(variable().binary());
                back_to_back_links.push((starts_at_k_plus_1, ends_at_k, penalty_var));
            }
        }
    }
    // soft constraints
    // soft constraint weights
    let morning_preference_weight = 1.0;
    let back_to_back_penalty_weight = 0.5;

    let morning_cutoff = input.total_timeslots / 2; //assume morining is from 6-12 out of assumed 12 slots
    let morning_score: Expression = assignment_vars_map
        .iter()
        .filter(|((_, _, start_slot), _)| *start_slot < morning_cutoff)
        .map(|(_, var)| *var)
        .sum();
    let back_to_back_penalty_score: Expression =
        back_to_back_links.iter().map(|(_, _, var)| *var).sum();

    let objective = morning_preference_weight * morning_score
        - back_to_back_penalty_weight * back_to_back_penalty_score;
    info!("Objective function defined with morning preference and back-to-back penalties.");

    let mut model = problem
        .maximise(objective)
        .using(default_solver)
        .set_option("threads", 1) // limit to 1 thread for reproducibility
        .set_option("random_seed", 1234) //set seed for reproducibility
        .set_option("log_to_console", "true");
    // begin hard constraints

    // sanity check so course schedule makes sense
    info!("Adding 'course scheduled once' constraints...");
    for course in &input.courses {
        let scheduled_once: Expression = assignment_vars_map
            .iter()
            .filter(|((c_id, _, _), _)| *c_id == course.id)
            .map(|(_, var)| *var)
            .sum();
        model.add_constraint(constraint!(scheduled_once == 1));
    }

    // no room double-booking
    info!("Adding 'no room overlap' constraints...");
    for room in &input.rooms {
        for k in 0..input.total_timeslots {
            let room_occupied: Expression = assignment_vars_map
                .iter()
                .filter(|((_, r_id, _), _)| *r_id == room.id)
                .filter(|((c_id, _, start_slot), _)| {
                    let course = course_map.get(c_id).unwrap();
                    // Check if the course occupies the room at timeslot k
                    k >= *start_slot && k < *start_slot + course.duration_slots
                })
                .map(|(_, var)| *var)
                .sum();
            model.add_constraint(constraint!(room_occupied <= 1));
        }
    }

    // no instructor overlap
    info!("Adding 'no instructor overlap' constraints...");
    for instructor in &input.instructors {
        if let Some(courses_for_instructor) = instructor_courses.get(&instructor.id) {
            for k in 0..input.total_timeslots {
                let instructor_busy: Expression = assignment_vars_map
                    .iter()
                    .filter(|((c_id, _, _), _)| courses_for_instructor.contains(c_id))
                    .filter(|((c_id, _, start_slot), _)| {
                        let course = course_map.get(c_id).unwrap();
                        k >= *start_slot && k < *start_slot + course.duration_slots
                    })
                    .map(|(_, var)| *var)
                    .sum();
                model.add_constraint(constraint!(instructor_busy <= 1));
            }
        }
    }

    //solve
    info!("Starting ILP solver...");
    let solution = match model.solve() {
        Ok(s) => s,
        Err(e) => {
            return Err(format!(
                "No solution found. The problem might be too constrained. Solver error: {}",
                e
            ));
        }
    };
    let duration = start_time.elapsed();
    info!("Solution found in {:.2?}", duration);

    // get assignments from solution
    let mut assignments = Vec::new();
    for ((course_id, room_id, start_slot), var) in &assignment_vars_map {
        if solution.value(*var) > 0.9 {
            assignments.push(Assignment {
                course_id: *course_id,
                room_id: *room_id,
                start_slot: *start_slot,
            });
        }
    }
    assignments.sort();

    // get score
    let (score, unmet_soft_constraints) =
        calculate_score_and_unmet_constraints(&assignments, input, &course_map);

    // build the final output
    Ok(SchedulingOutput {
        assignments,
        score,
        unmet_soft_constraints,
    })

}

// implicitly checks the hard constraints on overlap and capacity
fn is_assignment_possible(
    course: &Course,
    room: &Room,
    start_slot: Timeslot,
    input: &SchedulingInput,
    instructor_map: &HashMap<InstructorId, &Instructor>,
) -> bool {
    // course fits in remaining timeslots
    if start_slot + course.duration_slots > input.total_timeslots {
        return false;
    }

    // room has capacity
    if room.capacity < course.required_capacity {
        return false;
    }

    // instructor has to be available
    if let Some(instructor) = instructor_map.get(&course.instructor_id) {
        let required_slots: HashSet<Timeslot> =
            (start_slot..start_slot + course.duration_slots).collect();
        let unavailable_set: HashSet<Timeslot> =
            instructor.unavailable_slots.iter().cloned().collect();

        if !required_slots.is_disjoint(&unavailable_set) {
            return false; // not available
        }
    } else {
        return false;
    }

    true
}

fn calculate_score_and_unmet_constraints(
    assignments: &[Assignment],
    input: &SchedulingInput,
    course_map: &HashMap<CourseId, &Course>,
) -> (i32, Vec<UnmetSoftConstraint>) {
    let mut score = 0;
    let mut unmet = Vec::new();
    let morning_cutoff = input.total_timeslots / 2;

    // prefer morning slots.
    for assignment in assignments {
        if assignment.start_slot < morning_cutoff {
            score += 1; //add score if met
        } else {
            score -= 1; //penalize if not met
            unmet.push(UnmetSoftConstraint {
                constraint_type: "Prefer Mornings".to_string(),
                description: format!(
                    "Course {} is scheduled at slot {}, which is not in the morning. Morning starts at 6 am (slot 0) and ends at 12 pm (slot 6)",
                    assignment.course_id, assignment.start_slot
                ),
            });
        }
    }

    // avoid back-to-back classes for instructors
    let instructor_assignments: HashMap<InstructorId, Vec<&Assignment>> = assignments
        .iter()
        .filter_map(|a| course_map.get(&a.course_id).map(|c| (c.instructor_id, a)))
        .into_group_map();

    for (instructor_id, mut instructor_assigns) in instructor_assignments {
        instructor_assigns.sort_by_key(|a| a.start_slot);

        for i in 0..instructor_assigns.len().saturating_sub(1) {
            let current = instructor_assigns[i];
            let next = instructor_assigns[i + 1];

            let current_course = course_map.get(&current.course_id).unwrap();
            let current_end_slot = current.start_slot + current_course.duration_slots;

            if current_end_slot != next.start_slot {
                score += 1; // reward for not back-to-back
            } else {
                score -= 1; // penalty for back-to-back
                unmet.push(UnmetSoftConstraint {
                    constraint_type: "Avoid Back-to-Back Classes".to_string(),
                    description: format!(
                        "Instructor {} has back-to-back classes: Course {} (ends at slot {}) and Course {} (starts at slot {}).",
                        instructor_id,
                        current.course_id,
                        current_end_slot,
                        next.course_id,
                        next.start_slot
                    ),
                });
            }
        }
    }

    (score, unmet)
}
