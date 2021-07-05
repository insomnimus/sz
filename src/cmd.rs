use clap::{crate_version, App, AppSettings, Arg};
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};
use walkdir::WalkDir;
pub struct Cmd {
    ignore_hidden: bool,
    follow_links: bool,
    pub bytes: bool,
    files: Vec<String>,
}

impl Cmd {
    pub fn app() -> App<'static> {
        let app = App::new("sz")
            .about("calculates file sizes")
            .version(crate_version!())
            .setting(AppSettings::UnifiedHelpMessage);

        let bytes = Arg::new("bytes")
            .short('b')
            .long("bytes")
            .about("do not pretty print, only display total byte value");

        let ignore_hidden = Arg::new("ignore-hidden")
            .short('i')
            .long("ignore-hidden")
            .about("ignore hidden files and directories");

        let file = Arg::new("file").multiple(true);

        let follow_links = Arg::new("follow-links")
            .short('f')
            .long("follow-links")
            .about("follow links");

        app.arg(follow_links)
            .arg(bytes)
            .arg(ignore_hidden)
            .arg(file)
    }

    pub fn from_args() -> Self {
        let matches = Self::app().get_matches();

        let follow_links = matches.is_present("follow-links");
        let bytes = matches.is_present("bytes");
        let ignore_hidden = matches.is_present("ignore-hidden");

        let files: Vec<String> = match matches.values_of("file") {
            Some(fs) => fs.map(|s| s.to_string()).collect(),
            None => vec![],
        };
        Self {
            ignore_hidden,
            follow_links,
            bytes,
            files,
        }
    }

    // windows powershell/cmd.exe does not have glob expansion, so we provide one.
    #[cfg(windows)]
    pub fn calculate(&self) -> u64 {
        use glob::{glob_with, MatchOptions};

        if self.files.is_empty() {
            return self.calc_dir("./");
        }

        let opt = MatchOptions {
            case_sensitive: false,
            require_literal_separator: true,
            require_literal_leading_dot: self.ignore_hidden,
        };

        self.files
            .iter()
            .map(|s| {
                if is_glob(s) {
                    glob_with(s, opt)
                        .unwrap_or_else(|e| {
                            eprintln!("failed to read the glob pattern: {:?}", e);
                            process::exit(1);
                        })
                        .filter_map(Result::ok)
                        .collect::<Vec<_>>()
                } else {
                    vec![PathBuf::from(s)]
                }
            })
            .flatten()
            .map(|p| self.calc_size(&p))
            .sum()
    }

    #[cfg(not(windows))]
    pub fn calculate(&self) -> u64 {
        if self.files.is_empty() {
            return self.calc_dir("./");
        }
        self.files.iter().map(|x| self.calc_size(x)).sum()
    }

    fn calc_dir(&self, name: impl AsRef<Path>) -> u64 {
        WalkDir::new(name)
            .follow_links(self.follow_links)
            .into_iter()
            .filter_entry(|e| !self.ignore_hidden || !is_hidden(&e.file_name()))
            .filter_map(|r| {
                r.map_err(|e| {
                    eprintln!(
                        "error accessing {}: {}",
                        e.path().unwrap_or_else(|| Path::new("")).display(),
                        &e
                    );
                })
                .ok()
            })
            .filter_map(|x| {
                x.metadata()
                    .map_err(|e| {
                        eprintln!("error reading metadata of {}: {}", x.path().display(), &e);
                    })
                    .map(|md| md.len())
                    .ok()
            })
            .sum()
    }

    fn calc_size(&self, name: impl AsRef<Path>) -> u64 {
        if self.follow_links {
            fs::metadata(name.as_ref())
        } else {
            fs::symlink_metadata(name.as_ref())
        }
        .map(|md| {
            if md.is_dir() {
                self.calc_dir(name.as_ref())
            } else {
                md.len()
            }
        })
        .unwrap_or_else(|e| {
            eprintln!("error accessing {}: {}", name.as_ref().display(), &e);
            process::exit(2);
        })
    }
}

fn is_hidden(p: impl AsRef<Path>) -> bool {
    p.as_ref().file_name().map_or(false, |s| {
        s.to_str().map(|s| s.starts_with('.')).unwrap_or(false)
    })
}

fn is_glob(s: &str) -> bool {
    s.contains('*') || s.contains('?') || (s.contains('[') && s.contains(']'))
}
