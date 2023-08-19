use crate::utils::{
    compute, create_input_buffer, create_output_buffers, create_pipeline, create_staging_buffer,
    ExampleStruct
};
use super::{
    create_bind_group,
    structs::{AdvancedInner, AsWgslBytes, FromWgslBuffers},
    Advanced, Beginner, InUniform, InUniformInner, Intermediate, SystemContext,
};
use pollster::FutureExt;

#[test]
fn rust_struct_to_wgsl_test_beginner() {
    fn t(input: Beginner, desired: Beginner, sc: &SystemContext) {
        assert_eq!(input.run_as_example(sc, include_str!("beginner.wgsl"), false), desired);
    }

    let sc = SystemContext::new().block_on();

    t(
        Beginner { a: 0, b: [0f32; 2] },
        Beginner { a: 1, b: [1f32; 2] },
        &sc,
    );
}

#[test]
fn rust_struct_to_wgsl_test_intermediate() {
    fn t(input: Intermediate, desired: Intermediate, sc: &SystemContext) {
        assert_eq!(input.run_as_example(sc, include_str!("intermediate.wgsl"), false), desired);
    }

    let sc = SystemContext::new().block_on();

    t(
        Intermediate {
            a: 0,
            b: [0.0; 3],
            c: [0; 2],
        },
        Intermediate {
            a: 1,
            b: [1.0; 3],
            c: [1; 2],
        },
        &sc,
    );
}

#[test]
fn rust_struct_to_wgsl_test_advanced_inner() {
    fn t(input: AdvancedInner, desired: AdvancedInner, sc: &SystemContext) {
        assert_eq!(input.run_as_example(sc, include_str!("test-advanced-inner.wgsl"), false), desired);
    }

    let sc = SystemContext::new().block_on();

    t(
        AdvancedInner {
            a: [0; 2],
            b: [[0.0; 2]; 4],
            c: 0,
        },
        AdvancedInner {
            a: [1; 2],
            b: [[1.0; 2]; 4],
            c: 1,
        },
        &sc,
    );
}

#[test]
fn rust_struct_to_wgsl_test_advanced() {
    fn t(input: Advanced, desired: Advanced, sc: &SystemContext) {
        assert_eq!(input.run_as_example(sc, include_str!("advanced.wgsl"), false), desired);
    }

    let sc = SystemContext::new().block_on();

    t(
        Advanced {
            a: 0,
            b: [0; 3],
            c: AdvancedInner {
                a: [0; 2],
                b: [[0.0; 2]; 4],
                c: 0,
            },
            d: 0,
        },
        Advanced {
            a: 1,
            b: [1; 3],
            c: AdvancedInner {
                a: [1; 2],
                b: [[1.0; 2]; 4],
                c: 1,
            },
            d: 1,
        },
        &sc,
    );
}

#[test]
fn rust_struct_to_wgsl_test_in_uniform() {
    fn t(input: InUniform, desired: InUniform, sc: &SystemContext) {
        assert_eq!(input.run_as_example(sc, include_str!("in-uniform.wgsl"), true), desired);
    }

    let sc = SystemContext::new().block_on();

    t(
        InUniform {
            a: InUniformInner { a: 0, b: 0 },
            b: 0,
            c: [0; 2],
        },
        InUniform {
            a: InUniformInner { a: 1, b: 1 },
            b: 1,
            c: [1; 2],
        },
        &sc,
    )
}
