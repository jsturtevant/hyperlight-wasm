//! Simple synchronous WASI implementations for HostState.
//!
//! All resource handle types are `u32`. Most operations are stubs that return
//! errors or no-ops — the guest Python captures stdout/stderr internally
//! and returns them through the WIT export, so WASI I/O is not critical.
#![allow(unused_variables)]

use crate::bindings::wasi;
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
        Err(streams::StreamError::Closed)
    }
    fn blocking_read(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _len: u64,
    ) -> Result<Vec<u8>, streams::StreamError<u32>> {
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
        Ok(())
    }
    fn blocking_write_and_flush(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _contents: Vec<u8>,
    ) -> Result<(), streams::StreamError<u32>> {
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
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn write_via_stream(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _offset: fs_types::Filesize,
    ) -> Result<u32, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn append_via_stream(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u32, fs_types::ErrorCode> {
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
        Ok(fs_types::DescriptorType::Directory)
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
        Err(fs_types::ErrorCode::NoEntry)
    }
    fn write(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _buffer: Vec<u8>,
        _offset: fs_types::Filesize,
    ) -> Result<fs_types::Filesize, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn read_directory(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
    ) -> Result<u32, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::NoEntry)
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
        Err(fs_types::ErrorCode::Unsupported)
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
        Err(fs_types::ErrorCode::Unsupported)
    }
    fn stat_at(
        &mut self,
        _self_: BorrowedResourceGuard<u32>,
        _path_flags: fs_types::PathFlags,
        _path: String,
    ) -> Result<fs_types::DescriptorStat<wall_clock::Datetime>, fs_types::ErrorCode> {
        Err(fs_types::ErrorCode::Unsupported)
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
        Err(fs_types::ErrorCode::Unsupported)
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
        Vec::new()
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
