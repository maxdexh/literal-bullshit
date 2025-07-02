pub type CommandError = anyhow::Error;
pub type Result<T = String, E = CommandError> = std::result::Result<T, E>;

mod commands;

use anyhow::bail;

use crate::model::Model;

pub struct CommandResult {
    pub command_output: String,
    pub is_error: bool,
    pub is_quitting: bool,
}

pub struct CommandHandler {
    model: Model,
}

const QUIT_CMD: &str = "quit";

impl CommandHandler {
    pub fn new(model: Model) -> Self {
        Self { model }
    }
    pub fn handle_command(&mut self, command: String) -> CommandResult {
        match self.handle_command_split(command) {
            Ok((command_output, is_quitting)) => CommandResult {
                command_output,
                is_error: false,
                is_quitting,
            },
            Err(err) => CommandResult {
                command_output: format!("Error: {err}"),
                is_error: true,
                is_quitting: false,
            },
        }
    }
    fn handle_command_split(&mut self, command: String) -> Result<(String, bool)> {
        let mut args = command.split_whitespace();
        let Some(command) = args.next() else {
            bail!("Please enter a command")
        };
        self.handle_command_args(command, args.collect())
    }
    fn handle_command_args(&mut self, command: &str, args: Vec<&str>) -> Result<(String, bool)> {
        match command {
            QUIT_CMD => {
                if let Some(arg) = args.first() {
                    bail!("Unexpected argument {arg}")
                }
                Ok((Default::default(), true))
            }
            _ => self
                .handle_normal_command(command, args)
                .map(|message| (message, false)),
        }
    }
    fn handle_normal_command(&mut self, command: &str, args: Vec<&str>) -> Result {
        commands::handle(command, &mut self.model, args)
    }
}
