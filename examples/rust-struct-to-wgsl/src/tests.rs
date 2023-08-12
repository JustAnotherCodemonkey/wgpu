use crate::get_value_from_buffer;

use super::{
    SystemContext, Beginner, beginner, Intermediate, intermediate, Advanced, advanced,
    example_create_bind_group,
    structs::{AdvancedInner, AsWgslBytes},
};

use pollster::FutureExt;

async fn advanced_inner(sc: &SystemContext, input: &AdvancedInner) -> AdvancedInner {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let a_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 8,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let b_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 32,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let c_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let output_staging_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 32,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let (bind_group_layout, bind_group) = example_create_bind_group(
        &sc.device,
        &input_buffer,
        &[&a_output_buffer, &b_output_buffer, &c_output_buffer]
    );

    let shader_module = sc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("test-advanced-inner.wgsl"))
        ),
    });

    let pipeline_layout = sc.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let compute_pipeline = sc.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: "main",
    });

    sc.queue.write_buffer(&input_buffer, 0, &input_bytes);
    let mut command_encoder =
        sc.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    sc.queue.submit(Some(command_encoder.finish()));

    let final_a = get_value_from_buffer(
        &a_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut output = [0i32; 2];
            for (i, v) in output.iter_mut().enumerate() {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&view[(i * 4)..(i * 4 + 4)]);
                *v = i32::from_le_bytes(bytes);
            }
            output
        }
    ).await;
    let final_b = get_value_from_buffer(
        &b_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut floats = [0f32; 8];
            for (i, v) in floats.iter_mut().enumerate() {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&view[(i * 4)..(i * 4 + 4)]);
                *v = f32::from_le_bytes(bytes);
            }
            let mut output = [[0f32; 2]; 4];
            for (rn, row) in output.iter_mut().enumerate() {
                let row_len = row.len();
                for (vn, v) in row.iter_mut().enumerate() {
                    *v = floats[rn * row_len + vn];
                }
            }
            output
        }
    ).await;
    let final_c = get_value_from_buffer(
        &c_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view);
            i32::from_le_bytes(bytes)
        }
    ).await;

    AdvancedInner {
        a: final_a,
        b: final_b,
        c: final_c,
    }
}

#[test]
fn rust_struct_to_wgsl_test_beginner() {
    fn t(input: Beginner, desired: Beginner, sc: &SystemContext) {
        assert_eq!(beginner(sc, &input).block_on(), desired);
    }

    let sc = SystemContext::new().block_on();

    t(
        Beginner {
            a: 0,
            b: [0f32; 2]
        },
        Beginner {
            a: 1,
            b: [1f32; 2],
        },
        &sc
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
        &sc
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
        &sc
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
                c: 0
            },
            d: 0,
        },
        Advanced {
            a: 1,
            b: [1; 3],
            c: AdvancedInner {
                a: [1; 2],
                b: [[1.0; 2]; 4],
                c: 1
            },
            d: 1,
        },
        &sc
    );
}