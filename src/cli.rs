use anyhow::Result;
use clap::{crate_name, CommandFactory};
use clap_complete::{generate, Shell};

use clap::Parser;

use crate::{config::RummageGlobal, matches::find_workspaces_in_dir};

#[derive(Parser, Default, Debug)]
#[clap(author = "Mitchell Hanberg", version)]
/// rummage is a cli tool for finding directories on the file system that match certain criteria.
///
/// Workspaces are defined as a directory matching any workspace pattern from your configuration. If no configuration is set, any directory containing a `.git` file/folder or a `.rummage.yaml` file is considered a workspace.
pub struct Arguments {
    #[clap(long)]
    /// Print bash completions to stdout
    pub print_bash_completion: bool,

    #[clap(long)]
    /// Print zsh completions to stdout
    pub print_zsh_completion: bool,

    #[clap(long)]
    /// Print fish completions to stdout
    pub print_fish_completion: bool,

    #[clap(long)]
    /// Print man(1) page to stdout
    pub print_man: bool,
}

fn print_completion(shell: Shell) -> Result<()> {
    let mut cmd = Arguments::command();
    generate(shell, &mut cmd, crate_name!(), &mut std::io::stdout());
    Ok(())
}

/// Parses the command line arguments and runs the program. Called from `main.rs`.
/// Since not every command needs a TUI, we start one up as necessary in each handler that needs one.
pub fn parse() -> Result<()> {
    let args = Arguments::parse();

    match args {
        Arguments {
            print_bash_completion: true,
            ..
        } => print_completion(Shell::Bash),
        Arguments {
            print_zsh_completion: true,
            ..
        } => print_completion(Shell::Zsh),
        Arguments {
            print_fish_completion: true,
            ..
        } => print_completion(Shell::Fish),
        Arguments {
            print_man: true, ..
        } => {
            let cmd = Arguments::command();
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut std::io::stdout())?;
            Ok(())
        }
        _ => search(),
    }
}

fn search() -> std::result::Result<(), anyhow::Error> {
    let config = RummageGlobal::load()?;

    for dir in &config.search_paths {
        find_workspaces_in_dir(dir, &config);
    }

    Ok(())
}
