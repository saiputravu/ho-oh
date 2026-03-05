use model::Layer;
use safetensors::SafeTensors;

use crate::model;

struct RotaryEmbeddingLayer<'t> {
    xq: SafeTensors<'t>,
    xc: SafeTensors<'t>,
    freq_cis: SafeTensors<'t>,
}

impl<'p> Layer for RotaryEmbeddingLayer<'p> {
    fn new() {}

    fn forward<'t>(input: SafeTensors<'t>) -> SafeTensors<'t> {
        input
    }
}
