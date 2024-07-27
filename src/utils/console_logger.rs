use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

pub struct ConsoleLogger {
    stdout: std::io::Stdout,
}

impl ConsoleLogger {
    pub fn new() -> Self {
        let mut stdout = stdout();
        stdout.execute(cursor::Hide).unwrap();
        stdout.queue(cursor::SavePosition).unwrap();
        Self {
            stdout,
        }
    }

    pub fn fps(&mut self, fps: f32) {
        self.stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
        self.stdout.write_all(format!("FPS: {:.2}", fps).as_bytes()).unwrap();
        self.stdout.queue(cursor::RestorePosition).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn cleanup(&mut self) {
        self.stdout.execute(cursor::Show).unwrap();
    }
}