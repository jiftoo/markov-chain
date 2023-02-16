use std::fs::File;

use std::io::Read;

use clap::arg;
use clap::command;
use clap::CommandFactory;
use clap::Parser;
use input::format_input;
use markov::MarkovChain;

mod dist;
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
			"{}",
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

// fn print_table(markov_chain: &markov::MarkovChain<String>) {
// 	let headers = markov_chain
// 		.tokens()
// 		.iter()
// 		.map(|x| x.chars().take(7).collect::<String>())
// 		.collect::<Vec<_>>();
// 	println!("\t{}", headers.join("\t"));
// 	for (c, h) in markov_chain
// 		.matrix()
// 		.chunks_exact(markov_chain.tokens().len())
// 		.zip(&headers)
// 	{
// 		println!("{}\t{}", h, c.iter().map(|x| x.to_string()).join("\t"));
// 	}
// }

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Parse as binary instead of sentences
	#[arg(short, default_value_t = false)]
	binary: bool,
	/// Treat input as file path
	#[arg(short, default_value_t = false)]
	file: bool,
	/// Number of sentences to generate
	#[arg(short, default_value_t = 1)]
	s_number_of_sentences: usize,
	/// Number or range of tokens per sentence <length|low-high>
	#[arg(short, default_value = "20")]
	t_number_of_tokens: String,
	/// Input string, or file path (if -f is set)
	input: Vec<String>,
}

fn read_input(args: &Args) -> Result<String, String> {
	let input = args.input.join(" ");
	if args.file {
		let mut file = File::open(input).map_err(|x| x.to_string())?;
		let mut buf = Vec::new();
		file.read_to_end(&mut buf).map_err(|x| x.to_string())?;
		String::from_utf8(buf).map_err(|x| x.to_string())
	} else {
		Ok(input)
	}
}

fn print_help_and_abort() -> ! {
	let mut cmd = Args::command();
	cmd.print_long_help();
	std::process::exit(1);
}

fn main() {
	let Ok(args) = Args::try_parse() else {
		print_help_and_abort();
	};
	if args.input.is_empty() {
		println!("No input provided!");
		print_help_and_abort();
	}

	let length_range = if args.t_number_of_tokens.contains('-') {
		let range = args
			.t_number_of_tokens
			.split('-')
			.map(|x| x.parse::<usize>().unwrap())
			.collect::<Vec<_>>();
		if range.len() != 2 {
			println!("Invalid range");
			return;
		}
		range[0]..=range[1]
	} else {
		let n = args.t_number_of_tokens.parse::<usize>().unwrap();
		n..=n
	};

	let input_data = match read_input(&args) {
		Ok(input_data) => input_data,
		Err(e) => {
			println!("{}", e);
			return;
		}
	};

	if args.binary {
		let markov_chain = MarkovChain::from_continuous_data(input_data.into_bytes());
		let output = markov_chain.generate(args.s_number_of_sentences, length_range);
		match output {
			Ok(garbage) => display_text_bytes(garbage),
			Err(e) => println!("{}", e),
		}
	} else {
		let markov_chain = MarkovChain::from_grouped_data(format_input(&input_data));
		let output = markov_chain.generate(args.s_number_of_sentences, length_range);
		match output {
			Ok(text) => display_words(text),
			Err(e) => println!("{}", e),
		}
	}
}
