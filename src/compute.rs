pub async fn run() {
    env_logger::init();
    // todo: get the logger working
    log::info!("SHADER PROJ 🌐");
    print!("Shader proj 😭");
    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::METAL);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();
    let features = adapter.features();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: features & wgpu::Features::TIMESTAMP_QUERY,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Vec3 Bind Group"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: true,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("proj.wgsl").into()),
    });
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        entry_point: "main",
        module: &shader_module,
    });

    const BUFFER_SIZE: u64 = 1000;

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: BUFFER_SIZE,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: true,
    });

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: BUFFER_SIZE,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("A bind group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 1,
            resource: output_buffer.as_entire_binding(),
        }],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute Encoder"),
    });

    {
        let mut pass_encoder = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass Encoder"),
        });

        pass_encoder.set_pipeline(&compute_pipeline);
        pass_encoder.set_bind_group(0, &bind_group, &[]);

        let dispatches: u32 = { BUFFER_SIZE / 64 }.try_into().unwrap();
        pass_encoder.dispatch(dispatches, 1, 1);
    }

    // pass_encoder.
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, BUFFER_SIZE);

    // submit will accept anything that implements IntoIter
    // this is where Rust hands off to the gpu render queue
    queue.submit(std::iter::once(encoder.finish()));

    let buf_slice = staging_buffer.slice(..);
    buf_slice.map_async(wgpu::MapMode::Read).await.unwrap();
    let result_buf = &*buf_slice.get_mapped_range();
    let data: &[f32] = bytemuck::cast_slice(result_buf);
    staging_buffer.unmap();
    println!("data: {:?}", &*data);
}
