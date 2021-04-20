use pretty_bytes::converter::convert;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;
use walkdir::WalkDir;

/// calc_dir calculates the total size of every file under a directory, recursively.
fn calc_dir(dir: &str) -> u64 {
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

/// calc_size calculates the size of either a directory or a plain file.
fn calc_size(name: &str) -> u64 {
    match fs::metadata(name) {
        Ok(m) => {
            if m.is_dir() {
                calc_dir(name)
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

fn show_help() {
    let cmd = match env::current_exe() {
        Ok(p) => match p.file_name() {
            Some(s) => s.to_str().unwrap_or("rs-size").to_owned(),
            None => String::from("rs-size"),
        },
        Err(_) => String::from("rs-size"),
    };
    eprintln!(
        "{}, calculate file sizes
usage:
	{} [options] [file...]
options:
	-b, --bytes: do not pretty print
	-h, --help: show this message and exit",
        cmd, cmd
    );
    process::exit(0);
}

struct CmdArgs {
    flag_help: bool,
    flag_bytes: bool,
    files: Vec<String>,
}

impl CmdArgs {
    pub fn parse(args: Vec<String>) -> Self {
        if args.len() == 0 {
            return CmdArgs {
                flag_help: false,
                flag_bytes: false,
                files: vec![],
            };
        }
        let mut flag_bytes = false;
        let mut files: Vec<String> = vec![];
        for a in &args {
            match &a[..] {
                "-h" | "--help" => {
                    return CmdArgs {
                        flag_help: true,
                        flag_bytes: false,
                        files,
                    }
                }
                "-b" | "--bytes" | "--byte" => flag_bytes = true,
                _ => files.push(a.to_string()),
            };
        }

        CmdArgs {
            flag_help: false,
            flag_bytes: flag_bytes,
            files: files,
        }
    }

    // windows powershell/cmd.exe does not have glob expansion, so we provide one.
    #[cfg(windows)]
    pub fn calculate(&self) -> u64 {
        use glob::glob;
        use itertools::Itertools;

        if self.files.len() == 0 {
            return calc_dir("./");
        }

        let mut files: Vec<String> = vec![];

        for s in &self.files {
            if s.contains("*") || s.contains("[") || s.contains("?") {
                for result in glob(&s[..])
                    .unwrap_or_else(|e| {
                        eprintln!("failed to read the glob pattern: {:?}", e);
                        process::exit(1);
                    })
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap().into_os_string().into_string())
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap())
                {
                    files.push(result)
                }
                continue;
            }
            files.push(s.to_string());
        }
        if files.len() == 0 {
            return 0;
        }

        files.iter().unique().map(|x| calc_size(&x[..])).sum()
    }

    #[cfg(not(windows))]
    pub fn calculate(&self) -> u64 {
        if self.files.len() == 0 {
            return calc_dir("./");
        }
        self.files.iter().map(|x| calc_size(&x[..])).sum()
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let opt = if args.len() < 1 {
        CmdArgs::parse(args)
    } else {
        CmdArgs::parse(args[1..].to_vec())
    };

    if opt.flag_help {
        show_help();
        return;
    }
    let total = opt.calculate();
    if opt.flag_bytes {
        println!("{}", total);
        return;
    }
    println!("{}", convert(total as f64));
}
