use std::env::Args;

/// An abstract type representing a command-line flag supplied at runtime.
pub enum Flag {
	/// The path to a configuration file.
	ConfigPath(String)
}

macro_rules! error {
	($fmt:expr, $($arg:tt)*) => ({
		Err(Some(format!($fmt, $($arg),*)))
	});
	() => (Err(None));
}

impl Flag {
	/// Converts the given arguments to an array of parsed flags, aborting with a help message if the conversion fails (due to an unknown flag, probably).
	///
	/// # Panics
	/// This method will panic if `args` contains an unrecognized argument outside of index 0.
	pub fn from(args: Args) -> Result<Vec<Self>, Option<String>> {
		let mut flags = vec!();
		let mut skip_next = 0;
		let args = args.skip(1).enumerate().collect::<Vec<_>>();
		for (index, value) in args.clone() { // TODO: Do we need this clone?
			if skip_next > 0 {
				skip_next -= 1;
				continue;
			}
			let flag = match value.as_str() {
				"-c" | "--c" | "--config-path" => {
					skip_next = 1;
					if let Some(path) = args.clone().get(index + 1) {
						Ok(Flag::ConfigPath(path.1.clone()))
					} else {
						error!("{} argument was given with no path.", value)
					}
				},
				"-h" | "--h" | "--help" | "help" => error!(),
				_ => {
					error!("Unrecognized argument {}", value)
				}
			};
			match flag {
				Ok(value) => flags.push(value),
				Err(_) => return flag.map(|v| vec![v]) // This map closure will never be called, but that's okay
			};
		}
		Ok(flags)
	}
}
