use serde::Serialize;

#[derive(Serialize)]
pub struct ServerStatus {
    pub workers: Vec<WorkerScore>,
    // TODO Other info
}

/// Analog to the 'worker_score' struct:
/// https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L88
/// 
/// Added suffices for time units because they're often converted from microseconds to something else by mod_status.
#[derive(Debug, Serialize)]
pub struct WorkerScore {
    // `tid` and `thread_num` not available through mod_status

    /// Not available if state = "dead"
    pub pid: Option<i32>,

    pub generation: i32,

    pub status: WorkerStatus,

    pub access_counts: AccessCounts,

    /// `conn_bytes` converted to KiB by mod_status
    /// 
    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L920
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L104
    pub conn_kib: f32,

    
    /// `bytes_served` converted to MiB by mod_status
    /// 
    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L920
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L106
    pub child_mib: f32,
    
    /// `my_bytes_served` converted to MiB by mod_status.
    /// 
    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L920
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L108
    pub slot_mib: f32,

    /// Replacing `stop_time` and `start_time` in mod_status
    ///
    /// `req_time` = `stop_time` - `start_time`
    /// 
    /// See:
    /// https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L112
    /// https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L741
    pub request_time_ms: u32,

    /// Derived from `last_used` by mod_status
    /// 
    /// `seconds_since` = now - `last_used`
    /// 
    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L111
    /// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L916
    pub seconds_since_s: u32,
    
    /// Derived from `times` by mod_status
    /// 
    /// Note: needs `define HAVE_TIMES`.
    /// 
    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L113
    /// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L904
    pub cpu: f32,

    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L116
    pub request: String,

    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L117
    pub vhost: String,

    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L118
    pub protocol: String,

    /// `duration` converted to milliseconds by mod_status
    /// 
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L119
    /// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L918
    pub duration_ms: u32,

    /// From `client64`
    /// 
    /// Note: the `client` field is deprecated, hence using `client64`.
    /// 
    /// See:
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L115
    /// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L120
    pub client: String,
}

/// These are named differently in `scoreboard.h` and `mod_status.c`. These are in order in which they appear in the mod_status "Acc" column:
/// 1. `conn_count` is called `conn_lres` in `mod_status.c`,
/// 2. `my_lres` is called `my_access_count` in `mod_status.c`,
/// 3. `lres` is called `access_count` in `mod_status.c`.
/// 
/// See:
/// - https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L90
/// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L747
/// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L854
#[derive(Debug, Serialize)]
pub struct AccessCounts {
    /// aka `conn_count`, see the `AccessCounts` doc.
    pub connection: u32,
    /// aka `my_lres`, see the `AccessCounts` doc.
    pub child: u32,
    /// aka `lres`, see the `AccessCounts` doc.
    pub slot: u32,
}

/// Analogous to constants from:
/// https://github.com/apache/httpd/blob/2.4.56/include/scoreboard.h#L56
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum WorkerStatus {
    Dead,           // SERVER_DEAD
    Starting,       // SERVER_STARTING        Server Starting up
    Ready,          // SERVER_READY           Waiting for connection (or accept() lock)
    BusyRead,       // SERVER_BUSY_READ       Reading a client request
    BusyWrite,      // SERVER_BUSY_WRITE      Processing a client request
    BusyKeepAlive,  // SERVER_BUSY_KEEPALIVE  Waiting for more requests via keepalive
    BusyLog,        // SERVER_BUSY_LOG        Logging the request
    BusyDns,        // SERVER_BUSY_DNS        Looking up a hostname
    Closing,        // SERVER_CLOSING         Closing the connection
    Graceful,       // SERVER_GRACEFUL        server is gracefully finishing request
    IdleKill,       // SERVER_IDLE_KILL       Server is cleaning up idle children.
}
