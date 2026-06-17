mod cli;
mod commands;
mod compiler;
mod config;
mod errors;
mod output;
mod platform;
mod template;

use clap::Parser;

use cli::{Cli, Commands};
use output::OutputConfig;

fn main() {
    let cli = Cli::parse();
    let output_config = OutputConfig::detect(cli.no_color);

    let exit_code = match cli.command {
        Commands::Doctor { fix } => commands::doctor::execute(fix),
        Commands::New { name, template } => commands::new::execute(&name, &template),
        Commands::Run { file } => {
            commands::run::execute(file.as_deref(), &output_config)
        }
        Commands::Build { release } => commands::build::execute(release, &output_config),
        Commands::Add { package } => commands::add::execute(&package, &output_config),
        Commands::Uninstall { force } => commands::uninstall::execute(force),
    };

    std::process::exit(exit_code);
}
