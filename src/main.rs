#[macro_use]
extern crate clap;
use clap::AppSettings as settings;

extern crate deoxy;

fn main() {
	let matches = clap_app!(deoxy =>
		(version: "0.2.0")
		(author: "Alex Hamilton <alex.hamilton@ou.edu>")
		(about: "For all your buffer exchange needs!")
		(setting: settings::SubcommandRequired)
		(@arg CONFIG: -c --config +takes_value "Sets a custom config file")
		(@subcommand run =>
			// (about: "starts the deoxy daemon, or aborts if another instance is detected")
			(about: "runs deoxy::main()")
			)
		).get_matches();

	// let config = matches.value_of("config").unwrap_or("default.conf");

	match matches.subcommand_name() {
		Some("run") => deoxy::main(),
		None => {},
		_ => unreachable!()
	}
}
