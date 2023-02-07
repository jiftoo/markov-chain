pub type Sentences = Vec<Vec<String>>;

pub fn format_input(text: &str) -> Sentences {
	let tokens = text.replace(
		|x: char| !x.is_alphabetic() && x != '.' && x != ',' && x != ' ',
		"",
	);

	let mut first_tokens: Vec<String> = vec![];

	tokens
		.split('.') // make sentences
		.map(|x| x.split_whitespace())
		.map(|x| {
			x.flat_map(|x| {
				let end_char = if x.ends_with(',') { "," } else { "" };
				let trimmed = x.trim_end_matches(|x: char| x == ',');
				[trimmed, end_char]
			})
			.collect::<Vec<_>>()
		})
		.filter(|x| !x.is_empty())
		.inspect(|x| {
			first_tokens.extend(
				x.iter()
					.filter(|x| x.starts_with(|x: char| x.is_uppercase()))
					.map(|x| x.to_string()),
			);
		})
		.map(|x| {
			x.iter()
				.filter(|x| !x.is_empty())
				.map(|x| x.to_lowercase())
				.collect()
		})
		.collect::<Vec<_>>()
}
