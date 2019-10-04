use core::fmt::Write;

use ignore_result::Ignore;

use crate::uart::Command;
use crate::uart::Uart;

use crate::path::PathConfig;

#[derive(Debug)]
pub struct BotConfig {
    pub path: PathConfig,
}

impl Command for BotConfig {
    fn keyword_command(&self) -> &str {
        "config"
    }

    fn handle_command<'a, I: Iterator<Item = &'a str>>(
        &mut self,
        uart: &mut Uart,
        mut args: I,
    ) {
        match args.next() {
            Some(_) => writeln!(uart, "config: unknown key").ignore(),
            None => writeln!(uart, "{:#?}", &self).ignore(),
        }
    }
}
