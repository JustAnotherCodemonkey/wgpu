@group(0)
@binding(0)
var<uniform> input: InUniform;
@group(0)
@binding(1)
var<storage, read_write> aa_output: i32;
@group(0)
@binding(2)
var<storage, read_write> ab_output: i32;
@group(0)
@binding(3)
var<storage, read_write> b_output: i32;
@group(0)
@binding(4)
var<storage, read_write> c_output: vec2<i32>;

struct InUniform {
    a: InUniformInner,
    @align(16)
    b: i32,
    @align(16)
    c: array<i32_wrapper, 2>,
}

struct InUniformInner {
    a: i32,
    b: i32,
}

struct i32_wrapper {
    @size(16)
    inner: i32
}

@compute
@workgroup_size(1)
fn main() {
    aa_output = input.a.a + 1;
    ab_output = input.a.b + 1;
    b_output = input.b + 1;
    c_output.x = input.c[0].inner + 1;
    c_output.y = input.c[1].inner + 1;
}