use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::{distributions::WeightedIndex, prelude::*};
use rulinalg::matrix::{BaseMatrix, Matrix};
use std::ops::{Range, RangeBounds, RangeInclusive};
use std::{
	collections::{HashMap, HashSet},
	time::Duration,
};

fn get_pi_matrix(token: &str, possible_tokens: &[&str]) -> Matrix<f64> {
	let mut pi_vec = vec![0.0; possible_tokens.len()];
	let token_index = possible_tokens.iter().position(|x| *x == token).unwrap();
	pi_vec[token_index] = 1.0;
	Matrix::new(1, possible_tokens.len(), pi_vec)
}

fn main() {
	let text = include_str!("genesis.txt");

	let tokens = text.replace(
		|x: char| !x.is_alphabetic() && x != '.' && x != ',' && x != ' ',
		"",
	);

	let mut first_tokens: Vec<String> = vec![];
	let sentences: Vec<Vec<String>> = tokens
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
		.collect::<Vec<_>>();

	let possible_tokens = sentences
		.iter()
		.flat_map(|x| x.iter())
		.collect::<HashSet<_>>();
	let possible_tokens_vec = possible_tokens
		.iter()
		.map(|x| x.as_str())
		.collect::<Vec<_>>();
	let number_of_tokens = possible_tokens.len();
	println!("Number of tokens: {}", number_of_tokens);
	// let mut token_map: HashMap<&str, Vec<&str>> = HashMap::new();

	let mut mat_vec = vec![0.0; number_of_tokens * number_of_tokens];
	for (token_index, token) in possible_tokens.iter().enumerate() {
		let mut next_tokens = vec![];
		sentences.iter().for_each(|x| {
			if x.len() < 2 {
				panic!("sentence shorter than 2 words");
			}
			x.windows(2).for_each(|ab| {
				if let Ok([a, b]) = TryInto::<&[String; 2]>::try_into(ab) {
					if a == *token {
						next_tokens.push(b.as_str());
					}
				}
			})
		});
		// token_map.insert(token, next_tokens);

		let accessible_token_count = next_tokens.len() as f64;
		for next_token in next_tokens {
			let next_token_index = possible_tokens
				.iter()
				.position(|x| *x == next_token)
				.unwrap();
			mat_vec[token_index * number_of_tokens + next_token_index] +=
				1.0 / accessible_token_count;
		}
	}

	let markov_matrix = Matrix::new(number_of_tokens, number_of_tokens, mat_vec);

	println!("Matrix created.");

	let first_tokens_full = possible_tokens_vec
		.iter()
		.filter(|&&x| x != "," && x != " ")
		.map(|x| x.to_string())
		.collect::<Vec<_>>();
	let text = generate_text(
		10,
		5..32,
		// &first_tokens,
		first_tokens_full.as_slice(),
		&markov_matrix,
		&possible_tokens_vec,
	);

	for sentence in text {
		println!("{}", sentence);
	}
}

fn generate_text(
	sentence_count: usize,
	sentence_length_bounds: Range<usize>,
	first_tokens: &[String],
	markov_matrix: &Matrix<f64>,
	possible_tokens: &[&str],
) -> Vec<String> {
	let mut rng = thread_rng();

	let mut sentences = vec![String::new(); sentence_count];
	for sentence in sentences.iter_mut() {
		let mut current_token = first_tokens.choose(&mut rng).unwrap().to_lowercase();
		*sentence += &(current_token
			.chars()
			.next()
			.unwrap()
			.to_uppercase()
			.to_string() + &current_token.as_str()[1..]);

		let random_sentence_length = rng.gen_range(sentence_length_bounds.clone());
		for _ in 0..random_sentence_length {
			let Some(next_token) = generate_next_token(&current_token, markov_matrix, possible_tokens) else {
                // println!("No next token found for <{}>", current_token);
                break;
        };

			current_token = next_token.to_owned();
			if current_token == "," {
				*sentence += &current_token;
			} else {
				*sentence += &format!(" {}", current_token);
			}
		}

		*sentence += ".";
	}

	sentences
}

fn generate_next_token<'a>(
	current_token: &'a str,
	markov_matrix: &'a Matrix<f64>,
	possible_tokens: &'a [&str],
) -> Option<&'a str> {
	let mut rng = thread_rng();
	let random_pi_matrix = get_pi_matrix(current_token, possible_tokens);

	// println!(
	// 	"Pi matrix for <{}>: {:?}",
	// 	current_token,
	// 	random_pi_matrix.data()
	// );
	let next_token_weights = random_pi_matrix * markov_matrix;

	if next_token_weights.data().iter().all(|x| *x == 0.0) {
		return None;
	}
	let dist = WeightedIndex::new(next_token_weights.data()).unwrap();
	let next_token_index = dist.sample(&mut rng);

	Some(possible_tokens[next_token_index])
}
