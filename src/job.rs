use std::path::{
	Component,
	Path,
	PathBuf,
};

use globset::{
	GlobBuilder,
	GlobMatcher,
};
use ignore::{
	overrides::OverrideBuilder,
	WalkBuilder,
};

#[allow(clippy::large_enum_variant)]
pub enum Job {
	Plain(PathBuf),
	Walk {
		walker: WalkBuilder,
		matcher: GlobMatcher,
	},
}

fn walker<P: AsRef<Path>>(root: P, glob: &str, depth: usize) -> WalkBuilder {
	let mut ov = OverrideBuilder::new(root.as_ref());
	let _ = ov.case_insensitive(true);

	if let Err(e) = ov.add(glob) {
		eprintln!("pattern error: {}", e);
		std::process::exit(2);
	};

	let ov = ov.build().unwrap();

	let mut walk = WalkBuilder::new(root.as_ref());

	walk.overrides(ov)
		.max_depth(if depth == 0 { None } else { Some(depth) });

	walk
}

fn is_glob(s: &str) -> bool {
	s.chars().any(|c| c == '*' || c == '?' || c == '[')
}

impl Job {
	pub fn new(s: &str) -> Self {
		if !is_glob(s) {
			return Self::Plain(PathBuf::from(s));
		}

		let p = PathBuf::from(s);
		let comps = p.components();
		let mut comps = comps.collect::<Vec<_>>();
		if matches!(&comps[0], Component::Normal(_)) {
			comps.insert(0, Component::CurDir);
		}

		// Find the index of the first glob.
		let idx = comps
			.iter()
			.position(|c| c.as_os_str().to_str().map_or(false, is_glob))
			.expect("internal logic error: expected a glob but there was none");

		let glob = comps[idx..].iter().collect::<PathBuf>();
		let first = glob.components().next();
		let glob = glob.as_os_str().to_str().unwrap();
		let glob_string = if matches!(first, Some(Component::Normal(_))) {
			format!(".\\{}", &glob)
		} else {
			glob.to_string()
		};
		let matcher = GlobBuilder::new(&glob_string)
			.case_insensitive(true)
			.literal_separator(true)
			.build()
			.unwrap_or_else(|e| {
				eprintln!("pattern error: {}", e);
				std::process::exit(2);
			})
			.compile_matcher();

		let walker = if glob.contains("**") {
			// No depth limit.
			if idx == 0 {
				walker("./", glob, 0)
			} else {
				let root = comps[..idx].iter().collect::<PathBuf>();
				walker(&root, glob, 0)
			}
		} else {
			let depth = comps.len() - idx;
			let mut root = comps[..idx].iter().collect::<PathBuf>();
			if idx == 0 {
				root.push("./");
			}
			walker(&root, glob, depth)
		};

		Self::Walk { matcher, walker }
	}
}
