/// Container for managing the process queue of warriors. A given core has
/// a single queue, but the queue itself may have numerous "threads" of execution
/// and determines what process is scheduled when.
use std::collections::{BTreeMap, VecDeque};

use thiserror::Error as ThisError;

use super::Offset;

#[derive(Debug, Eq, PartialEq)]
pub struct ProcessEntry {
    pub name: String,
    pub offset: Offset,
}

/// A representation of the process queue. This is effectively a simple FIFO queue.
// TODO enforce size limits based on MAXPROCESSES
#[derive(Debug)]
pub struct Queue {
    /// The actual offsets enqueued to be executed
    queue: VecDeque<ProcessEntry>,
    /// A map of process names to the number of tasks each has in the queue.
    /// This is updated whenever instructions are added to/removed from the queue,
    /// and can be used to determine whether a process is alive or not.
    processes: BTreeMap<String, usize>,
}

impl Queue {
    /// Create an empty queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            processes: BTreeMap::new(),
        }
    }

    /// Get the next offset for execution, removing it from the queue.
    pub fn pop(&mut self) -> Result<ProcessEntry, Error> {
        if let Some(entry) = self.queue.pop_front() {
            let decremented = self.processes[&entry.name].saturating_sub(1);
            self.processes
                .entry(entry.name.clone())
                .and_modify(|count| *count = decremented);

            Ok(entry)
        } else {
            Err(Error::NoRemainingProcesses)
        }
    }

    /// Get the next offset for execution without modifying the queue.
    // TODO: this should probably just return Option<&ProcessEntry>
    pub fn peek(&self) -> Result<&ProcessEntry, Error> {
        if let Some(entry) = self.queue.get(0) {
            Ok(entry)
        } else {
            Err(Error::NoRemainingProcesses)
        }
    }

    /// Add an entry to the process queue.
    pub fn push(&mut self, process_name: String, offset: Offset) {
        self.queue.push_back(ProcessEntry {
            name: process_name.clone(),
            offset,
        });

        *self.processes.entry(process_name).or_insert(0) += 1;
    }

    /// Check the status of a process in the queue. Panics if the process was
    /// never added to the queue.
    pub fn is_process_alive(&self, name: &str) -> bool {
        self.processes[name] > 0
    }
}

/// An process-related error occurred
#[derive(ThisError, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// All processes terminated
    #[error("no process running to execute")]
    NoRemainingProcesses,

    /// The process with this name has no remaining threads
    #[error("process '{0}' has no remaining threads")]
    NoRemainingThreads(String),

    /// A process already exists with the given name
    #[error("a process with the name '{0}' already exists")]
    ProcessNameExists(String),

    /// The warrior attempted to execute a DAT instruction
    #[error("terminated due to reaching a DAT at offset {0}")]
    ExecuteDat(Offset),

    /// The warrior attempted to execute a division by zero
    #[error("terminated due to division by 0")]
    DivideByZero,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue() {
        let mut queue = Queue::new();

        assert_eq!(queue.peek().unwrap_err(), Error::NoRemainingProcesses);
        assert_eq!(queue.pop().unwrap_err(), Error::NoRemainingProcesses);

        let starting_offset = Offset::new(10, 8000);

        queue.push("p1".into(), starting_offset);
        assert_eq!(
            queue.peek().unwrap(),
            &ProcessEntry {
                name: "p1".into(),
                offset: starting_offset
            }
        );
        assert!(queue.is_process_alive("p1"));

        queue.push("p2".into(), starting_offset + 5);
        assert!(queue.is_process_alive("p2"));

        assert_eq!(
            queue.pop().unwrap(),
            ProcessEntry {
                name: "p1".into(),
                offset: starting_offset
            }
        );
        assert_eq!(
            queue.peek().unwrap(),
            &ProcessEntry {
                name: "p2".into(),
                offset: starting_offset + 5
            }
        );
        assert!(!queue.is_process_alive("p1"));
        assert!(queue.is_process_alive("p2"));

        assert_eq!(
            queue.pop().unwrap(),
            ProcessEntry {
                name: "p2".into(),
                offset: starting_offset + 5
            }
        );
        assert!(!queue.is_process_alive("p1"));
        assert!(!queue.is_process_alive("p2"));

        assert_eq!(queue.peek().unwrap_err(), Error::NoRemainingProcesses);
        assert_eq!(queue.pop().unwrap_err(), Error::NoRemainingProcesses);

        assert!(!queue.is_process_alive("p1"));
        assert!(!queue.is_process_alive("p2"));
    }
}
