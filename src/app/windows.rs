use std::path::Path;

use globset::GlobMatcher;

use super::*;
use crate::job::Job;

fn new_walker(p: &Path, follow_links: bool, hidden: bool, ignore: bool) -> WalkParallel {
	WalkBuilder::new(p)
		.standard_filters(false)
		.follow_links(follow_links)
		.ignore(ignore)
		.hidden(hidden)
		.build_parallel()
}

fn spawn_job_glob(
	walker: WalkParallel,
	glob: GlobMatcher,
	tx: Sender<PathBuf>,
	hidden: bool,
	follow_links: bool,
	quiet: bool,
	ignore: bool,
) {
	thread::spawn(move || {
		walker.run(move || {
			let tx = tx.clone();
			let glob = glob.clone();
			// Outer
			Box::new(move |p| {
				let p = match p {
					Ok(p) if p.path().ne(Path::new(".")) && glob.is_match(p.path()) => p,
					Err(e) if !quiet => {
						eprintln!("error: {}", e);
						return Continue;
					}
					Err(_) | Ok(_) => return Continue,
				};

				let tx = tx.clone();
				// Inner
				new_walker(p.path(), follow_links, hidden, ignore).run(move || {
					let tx = tx.clone();
					Box::new(move |entry| {
						match entry {
							Ok(e) => tx.send(e.into_path()).unwrap(),
							Err(_) if quiet => (),
							Err(e) => eprintln!("error: {}", e),
						};
						Continue
					})
				});
				Continue
			})
		});
	});
}

impl Cmd {
	pub fn calculate(&self) -> u64 {
		let (tx, rx) = mpsc::channel();

		if self.files.is_empty() {
			let walker = new_walker(
				".".as_ref(),
				self.follow_links,
				self.ignore_hidden,
				self.ignore,
			);
			spawn_job(walker, tx.clone(), self.quiet)
		} else {
			let mut pwd_files = Vec::new();
			for s in &self.files {
				match Job::new(s) {
					Job::Plain(f) => pwd_files.push(f),
					Job::Walk {
						mut walker,
						matcher,
					} => {
						let walker = walker
							.standard_filters(false)
							.follow_links(self.follow_links)
							.hidden(self.ignore_hidden)
							.ignore(self.ignore)
							.build_parallel();
						spawn_job_glob(
							walker,
							matcher,
							tx.clone(),
							self.follow_links,
							self.ignore_hidden,
							self.quiet,
							self.ignore,
						);
					}
				};
			}

			if !pwd_files.is_empty() {
				let mut walker = WalkBuilder::new(&pwd_files[0]);
				for p in &pwd_files[1..] {
					walker.add(p);
				}
				let walker = walker
					.standard_filters(false)
					.follow_links(self.follow_links)
					.ignore(self.ignore)
					.hidden(self.ignore_hidden)
					.build_parallel();

				spawn_job(walker, tx.clone(), self.quiet);
			}
		}

		drop(tx);

		rx.into_iter()
			.par_bridge()
			.filter_map(move |p| match p.metadata() {
				Ok(md) => Some(md.len()),
				Err(_) if self.quiet => None,
				Err(e) => {
					eprintln!("error: {}", e);
					None
				}
			})
			.sum()
	}
}
