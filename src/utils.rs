use nexosim::ports::EventQueueReader;
use crate::models;
use std::collections::HashMap;

pub fn calculate_statistics(output_reader: EventQueueReader<models::Person>) -> HashMap<String, f64> {
    let mut total_time = 0.0;
    let mut min_time = f64::MAX;
    let mut max_time: f64 = 0.0;
    let mut count = 0;
    let mut stats = HashMap::new();

    // Process all people who completed swimming
    for person in output_reader {
        let completion_time = person.exit_time.unwrap_or(0.0 as u64) - person.entry_time;

        total_time += completion_time as f64;
        min_time = min_time.min(completion_time as f64);
        max_time = max_time.max(completion_time as f64);
        count += 1;
    }

    // Calculate average
    let avg_time = if count > 0 { total_time / count as f64 } else { 0.0 };

    // Store statistics
    stats.insert("average_time".to_string(), avg_time);
    stats.insert("min_time".to_string(), min_time);
    stats.insert("max_time".to_string(), max_time);
    stats.insert("total_participants".to_string(), count as f64);

    println!("Statistics: Avg time: {:.2}, Min: {:.2}, Max: {:.2}, Participants: {}",
             avg_time, min_time, max_time, count);

    stats
}