use crate::metal::MetalGPU;
use block2::RcBlock;
use dispatch2::DispatchObject;
use dispatch2::DispatchSemaphore;
use dispatch2::DispatchTime;
use memmap2::MmapOptions;
use objc2::runtime::ProtocolObject;
use objc2_metal::*;
use std::fs::File;
use std::ptr::NonNull;

pub use safetensors::SafeTensors;
pub use safetensors::serialize;

mod embeddings;
mod metal;
mod model;

fn main() {
    let mut gpu = MetalGPU::new_metal_gpu().unwrap();
    let queue_name = String::from("first");
    gpu.new_command_queue(&queue_name, Some(true)).unwrap();

    println!("device: {:?}", gpu.device.name());
    println!("metal4 supported: {:?}", gpu.metal4_supported);

    // Load the compiled metallib
    let scale_tensor = gpu
        .load_kernel_file(
            &String::from("./src/kernels/kernels.metallib"),
            &String::from("scale_tensor"),
        )
        .unwrap();

    // --- Set up test data ---
    let input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let scale: f32 = 2.5;
    let count = input.len();
    let buffer_size = count * std::mem::size_of::<f32>();
    let count_size = std::mem::size_of::<f32>();

    // Declare buffers
    let input_buffer: objc2::rc::Retained<ProtocolObject<dyn MTLBuffer>>;
    let output_buffer: objc2::rc::Retained<ProtocolObject<dyn MTLBuffer>>;
    let scale_buffer: objc2::rc::Retained<ProtocolObject<dyn MTLBuffer>>;

    input_buffer = unsafe {
        let ptr = NonNull::new(input.as_ptr() as *mut _).expect("failed to cast to pointer");
        gpu.new_buffer_from_bytes(ptr, buffer_size, MTLResourceOptions::StorageModeShared)
    }
    .unwrap();

    scale_buffer = unsafe {
        let ptr = NonNull::new(&scale as *const f32 as *mut _).expect("failed to cast to pointer");
        gpu.new_buffer_from_bytes(ptr, count_size, MTLResourceOptions::StorageModeShared)
    }
    .unwrap();

    output_buffer = gpu
        .new_buffer(buffer_size, MTLResourceOptions::StorageModeShared)
        .unwrap();

    let grid_size = MTLSize {
        width: count as usize,
        height: 1,
        depth: 1,
    };
    let threadgroup_size = MTLSize {
        width: scale_tensor
            .maxTotalThreadsPerThreadgroup()
            .min(count as usize),
        height: 1,
        depth: 1,
    };

    // Encode and dispatch
    let command_queue = gpu.get_command_queue(&queue_name).unwrap();
    match command_queue {
        metal::CommandQueue::Metal(cq) => {
            let command_buffer = cq.commandBuffer().unwrap();

            // Setup the command buffer.
            let command_encoder = command_buffer.computeCommandEncoder().unwrap();
            command_encoder.setComputePipelineState(&scale_tensor);
            unsafe {
                command_encoder.setBuffer_offset_atIndex(Some(&input_buffer), 0, 0);
                command_encoder.setBuffer_offset_atIndex(Some(&output_buffer), 0, 1);
                command_encoder.setBuffer_offset_atIndex(Some(&scale_buffer), 0, 2);
            }
            command_encoder.dispatchThreadgroups_threadsPerThreadgroup(grid_size, threadgroup_size);
            command_encoder.endEncoding();

            // Commit and wait till completion.
            command_buffer.commit();
            command_buffer.waitUntilScheduled();
            println!("Scheduled.");
            command_buffer.waitUntilCompleted();
            println!("Completed.");
        }
        metal::CommandQueue::Metal4(cq) => {
            let allocator = gpu.device.newCommandAllocator().unwrap();
            let command_buffer = gpu.device.newCommandBuffer().unwrap();
            command_buffer.beginCommandBufferWithAllocator(&allocator);

            let arg_desc = MTL4ArgumentTableDescriptor::new();
            arg_desc.setMaxBufferBindCount(3);
            let argument_table = gpu
                .device
                .newArgumentTableWithDescriptor_error(&arg_desc)
                .unwrap();
            unsafe {
                argument_table.setAddress_atIndex(input_buffer.gpuAddress(), 0);
                argument_table.setAddress_atIndex(output_buffer.gpuAddress(), 1);
                argument_table.setAddress_atIndex(scale_buffer.gpuAddress(), 2);
            }

            let command_encoder = command_buffer.computeCommandEncoder().unwrap();
            command_encoder.setComputePipelineState(&scale_tensor);
            command_encoder.setArgumentTable(Some(&argument_table));
            command_encoder.dispatchThreadgroups_threadsPerThreadgroup(grid_size, threadgroup_size);
            command_encoder.endEncoding();

            command_buffer.endCommandBuffer();

            let mut command_buffers = [NonNull::from(&*command_buffer)];
            let command_buffers_ptr = NonNull::new(command_buffers.as_mut_ptr()).unwrap();

            let options = MTL4CommitOptions::new();
            let sem = DispatchSemaphore::new(0);
            let sem_clone = sem.retain();
            let callback =
                block2::RcBlock::new(move |x: NonNull<ProtocolObject<dyn MTL4CommitFeedback>>| {
                    let feedback = unsafe { x.as_ref() };
                    println!(
                        "Committed: start {:?}, end {:?}, diff {:?}",
                        feedback.GPUStartTime(),
                        feedback.GPUEndTime(),
                        feedback.GPUEndTime() - feedback.GPUStartTime(),
                    );
                    sem_clone.signal();
                });

            unsafe {
                options.addFeedbackHandler(RcBlock::as_ptr(&callback));
                cq.commit_count_options(command_buffers_ptr, 1, &options);
            }

            // Block until semaphore is done.
            sem.try_acquire(DispatchTime::FOREVER).unwrap();
        }
    };

    // Read back results
    let output_ptr = output_buffer.contents().as_ptr() as *const f32;
    let output: Vec<f32>;
    unsafe {
        output = std::slice::from_raw_parts(output_ptr, count).to_vec();
    }

    println!("input:  {:?}", input);
    println!("scale:  {}", scale);
    println!("output: {:?}", output);

    // Read safetensors.
    let file = File::open("./models/TinyMistral-248M-v3/model.safetensors").unwrap();
    let buffer = unsafe { MmapOptions::new().map(&file).unwrap() };
    let tensors = SafeTensors::deserialize(&buffer).unwrap();
    let names = tensors.names();
    println!("{:?}", names);
    let tensor = tensors
        .tensor("model.layers.2.self_attn.k_proj.weight")
        .unwrap();
    println!("{:?}", tensor.shape())
}
