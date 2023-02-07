use std::ops::{Bound, Range, RangeBounds};

use itertools::Itertools;
use rand::{
	distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, thread_rng, Rng,
};

use crate::input::Sentences;

// More comprehensible
pub trait ClosedRange {
	fn as_open_range(&self) -> Range<usize>;
}

impl<R> ClosedRange for R
where
	R: DoubleEndedIterator + RangeBounds<usize>,
{
	fn as_open_range(&self) -> Range<usize> {
		let lower = match self.start_bound() {
			Bound::Included(&x) => x,
			Bound::Excluded(&x) => x + 1,
			Bound::Unbounded => unreachable!(),
		};
		let upper = match self.end_bound() {
			Bound::Included(&x) => x + 1,
			Bound::Excluded(&x) => x,
			Bound::Unbounded => unreachable!(),
		};

		lower..upper
	}
}

pub struct MarkovChain {
	tokens: Vec<String>,
	first_token_count: usize,
	matrix: Vec<f32>, // Don't actually need a matrix struct
}

impl MarkovChain {
	/// Create a new MarkovChain from existing data
	pub fn new(tokens: &[String], first_token_count: usize, matrix: Vec<f32>) -> Self {
		Self {
			tokens: tokens.iter().unique().cloned().collect::<Vec<String>>(),
			first_token_count,
			matrix,
		}
	}

	pub fn from_sentences(sentences: Sentences) -> Self {
		let possible_tokens = sentences
			.iter()
			.flatten()
			.unique()
			.cloned()
			.collect::<Vec<_>>();

		let number_of_tokens = possible_tokens.len();

		let mut matrix = vec![0.0; number_of_tokens * number_of_tokens];
		for (token_index, token) in possible_tokens.iter().enumerate() {
			let mut next_tokens = vec![];
			for x in &sentences {
				if x.len() < 2 {
					break;
				}
				x.windows(2).for_each(|ab| {
					if let Ok([a, b]) = TryInto::<&[String; 2]>::try_into(ab) {
						if a == token {
							next_tokens.push(b.as_str());
						}
					}
				})
			}

			let accessible_token_count = next_tokens.len() as f32;
			for next_token in next_tokens {
				let next_token_index = possible_tokens
					.iter()
					.position(|x| *x == next_token)
					.unwrap();
				matrix[token_index * number_of_tokens + next_token_index] +=
					1.0 / accessible_token_count;
			}
		}

		Self {
			tokens: possible_tokens,
			first_token_count: sentences.len(),
			matrix,
		}
	}

	pub fn get_tokens(&self) -> &[String] {
		&self.tokens
	}

	pub fn get_first_tokens(&self) -> &[String] {
		&self.tokens[..self.first_token_count]
	}

	pub fn generate_text(
		&self,
		sentence_count: usize,
		sentence_length_bounds: impl ClosedRange,
	) -> Result<Vec<String>, &'static str> {
		self.assert_able_to_generate()?;

		let mut rng = thread_rng();

		let first_tokens = &self.get_first_tokens();

		let mut sentences = vec![String::new(); sentence_count];
		for sentence in sentences.iter_mut() {
			let mut current_token = first_tokens.choose(&mut rng).unwrap().as_str();

			*sentence += &format!(
				"{}{}",
				&(current_token.as_bytes()[0] as char).to_uppercase(),
				&current_token[1..]
			);

			let random_sentence_length = rng.gen_range(sentence_length_bounds.as_open_range());
			for _ in 0..random_sentence_length {
				let Some(next_token) =  self.generate_next_token(current_token) else {
                    // println!("No next token found for <{}>", current_token);
                    break;
                };

				current_token = next_token;
				if current_token == "," {
					*sentence += current_token;
				} else {
					*sentence += &format!(" {}", current_token);
				}
			}

			*sentence += ".";
		}

		Ok(sentences)
	}

	fn assert_able_to_generate(&self) -> Result<(), &'static str> {
		if self.get_tokens().is_empty() {
			return Err("No tokens found");
		}
		if self.get_first_tokens().is_empty() {
			return Err("No first tokens found");
		}
		// ...

		Ok(())
	}

	fn generate_next_token<'a>(&'a self, current_token: &'a str) -> Option<&'a str> {
		let mut rng = thread_rng();

		let next_token_weights = {
			let token_index = self
				.tokens
				.iter()
				.position(|x| *x == current_token)
				.unwrap();
			let row_offset = self.tokens.len() * token_index;
			&self.matrix[row_offset..row_offset + self.tokens.len()]
		};

		// `any` is faster because it can short-circuit
		if next_token_weights.iter().any(|x| *x != 0.0) {
			let dist = WeightedIndex::new(next_token_weights).unwrap();
			let next_token_index = dist.sample(&mut rng);

			Some(&self.tokens[next_token_index])
		} else {
			None
		}
	}
}
