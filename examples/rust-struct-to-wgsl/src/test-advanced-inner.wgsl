@group(0)
@binding(0)
var<storage, read> input: AdvancedInner;
@group(0)
@binding(1)
var<storage, read_write> a_output: vec2<i32>;
@group(0)
@binding(2)
var<storage, read_write> b_output: mat4x2<f32>;
@group(0)
@binding(3)
var<storage, read_write> c_output: i32;

struct AdvancedInner {
    a: vec2<i32>,
    b: mat4x2<f32>,
    c: i32,
}

@compute
@workgroup_size(1)
fn main() {
    a_output = input.a + 1;
    b_output = input.b + mat4x2(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    c_output = input.c + 1;
}