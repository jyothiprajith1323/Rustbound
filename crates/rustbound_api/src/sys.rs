//! Raw foreign function imports declared by the host runtime environment.

unsafe extern "C" {
    /// Queries machine status into guest memory buffer.
    /// Returns number of bytes written, or negative error code.
    pub fn host_query_telemetry(machine_id: u32, out_ptr: *mut u8, max_len: u32) -> i32;

    /// Submits a binary command packet from guest memory buffer.
    /// Returns 0 on success, or negative error code.
    pub fn host_submit_command(machine_id: u32, cmd_ptr: *const u8, cmd_len: u32) -> i32;

    /// Voluntarily relinquishes remaining execution fuel for the current simulation tick.
    pub fn host_yield_tick();

    /// Emits a debug log message to the in-game machine terminal.
    pub fn host_log(level: u32, msg_ptr: *const u8, msg_len: u32);
}
