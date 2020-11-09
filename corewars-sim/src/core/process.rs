/// Container for managing the process queue of warriors. A given core has
/// a single queue, but the queue itself may have numerous "threads" of execution
/// and determines what process is scheduled when.
use std::collections::hash_map::{Entry, HashMap};

use thiserror::Error as ThisError;

use super::Offset;

#[derive(Debug)]
pub struct Queue {
    processes: Vec<Process>,
    process_names: HashMap<String, usize>,
    current_process: usize,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            process_names: HashMap::new(),
            current_process: 0,
        }
    }

    pub fn add_process<T: ToString>(
        &mut self,
        name: T,
        starting_offset: Offset,
    ) -> Result<(), Error> {
        match self.process_names.entry(name.to_string()) {
            Entry::Occupied(_) => Err(Error::ProcessNameExists(name.to_string())),
            Entry::Vacant(entry) => {
                entry.insert(self.processes.len());
                self.processes
                    .push(Process::new(name.to_string(), starting_offset));
                Ok(())
            }
        }
    }

    pub fn current_offset(&self) -> Result<Offset, Error> {
        if self.processes.is_empty() {
            return Err(Error::NoRemainingProcesses);
        }

        Ok(self.processes[self.current_process].current_offset())
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        if self.processes.is_empty() {
            return Err(Error::NoRemainingProcesses);
        }

        self.processes[self.current_process].advance();

        self.current_process += 1;
        self.current_process %= self.processes.len();
        Ok(())
    }

    pub fn stop_thread(&mut self) -> Result<(), Error> {
        self.processes[self.current_process].stop_thread()?;
        Ok(())
    }

    pub fn add_offset(&mut self, offset: Offset) {
        self.processes[self.current_process].add_offset(offset);
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
    #[error("terminated due to reaching a DAT")]
    ExecuteDat,

    /// The warrior attempted to execute a division by zero
    #[error("terminated due to division by 0")]
    DivideByZero,
}

/// Representation of a single process in the core.
#[derive(Debug)]
struct Process {
    name: String,
    threads: Vec<Offset>,
    current_thread: usize,
}

impl Process {
    fn new(name: String, start: Offset) -> Self {
        Self {
            name,
            threads: vec![start],
            current_thread: 0,
        }
    }

    fn advance(&mut self) {
        self.current_thread += 1;
        self.current_thread %= self.threads.len();
    }

    fn add_offset(&mut self, offset: Offset) {
        self.threads[self.current_thread] += offset;
    }

    fn current_offset(&self) -> Offset {
        self.threads[self.current_thread]
    }

    fn stop_thread(&mut self) -> Result<(), Error> {
        self.threads.remove(self.current_thread);
        if self.threads.is_empty() {
            return Err(Error::NoRemainingThreads(self.name.clone()));
        }
        self.advance();
        Ok(())
    }
}
