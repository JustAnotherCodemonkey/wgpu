@group(0)
@binding(0)
var<storage, read> struct_input: Beginner;
@group(0)
@binding(1)
var<storage, read_write> a_output: i32;
@group(0)
@binding(2)
var<storage, read_write> b_output: vec2<f32>;

struct Beginner {
    a: i32,
    b: vec2<f32>,
}

@compute
@workgroup_size(1)
fn main() {
    a_output = struct_input.a + 1;
    b_output = struct_input.b + 1.0;
}