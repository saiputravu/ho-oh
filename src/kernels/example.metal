#include <metal_stdlib>
using namespace metal;

kernel void scale_tensor(device const float *in_data [[buffer(0)]],
                         device float *out_data [[buffer(1)]],
                         constant float &scale [[buffer(2)]],
                         uint id [[thread_position_in_grid]]) {
  out_data[id] = in_data[id] * scale;
}
