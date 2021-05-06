mod cmd;

use cmd::Cmd;
use pretty_bytes::converter::convert;

fn main() {
    let opt = Cmd::from_args();
    let total = opt.calculate();
    if opt.bytes {
        println!("{}", total);
        return;
    }
    println!("{}", convert(total as f64));
}
