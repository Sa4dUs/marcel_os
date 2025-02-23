/// Enum representing the different log message types used for logging kernel events.
/// Each variant corresponds to a specific level or type of message.
///
/// - `Info`: A general informational message.
/// - `Success`: A message indicating successful completion of an operation.
/// - `Failed`: A message indicating a failure or error in an operation.
/// - `Warning`: A message indicating a potential issue or something that requires attention.
pub enum LogType {
    /// Represents an informational log message.
    Info,

    /// Represents a successful operation log message.
    Success,

    /// Represents a failure or error log message.
    Failed,

    /// Represents a warning log message.
    Warning,
}
