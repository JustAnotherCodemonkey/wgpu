mod structs;
#[cfg(test)]
mod tests;
mod utils;

use structs::{
    Advanced, AdvancedInner, AsWgslBytes, Beginner, FromWgslBuffers, InUniform, InUniformInner,
    Intermediate,
};
use utils::{
    compute, create_bind_group, create_input_buffer, create_output_buffers, create_pipeline,
    create_staging_buffer, SystemContext,
};

use pollster::FutureExt;

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

    let compute_pipeline = create_pipeline(&sc.device, &bind_group_layout, &shader_module);

    compute(
        &input_buffer,
        &input_bytes,
        &sc.device,
        &sc.queue,
        &compute_pipeline,
        &bind_group,
    );

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
            8,  // c
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

    let compute_pipeline = create_pipeline(&sc.device, &bind_group_layout, &shader_module);

    compute(
        &input_buffer,
        &input_bytes,
        &sc.device,
        &sc.queue,
        &compute_pipeline,
        &bind_group,
    );

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
            4,  // d
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

    let compute_pipeline = create_pipeline(&sc.device, &bind_group_layout, &shader_module);

    compute(
        &input_buffer,
        &input_bytes,
        &sc.device,
        &sc.queue,
        &compute_pipeline,
        &bind_group,
    );

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

    let compute_pipeline = create_pipeline(&sc.device, &bind_group_layout, &shader_module);

    compute(
        &input_buffer,
        &input_bytes,
        &sc.device,
        &sc.queue,
        &compute_pipeline,
        &bind_group,
    );

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
