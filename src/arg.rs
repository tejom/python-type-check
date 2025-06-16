use clap::{Arg, ArgAction, ArgMatches, command};

pub fn get_args() -> ArgMatches {
    command!()
        .arg(Arg::new("file_name"))
        .arg(
            Arg::new("pretty-print")
                .short('p')
                .long("pretty-print")
                .help("Pretty print the ast")
                .action(ArgAction::SetTrue),
        )
        .get_matches()
}
