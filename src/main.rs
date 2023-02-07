mod input;
mod markov;

// fn get_pi_matrix(token: &str, possible_tokens: &[&str]) -> Matrix<f32> {
// 	let mut pi_vec = vec![0.0; possible_tokens.len()];
// 	let token_index = possible_tokens.iter().position(|x| *x == token).unwrap();
// 	pi_vec[token_index] = 1.0;
// 	Matrix::new(1, possible_tokens.len(), pi_vec)
// }

fn display_words(input: Vec<Vec<String>>) {
	for sentence in input {
		println!(
			"{:?}",
			sentence.iter().skip(1).fold(
				sentence[0]
					.chars()
					.enumerate()
					.map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
					.collect::<String>(),
				|a, b| { format!("{}{}{}", a, if b == "," { "" } else { " " }, b) }
			)
		);
	}
}

fn display_text_bytes(input: Vec<Vec<u8>>) {
	for sentence in input {
		println!("{}", String::from_utf8(sentence).unwrap());
	}
}

fn main() {
	let text = include_str!("genesis.txt");

	let sentences = input::format_input(text);
	let markov_chain = markov::MarkovChain::from_grouped_data(sentences);

	// let bytes = input::format_input(text).into_iter().flatten().flat_map(|x| x.into_bytes()).collect::<Vec<_>>();
	// let markov_chain = markov::MarkovChain::from_continuous_data(bytes);

	let text = markov_chain.generate(20, 5..128);

	match text {
		Ok(text) => display_words(text),
		// Ok(text) => display_text_bytes(text),
		Err(e) => println!("{}", e),
	}
}
