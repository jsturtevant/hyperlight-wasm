//! Simple synchronous WASI implementations for HostState.
//!
//! All resource handle types are `u32`. Most operations are stubs that return
//! errors or no-ops — the guest Python captures stdout/stderr internally
//! and returns them through the WIT export, so WASI I/O is not critical.
#![allow(unused_variables)]

use crate::bindings::wasi;
use crate::virtual_fs;
use crate::HostState;
use hyperlight_common::resource::BorrowedResourceGuard;

use wasi::clocks::monotonic_clock;
use wasi::clocks::wall_clock;
use wasi::filesystem::types as fs_types;
use wasi::io::streams;
use wasi::sockets::ip_name_lookup;
use wasi::sockets::network;
use wasi::sockets::tcp;
use wasi::sockets::udp;

// ---------------------------------------------------------------------------
// IO: Error
// ---------------------------------------------------------------------------

impl wasi::io::error::Error for HostState {
    type T = u32;
    fn to_debug_string(&mut self, _self_: BorrowedResourceGuard<u32>) -> String {
        String::from("error")
    }
}

impl wasi::io::Error for HostState {}

// ---------------------------------------------------------------------------
// IO: Poll
// ---------------------------------------------------------------------------

impl wasi::io::poll::Pollable for HostState {
    type T = u32;
    fn ready(&mut self, _self_: BorrowedResourceGuard<u32>) -> bool {
        true
    }
    fn block(&mut self, _self_: BorrowedResourceGuard<u32>) {}
}

impl wasi::io::Poll for HostState {
    fn poll(&mut self, _in: Vec<BorrowedResourceGuard<u32>>) -> Vec<u32> {
        (0.._in.len() as u32).collect()
    }
}

// ---------------------------------------------------------------------------
// IO: Streams
// ---------------------------------------------------------------------------

impl streams::InputStream<u32, u32> for HostState {
    type T = u32;
    fn read(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<Vec<u8>, streams::StreamError<u32>> {
        let stream_id = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.has_stream(stream_id) && !fs.is_write_stream(stream_id) {
            return fs.stream_read(stream_id, _len)
                .map_err(|_| streams::StreamError::Closed);
        }
        Err(streams::StreamError::Closed)
    }
    fn blocking_read(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<Vec<u8>, streams::StreamError<u32>> {
        let stream_id = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.has_stream(stream_id) && !fs.is_write_stream(stream_id) {
            return fs.stream_read(stream_id, _len)
                .map_err(|_| streams::StreamError::Closed);
        }
        Err(streams::StreamError::Closed)
    }
    fn skip(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<u64, streams::StreamError<u32>> {
        Err(streams::StreamError::Closed)
    }
    fn blocking_skip(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<u64, streams::StreamError<u32>> {
        Err(streams::StreamError::Closed)
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
}

impl streams::OutputStream<u32, u32, u32> for HostState {
    type T = u32;
    fn check_write(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u64, streams::StreamError<u32>> {
        Ok(65536)
    }
    fn write(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _contents: Vec<u8>,
    ) -> Result<(), streams::StreamError<u32>> {
        let stream_id = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.has_stream(stream_id) && fs.is_write_stream(stream_id) {
            fs.stream_write(stream_id, &_contents);
            return Ok(());
        }
        Ok(())
    }
    fn blocking_write_and_flush(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _contents: Vec<u8>,
    ) -> Result<(), streams::StreamError<u32>> {
        let stream_id = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.has_stream(stream_id) && fs.is_write_stream(stream_id) {
            fs.stream_write(stream_id, &_contents);
            return Ok(());
        }
        Ok(())
    }
    fn flush(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), streams::StreamError<u32>> {
        Ok(())
    }
    fn blocking_flush(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), streams::StreamError<u32>> {
        Ok(())
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
    fn write_zeroes(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<(), streams::StreamError<u32>> {
        Ok(())
    }
    fn blocking_write_zeroes_and_flush(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<(), streams::StreamError<u32>> {
        Ok(())
    }
    fn splice(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _src: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<u64, streams::StreamError<u32>> {
        Err(streams::StreamError::Closed)
    }
    fn blocking_splice(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _src: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<u64, streams::StreamError<u32>> {
        Err(streams::StreamError::Closed)
    }
}

impl wasi::io::Streams<u32, u32> for HostState {}

// ---------------------------------------------------------------------------
// CLI: Environment, Exit, Stdin/Stdout/Stderr
// ---------------------------------------------------------------------------

impl wasi::cli::Environment for HostState {
    fn get_environment(&mut self) -> Vec<(String, String)> {
        Vec::new()
    }
    fn get_arguments(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn initial_cwd(&mut self) -> Option<String> {
        None
    }
}

impl wasi::cli::Exit for HostState {
    fn exit(&mut self, _status: Result<(), ()>) {}
}

impl wasi::cli::Stdin<u32> for HostState {
    fn get_stdin(&mut self) -> u32 {
        0
    }
}

impl wasi::cli::Stdout<u32> for HostState {
    fn get_stdout(&mut self) -> u32 {
        0
    }
}

impl wasi::cli::Stderr<u32> for HostState {
    fn get_stderr(&mut self) -> u32 {
        0
    }
}

// ---------------------------------------------------------------------------
// CLI: Terminals (stubs — no terminal support)
// ---------------------------------------------------------------------------

impl wasi::cli::terminal_input::TerminalInput for HostState {
    type T = u32;
}
impl wasi::cli::TerminalInput for HostState {}

impl wasi::cli::terminal_output::TerminalOutput for HostState {
    type T = u32;
}
impl wasi::cli::TerminalOutput for HostState {}

impl wasi::cli::TerminalStdin<u32> for HostState {
    fn get_terminal_stdin(&mut self) -> Option<u32> {
        None
    }
}

impl wasi::cli::TerminalStdout<u32> for HostState {
    fn get_terminal_stdout(&mut self) -> Option<u32> {
        None
    }
}

impl wasi::cli::TerminalStderr<u32> for HostState {
    fn get_terminal_stderr(&mut self) -> Option<u32> {
        None
    }
}

// ---------------------------------------------------------------------------
// Clocks
// ---------------------------------------------------------------------------

impl wasi::clocks::MonotonicClock<u32> for HostState {
    fn now(&mut self) -> monotonic_clock::Instant {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    fn resolution(&mut self) -> monotonic_clock::Duration {
        1
    }
    fn subscribe_instant(&mut self, _when: monotonic_clock::Instant) -> u32 {
        0
    }
    fn subscribe_duration(&mut self, _when: monotonic_clock::Duration) -> u32 {
        0
    }
}

impl wasi::clocks::WallClock for HostState {
    fn now(&mut self) -> wall_clock::Datetime {
        let d = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        wall_clock::Datetime {
            seconds: d.as_secs(),
            nanoseconds: d.subsec_nanos(),
        }
    }
    fn resolution(&mut self) -> wall_clock::Datetime {
        wall_clock::Datetime {
            seconds: 0,
            nanoseconds: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Filesystem (stub — returns errors for most operations)
// ---------------------------------------------------------------------------

impl fs_types::DirectoryEntryStream for HostState {
    type T = u32;
    fn read_directory_entry(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<Option<fs_types::DirectoryEntry>, fs_types::ErrorCode> {
        let stream_id = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.has_dir_stream(stream_id) {
            match fs.read_dir_entry(stream_id) {
                Some(Some((name, is_dir))) => {
                    let dtype = if is_dir {
                        fs_types::DescriptorType::Directory
                    } else {
                        fs_types::DescriptorType::RegularFile
                    };
                    return Ok(Some(fs_types::DirectoryEntry {
                        r#type: dtype,
                        r#name: name,
                    }));
                }
                Some(None) => return Ok(None),
                None => {}
            }
        }
        Ok(None)
    }
}

impl fs_types::Descriptor<wall_clock::Datetime, u32, u32, u32> for HostState {
    type T = u32;

    fn read_via_stream(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _offset: fs_types::Filesize,
    ) -> Result<u32, fs_types::ErrorCode> {
        let fd = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.is_file(fd) {
            return fs.create_read_stream(fd, _offset)
                .ok_or(fs_types::ErrorCode::BadDescriptor);
        }
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn write_via_stream(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _offset: fs_types::Filesize,
    ) -> Result<u32, fs_types::ErrorCode> {
        let fd = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.is_file(fd) {
            return fs.create_write_stream(fd, _offset)
                .ok_or(fs_types::ErrorCode::BadDescriptor);
        }
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn append_via_stream(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u32, fs_types::ErrorCode> {
        let fd = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.is_file(fd) {
            return fs.create_append_stream(fd)
                .ok_or(fs_types::ErrorCode::BadDescriptor);
        }
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn advise(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _offset: fs_types::Filesize,
        _length: fs_types::Filesize,
        _advice: fs_types::Advice,
    ) -> Result<(), fs_types::ErrorCode> {
        Ok(())
    }
    fn sync_data(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn get_type(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<fs_types::DescriptorType, fs_types::ErrorCode> {
        let fd = *_self_;
        let fs = self.fs.lock().unwrap();
        if fs.is_directory(fd) {
            Ok(fs_types::DescriptorType::Directory)
        } else if fs.is_file(fd) {
            Ok(fs_types::DescriptorType::RegularFile)
        } else {
            Ok(fs_types::DescriptorType::Directory)
        }
    }
    fn set_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _size: fs_types::Filesize,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn set_times(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _data_access_timestamp: fs_types::NewTimestamp<wall_clock::Datetime>,
        _data_modification_timestamp: fs_types::NewTimestamp<wall_clock::Datetime>,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn read(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _length: fs_types::Filesize,
        _offset: fs_types::Filesize,
    ) -> Result<(Vec<u8>, bool), fs_types::ErrorCode> {
        let fd = *_self_;
        let fs = self.fs.lock().unwrap();
        if fs.is_file(fd) {
            return fs.read_file(fd, _offset, _length)
                .ok_or(fs_types::ErrorCode::BadDescriptor);
        }
        Err(fs_types::ErrorCode::BadDescriptor)
    }
    fn write(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _buffer: Vec<u8>,
        _offset: fs_types::Filesize,
    ) -> Result<fs_types::Filesize, fs_types::ErrorCode> {
        let fd = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.is_file(fd) {
            return fs.write_file(fd, _offset, &_buffer)
                .ok_or(fs_types::ErrorCode::BadDescriptor);
        }
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn read_directory(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u32, fs_types::ErrorCode> {
        let fd = *_self_;
        let mut fs = self.fs.lock().unwrap();
        if fs.is_directory(fd) {
            return fs.create_dir_stream(fd)
                .ok_or(fs_types::ErrorCode::BadDescriptor);
        }
        Err(fs_types::ErrorCode::BadDescriptor)
    }
    fn sync(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn create_directory_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path: String,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn stat(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<fs_types::DescriptorStat<wall_clock::Datetime>, fs_types::ErrorCode> {
        let fd = *_self_;
        let fs = self.fs.lock().unwrap();
        if fs.is_directory(fd) {
            return Ok(fs_types::DescriptorStat {
                r#type: fs_types::DescriptorType::Directory,
                r#link_count: 1,
                r#size: 0,
                r#data_access_timestamp: None,
                r#data_modification_timestamp: None,
                r#status_change_timestamp: None,
            });
        }
        if let Some(size) = fs.file_size(fd) {
            return Ok(fs_types::DescriptorStat {
                r#type: fs_types::DescriptorType::RegularFile,
                r#link_count: 1,
                r#size: size,
                r#data_access_timestamp: None,
                r#data_modification_timestamp: None,
                r#status_change_timestamp: None,
            });
        }
        Err(fs_types::ErrorCode::BadDescriptor)
    }
    fn readlink_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path: String,
    ) -> Result<String, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::NoEntry)
    }
    fn remove_directory_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path: String,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn rename_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _old_path: String,
        _new_descriptor: BorrowedResourceGuard<u32>,
        _new_path: String,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn symlink_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _old_path: String,
        _new_path: String,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn unlink_file_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path: String,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn is_same_object(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _other: BorrowedResourceGuard<u32>,
    ) -> bool {
        false
    }
    fn metadata_hash(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<fs_types::MetadataHashValue, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn get_flags(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<fs_types::DescriptorFlags, fs_types::ErrorCode> {
        let fd = *_self_;
        let fs = self.fs.lock().unwrap();
        if fs.is_directory(fd) {
            let writable = fs.is_output_dir(fd);
            return Ok(fs_types::DescriptorFlags {
                r#read: true,
                r#write: writable,
                r#file_integrity_sync: false,
                r#data_integrity_sync: false,
                r#requested_write_sync: false,
                r#mutate_directory: writable,
            });
        }
        if fs.is_file(fd) {
            let writable = fs.is_output_file(fd);
            return Ok(fs_types::DescriptorFlags {
                r#read: true,
                r#write: writable,
                r#file_integrity_sync: false,
                r#data_integrity_sync: false,
                r#requested_write_sync: false,
                r#mutate_directory: false,
            });
        }
        Err(fs_types::ErrorCode::BadDescriptor)
    }
    fn stat_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path_flags: fs_types::PathFlags,
        _path: String,
    ) -> Result<fs_types::DescriptorStat<wall_clock::Datetime>, fs_types::ErrorCode> {
        let dir_fd = *_self_;
        let fs = self.fs.lock().unwrap();
        if let Some(file_fd) = fs.find_file_in_dir(dir_fd, &_path) {
            let size = fs.file_size(file_fd).unwrap_or(0);
            return Ok(fs_types::DescriptorStat {
                r#type: fs_types::DescriptorType::RegularFile,
                r#link_count: 1,
                r#size: size,
                r#data_access_timestamp: None,
                r#data_modification_timestamp: None,
                r#status_change_timestamp: None,
            });
        }
        Err(fs_types::ErrorCode::NoEntry)
    }
    fn set_times_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path_flags: fs_types::PathFlags,
        _path: String,
        _data_access_timestamp: fs_types::NewTimestamp<wall_clock::Datetime>,
        _data_modification_timestamp: fs_types::NewTimestamp<wall_clock::Datetime>,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn link_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _old_path_flags: fs_types::PathFlags,
        _old_path: String,
        _new_descriptor: BorrowedResourceGuard<u32>,
        _new_path: String,
    ) -> Result<(), fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn open_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path_flags: fs_types::PathFlags,
        _path: String,
        _open_flags: fs_types::OpenFlags,
        _flags: fs_types::DescriptorFlags,
    ) -> Result<u32, fs_types::ErrorCode> {
        let dir_fd = *_self_;
        let mut fs = self.fs.lock().unwrap();
        let create = _open_flags.r#create;
        let truncate = _open_flags.r#truncate;
        fs.open_at(dir_fd, &_path, create, truncate)
            .ok_or(fs_types::ErrorCode::NoEntry)
    }
    fn metadata_hash_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path_flags: fs_types::PathFlags,
        _path: String,
    ) -> Result<fs_types::MetadataHashValue, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
}

impl wasi::filesystem::Types<wall_clock::Datetime, u32, u32, u32> for HostState {
    fn filesystem_error_code(
        &mut self,
        _err: BorrowedResourceGuard<u32>,
    ) -> Option<fs_types::ErrorCode> {
        None
    }
}

impl wasi::filesystem::Preopens<u32> for HostState {
    fn get_directories(&mut self) -> Vec<(u32, String)> {
        vec![
            (virtual_fs::INPUT_DIR_FD, "/input".to_string()),
            (virtual_fs::OUTPUT_DIR_FD, "/output".to_string()),
        ]
    }
}

// ---------------------------------------------------------------------------
// Sockets: Network
// ---------------------------------------------------------------------------

impl network::Network for HostState {
    type T = u32;
}

impl wasi::sockets::Network for HostState {}

impl wasi::sockets::InstanceNetwork<u32> for HostState {
    fn instance_network(&mut self) -> u32 {
        0
    }
}

// ---------------------------------------------------------------------------
// Sockets: TCP
// ---------------------------------------------------------------------------

impl tcp::TcpSocket<
    monotonic_clock::Duration,
    network::ErrorCode,
    u32,
    network::IpAddressFamily,
    network::IpSocketAddress,
    u32,
    u32,
    u32,
> for HostState
{
    type T = u32;
    fn start_bind(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _network: BorrowedResourceGuard<u32>,
        _local_address: network::IpSocketAddress,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn finish_bind(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn start_connect(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _network: BorrowedResourceGuard<u32>,
        _remote_address: network::IpSocketAddress,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn finish_connect(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(u32, u32), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn start_listen(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn finish_listen(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn accept(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(u32, u32, u32), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn local_address(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<network::IpSocketAddress, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn remote_address(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<network::IpSocketAddress, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn is_listening(&mut self, _self_: BorrowedResourceGuard<u32>) -> bool {
        false
    }
    fn address_family(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> network::IpAddressFamily {
        network::IpAddressFamily::Ipv4
    }
    fn set_listen_backlog_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u64,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn keep_alive_enabled(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<bool, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_keep_alive_enabled(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: bool,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn keep_alive_idle_time(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<monotonic_clock::Duration, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_keep_alive_idle_time(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: monotonic_clock::Duration,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn keep_alive_interval(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<monotonic_clock::Duration, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_keep_alive_interval(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: monotonic_clock::Duration,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn keep_alive_count(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u32, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_keep_alive_count(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u32,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn hop_limit(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u8, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_hop_limit(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u8,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn receive_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u64, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_receive_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u64,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn send_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u64, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_send_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u64,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
    fn shutdown(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _shutdown_type: tcp::ShutdownType,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
}

impl wasi::sockets::Tcp<
    monotonic_clock::Duration,
    network::ErrorCode,
    u32,
    network::IpAddressFamily,
    network::IpSocketAddress,
    u32,
    u32,
    u32,
> for HostState
{
}

impl wasi::sockets::TcpCreateSocket<network::ErrorCode, network::IpAddressFamily, u32>
    for HostState
{
    fn create_tcp_socket(
        &mut self,
        _address_family: network::IpAddressFamily,
    ) -> Result<u32, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
}

// ---------------------------------------------------------------------------
// Sockets: UDP
// ---------------------------------------------------------------------------

impl udp::IncomingDatagramStream<network::ErrorCode, network::IpSocketAddress, u32>
    for HostState
{
    type T = u32;
    fn receive(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _max_results: u64,
    ) -> Result<Vec<udp::IncomingDatagram<network::IpSocketAddress>>, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
}

impl udp::OutgoingDatagramStream<network::ErrorCode, network::IpSocketAddress, u32>
    for HostState
{
    type T = u32;
    fn check_send(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u64, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn send(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _datagrams: Vec<udp::OutgoingDatagram<network::IpSocketAddress>>,
    ) -> Result<u64, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
}

impl udp::UdpSocket<
    network::ErrorCode,
    u32,
    network::IpAddressFamily,
    network::IpSocketAddress,
    u32,
    u32,
    u32,
> for HostState
{
    type T = u32;
    fn start_bind(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _network: BorrowedResourceGuard<u32>,
        _local_address: network::IpSocketAddress,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn finish_bind(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn stream(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _remote_address: Option<network::IpSocketAddress>,
    ) -> Result<(u32, u32), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn local_address(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<network::IpSocketAddress, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn remote_address(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<network::IpSocketAddress, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn address_family(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> network::IpAddressFamily {
        network::IpAddressFamily::Ipv4
    }
    fn unicast_hop_limit(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u8, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_unicast_hop_limit(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u8,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn receive_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u64, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_receive_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u64,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn send_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u64, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn set_send_buffer_size(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _value: u64,
    ) -> Result<(), network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
}

impl wasi::sockets::Udp<
    network::ErrorCode,
    network::IpAddressFamily,
    network::IpSocketAddress,
    u32,
    u32,
> for HostState
{
}

impl wasi::sockets::UdpCreateSocket<network::ErrorCode, network::IpAddressFamily, u32>
    for HostState
{
    fn create_udp_socket(
        &mut self,
        _address_family: network::IpAddressFamily,
    ) -> Result<u32, network::ErrorCode> {
        Err(network::ErrorCode::NotSupported)
    }
}

// ---------------------------------------------------------------------------
// Sockets: IP Name Lookup
// ---------------------------------------------------------------------------

impl ip_name_lookup::ResolveAddressStream<network::ErrorCode, network::IpAddress, u32>
    for HostState
{
    type T = u32;
    fn resolve_next_address(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<Option<network::IpAddress>, network::ErrorCode> {
        Ok(None)
    }
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
}

impl wasi::sockets::IpNameLookup<network::ErrorCode, network::IpAddress, u32, u32>
    for HostState
{
    fn resolve_addresses(
        &mut self,
        _network: BorrowedResourceGuard<u32>,
        _name: String,
    ) -> Result<u32, network::ErrorCode> {
        Err(network::ErrorCode::PermanentResolverFailure)
    }
}

// ---------------------------------------------------------------------------
// Random
// ---------------------------------------------------------------------------

impl wasi::random::Random for HostState {
    fn get_random_bytes(&mut self, len: u64) -> Vec<u8> {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut state = seed as u64;
        (0..len)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                (state >> 33) as u8
            })
            .collect()
    }
    fn get_random_u64(&mut self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

impl wasi::random::Insecure for HostState {
    fn get_insecure_random_bytes(&mut self, len: u64) -> Vec<u8> {
        vec![0u8; len as usize]
    }
    fn get_insecure_random_u64(&mut self) -> u64 {
        42
    }
}

impl wasi::random::InsecureSeed for HostState {
    fn insecure_seed(&mut self) -> (u64, u64) {
        (0, 0)
    }
}

// ---------------------------------------------------------------------------
// HTTP: Types + OutgoingHandler (Phase 3.5 — WASI-HTTP networking)
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use wasi::http::types as http_types;

/// Stored outgoing request state.
struct HttpRequest {
    method: String,
    scheme: String,
    authority: Option<String>,
    path_with_query: Option<String>,
    headers_handle: u32,
    body_handle: Option<u32>,
}

/// In-memory store for HTTP resources (fields, requests, responses, bodies).
/// All keyed by `u32` handles, matching the resource handle pattern used
/// throughout this file.
struct HttpStore {
    next_handle: u32,
    fields: HashMap<u32, Vec<(String, Vec<u8>)>>,
    requests: HashMap<u32, HttpRequest>,
    /// Incoming response state: (status_code, headers_handle, body bytes)
    responses: HashMap<u32, (u16, u32, Vec<u8>)>,
    /// Outgoing body buffers
    outgoing_bodies: HashMap<u32, Vec<u8>>,
    /// Incoming body read cursors
    incoming_body_cursors: HashMap<u32, usize>,
    /// Future responses: handle → Option<Result<response_handle, String>>
    future_responses: HashMap<u32, Option<Result<u32, String>>>,
    /// Request options
    request_options: HashMap<u32, (Option<u64>, Option<u64>, Option<u64>)>,
}

impl HttpStore {
    fn new() -> Self {
        Self {
            next_handle: 5000, // start high to avoid collision with FS handles
            fields: HashMap::new(),
            requests: HashMap::new(),
            responses: HashMap::new(),
            outgoing_bodies: HashMap::new(),
            incoming_body_cursors: HashMap::new(),
            future_responses: HashMap::new(),
            request_options: HashMap::new(),
        }
    }
    fn alloc(&mut self) -> u32 {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }
}

use std::sync::OnceLock;
use std::sync::Mutex as StdMutex;

fn http_store() -> &'static StdMutex<HttpStore> {
    static STORE: OnceLock<StdMutex<HttpStore>> = OnceLock::new();
    STORE.get_or_init(|| StdMutex::new(HttpStore::new()))
}

// -- Fields (headers/trailers) resource --

impl http_types::Fields<u32> for HostState {
    type T = u32;

    fn new(&mut self) -> u32 {
        let mut store = http_store().lock().unwrap();
        let h = store.alloc();
        store.fields.insert(h, Vec::new());
        h
    }
    fn from_list(
        &mut self,
        entries: Vec<(String, Vec<u8>)>,
    ) -> Result<u32, u32> {
        let mut store = http_store().lock().unwrap();
        let h = store.alloc();
        store.fields.insert(h, entries);
        Ok(h)
    }
    fn get(&mut self, self_: BorrowedResourceGuard<u32>, name: String) -> Vec<Vec<u8>> {
        let store = http_store().lock().unwrap();
        store.fields.get(&*self_)
            .map(|f| f.iter().filter(|(k, _)| k == &name).map(|(_, v)| v.clone()).collect())
            .unwrap_or_default()
    }
    fn has(&mut self, self_: BorrowedResourceGuard<u32>, name: String) -> bool {
        let store = http_store().lock().unwrap();
        store.fields.get(&*self_)
            .map(|f| f.iter().any(|(k, _)| k == &name))
            .unwrap_or(false)
    }
    fn set(
        &mut self,
        self_: BorrowedResourceGuard<u32>,
        name: String,
        value: Vec<Vec<u8>>,
    ) -> Result<(), u32> {
        let mut store = http_store().lock().unwrap();
        if let Some(f) = store.fields.get_mut(&*self_) {
            f.retain(|(k, _)| k != &name);
            for v in value {
                f.push((name.clone(), v));
            }
        }
        Ok(())
    }
    fn delete(&mut self, self_: BorrowedResourceGuard<u32>, name: String) -> Result<(), u32> {
        let mut store = http_store().lock().unwrap();
        if let Some(f) = store.fields.get_mut(&*self_) {
            f.retain(|(k, _)| k != &name);
        }
        Ok(())
    }
    fn append(
        &mut self,
        self_: BorrowedResourceGuard<u32>,
        name: String,
        value: Vec<u8>,
    ) -> Result<(), u32> {
        let mut store = http_store().lock().unwrap();
        if let Some(f) = store.fields.get_mut(&*self_) {
            f.push((name, value));
        }
        Ok(())
    }
    fn entries(&mut self, self_: BorrowedResourceGuard<u32>) -> Vec<(String, Vec<u8>)> {
        let store = http_store().lock().unwrap();
        store.fields.get(&*self_).cloned().unwrap_or_default()
    }
    fn clone(&mut self, self_: BorrowedResourceGuard<u32>) -> u32 {
        let mut store = http_store().lock().unwrap();
        let entries = store.fields.get(&*self_).cloned().unwrap_or_default();
        let h = store.alloc();
        store.fields.insert(h, entries);
        h
    }
}

// -- IncomingRequest (stub — we don't serve incoming HTTP) --

impl http_types::IncomingRequest<u32, u32> for HostState {
    type T = u32;
    fn method(&mut self, _self_: BorrowedResourceGuard<u32>) -> http_types::Method {
        http_types::Method::Get
    }
    fn path_with_query(&mut self, _self_: BorrowedResourceGuard<u32>) -> Option<String> {
        None
    }
    fn scheme(&mut self, _self_: BorrowedResourceGuard<u32>) -> Option<http_types::Scheme> {
        None
    }
    fn authority(&mut self, _self_: BorrowedResourceGuard<u32>) -> Option<String> {
        None
    }
    fn headers(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0
    }
    fn consume(&mut self, _self_: BorrowedResourceGuard<u32>) -> Result<u32, ()> {
        Err(())
    }
}

// -- OutgoingRequest --

impl http_types::OutgoingRequest<u32, u32> for HostState {
    type T = u32;
    fn new(&mut self, headers: u32) -> u32 {
        let mut store = http_store().lock().unwrap();
        let h = store.alloc();
        store.requests.insert(h, HttpRequest {
            method: "GET".into(),
            scheme: "https".into(),
            authority: None,
            path_with_query: None,
            headers_handle: headers,
            body_handle: None,
        });
        h
    }
    fn body(&mut self, self_: BorrowedResourceGuard<u32>) -> Result<u32, ()> {
        let mut store = http_store().lock().unwrap();
        let body_h = store.alloc();
        store.outgoing_bodies.insert(body_h, Vec::new());
        if let Some(req) = store.requests.get_mut(&*self_) {
            req.body_handle = Some(body_h);
        }
        Ok(body_h)
    }
    fn method(&mut self, self_: BorrowedResourceGuard<u32>) -> http_types::Method {
        let store = http_store().lock().unwrap();
        let m = store.requests.get(&*self_).map(|r| r.method.as_str()).unwrap_or("GET");
        match m {
            "GET" => http_types::Method::Get,
            "POST" => http_types::Method::Post,
            "PUT" => http_types::Method::Put,
            "DELETE" => http_types::Method::Delete,
            "HEAD" => http_types::Method::Head,
            "OPTIONS" => http_types::Method::Options,
            "PATCH" => http_types::Method::Patch,
            other => http_types::Method::Other(other.to_string()),
        }
    }
    fn set_method(&mut self, self_: BorrowedResourceGuard<u32>, method: http_types::Method) -> Result<(), ()> {
        let s = match &method {
            http_types::Method::Get => "GET",
            http_types::Method::Post => "POST",
            http_types::Method::Put => "PUT",
            http_types::Method::Delete => "DELETE",
            http_types::Method::Head => "HEAD",
            http_types::Method::Options => "OPTIONS",
            http_types::Method::Patch => "PATCH",
            http_types::Method::Connect => "CONNECT",
            http_types::Method::Trace => "TRACE",
            http_types::Method::Other(m) => m.as_str(),
        };
        let mut store = http_store().lock().unwrap();
        if let Some(req) = store.requests.get_mut(&*self_) { req.method = s.to_string(); }
        Ok(())
    }
    fn path_with_query(&mut self, self_: BorrowedResourceGuard<u32>) -> Option<String> {
        http_store().lock().unwrap().requests.get(&*self_).and_then(|r| r.path_with_query.clone())
    }
    fn set_path_with_query(&mut self, self_: BorrowedResourceGuard<u32>, path: Option<String>) -> Result<(), ()> {
        let mut store = http_store().lock().unwrap();
        if let Some(req) = store.requests.get_mut(&*self_) { req.path_with_query = path; }
        Ok(())
    }
    fn scheme(&mut self, self_: BorrowedResourceGuard<u32>) -> Option<http_types::Scheme> {
        let store = http_store().lock().unwrap();
        store.requests.get(&*self_).map(|r| match r.scheme.as_str() {
            "http" => http_types::Scheme::HTTP,
            "https" => http_types::Scheme::HTTPS,
            s => http_types::Scheme::Other(s.to_string()),
        })
    }
    fn set_scheme(&mut self, self_: BorrowedResourceGuard<u32>, scheme: Option<http_types::Scheme>) -> Result<(), ()> {
        let s = match &scheme {
            Some(http_types::Scheme::HTTP) => "http",
            Some(http_types::Scheme::HTTPS) | None => "https",
            Some(http_types::Scheme::Other(s)) => s.as_str(),
        };
        let mut store = http_store().lock().unwrap();
        if let Some(req) = store.requests.get_mut(&*self_) { req.scheme = s.to_string(); }
        Ok(())
    }
    fn authority(&mut self, self_: BorrowedResourceGuard<u32>) -> Option<String> {
        http_store().lock().unwrap().requests.get(&*self_).and_then(|r| r.authority.clone())
    }
    fn set_authority(&mut self, self_: BorrowedResourceGuard<u32>, authority: Option<String>) -> Result<(), ()> {
        let mut store = http_store().lock().unwrap();
        if let Some(req) = store.requests.get_mut(&*self_) { req.authority = authority; }
        Ok(())
    }
}

// -- RequestOptions --

impl http_types::RequestOptions for HostState {
    type T = u32;
    fn new(&mut self) -> u32 {
        let mut store = http_store().lock().unwrap();
        let h = store.alloc();
        store.request_options.insert(h, (None, None, None));
        h
    }
    fn connect_timeout(&mut self, self_: BorrowedResourceGuard<u32>) -> Option<u64> {
        http_store().lock().unwrap().request_options.get(&*self_).and_then(|o| o.0)
    }
    fn set_connect_timeout(&mut self, self_: BorrowedResourceGuard<u32>, duration: Option<u64>) -> Result<(), ()> {
        if let Some(o) = http_store().lock().unwrap().request_options.get_mut(&*self_) { o.0 = duration; }
        Ok(())
    }
    fn first_byte_timeout(&mut self, self_: BorrowedResourceGuard<u32>) -> Option<u64> {
        http_store().lock().unwrap().request_options.get(&*self_).and_then(|o| o.1)
    }
    fn set_first_byte_timeout(&mut self, self_: BorrowedResourceGuard<u32>, duration: Option<u64>) -> Result<(), ()> {
        if let Some(o) = http_store().lock().unwrap().request_options.get_mut(&*self_) { o.1 = duration; }
        Ok(())
    }
    fn between_bytes_timeout(&mut self, self_: BorrowedResourceGuard<u32>) -> Option<u64> {
        http_store().lock().unwrap().request_options.get(&*self_).and_then(|o| o.2)
    }
    fn set_between_bytes_timeout(&mut self, self_: BorrowedResourceGuard<u32>, duration: Option<u64>) -> Result<(), ()> {
        if let Some(o) = http_store().lock().unwrap().request_options.get_mut(&*self_) { o.2 = duration; }
        Ok(())
    }
}

// -- ResponseOutparam (stub — not needed for outgoing requests) --

impl http_types::ResponseOutparam<u32> for HostState {
    type T = u32;
    fn set(
        &mut self,
        _param: u32,
        _response: Result<u32, http_types::ErrorCode>,
    ) {}
}

// -- IncomingResponse --

impl http_types::IncomingResponse<u32, u32> for HostState {
    type T = u32;
    fn status(&mut self, self_: BorrowedResourceGuard<u32>) -> u16 {
        let store = http_store().lock().unwrap();
        store.responses.get(&*self_).map(|r| r.0).unwrap_or(0)
    }
    fn headers(&mut self, self_: BorrowedResourceGuard<u32>) -> u32 {
        let store = http_store().lock().unwrap();
        store.responses.get(&*self_).map(|r| r.1).unwrap_or(0)
    }
    fn consume(&mut self, self_: BorrowedResourceGuard<u32>) -> Result<u32, ()> {
        // Return a body handle that contains the response bytes as a readable stream
        let mut store = http_store().lock().unwrap();
        let resp_handle = *self_;
        let body_data = store.responses.get(&resp_handle).map(|r| r.2.clone()).unwrap_or_default();
        let body_h = store.alloc();
        // Store the body data as an "incoming body" keyed by body_h
        // We'll use the outgoing_bodies map for storage (reusing the buffer map)
        store.outgoing_bodies.insert(body_h, body_data);
        store.incoming_body_cursors.insert(body_h, 0);
        Ok(body_h)
    }
}

// -- IncomingBody --

impl http_types::IncomingBody<u32, u32> for HostState {
    type T = u32;
    fn stream(&mut self, self_: BorrowedResourceGuard<u32>) -> Result<u32, ()> {
        // Return a stream handle for reading the body.
        // We use the VirtualFs stream system for this.
        let body_h = *self_;
        let store = http_store().lock().unwrap();
        let data = store.outgoing_bodies.get(&body_h).cloned().unwrap_or_default();
        drop(store);
        // Create a read stream in the virtual FS
        let mut fs = self.fs.lock().unwrap();
        let stream_id = fs.create_http_read_stream(&data);
        Ok(stream_id)
    }
    fn finish(&mut self, _this: u32) -> u32 {
        // Return a future-trailers handle (stub — no trailers)
        let mut store = http_store().lock().unwrap();
        store.alloc()
    }
}

// -- FutureTrailers --

impl http_types::FutureTrailers<u32, u32> for HostState {
    type T = u32;
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0 // pollable that's always ready
    }
    fn get(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Option<Result<Result<Option<u32>, http_types::ErrorCode>, ()>> {
        Some(Ok(Ok(None))) // no trailers
    }
}

// -- OutgoingResponse (stub — not needed for outgoing requests) --

impl http_types::OutgoingResponse<u32, u32> for HostState {
    type T = u32;
    fn new(&mut self, _headers: u32) -> u32 { 0 }
    fn status_code(&mut self, _self_: BorrowedResourceGuard<u32>) -> u16 { 200 }
    fn set_status_code(&mut self, _self_: BorrowedResourceGuard<u32>, _status_code: u16) -> Result<(), ()> { Ok(()) }
    fn headers(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 { 0 }
    fn body(&mut self, _self_: BorrowedResourceGuard<u32>) -> Result<u32, ()> { Err(()) }
}

// -- OutgoingBody --

impl http_types::OutgoingBody<u32, u32> for HostState {
    type T = u32;
    fn write(&mut self, self_: BorrowedResourceGuard<u32>) -> Result<u32, ()> {
        // Return a writable stream handle
        let body_h = *self_;
        let mut fs = self.fs.lock().unwrap();
        let stream_id = fs.create_http_write_stream(body_h);
        Ok(stream_id)
    }
    fn finish(&mut self, _this: u32, _trailers: Option<u32>) -> Result<(), http_types::ErrorCode> {
        Ok(())
    }
}

// -- FutureIncomingResponse --

impl http_types::FutureIncomingResponse<u32, u32> for HostState {
    type T = u32;
    fn subscribe(&mut self, _self_: BorrowedResourceGuard<u32>) -> u32 {
        0 // pollable that's always ready (we do sync HTTP)
    }
    fn get(
        &mut self,
        self_: BorrowedResourceGuard<u32>,
    ) -> Option<Result<Result<u32, http_types::ErrorCode>, ()>> {
        let store = http_store().lock().unwrap();
        let future = store.future_responses.get(&*self_)?;
        match future {
            Some(Ok(resp_h)) => Some(Ok(Ok(*resp_h))),
            Some(Err(msg)) => Some(Ok(Err(http_types::ErrorCode::InternalError(Some(msg.clone()))))),
            None => None,
        }
    }
}

// -- HTTP Types namespace --

impl wasi::http::Types<u32, u32, u32, u32> for HostState {
    fn http_error_code(
        &mut self,
        _err: BorrowedResourceGuard<u32>,
    ) -> Option<http_types::ErrorCode> {
        Some(http_types::ErrorCode::InternalError(Some("error".to_string())))
    }
}

// -- OutgoingHandler — the actual HTTP implementation with domain filtering --

impl wasi::http::OutgoingHandler<http_types::ErrorCode, u32, u32, u32> for HostState {
    fn handle(
        &mut self,
        request: u32,
        _options: Option<u32>,
    ) -> Result<u32, http_types::ErrorCode> {
        let mut store = http_store().lock().unwrap();

        let req = store.requests.remove(&request)
            .ok_or(http_types::ErrorCode::InternalError(Some("unknown request handle".into())))?;

        let scheme_str = &req.scheme;
        let authority_str = req.authority.as_deref().unwrap_or("");
        let path = req.path_with_query.as_deref().unwrap_or("/");
        let path = if path.starts_with('/') { path.to_string() } else { format!("/{}", path) };
        let url = format!("{}://{}{}", scheme_str, authority_str, path);

        // Domain allowlist check
        let domain = authority_str.split(':').next().unwrap_or("");
        {
            let allowed = self.allowed_domains.lock().unwrap();
            if !allowed.contains(domain) {
                return Err(http_types::ErrorCode::HTTPRequestDenied);
            }
        }

        let reqwest_method = match req.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "HEAD" => reqwest::Method::HEAD,
            "OPTIONS" => reqwest::Method::OPTIONS,
            "PATCH" => reqwest::Method::PATCH,
            "CONNECT" => reqwest::Method::CONNECT,
            "TRACE" => reqwest::Method::TRACE,
            m => reqwest::Method::from_bytes(m.as_bytes())
                .map_err(|_| http_types::ErrorCode::HTTPRequestMethodInvalid)?,
        };

        let headers = store.fields.get(&req.headers_handle).cloned().unwrap_or_default();
        let body_bytes = req.body_handle.and_then(|bh| store.outgoing_bodies.get(&bh).cloned()).unwrap_or_default();

        // Allocate a future response handle
        let future_h = store.alloc();
        store.future_responses.insert(future_h, None);
        drop(store);

        // Perform the HTTP request synchronously
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| http_types::ErrorCode::InternalError(Some(e.to_string())))?;

        let mut builder = client.request(reqwest_method, &url);
        for (k, v) in &headers {
            builder = builder.header(k, v.as_slice());
        }
        if !body_bytes.is_empty() {
            builder = builder.body(body_bytes);
        }

        let result = builder.send();

        let mut store = http_store().lock().unwrap();
        match result {
            Ok(resp) => {
                let status = resp.status().as_u16();

                let resp_headers_h = store.alloc();
                let mut resp_headers = Vec::new();
                for (name, value) in resp.headers() {
                    resp_headers.push((name.to_string(), value.as_bytes().to_vec()));
                }
                store.fields.insert(resp_headers_h, resp_headers);

                let body_bytes = resp.bytes()
                    .map(|b| b.to_vec())
                    .unwrap_or_default();

                let resp_h = store.alloc();
                store.responses.insert(resp_h, (status, resp_headers_h, body_bytes));
                store.future_responses.insert(future_h, Some(Ok(resp_h)));
            }
            Err(e) => {
                store.future_responses.insert(future_h, Some(Err(e.to_string())));
            }
        }

        Ok(future_h)
    }
}
