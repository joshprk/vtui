pub enum Command {
    Shutdown,
}

impl Command {
    pub fn reduce(self, ctx: &mut Context) {
        match self {
            Self::Shutdown => ctx.shutdown_requested = true,
        }
    }
}

#[derive(Default)]
pub struct Context {
    pub(crate) shutdown_requested: bool,

    enqueued: Vec<Command>,
}

impl Context {
    pub fn commit(&mut self) {
        for cmd in core::mem::take(&mut self.enqueued) {
            cmd.reduce(self);
        }
    }

    pub fn enqueue(&mut self, cmd: Command) {
        self.enqueued.push(cmd);
    }
}
