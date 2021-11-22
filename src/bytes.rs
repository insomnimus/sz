use core::fmt;

type Unit = u64;

pub const KB: Unit = 1000;
pub const KIB: Unit = 1024;
pub const MB: Unit = 1000000;
pub const MIB: Unit = 1024 * 1024;
pub const GB: Unit = 1000000000;
pub const GIB: Unit = 1024 * 1024 * 1024;
pub const TB: Unit = 1000000000000;
pub const TIB: Unit = 1024 * 1024 * 1024 * 1024;

#[inline]
fn unit_str(n: Unit) -> &'static str {
	match n {
		KB => "KB",
		KIB => "KiB",
		MB => "MB",
		MIB => "MiB",
		GB => "GB",
		GIB => "GiB",
		TB => "TB",
		TIB => "TiB",
		_ => unreachable!(),
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Bytes(pub u64);

impl fmt::Display for Bytes {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let n = self.0;

		const UNITS: &[(Unit, Unit)] = &[(KB, KIB), (MB, MIB), (GB, GIB), (TB, TIB)];

		for u in UNITS.iter().rev() {
			let u = if f.alternate() { u.0 } else { u.1 };
			if n >= u {
				let val = n as f64 / u as f64;
				return if let Some(precision) = f.precision() {
					write!(
						f,
						"{val:.precision$}{unit}",
						val = val,
						precision = precision,
						unit = unit_str(u)
					)
				} else {
					write!(f, "{}{}", val, unit_str(u))
				};
			}
		}

		write!(f, "{}b", n)
	}
}
