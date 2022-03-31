use crate::misc::chars_take;

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint(input: String) -> HintEnum<'static> {
	if input.is_empty() {
		return HintEnum::Single("x^2");
	}

	let chars: Vec<char> = input.chars().collect();

	let mut open_parens: usize = 0;
	let mut closed_parens: usize = 0;
	chars.iter().for_each(|chr| match *chr {
		'(' => open_parens += 1,
		')' => closed_parens += 1,
		_ => {}
	});

	if open_parens > closed_parens {
		return HintEnum::Single(")");
	}

	let len = chars.len();

	for i in (2..=5).rev().filter(|i| len >= *i) {
		if let Some(output) = get_completion(chars_take(&chars, i)) {
			return output;
		}
	}

	HintEnum::None
}

#[derive(Clone, PartialEq)]
pub enum HintEnum<'a> {
	Single(&'static str),
	Many(&'a [&'static str]),
	None,
}

impl std::fmt::Debug for HintEnum<'static> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

impl ToString for HintEnum<'static> {
	fn to_string(&self) -> String {
		match self {
			HintEnum::Single(single_data) => single_data.to_string(),
			HintEnum::Many(multi_data) => multi_data
				.iter()
				.map(|a| a.to_string())
				.collect::<String>()
				.to_string(),
			HintEnum::None => String::new(),
		}
	}
}

impl HintEnum<'static> {
	pub fn get_single(&self) -> Option<String> {
		match self {
			HintEnum::Single(x) => Some(x.to_string()),
			_ => None,
		}
	}

	pub fn is_multi(&self) -> bool {
		match self {
			HintEnum::Many(_) => true,
			_ => false,
		}
	}
}

// include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
static COMPLETION_HASHMAP: phf::Map<&'static str, HintEnum> = ::phf::Map {
	key: 2980949210194914378,
	disps: &[
		(0, 5),
		(0, 24),
		(1, 0),
		(3, 14),
		(51, 0),
		(0, 11),
		(2, 0),
		(0, 29),
		(3, 23),
		(23, 59),
		(0, 5),
		(0, 7),
		(39, 43),
	],
	entries: &[
		("co", HintEnum::Many(&["s(", "sh("])),
		("c", HintEnum::Many(&["os(", "osh(", "eil(", "brt("])),
		("frac", HintEnum::Single("t(")),
		("fl", HintEnum::Single("oor(")),
		("sq", HintEnum::Single("rt(")),
		("fr", HintEnum::Single("act(")),
		("sig", HintEnum::Single("num(")),
		("ac", HintEnum::Single("os(")),
		("signum", HintEnum::Single("(")),
		("ln", HintEnum::Single("(")),
		("aco", HintEnum::Single("s(")),
		("fra", HintEnum::Single("ct(")),
		("round", HintEnum::Single("(")),
		("t", HintEnum::Many(&["an(", "anh(", "runc("])),
		("s", HintEnum::Many(&["ignum(", "in(", "inh(", "qrt("])),
		("acos", HintEnum::Single("(")),
		("exp", HintEnum::Single("(")),
		("tanh", HintEnum::Single("(")),
		("lo", HintEnum::Many(&["g2(", "g10("])),
		("log10", HintEnum::Single("(")),
		("fract", HintEnum::Single("(")),
		("trun", HintEnum::Single("c(")),
		("log1", HintEnum::Single("0(")),
		("at", HintEnum::Single("an(")),
		("tr", HintEnum::Single("unc(")),
		("floor", HintEnum::Single("(")),
		("ab", HintEnum::Single("s(")),
		("si", HintEnum::Many(&["gnum(", "n(", "nh("])),
		("asi", HintEnum::Single("n(")),
		("sin", HintEnum::Many(&["(", "h("])),
		("e", HintEnum::Single("xp(")),
		("flo", HintEnum::Single("or(")),
		("ex", HintEnum::Single("p(")),
		("sqr", HintEnum::Single("t(")),
		("log2", HintEnum::Single("(")),
		("atan", HintEnum::Single("(")),
		("sinh", HintEnum::Single("(")),
		("tru", HintEnum::Single("nc(")),
		("cei", HintEnum::Single("l(")),
		("l", HintEnum::Many(&["n(", "og2(", "og10("])),
		("asin", HintEnum::Single("(")),
		("tan", HintEnum::Many(&["(", "h("])),
		("cos", HintEnum::Many(&["(", "h("])),
		("roun", HintEnum::Single("d(")),
		("as", HintEnum::Single("in(")),
		("r", HintEnum::Single("ound(")),
		("log", HintEnum::Many(&["2(", "10("])),
		("ta", HintEnum::Many(&["n(", "nh("])),
		("floo", HintEnum::Single("r(")),
		("cbrt", HintEnum::Single("(")),
		("ata", HintEnum::Single("n(")),
		("ce", HintEnum::Single("il(")),
		("abs", HintEnum::Single("(")),
		("cosh", HintEnum::Single("(")),
		("cbr", HintEnum::Single("t(")),
		("rou", HintEnum::Single("nd(")),
		("signu", HintEnum::Single("m(")),
		("a", HintEnum::Many(&["bs(", "sin(", "cos(", "tan("])),
		("sqrt", HintEnum::Single("(")),
		("ceil", HintEnum::Single("(")),
		("ro", HintEnum::Single("und(")),
		("f", HintEnum::Many(&["loor(", "ract("])),
		("sign", HintEnum::Single("um(")),
		("trunc", HintEnum::Single("(")),
		("cb", HintEnum::Single("rt(")),
	],
};

/// Gets completion from `COMPLETION_HASHMAP`
pub fn get_completion(key: String) -> Option<HintEnum<'static>> {
	if key.is_empty() {
		return None;
	}

	match COMPLETION_HASHMAP.get(&key) {
		Some(data_x) => Some(data_x.clone()),
		None => None,
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;

	/// Tests to make sure hints are properly outputed based on input
	#[test]
	fn hint_test() {
		let values = HashMap::from([
			("", "x^2"),
			("sin(x", ")"),
			("sin(x)", ""),
			("x^x", ""),
			("(x+1)(x-1", ")"),
			("lo", "g"),
			("log", ""), // because there are multiple log functions
			("asi", "n("),
			("asin", "("),
			("fl", "oor("),
			("ata", "n("),
			("at", "an("),
			("roun", "d("),
			("floo", "r("),
			("flo", "or("),
		]);

		for (key, value) in values {
			println!("{} + {}", key, value);
			assert_eq!(generate_hint(key).unwrap_or_default(), value.to_owned());
		}
	}

	/*
	#[test]
	fn completion_hashmap_test() {
		let values = hashmap_test_gen();
		for (key, value) in values {
			println!(
				"{} + {}",
				key,
				match value.clone() {
					Some(x) => x.clone(),
					None => "(No completion)".to_string(),
				}
			);

			assert_eq!(
				get_completion(key.to_string())

					.unwrap_or(String::new()),
				value.unwrap_or(String::new())
			);
		}
	}

	fn hashmap_test_gen() -> HashMap<String, Option<String>> {
		let mut values: HashMap<String, Option<String>> = HashMap::new();

		let processed_func: Vec<String> = [
			"abs", "signum", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh",
			"floor", "round", "ceil", "trunc", "fract", "exp", "sqrt", "cbrt", "ln", "log2",
			"log10",
		]
		.iter()
		.map(|ele| ele.to_string() + "(")
		.collect();

		let mut data_tuple: Vec<(String, Option<String>)> = Vec::new();
		for func in processed_func.iter() {
			for i in 1..=func.len() {
				let (first, last) = func.split_at(i);
				let value = match last {
					"" => None,
					x => Some(x.to_string()),
				};
				data_tuple.push((first.to_string(), value));
			}
		}

		let key_list: Vec<String> = data_tuple.iter().map(|(a, _)| a.clone()).collect();

		for (key, value) in data_tuple {
			if key_list.iter().filter(|a| **a == key).count() == 1 {
				values.insert(key, value);
			}
		}

		let values_old = values.clone();
		values = values
			.iter()
			.filter(|(key, _)| values_old.iter().filter(|(a, _)| a == key).count() == 1)
			.map(|(a, b)| (a.to_string(), b.clone()))
			.collect();

		let manual_values: Vec<(&str, Option<&str>)> =
			vec![("sin", None), ("cos", None), ("tan", None)];

		for (key, value) in manual_values {
			values.insert(key.to_string(), value.map(|x| x.to_string()));
		}
		values
	}
	*/
}
