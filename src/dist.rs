use std::hash::Hash;

use nalgebra::{DMatrix, MatrixXx1};

use crate::markov::MarkovChain;

pub fn steady_state<T: Clone + Eq + Hash>(chain: &MarkovChain<T>) -> Vec<f32> {
	let mat = &DMatrix::from_row_slice(chain.tokens().len(), chain.tokens().len(), chain.matrix())
		.transpose();
	let mut eigenvector = MatrixXx1::from_element(mat.nrows(), 1.0);
	let tolerance = 1e-8;

	for _ in 0..30 {
		let new_eigenvector = mat * eigenvector.clone();
		let diff = (new_eigenvector.clone() - eigenvector.clone()).norm();
		if diff < tolerance {
			break;
		}
		eigenvector = new_eigenvector;
	}

	// let norm = eigenvector.norm();
	let norm = eigenvector.sum();
	(eigenvector / norm).data.as_vec().to_vec()
}
