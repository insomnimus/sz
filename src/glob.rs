struct Compiler(Peekable<Char>);

pub fn compile(s: &str) -> Result<Regex> {
	Compiler(s.chars().copied().peekable()).compile()
}

impl Compiler {
	fn compile(mut self) -> Result<Regex> {
		
	}
}