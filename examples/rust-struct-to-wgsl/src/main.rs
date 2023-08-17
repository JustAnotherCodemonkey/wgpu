mod structs;
#[cfg(test)]
mod tests;
mod utils;

use structs::{
    Advanced, AdvancedInner, AsWgslBytes, Beginner, FromWgslBuffers, InUniform, InUniformInner,
    Intermediate,
};
use utils::{create_bind_group, create_input_buffer, create_output_buffers, create_staging_buffer};

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

    let input_buffer = create_input_buffer(&sc.device, input_bytes.len() as u64, false);
    let output_buffers = create_output_buffers(
        &sc.device,
        &[
            4, // a
            8, // b
        ],
    );
    let output_staging_buffer = create_staging_buffer(&sc.device, 8);

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
        &output_buffers.iter().collect::<Vec<&_>>(),
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
}

async fn intermediate(sc: &SystemContext, input: &Intermediate) -> Intermediate {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = create_input_buffer(&sc.device, input_bytes.len() as u64, false);
    let output_buffers = create_output_buffers(
        &sc.device,
        &[
            4,  // a
            12, // b
            8, // c
        ],
    );
    let output_staging_buffer = create_staging_buffer(&sc.device, 12);

    let (bind_group_layout, bind_group) = create_bind_group(
        &sc.device,
        &input_buffer,
        &output_buffers.iter().collect::<Vec<_>>(),
        false,
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
        output_buffers
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

    let input_buffer = create_input_buffer(&sc.device, input_bytes.len() as u64, false);
    let output_buffers = create_output_buffers(
        &sc.device,
        &[
            4,  // a
            12, // b
            8,  // c.a
            32, // c.b
            4,  // c.c
            4, // d
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
        &output_buffers.iter().collect::<Vec<&_>>(),
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
}

async fn in_uniform(sc: &SystemContext, input: &InUniform) -> InUniform {
    let input_bytes = input.as_wgsl_bytes();

    let input_buffer = create_input_buffer(&sc.device, input_bytes.len() as u64, true);
    let output_buffers = create_output_buffers(
        &sc.device,
        &[
            4, // aa
            4, // ab
            4, // b
            8, // c
        ],
    );
    let output_staging_buffer = create_staging_buffer(&sc.device, 8);

    let shader_module = sc
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "in-uniform.wgsl"
            ))),
        });

    let (bind_group_layout, bind_group) = create_bind_group(
        &sc.device,
        &input_buffer,
        &output_buffers.iter().collect::<Vec<&_>>(),
        true,
    );

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

    InUniform::from_wgsl_buffers(
        &output_buffers.iter().collect::<Vec<&_>>(),
        &output_staging_buffer,
        &sc.device,
        &sc.queue,
    )
}

fn main() {
    let sc = SystemContext::new().block_on();
    let shader_module = sc
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "in-uniform.wgsl"
            ))),
        });
}
