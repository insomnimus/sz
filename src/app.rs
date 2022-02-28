#[cfg(windows)]
mod windows;

use std::{
	path::PathBuf,
	sync::mpsc::{
		self,
		Sender,
	},
	thread,
};

use clap::Parser;
use ignore::{
	WalkBuilder,
	WalkParallel,
	WalkState::Continue,
};
use rayon::iter::{
	ParallelBridge,
	ParallelIterator,
};

/// Calculate file sizes.
#[derive(Parser)]
pub struct Cmd {
	/// Do not traverse hidden directories, do not count hidden files.
	#[clap(short = 'd', long)]
	ignore_hidden: bool,
	/// Read .ignore files and apply them.
	#[clap(short = 'i', long)]
	ignore: bool,
	/// Follow symbolic links.
	#[clap(short, long)]
	follow_links: bool,
	/// Do not display errors.
	#[clap(short, long)]
	quiet: bool,
	/// Do not pretty print, only display total number of bytes.
	#[clap(short, long)]
	pub bytes: bool,
	/// Any number of files or directories.
	files: Vec<String>,
}

fn spawn_job(walker: WalkParallel, tx: Sender<PathBuf>, quiet: bool) {
	thread::spawn(move || {
		walker.run(move || {
			let tx = tx.clone();
			Box::new(move |p| {
				match p {
					Ok(p) => tx.send(p.into_path()).unwrap(),
					Err(_) if quiet => (),
					Err(e) => {
						eprintln!("error: {}", e);
					}
				};
				Continue
			})
		});
	});
}

impl Cmd {
	pub fn from_args() -> Self {
		Self::parse()
	}

	#[cfg(not(windows))]
	pub fn calculate(&self) -> u64 {
		let (tx, rx) = mpsc::channel();

		let walker = if self.files.is_empty() {
			WalkBuilder::new(".")
		} else {
			let mut builder = WalkBuilder::new(&self.files[0]);
			for f in &self.files[1..] {
				builder.add(f);
			}
			builder
		}
		.standard_filters(false)
		.follow_links(self.follow_links)
		.ignore(self.ignore)
		.hidden(self.ignore_hidden)
		.build_parallel();

		spawn_job(walker, tx, self.quiet);

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
