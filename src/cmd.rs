use clap::{crate_version, App, Arg};
use std::io::ErrorKind;
use std::{fs, process};

use walkdir::WalkDir;

fn calc_dir(dir: &str, ignore_hidden: bool) -> u64 {
    if ignore_hidden && is_hidden(dir) {
        return 0;
    }
    if ignore_hidden {
        WalkDir::new(dir)
            .into_iter()
            .filter_entry(|x| {
                x.file_name()
                    .to_str()
                    .map(|s| !is_hidden(s))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
            .filter(|x| !x.file_type().is_dir())
            .fold(0u64, |sum, x| {
                sum + match x.metadata() {
                    Err(_) => 0,
                    Ok(m) => m.len(),
                }
            })
    } else {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|x| !x.file_type().is_dir())
            .fold(0u64, |sum, x| {
                sum + match x.metadata() {
                    Err(_) => 0,
                    Ok(m) => m.len(),
                }
            })
    }
}

fn calc_size(name: &str, ignore_hidden: bool) -> u64 {
    if ignore_hidden && is_hidden(name) {
        return 0;
    }
    match fs::metadata(name) {
        Ok(m) => {
            if m.is_dir() {
                calc_dir(name, ignore_hidden)
            } else {
                m.len()
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                eprintln!("{}: does not exist", name);
                process::exit(1);
            }
            _ => 0,
        },
    }
}

pub struct Cmd {
    ignore_hidden: bool,
    pub bytes: bool,
    files: Vec<String>,
}

impl Cmd {
    pub fn app() -> App<'static> {
        let app = App::new("sz")
            .about("calculates file sizes")
            .override_usage("sz [OPTIONS] [FILE...]")
            .version(crate_version!())
            .help_template(
                "\
sz, {about}
usage: {usage}

{all-args}\
	",
            );

        let bytes = Arg::new("bytes")
            .short('b')
            .long("bytes")
            .about("do not pretty print, only display total byte value");

        let ignore_hidden = Arg::new("ignore-hidden")
            .short('i')
            .long("ignore-hidden")
            .about("ignore hidden files and directories");

        let file = Arg::new("file").multiple(true);

        app.arg(bytes).arg(ignore_hidden).arg(file)
    }

    pub fn from_args() -> Self {
        let matches = Self::app().get_matches();

        let bytes = matches.is_present("bytes");
        let ignore_hidden = matches.is_present("ignore-hidden");

        let files: Vec<String> = match matches.values_of("file") {
            Some(fs) => fs.map(|s| s.to_string()).collect(),
            None => vec![],
        };
        Self {
            ignore_hidden,
            bytes,
            files,
        }
    }

    // windows powershell/cmd.exe does not have glob expansion, so we provide one.
    #[cfg(windows)]
    pub fn calculate(&self) -> u64 {
        use glob::{glob_with, MatchOptions};
        use itertools::Itertools;

        if self.files.is_empty() {
            return calc_dir("./", self.ignore_hidden);
        }

        let mut files: Vec<String> = vec![];
        let opt = MatchOptions {
            case_sensitive: false,
            require_literal_separator: true,
            require_literal_leading_dot: self.ignore_hidden,
        };

        for s in &self.files {
            if s.contains('*') || s.contains('[') || s.contains('?') {
                for result in glob_with(&s[..], opt)
                    .unwrap_or_else(|e| {
                        eprintln!("failed to read the glob pattern: {:?}", e);
                        process::exit(1);
                    })
                    .filter_map(|x| x.ok())
                    .map(|x| x.into_os_string().into_string())
                    .filter_map(|e| e.ok())
                {
                    files.push(result)
                }
                continue;
            }
            files.push(s.to_string());
        }

        if files.is_empty() {
            return 0;
        }

        files
            .iter()
            .unique()
            .map(|x| calc_size(&x[..], self.ignore_hidden))
            .sum()
    }

    #[cfg(not(windows))]
    pub fn calculate(&self) -> u64 {
        if self.files.is_empty() {
            return calc_dir("./", self.ignore_hidden);
        }
        self.files
            .iter()
            .map(|x| calc_size(&x[..], self.ignore_hidden))
            .sum()
    }
}

fn is_hidden(s: &str) -> bool {
    if s == "." || s == "./" || s == ".\\" {
        false
    } else {
        s.starts_with('.')
    }
}
