mod structs;
#[cfg(test)]
mod tests;
mod utils;

use structs::{
    Advanced, AdvancedInner, AsWgslBytes, Beginner, FromWgslBuffers, InUniform, Intermediate,
};
use utils::example_create_bind_group;

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
        &[&a_output_buffer, &b_output_buffer],
    );

    let shader_module = sc
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "beginner.wgsl"
            ))),
        });

    let pipeline_layout = sc
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
    let compute_pipeline = sc
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

    sc.queue.write_buffer(&input_buffer, 0, &input_bytes);
    let mut command_encoder = sc
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    sc.queue.submit(Some(command_encoder.finish()));

    Beginner::from_wgsl_buffers(
        &[&a_output_buffer, &b_output_buffer],
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
}

async fn intermediate(sc: &SystemContext, input: &Intermediate) -> Intermediate {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = sc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut member_buffers = Vec::<wgpu::Buffer>::with_capacity(3);
    // 4, 12, and 8 are our member sizes.
    for size in [4, 12, 8] {
        member_buffers.push(sc.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
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
        &member_buffers.iter().collect::<Vec<_>>(),
    );

    let shader_module = sc
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "intermediate.wgsl"
            ))),
        });

    let pipeline_layout = sc
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
    let compute_pipeline = sc
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

    sc.queue.write_buffer(&input_buffer, 0, &input_bytes);
    let mut command_encoder = sc
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    sc.queue.submit(Some(command_encoder.finish()));

    Intermediate::from_wgsl_buffers(
        member_buffers
            .iter()
            .collect::<Vec<&wgpu::Buffer>>()
            .as_slice(),
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
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

    let shader_module = sc
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "advanced.wgsl"
            ))),
        });

    let pipeline_layout = sc
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
    let compute_pipeline = sc
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

    sc.queue.write_buffer(&input_buffer, 0, &input_bytes);
    let mut command_encoder = sc
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    sc.queue.submit(Some(command_encoder.finish()));

    Advanced::from_wgsl_buffers(
        &[
            &a_output_buffer,
            &b_output_buffer,
            &ca_output_buffer,
            &cb_output_buffer,
            &cc_output_buffer,
            &d_output_buffer,
        ],
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
}

fn main() {
    println!("Hello, world!");
}
