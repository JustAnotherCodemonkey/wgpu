pub fn example_create_bind_group(
    device: &wgpu::Device,
    input_struct_buffer: &wgpu::Buffer,
    member_buffers: &[&wgpu::Buffer],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let mut layout_entries =
        Vec::<wgpu::BindGroupLayoutEntry>::with_capacity(member_buffers.len() + 1);
    layout_entries.push(wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
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