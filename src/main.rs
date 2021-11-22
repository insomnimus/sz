mod app;
mod bytes;
#[cfg(windows)]
mod job;

use app::Cmd;
use bytes::Bytes;

fn main() {
	let opt = Cmd::from_args();
	let total = opt.calculate();
	if opt.bytes {
		println!("{}", total);
	} else {
		println!("{:.2}", Bytes(total));
	}
}
