mod input;
mod markov;

// fn get_pi_matrix(token: &str, possible_tokens: &[&str]) -> Matrix<f32> {
// 	let mut pi_vec = vec![0.0; possible_tokens.len()];
// 	let token_index = possible_tokens.iter().position(|x| *x == token).unwrap();
// 	pi_vec[token_index] = 1.0;
// 	Matrix::new(1, possible_tokens.len(), pi_vec)
// }

fn main() {
	let text = include_str!("genesis.txt");

	let sentences = input::format_input(text);
	let markov_chain = markov::MarkovChain::from_sentences(sentences);

	let text = markov_chain.generate_text(10, 5..=32);

	match text {
		Ok(text) => {
			for sentence in text {
				println!("{}", sentence);
			}
		}
		Err(e) => println!("{}", e),
	}
}
