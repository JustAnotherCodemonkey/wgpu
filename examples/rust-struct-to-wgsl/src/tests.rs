use crate::utils::{
    compute, create_input_buffer, create_output_buffers, create_pipeline, create_staging_buffer,
};

use super::{
    advanced, beginner, create_bind_group, in_uniform, intermediate,
    structs::{AdvancedInner, AsWgslBytes, FromWgslBuffers},
    Advanced, Beginner, InUniform, InUniformInner, Intermediate, SystemContext,
};

use pollster::FutureExt;

async fn advanced_inner(sc: &SystemContext, input: &AdvancedInner) -> AdvancedInner {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = create_input_buffer(&sc.device, input_bytes.len() as u64, false);
    let output_buffers = create_output_buffers(
        &sc.device,
        &[
            8,  // a
            32, // b
            4,  // c
        ],
    );
    let output_staging_buffer = create_staging_buffer(&sc.device, 32);

    let (bind_group_layout, bind_group) = create_bind_group(
        &sc.device,
        &input_buffer,
        &output_buffers.iter().collect::<Vec<&_>>(),
        false,
    );

    let shader_module = sc
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "test-advanced-inner.wgsl"
            ))),
        });

    let compute_pipeline = create_pipeline(&sc.device, &bind_group_layout, &shader_module);

    compute(
        &input_buffer,
        &input_bytes,
        &sc.device,
        &sc.queue,
        &compute_pipeline,
        &bind_group,
    );

    AdvancedInner::from_wgsl_buffers(
        &output_buffers.iter().collect::<Vec<&_>>(),
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
}

#[test]
fn rust_struct_to_wgsl_test_beginner() {
    fn t(input: Beginner, desired: Beginner, sc: &SystemContext) {
        assert_eq!(beginner(sc, &input).block_on(), desired);
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
        assert_eq!(intermediate(sc, &input).block_on(), desired);
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
        assert_eq!(advanced_inner(sc, &input).block_on(), desired);
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
        assert_eq!(advanced(sc, &input).block_on(), desired);
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
        assert_eq!(in_uniform(sc, &input).block_on(), desired);
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
