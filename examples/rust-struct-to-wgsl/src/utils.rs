pub fn compute(
    input_buffer: &wgpu::Buffer,
    input_bytes: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    compute_pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
) {
    queue.write_buffer(input_buffer, 0, input_bytes);
    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    queue.submit(Some(command_encoder.finish()));
}

pub fn create_input_buffer(device: &wgpu::Device, size: u64, is_in_uniform: bool) -> wgpu::Buffer {
    let memory_space_usage = if is_in_uniform {
        wgpu::BufferUsages::UNIFORM
    } else {
        wgpu::BufferUsages::STORAGE
    };
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: memory_space_usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_output_buffers(device: &wgpu::Device, sizes: &[u64]) -> Vec<wgpu::Buffer> {
    let mut output_vec = Vec::<wgpu::Buffer>::with_capacity(sizes.len());

    for size in sizes.iter().copied() {
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        output_vec.push(buf);
    }

    output_vec
}

pub fn create_staging_buffer(device: &wgpu::Device, largest_member_size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: largest_member_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    input_struct_buffer: &wgpu::Buffer,
    member_buffers: &[&wgpu::Buffer],
    input_is_uniform: bool,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let mut layout_entries =
        Vec::<wgpu::BindGroupLayoutEntry>::with_capacity(member_buffers.len() + 1);
    // Input buffer
    let input_buffer_binding_type = if input_is_uniform {
        wgpu::BufferBindingType::Uniform
    } else {
        wgpu::BufferBindingType::Storage { read_only: true }
    };
    layout_entries.push(wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: input_buffer_binding_type,
            has_dynamic_offset: false,
            min_binding_size: Some(std::num::NonZeroU64::new(input_struct_buffer.size()).unwrap()),
        },
        count: None,
    });
    for (i, b) in member_buffers.iter().enumerate() {
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: i as u32 + 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: Some(std::num::NonZeroU64::new(b.size()).unwrap()),
            },
            count: None,
        });
    }
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &layout_entries,
    });

    let mut bind_group_entries =
        Vec::<wgpu::BindGroupEntry>::with_capacity(member_buffers.len() + 1);
    bind_group_entries.push(wgpu::BindGroupEntry {
        binding: 0,
        resource: input_struct_buffer.as_entire_binding(),
    });
    for (i, b) in member_buffers.iter().enumerate() {
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: i as u32 + 1,
            resource: b.as_entire_binding(),
        });
    }
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &bind_group_entries,
    });

    (bind_group_layout, bind_group)
}

/// Creates a basic compute pipeline suitable for the sub-examples.
///
/// Note that the entry point will always be "main".
pub fn create_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader_module: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&layout),
        module: shader_module,
        entry_point: "main",
    })
}

pub async fn get_bytes_from_buffer(
    buffer: &wgpu::Buffer,
    staging_buffer: &wgpu::Buffer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Vec<u8> {
    let size_in_buffer = buffer.size();

    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    command_encoder.copy_buffer_to_buffer(buffer, 0, staging_buffer, 0, size_in_buffer);
    queue.submit(Some(command_encoder.finish()));

    let buffer_slice = staging_buffer.slice(..size_in_buffer);

    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
    device.poll(wgpu::Maintain::Wait);
    receiver.receive().await.unwrap().unwrap();

    let output = buffer_slice.get_mapped_range().to_vec();

    staging_buffer.unmap();

    output
}

/// Used for implementations of [`FromWgslBuffers`](super::structs::FromWgslBuffers).
pub fn validate_buffers_for_from_wgsl_bytes<T>(
    buffers_vec: &[&wgpu::Buffer],
    buffer_sizes: &[u64],
    staging_buffer: &wgpu::Buffer,
) {
    if !staging_buffer
        .usage()
        .contains(wgpu::BufferUsages::MAP_READ)
        || !staging_buffer
            .usage()
            .contains(wgpu::BufferUsages::COPY_DST)
    {
        panic!(
            "Staging buffer was did not have the proper usages.
        Needs MAP_READ and COPY_DST."
        );
    }

    if buffers_vec.len() < buffer_sizes.len() {
        panic!(
            "Vec of input buffers {:?} did not contain enough buffers to \
        construct struct {}",
            buffers_vec,
            std::any::type_name::<T>()
        );
    }
    if buffers_vec.len() > buffer_sizes.len() {
        log::warn!(
            "Input buffers vec had more buffers than was necessary to \
        construct struct {}.",
            std::any::type_name::<T>()
        );
    }

    for (i, s) in buffer_sizes.iter().enumerate() {
        if *s != buffers_vec[i].size() {
            panic!(
                "Struct {} expected buffer {:?} to have a size of {} but the actual \
            size was {}.",
                std::any::type_name::<T>(),
                buffers_vec[i],
                s,
                buffers_vec[i].size()
            );
        }
    }
}
