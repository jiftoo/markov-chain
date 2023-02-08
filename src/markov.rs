use std::{
	hash::Hash,
	ops::{Bound, Range, RangeBounds},
};

use itertools::Itertools;
use rand::{
	distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, thread_rng, Rng,
};

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

pub struct MarkovChain<T> {
	tokens: Vec<T>,
	matrix: Vec<f32>, // Don't actually need a matrix struct
}

impl<T: Clone + Eq + Hash> MarkovChain<T> {
	/// Create a new MarkovChain from existing data
	pub fn new(tokens: &[T], matrix: Vec<f32>) -> Self {
		Self {
			tokens: tokens.iter().unique().cloned().collect::<Vec<T>>(),
			matrix,
		}
	}

	pub fn from_grouped_data(data: Vec<Vec<T>>) -> MarkovChain<T> {
		let possible_tokens = data.iter().flatten().unique().cloned().collect::<Vec<_>>();

		let number_of_tokens = possible_tokens.len();

		let mut matrix = vec![0.0; number_of_tokens * number_of_tokens];
		for (token_index, token) in possible_tokens.iter().enumerate() {
			let next_tokens = Self::find_next_tokens(token, &data);

			let accessible_token_count = next_tokens.len();
			if next_tokens.is_empty() {
				// Can go to any token if there's not real next tokens
				let l = token_index * number_of_tokens;
				let h = l + number_of_tokens;
				matrix[l..h].fill(1.0 / possible_tokens.len() as f32);
			} else {
				for next_token in next_tokens {
					let next_token_index = possible_tokens
						.iter()
						.position(|x| x == next_token)
						.unwrap();
					matrix[token_index * number_of_tokens + next_token_index] +=
						1.0 / accessible_token_count as f32;
				}
			}
		}

		assert!(matrix.chunks_exact(number_of_tokens).all(|chunk| {
			let sum: f32 = chunk.iter().sum();
			(sum - 1.0).abs() < 0.0001
		}));

		Self {
			tokens: possible_tokens,
			matrix,
		}
	}

	pub fn from_continuous_data(data: Vec<T>) -> Self {
		Self::from_grouped_data(vec![data])
	}

	pub fn tokens(&self) -> &[T] {
		&self.tokens
	}

	pub fn matrix(&self) -> &[f32] {
		&self.matrix
	}

	fn find_next_tokens<'a>(current_token: &'a T, data: &'a Vec<Vec<T>>) -> Vec<&'a T> {
		let mut next_tokens = vec![];
		// data.len() == 1 in case of continuous data
		for x in data {
			if x.len() < 2 {
				continue;
			}
			x.windows(2).for_each(|ab| {
				if let Ok([a, b]) = TryInto::<&[T; 2]>::try_into(ab) {
					if a == current_token {
						next_tokens.push(b);
					}
				}
			})
		}

		next_tokens
	}

	pub fn generate(
		&self,
		sentence_count: usize,
		sentence_length_bounds: impl ClosedRange,
	) -> Result<Vec<Vec<T>>, &'static str> {
		self.assert_able_to_generate()?;

		let mut rng = thread_rng();

		let mut sentences = Vec::with_capacity(sentence_count);
		for _ in 0..sentence_count {
			let mut sentence = Vec::new();

			let mut current_token = self.tokens.choose(&mut rng).unwrap();

			sentence.push(current_token.clone());

			let random_sentence_length = rng.gen_range(sentence_length_bounds.as_open_range());
			for _ in 0..random_sentence_length {
				let Some(next_token) =  self.generate_next_token(current_token) else {
                    // println!("No next token found for <{}>", current_token);
                    break;
                };

				current_token = next_token;
				sentence.push(current_token.clone());
			}

			sentences.push(sentence);
		}

		Ok(sentences)
	}

	fn assert_able_to_generate(&self) -> Result<(), &'static str> {
		if self.tokens().is_empty() {
			return Err("No tokens found");
		}
		// ...

		Ok(())
	}

	fn generate_next_token<'a>(&'a self, current_token: &'a T) -> Option<&'a T> {
		let mut rng = thread_rng();

		let next_token_weights = {
			let token_index = self.tokens.iter().position(|x| x == current_token).unwrap();
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
