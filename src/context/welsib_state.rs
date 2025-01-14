#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WelsibState {
    AwaitBegin,
    AwaitUpdateExecutorStatus,
    AwaitReadRequest,
    AwaitReadFile,
    AwaitRead404File,
    AwaitRead400File,
    AwaitRead500File,
    AwaitExecutor,
    AwaitPaymentNotification,
    AwaitUpgrade,
    AwaitWriteResponse,
    AwaitWrite101Response,
    AwaitHandshake,
    AwaitInitiator,
    Done,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RequestType {
    StaticFile, // GET
    Api,        // POST
    NotFound,
    Activator, // ACTIVATION
}
