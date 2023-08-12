@group(0)
@binding(0)
var<storage, read> struct_input: Intermediate;
@group(0)
@binding(1)
var<storage, read_write> a_output: i32;
@group(0)
@binding(2)
var<storage, read_write> b_output: vec3<f32>;
@group(0)
@binding(3)
var<storage, read_write> c_output: vec2<i32>;

struct Intermediate {
    a: i32,
    b: vec3<f32>,
    c: vec2<i32>,
}

@compute
@workgroup_size(1)
fn main() {
    a_output = struct_input.a + 1;
    b_output = struct_input.b + 1.0;
    c_output = struct_input.c + 1;
}