use pretty_bytes::converter::convert;
use std::env;
use std::fs;
use std::io::ErrorKind;
use walkdir::WalkDir;

fn calc_dir(dir: &str) -> u64 {
	let mut sum: u64 = 0;
	for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok())
.filter(|x| !x.file_type().is_dir()){
		sum += match entry.metadata() {
			Err(_) => 0,
			Ok(m) => m.len(),
		};
	}
	sum
}

fn calc_size(name: &str)-> u64 {
	let md = match fs::metadata(name){
		Ok(m) => m,
		Err(err) => match err.kind() {
			ErrorKind::NotFound => {
				eprintln!("{}: does not exist", name);
				std::process::exit(1);
			}
			_ => return 0,
		},
	};
	if md.is_dir() {
		return calc_dir(name);
	}
	md.len()
}

fn show_help() {
	let cmd = match env::current_exe() {
		Ok(p) => match p.file_name() {
			Some(s) => s.to_str().unwrap().to_owned(),
			None => String::from("which"),
		},
		Err(_) => String::from("which"),
	};
	eprintln!(
		"{}, calculate file sizes
usage:
{} [file...]",
		cmd, cmd
	);
	std::process::exit(0);
}

struct CmdArgs{
	flag_help: bool,
	flag_bytes: bool,
	files: Vec<String>,
}

impl CmdArgs{
	pub fn parse(args: Vec<String>) -> Self{
		if args.len() == 0{
			return CmdArgs{
				flag_help: false,
				flag_bytes: false,
				files: vec![String::from("./")],
			};
		}
		let mut flag_bytes= false;
		let mut files: Vec<String>= vec![];
		for a in &args{
			match &a[..]{
				"-h"|"--help"=> return CmdArgs{
					flag_help: true, flag_bytes: false, files,
				},
				"-b"|"--bytes"|"--byte"=> flag_bytes= true,
				_=> files.push(a.to_string()),
			};
		}
		
		if files.len()== 0{
			files.push(String::from("./"));
		}
		
		CmdArgs{
			flag_help: false,
			flag_bytes: flag_bytes,
			files: files,
		}
	}
	
	#[cfg(windows)]
	pub fn calculate(&self) -> u64{
		use glob::glob;
		use itertools::Itertools;
		let mut files: Vec<String>= vec![];
		for s in &self.files{
			if s.contains("*") || s.contains("[") || s.contains("?") {
				for result in glob(&s[..])
				.expect("failed to read the glob pattern")
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
		if files.len()== 0{
			return 0;
		}
		
		files
		.iter()
		.unique()
		.map(|x| calc_size(&x[..]))
		.sum()
	}
	
	#[cfg(not(windows))]
	pub fn calculate(&self) -> u64{
		if self.files.len()==0{
			return 0;
		}
		self.files.iter()
		.map(|x| calc_size(&x[..]))
		.sum()
	}
}

fn main() {
	let opt= CmdArgs::parse(env::args().collect());
	if opt.flag_help{
		show_help();
		return;
	}
	let total= opt.calculate();
	if opt.flag_bytes{
		println!("{}", total);
		return;
	}
	println!("{}", convert(total as f64));
}
