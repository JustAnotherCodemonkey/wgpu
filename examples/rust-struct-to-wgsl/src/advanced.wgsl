@group(0)
@binding(0)
var<storage, read> input: Advanced;
@group(0)
@binding(1)
var<storage, read_write> a_output: u32;
@group(0)
@binding(2)
var<storage, read_write> b_output: array<i32, 3>;
@group(0)
@binding(3)
var<storage, read_write> ca_output: vec2<i32>;
@group(0)
@binding(4)
var<storage, read_write> cb_output: mat4x2<f32>;
@group(0)
@binding(5)
var<storage, read_write> cc_output: i32;
@group(0)
@binding(6)
var<storage, read_write> d_output: i32;

struct Advanced {
    a: u32,
    b: array<i32, 3>,
    c: AdvancedInner,
    d: i32,
}

struct AdvancedInner {
    a: vec2<i32>,
    b: mat4x2<f32>,
    c: i32,
}

@compute
@workgroup_size(1)
fn main() {
    a_output = input.a + 1u;
    b_output[0] = input.b[0] + 1;
    b_output[1] = input.b[1] + 1;
    b_output[2] = input.b[2] + 1;
    ca_output = input.c.a + 1;
    cb_output = input.c.b + mat4x2(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    cc_output = input.c.c + 1;
    d_output = input.d + 1;
}