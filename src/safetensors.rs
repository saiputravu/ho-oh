use memmap2::MmapOptions;
use safetensors::SafeTensors;
use std::fs::File;

pub fn test() {
    let modelpath = "../models/Qwen3-0.6B/model.safetensors";

    let file = File::open(modelpath).unwrap();

    let buffer = unsafe { MmapOptions::new().map(&file).unwrap() };

    let tensors = SafeTensors::deserialize(&buffer).unwrap();

    let tensor = tensors
        .tensor("model.layers.16.self_attn.k_norm.weight")
        .unwrap();
    println!("{:?}", tensor);
}
