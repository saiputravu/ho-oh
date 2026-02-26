use std::ptr::NonNull;

use dispatch2::DispatchData;
use objc2_foundation::NSString;
use objc2_metal::*;

static KERNELS_METALLIB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kernels.metallib"));

fn main() {
    // Create the system default Metal device
    let device = MTLCreateSystemDefaultDevice().expect("no Metal device found");
    println!("device: {:?}", device.name());

    // Load the compiled metallib
    let data = DispatchData::from_static_bytes(KERNELS_METALLIB);
    let library = device
        .newLibraryWithData_error(&data)
        .expect("failed to load metallib");

    // Get the scale_tensor kernel function
    let function_name = NSString::from_str("scale_tensor");
    let function = library
        .newFunctionWithName(&function_name)
        .expect("function 'scale_tensor' not found in metallib");

    // Create a compute pipeline state
    let pipeline = device
        .newComputePipelineStateWithFunction_error(&function)
        .expect("failed to create compute pipeline state");

    // Create a command queue
    let command_queue = device
        .newCommandQueue()
        .expect("failed to create command queue");

    // --- Set up test data ---
    let input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let scale: f32 = 2.5;
    let count = input.len();
    let buffer_size = count * std::mem::size_of::<f32>();

    // Create input buffer
    let input_buffer = device
        .newBufferWithBytes_length_options(
            NonNull::new(input.as_ptr() as *mut _).unwrap(),
            buffer_size,
            MTLResourceOptions::MTLResourceStorageModeShared,
        )
        .expect("failed to create input buffer");

    // Create output buffer
    let output_buffer = device
        .newBufferWithLength_options(
            buffer_size,
            MTLResourceOptions::MTLResourceStorageModeShared,
        )
        .expect("failed to create output buffer");

    // Create scale buffer
    let scale_buffer = device
        .newBufferWithBytes_length_options(
            NonNull::new(&scale as *const f32 as *mut _).unwrap(),
            std::mem::size_of::<f32>(),
            MTLResourceOptions::MTLResourceStorageModeShared,
        )
        .expect("failed to create scale buffer");

    // Encode and dispatch
    let command_buffer = command_queue
        .commandBuffer()
        .expect("failed to create command buffer");

    let encoder = command_buffer
        .computeCommandEncoder()
        .expect("failed to create compute command encoder");

    encoder.setComputePipelineState(&pipeline);

    unsafe {
        encoder.setBuffer_offset_atIndex(Some(&input_buffer), 0, 0);
        encoder.setBuffer_offset_atIndex(Some(&output_buffer), 0, 1);
        encoder.setBuffer_offset_atIndex(Some(&scale_buffer), 0, 2);
    }

    let grid_size = MTLSize {
        width: count as u64,
        height: 1,
        depth: 1,
    };
    let threadgroup_size = MTLSize {
        width: pipeline.maxTotalThreadsPerThreadgroup().min(count as u64),
        height: 1,
        depth: 1,
    };

    unsafe {
        encoder.dispatchThreads_threadsPerThreadgroup(grid_size, threadgroup_size);
    }
    encoder.endEncoding();

    command_buffer.commit();
    command_buffer.waitUntilCompleted();

    // Read back results
    let output_ptr = output_buffer.contents().as_ptr() as *const f32;
    let output: Vec<f32> = unsafe { std::slice::from_raw_parts(output_ptr, count).to_vec() };

    println!("input:  {:?}", input);
    println!("scale:  {}", scale);
    println!("output: {:?}", output);
}
