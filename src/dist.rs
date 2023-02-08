use std::hash::Hash;

use crate::markov::MarkovChain;

pub fn markov_matrix_dist<T: Clone + Eq + Hash>(chain: &MarkovChain<T>) -> Vec<f32> {
	todo!()
}
