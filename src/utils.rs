use nexosim::ports::EventQueueReader;
use crate::models;
use std::collections::HashMap;

pub fn calculate_statistics(output_reader: EventQueueReader<models::Person>) -> HashMap<String, f64> {
    let mut count = 0;
    let mut total_swim_time = 0.0;
    let mut total_wait_time = 0.0;
    let mut max_wait_time: f64 = 0.0;
    let mut stats = HashMap::new();

    // Process all people who completed swimming
    for person in output_reader {
        count += 1;

        if let Some(exit_time) = person.exit_time {
            let actual_swim_time = (exit_time - person.entry_time) as f64;
            total_swim_time += actual_swim_time;
        }

        total_wait_time += person.wait_time as f64;
        max_wait_time = max_wait_time.max(person.wait_time as f64);
    }

    // Calculate average
    let avg_wait_time = if count > 0 {
        total_wait_time as f64 / count as f64 / 60.0 // in minutes
    } else {
        0.0
    };

    let avg_swim_time = if count > 0 {
        total_swim_time as f64 / count as f64 / 60.0 // in minutes
    } else {
        0.0
    };

    // Store statistics
    stats.insert("avg_swim_time".to_string(), avg_swim_time);
    stats.insert("avg_wait_time".to_string(), avg_wait_time);
    stats.insert("max_wait_time".to_string(), max_wait_time);
    stats.insert("total_participants".to_string(), count as f64);

    println!("Swimming Pool Simulation Statistics:\n\
        - Total participants: {}\n\
        - Average swim time: {:.2} minutes\n\
        - Average wait time: {:.2} minutes\n\
        - Maximum wait time: {:.2} minutes",
        count,
        avg_swim_time,
        avg_wait_time,
        max_wait_time / 60.0 // convert to minutes
    );

    stats
}

/// A macro for conditional debug printing that can be optimized out when disabled
#[macro_export]
macro_rules! debug_println {
    ($debug_enabled:expr, $($arg:tt)*) => {
        if $debug_enabled {
            println!($($arg)*);
        }
    };
}
