/// Container for managing the process queue of warriors. A given core has
/// a single queue, but the queue itself may have numerous "threads" of execution
/// and determines what process is scheduled when.
use std::collections::{BTreeMap, VecDeque};

use thiserror::Error as ThisError;

use super::Offset;

#[derive(Debug, Eq, PartialEq)]
pub struct Entry {
    pub name: String,
    pub thread: usize,
    pub offset: Offset,
}

/// A representation of the process queue. This is effectively a simple FIFO queue.
// TODO enforce size limits based on MAXPROCESSES
#[derive(Debug)]
pub struct Queue {
    /// The actual offsets enqueued to be executed
    queue: VecDeque<Entry>,

    /// A map of process names to the number of tasks each has in the queue.
    /// This is updated whenever instructions are added to/removed from the queue,
    /// and can be used to determine whether a process is alive or not.
    processes: BTreeMap<String, usize>,

    /// An increasing counter per process to give unique thread ids
    next_thread_id: BTreeMap<String, usize>,
}

impl Queue {
    /// Create an empty queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            processes: BTreeMap::new(),
            next_thread_id: BTreeMap::new(),
        }
    }

    /// Get the next offset for execution, removing it from the queue.
    pub fn pop(&mut self) -> Result<Entry, Error> {
        self.queue
            .pop_front()
            .map_or(Err(Error::NoRemainingProcesses), |entry| {
                let decremented = self.processes[&entry.name].saturating_sub(1);
                self.processes
                    .entry(entry.name.clone())
                    .and_modify(|count| *count = decremented);

                Ok(entry)
            })
    }

    /// Get the next offset for execution without modifying the queue.
    // TODO: this should probably just return Option<&ProcessEntry>
    pub fn peek(&self) -> Result<&Entry, Error> {
        self.queue.get(0).ok_or(Error::NoRemainingProcesses)
    }

    /// Add an entry to the process queue. If specified, it will use the given thread ID,
    /// otherwise a new thread ID will be created based on the current number of
    /// threads active for this process name.
    pub fn push(&mut self, process_name: String, offset: Offset, thread: Option<usize>) {
        let thread_id = thread.map_or_else(
            || {
                let entry = self.next_thread_id.entry(process_name.clone()).or_insert(0);
                let id = *entry;
                *entry += 1;
                id
            },
            |id| id,
        );

        self.queue.push_back(Entry {
            name: process_name.clone(),
            thread: thread_id,
            offset,
        });

        *self.processes.entry(process_name).or_insert(0) += 1;
    }

    /// Check the status of a process in the queue. Panics if the process was
    /// never added to the queue.
    pub fn thread_count(&self, name: &str) -> usize {
        self.processes[name]
    }
}

/// An process-related error occurred
#[derive(ThisError, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// All processes terminated
    #[error("no process running to execute")]
    NoRemainingProcesses,

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
    fn queue_multiple_processes() {
        let mut queue = Queue::new();

        assert_eq!(queue.peek().unwrap_err(), Error::NoRemainingProcesses);
        assert_eq!(queue.pop().unwrap_err(), Error::NoRemainingProcesses);

        let starting_offset = Offset::new(10, 8000);

        queue.push("p1".into(), starting_offset, None);
        assert_eq!(
            queue.peek().unwrap(),
            &Entry {
                name: "p1".into(),
                thread: 0,
                offset: starting_offset
            }
        );
        assert!(queue.thread_count("p1") > 0);

        queue.push("p2".into(), starting_offset + 5, None);
        assert!(queue.thread_count("p2") > 0);

        assert_eq!(
            queue.pop().unwrap(),
            Entry {
                name: "p1".into(),
                thread: 0,
                offset: starting_offset
            }
        );
        assert_eq!(
            queue.peek().unwrap(),
            &Entry {
                name: "p2".into(),
                thread: 0,
                offset: starting_offset + 5
            }
        );
        assert!(!queue.thread_count("p1") > 0);
        assert!(queue.thread_count("p2") > 0);

        assert_eq!(
            queue.pop().unwrap(),
            Entry {
                name: "p2".into(),
                thread: 0,
                offset: starting_offset + 5
            }
        );
        assert!(!queue.thread_count("p1") > 0);
        assert!(!queue.thread_count("p2") > 0);

        assert_eq!(queue.peek().unwrap_err(), Error::NoRemainingProcesses);
        assert_eq!(queue.pop().unwrap_err(), Error::NoRemainingProcesses);

        assert!(!queue.thread_count("p1") > 0);
        assert!(!queue.thread_count("p2") > 0);
    }

    #[test]
    fn queue_single_process() {
        let mut queue = Queue::new();
        let starting_offset = Offset::new(10, 8000);

        queue.push("p1".into(), starting_offset, None);
        assert_eq!(
            queue.peek().unwrap(),
            &Entry {
                name: "p1".into(),
                thread: 0,
                offset: starting_offset
            }
        );
        assert!(queue.thread_count("p1") > 0);

        // should increment the thread id to 1
        queue.push("p1".into(), starting_offset, None);
        queue.pop().unwrap();
        assert_eq!(
            queue.peek().unwrap(),
            &Entry {
                name: "p1".into(),
                thread: 1,
                offset: starting_offset
            }
        );
        assert!(queue.thread_count("p1") > 0);

        queue.push("p1".into(), starting_offset, Some(1));
        queue.pop().unwrap();
        assert_eq!(
            queue.peek().unwrap(),
            &Entry {
                name: "p1".into(),
                thread: 1,
                offset: starting_offset
            }
        );
        assert!(queue.thread_count("p1") > 0);
    }
}
