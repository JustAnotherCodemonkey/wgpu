mod structs;
mod utils;
#[cfg(test)]
mod tests;

use structs::{Advanced, Beginner, InUniform, Intermediate, AsWgslBytes, AdvancedInner};
use utils::{example_create_bind_group, get_value_from_buffer};

use pollster::FutureExt;

/// This struct will allow us to easily keep track of common variables and easily
/// set up for an example.
struct SystemContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl SystemContext {
    async fn new() -> SystemContext {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Use high limits because regular downlevel allows only 4 storage buffers.
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        SystemContext {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

// Running functions

async fn beginner(sc: &SystemContext, input: &Beginner) -> Beginner {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let a_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let b_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 8,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let output_staging_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        // Largest member
        size: 8,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let (bind_group_layout, bind_group) = example_create_bind_group(
        &sc.device,
        &input_buffer,
        &[&a_output_buffer, &b_output_buffer]
    );

    let shader_module = sc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("beginner.wgsl"))
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
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view);
            i32::from_le_bytes(bytes)
        }
    ).await;
    let final_b = get_value_from_buffer(
        &b_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut output = [0f32; 2];
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view[..4]);
            output[0] = f32::from_le_bytes(bytes.clone());
            bytes.copy_from_slice(&view[4..]);
            output[1] = f32::from_le_bytes(bytes);
            output
        }
    ).await;

    Beginner {
        a: final_a,
        b: final_b,
    }
}

async fn intermediate(sc: &SystemContext, input: &Intermediate) -> Intermediate {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false
    });
    let mut member_buffers = Vec::<wgpu::Buffer>::with_capacity(3);
    // 4, 12, and 8 are our member sizes.
    for size in [4, 12, 8] {
        member_buffers.push(sc.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));
    }
    let output_staging_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        // Largest member.
        size: 12,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let (bind_group_layout, bind_group) = example_create_bind_group(
        &sc.device,
        &input_buffer,
        &member_buffers.iter().collect::<Vec<_>>()
    );

    let shader_module = sc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("intermediate.wgsl"))
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
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    sc.queue.submit(Some(command_encoder.finish()));

    let final_a = get_value_from_buffer(
        &member_buffers[0],
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view);
            i32::from_le_bytes(bytes)
        }
    ).await;
    let final_b = get_value_from_buffer(
        &member_buffers[1],
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut floats = [0f32; 3];
            for i in 0..3 {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&view[(i * 4)..(i * 4 + 4)]);
                floats[i] = f32::from_le_bytes(bytes);
            }
            floats
        }
    ).await;
    let final_c = get_value_from_buffer(
        &member_buffers[2],
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut ints = [0i32; 2];
            for i in 0..2 {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&view[(i * 4)..(i * 4 + 4)]);
                ints[i] = i32::from_le_bytes(bytes);
            }
            ints
        }
    ).await;

    Intermediate {
        a: final_a,
        b: final_b,
        c: final_c,
    }
}

async fn advanced(sc: &SystemContext, input: &Advanced) -> Advanced {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let a_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let b_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 12,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let ca_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 8,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let cb_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 32,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let cc_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let d_output_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
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
        &[
            &a_output_buffer,
            &b_output_buffer,
            &ca_output_buffer,
            &cb_output_buffer,
            &cc_output_buffer,
            &d_output_buffer,
        ],
    );

    let shader_module = sc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("advanced.wgsl"))
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
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view);
            u32::from_le_bytes(bytes)
        }
    ).await;
    let final_b = get_value_from_buffer(
        &b_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut output = [0i32; 3];
            for (i, v) in output.iter_mut().enumerate() {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&view[(i * 4)..(i * 4 + 4)]);
                *v = i32::from_le_bytes(bytes);
            }
            output
        }
    ).await;
    let final_ca = get_value_from_buffer(
        &ca_output_buffer,
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
    let final_cb = get_value_from_buffer(
        &cb_output_buffer,
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
    let final_cc = get_value_from_buffer(
        &cc_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view);
            i32::from_le_bytes(bytes)
        }
    ).await;
    let final_d = get_value_from_buffer(
        &d_output_buffer,
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
        |view| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&view);
            i32::from_le_bytes(bytes)
        }
    ).await;

    Advanced {
        a: final_a,
        b: final_b,
        c: AdvancedInner {
            a: final_ca,
            b: final_cb,
            c: final_cc
        },
        d: final_d
    }
}

fn main() {
    println!("Hello, world!");
}