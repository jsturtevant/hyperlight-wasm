//! In-memory virtual filesystem for WASI preopens.
//!
//! Manages `/input/` (read-only) and `/output/` (writable) directories
//! entirely in memory. No host filesystem access is needed.

use std::collections::HashMap;

/// Pre-allocated descriptor ID for the `/input` preopen directory.
pub const INPUT_DIR_FD: u32 = 3;
/// Pre-allocated descriptor ID for the `/output` preopen directory.
pub const OUTPUT_DIR_FD: u32 = 4;

const FIRST_FILE_FD: u32 = 10;
const FIRST_STREAM_ID: u32 = 100;
const FIRST_DIR_STREAM_ID: u32 = 500;

/// An entry in the virtual filesystem.
#[allow(dead_code)]
pub enum FsEntry {
    Directory { path: String, is_output: bool },
    File { name: String, dir_fd: u32, data: Vec<u8> },
}

/// Read/write stream tracking position within a file.
struct StreamState {
    file_fd: u32,
    offset: u64,
    is_write: bool,
}

/// Directory listing stream state.
struct DirStreamState {
    entries: Vec<(String, bool)>,
    cursor: usize,
}

/// In-memory virtual filesystem backing WASI preopens.
pub struct VirtualFs {
    entries: HashMap<u32, FsEntry>,
    streams: HashMap<u32, StreamState>,
    dir_streams: HashMap<u32, DirStreamState>,
    next_fd: u32,
    next_stream_id: u32,
    next_dir_stream_id: u32,
}

impl VirtualFs {
    pub fn new() -> Self {
        let mut entries = HashMap::new();
        entries.insert(
            INPUT_DIR_FD,
            FsEntry::Directory {
                path: "/input".to_string(),
                is_output: false,
            },
        );
        entries.insert(
            OUTPUT_DIR_FD,
            FsEntry::Directory {
                path: "/output".to_string(),
                is_output: true,
            },
        );
        Self {
            entries,
            streams: HashMap::new(),
            dir_streams: HashMap::new(),
            next_fd: FIRST_FILE_FD,
            next_stream_id: FIRST_STREAM_ID,
            next_dir_stream_id: FIRST_DIR_STREAM_ID,
        }
    }

    // ----- Host-side file management -----

    /// Add a file to `/input/`.
    pub fn add_input_file(&mut self, name: &str, data: Vec<u8>) {
        let fd = self.alloc_fd();
        self.entries.insert(
            fd,
            FsEntry::File {
                name: name.to_string(),
                dir_fd: INPUT_DIR_FD,
                data,
            },
        );
    }

    /// Get all files written to `/output/`.
    pub fn get_output_files(&self) -> HashMap<String, Vec<u8>> {
        let mut result = HashMap::new();
        for entry in self.entries.values() {
            if let FsEntry::File { name, dir_fd, data } = entry {
                if *dir_fd == OUTPUT_DIR_FD {
                    result.insert(name.clone(), data.clone());
                }
            }
        }
        result
    }

    /// Clear all files and streams (between runs).
    pub fn clear(&mut self) {
        self.entries
            .retain(|fd, _| *fd == INPUT_DIR_FD || *fd == OUTPUT_DIR_FD);
        self.streams.clear();
        self.dir_streams.clear();
        self.next_fd = FIRST_FILE_FD;
        self.next_stream_id = FIRST_STREAM_ID;
        self.next_dir_stream_id = FIRST_DIR_STREAM_ID;
    }

    // ----- Allocation -----

    fn alloc_fd(&mut self) -> u32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        fd
    }

    fn alloc_stream_id(&mut self) -> u32 {
        let id = self.next_stream_id;
        self.next_stream_id += 1;
        id
    }

    fn alloc_dir_stream_id(&mut self) -> u32 {
        let id = self.next_dir_stream_id;
        self.next_dir_stream_id += 1;
        id
    }

    // ----- Queries -----

    pub fn is_directory(&self, fd: u32) -> bool {
        matches!(self.entries.get(&fd), Some(FsEntry::Directory { .. }))
    }

    pub fn is_output_dir(&self, fd: u32) -> bool {
        matches!(
            self.entries.get(&fd),
            Some(FsEntry::Directory { is_output: true, .. })
        )
    }

    pub fn is_file(&self, fd: u32) -> bool {
        matches!(self.entries.get(&fd), Some(FsEntry::File { .. }))
    }

    pub fn file_size(&self, fd: u32) -> Option<u64> {
        match self.entries.get(&fd) {
            Some(FsEntry::File { data, .. }) => Some(data.len() as u64),
            _ => None,
        }
    }

    pub fn is_output_file(&self, fd: u32) -> bool {
        matches!(
            self.entries.get(&fd),
            Some(FsEntry::File { dir_fd, .. }) if *dir_fd == OUTPUT_DIR_FD
        )
    }

    /// Find a file by name within a directory. Returns the fd if found.
    pub fn find_file_in_dir(&self, dir_fd: u32, name: &str) -> Option<u32> {
        for (&fd, entry) in &self.entries {
            if let FsEntry::File {
                name: fname,
                dir_fd: dfd,
                ..
            } = entry
            {
                if *dfd == dir_fd && fname == name {
                    return Some(fd);
                }
            }
        }
        None
    }

    // ----- File operations -----

    /// Open (or create) a file relative to a directory descriptor.
    pub fn open_at(&mut self, dir_fd: u32, path: &str, create: bool, truncate: bool) -> Option<u32> {
        let is_output = match self.entries.get(&dir_fd) {
            Some(FsEntry::Directory { is_output, .. }) => *is_output,
            _ => return None,
        };

        // Check if file already exists
        if let Some(fd) = self.find_file_in_dir(dir_fd, path) {
            if truncate {
                if let Some(FsEntry::File { data, .. }) = self.entries.get_mut(&fd) {
                    data.clear();
                }
            }
            return Some(fd);
        }

        // For output dirs, always allow creation. For input, only if create flag.
        if is_output || create {
            let fd = self.alloc_fd();
            self.entries.insert(
                fd,
                FsEntry::File {
                    name: path.to_string(),
                    dir_fd,
                    data: Vec::new(),
                },
            );
            return Some(fd);
        }

        None
    }

    /// Positioned read from a file.
    pub fn read_file(&self, fd: u32, offset: u64, len: u64) -> Option<(Vec<u8>, bool)> {
        match self.entries.get(&fd) {
            Some(FsEntry::File { data, .. }) => {
                let start = (offset as usize).min(data.len());
                let end = (start + len as usize).min(data.len());
                let at_end = end >= data.len();
                Some((data[start..end].to_vec(), at_end))
            }
            _ => None,
        }
    }

    /// Positioned write to a file.
    pub fn write_file(&mut self, fd: u32, offset: u64, buffer: &[u8]) -> Option<u64> {
        match self.entries.get_mut(&fd) {
            Some(FsEntry::File { data, .. }) => {
                let start = offset as usize;
                if start + buffer.len() > data.len() {
                    data.resize(start + buffer.len(), 0);
                }
                data[start..start + buffer.len()].copy_from_slice(buffer);
                Some(buffer.len() as u64)
            }
            _ => None,
        }
    }

    // ----- Stream operations -----

    /// Create a read stream for a file at the given offset.
    pub fn create_read_stream(&mut self, file_fd: u32, offset: u64) -> Option<u32> {
        if !self.is_file(file_fd) {
            return None;
        }
        let id = self.alloc_stream_id();
        self.streams.insert(id, StreamState { file_fd, offset, is_write: false });
        Some(id)
    }

    /// Create a write stream for a file at the given offset.
    pub fn create_write_stream(&mut self, file_fd: u32, offset: u64) -> Option<u32> {
        if !self.is_file(file_fd) {
            return None;
        }
        let id = self.alloc_stream_id();
        self.streams.insert(id, StreamState { file_fd, offset, is_write: true });
        Some(id)
    }

    /// Create a write stream positioned at end of file (append mode).
    pub fn create_append_stream(&mut self, file_fd: u32) -> Option<u32> {
        let size = self.file_size(file_fd)?;
        self.create_write_stream(file_fd, size)
    }

    /// Read from a stream. Returns Ok(bytes) or Err(()) at EOF.
    pub fn stream_read(&mut self, stream_id: u32, len: u64) -> Result<Vec<u8>, ()> {
        let stream = self.streams.get(&stream_id).ok_or(())?;
        let file_fd = stream.file_fd;
        let offset = stream.offset;

        let data = match self.entries.get(&file_fd) {
            Some(FsEntry::File { data, .. }) => data,
            _ => return Err(()),
        };

        if offset as usize >= data.len() {
            return Err(());
        }

        let start = offset as usize;
        let end = (start + len as usize).min(data.len());
        let bytes = data[start..end].to_vec();

        let stream = self.streams.get_mut(&stream_id).unwrap();
        stream.offset = end as u64;
        Ok(bytes)
    }

    /// Write to a stream. Returns bytes written.
    pub fn stream_write(&mut self, stream_id: u32, buffer: &[u8]) -> Option<u64> {
        let stream = self.streams.get(&stream_id)?;
        let file_fd = stream.file_fd;
        let offset = stream.offset;
        let written = self.write_file(file_fd, offset, buffer)?;
        let stream = self.streams.get_mut(&stream_id)?;
        stream.offset += written;
        Some(written)
    }

    /// Check if a stream ID is tracked by this filesystem.
    pub fn has_stream(&self, id: u32) -> bool {
        self.streams.contains_key(&id)
    }

    /// Check if a stream is a write stream.
    pub fn is_write_stream(&self, id: u32) -> bool {
        self.streams.get(&id).map_or(false, |s| s.is_write)
    }

    // ----- Directory listing -----

    /// Create a directory listing stream.
    pub fn create_dir_stream(&mut self, dir_fd: u32) -> Option<u32> {
        if !self.is_directory(dir_fd) {
            return None;
        }
        let mut file_entries = Vec::new();
        for entry in self.entries.values() {
            if let FsEntry::File { name, dir_fd: dfd, .. } = entry {
                if *dfd == dir_fd {
                    file_entries.push((name.clone(), false));
                }
            }
        }
        let id = self.alloc_dir_stream_id();
        self.dir_streams.insert(id, DirStreamState { entries: file_entries, cursor: 0 });
        Some(id)
    }

    /// Read next directory entry. Returns Some(Some((name, is_dir))) for an entry,
    /// Some(None) at end, None if invalid stream.
    pub fn read_dir_entry(&mut self, stream_id: u32) -> Option<Option<(String, bool)>> {
        let stream = self.dir_streams.get_mut(&stream_id)?;
        if stream.cursor >= stream.entries.len() {
            return Some(None);
        }
        let entry = stream.entries[stream.cursor].clone();
        stream.cursor += 1;
        Some(Some(entry))
    }

    /// Check if a directory stream ID is tracked.
    pub fn has_dir_stream(&self, id: u32) -> bool {
        self.dir_streams.contains_key(&id)
    }
}
