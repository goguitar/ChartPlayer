//! ChartPlayer - Main entry point for GUI

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use chart_player::{init, AudioOutput, PlayerScene3D, SceneVertex, SongInstrumentType, SongPlayer, SpriteFontDefinition, SpriteFontGlyph, SpriteLibrary, SpriteRegion, VERSION};
use image::GenericImageView;
use wgpu::util::DeviceExt;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.tex_coords = input.tex_coords;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas_texture, atlas_sampler, input.tex_coords);
    return texel * input.color;
}
"#;

struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    atlas_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    scenes: Vec<PlayerScene3D>,
    current_scene: usize,
    sprites: SpriteLibrary,
}

enum RenderResult {
    Rendered,
    SkipFrame,
    Reconfigure,
    SurfaceLost,
}

impl Renderer {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .context("failed to create rendering surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("failed to find a compatible GPU adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("chart-player-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("failed to create device")?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(surface_caps.present_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (sprites, atlas_view, atlas_sampler) = load_chartplayer_atlas(&device, &queue)?;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("chart-player-atlas-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chart-player-atlas-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chart-player-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("chart-player-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chart-player-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[scene_vertex_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let empty_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chart-player-vertices"),
            contents: bytemuck::cast_slice(&[SceneVertex {
                position: [0.0, 0.0],
                color: [0.0, 0.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let fret_player = SongPlayer::shared_demo_for(SongInstrumentType::LeadGuitar);
        let drum_player = SongPlayer::shared_demo_for(SongInstrumentType::Drums);
        let keys_player = SongPlayer::shared_demo_for(SongInstrumentType::Keys);

        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            atlas_bind_group,
            vertex_buffer: empty_vertex_buffer,
            vertex_count: 0,
            scenes: vec![
                PlayerScene3D::Fret(chart_player::FretPlayerScene3D::new(fret_player)),
                PlayerScene3D::Drum(chart_player::DrumPlayerScene3D::new(drum_player)),
                PlayerScene3D::Keys(chart_player::KeysPlayerScene3D::new(keys_player)),
            ],
            current_scene: 0,
            sprites,
        };
        renderer.update_geometry();
        Ok(renderer)
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            self.size = size;
            return;
        }

        self.size = size;
        self.reconfigure();
    }

    fn reconfigure(&mut self) {
        if self.size.width == 0 || self.size.height == 0 {
            return;
        }

        self.config.width = self.size.width;
        self.config.height = self.size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn select_scene(&mut self, index: usize) {
        if index < self.scenes.len() {
            self.current_scene = index;
        }
    }

    fn current_player_handle(&self) -> Option<chart_player::SharedSongPlayer> {
        self.scenes[self.current_scene].player_handle()
    }

    fn scene_name(&self) -> &'static str {
        self.scenes[self.current_scene].name()
    }

    fn update_geometry(&mut self) {
        let time = self.scenes[self.current_scene].current_time_seconds();
        let vertices = self.scenes[self.current_scene].build_vertices(time, self.size.width, self.size.height, &self.sprites);
        self.vertex_count = vertices.len() as u32;
        self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chart-player-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    }

    fn render(&mut self) -> RenderResult {
        if self.size.width == 0 || self.size.height == 0 {
            return RenderResult::SkipFrame;
        }

        self.update_geometry();

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return RenderResult::SkipFrame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Validation => return RenderResult::Reconfigure,
            wgpu::CurrentSurfaceTexture::Lost => return RenderResult::SurfaceLost,
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("chart-player-encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("chart-player-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.015,
                            g: 0.020,
                            b: 0.035,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        RenderResult::Rendered
    }
}

fn scene_vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SceneVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    }
}

fn load_chartplayer_atlas(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(SpriteLibrary, wgpu::TextureView, wgpu::Sampler)> {
    let manifest_path = Path::new("/home/csantz/ChartPlayer/ChartPlayerShared/Content/Textures/ImageManifest.xml");
    let atlas_path = Path::new("/home/csantz/ChartPlayer/ChartPlayerShared/Content/Textures/UISheet0.png");

    let manifest = fs::read_to_string(manifest_path).with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let atlas = image::open(atlas_path).with_context(|| format!("failed to open {}", atlas_path.display()))?;
    let rgba = atlas.to_rgba8();
    let (width, height) = atlas.dimensions();

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("chart-player-atlas"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("chart-player-atlas-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    Ok((parse_manifest(&manifest)?, view, sampler))
}

fn parse_manifest(manifest: &str) -> Result<SpriteLibrary> {
    let sheet_width: f32 = extract_tag(manifest, "SheetWidth")?
        .parse()
        .context("invalid SheetWidth")?;
    let sheet_height: f32 = extract_tag(manifest, "SheetHeight")?
        .parse()
        .context("invalid SheetHeight")?;

    let mut sprites = SpriteLibrary::new();
    for block in manifest.split("<ImageManifestSheetImage>").skip(1) {
        let Some((entry, _)) = block.split_once("</ImageManifestSheetImage>") else {
            continue;
        };

        let name = extract_tag(entry, "ImageName")?;
        let x: f32 = extract_tag(entry, "XOffset")?.parse().with_context(|| format!("invalid XOffset for {name}"))?;
        let y: f32 = extract_tag(entry, "YOffset")?.parse().with_context(|| format!("invalid YOffset for {name}"))?;
        let width: u32 = extract_tag(entry, "Width")?.parse().with_context(|| format!("invalid Width for {name}"))?;
        let height: u32 = extract_tag(entry, "Height")?.parse().with_context(|| format!("invalid Height for {name}"))?;

        sprites.insert(
            name,
            SpriteRegion {
                u0: x / sheet_width,
                v0: y / sheet_height,
                u1: (x + width as f32) / sheet_width,
                v1: (y + height as f32) / sheet_height,
                width,
                height,
            },
        );
    }

    for block in manifest.split("<SpriteFontDefinition>").skip(1) {
        let Some((entry, _)) = block.split_once("</SpriteFontDefinition>") else {
            continue;
        };

        let name = extract_tag(entry, "Name")?;
        let line_height: f32 = extract_tag(entry, "LineHeight")?
            .parse()
            .with_context(|| format!("invalid LineHeight for {name}"))?;
        let mut font = SpriteFontDefinition::new(line_height, 0.0);

        for glyph_block in entry.split("<SpriteFontGlyph>").skip(1) {
            let Some((glyph_entry, _)) = glyph_block.split_once("</SpriteFontGlyph>") else {
                continue;
            };

            let character_code: u32 = extract_tag(glyph_entry, "Character")?
                .parse()
                .with_context(|| format!("invalid Character for {name}"))?;
            let Some(character) = char::from_u32(character_code) else {
                continue;
            };
            let x: f32 = extract_tag(glyph_entry, "X")?
                .parse()
                .with_context(|| format!("invalid glyph X for {name}"))?;
            let y: f32 = extract_tag(glyph_entry, "Y")?
                .parse()
                .with_context(|| format!("invalid glyph Y for {name}"))?;
            let width: u32 = extract_tag(glyph_entry, "Width")?
                .parse()
                .with_context(|| format!("invalid glyph Width for {name}"))?;
            let height: u32 = extract_tag(glyph_entry, "Height")?
                .parse()
                .with_context(|| format!("invalid glyph Height for {name}"))?;

            font.insert_glyph(
                character,
                SpriteFontGlyph {
                    region: SpriteRegion {
                        u0: x / sheet_width,
                        v0: y / sheet_height,
                        u1: (x + width as f32) / sheet_width,
                        v1: (y + height as f32) / sheet_height,
                        width,
                        height,
                    },
                },
            );
        }

        sprites.insert_font(name, font);
    }

    Ok(sprites)
}

fn extract_tag(contents: &str, tag: &str) -> Result<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let (_, after_start) = contents
        .split_once(start_tag.as_str())
        .with_context(|| format!("missing tag {tag}"))?;
    let (value, _) = after_start
        .split_once(end_tag.as_str())
        .with_context(|| format!("missing end tag {tag}"))?;
    Ok(value.trim().to_string())
}

fn main() -> Result<()> {
    init();
    println!("ChartPlayer v{}\n", VERSION);

    let event_loop = EventLoop::new().context("failed to create event loop")?;
    let window = Arc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("CHARTPLAYER")
                    .with_inner_size(LogicalSize::new(1280.0, 720.0)),
            )
            .context("failed to create window")?,
    );

    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    let mut audio_output = renderer
        .current_player_handle()
        .map(AudioOutput::from_shared)
        .transpose()
        .map_err(|err| anyhow::anyhow!(err))?;
    let size = window.inner_size();
    println!("Window: {}x{}", size.width, size.height);
    println!("Running - close window to exit");
    println!("Scene keys: 1=fretboard 2=drums 3=keys\n");
    window.set_title(&format!("CHARTPLAYER - {}", renderer.scene_name()));

    let window_id = window.id();
    event_loop.run(move |event, target| match event {
        Event::WindowEvent {
            window_id: current_window_id,
            event,
        } if current_window_id == window_id => match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Resized(size) => renderer.resize(size),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && !event.repeat {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Digit1) => renderer.select_scene(0),
                        PhysicalKey::Code(KeyCode::Digit2) => renderer.select_scene(1),
                        PhysicalKey::Code(KeyCode::Digit3) => renderer.select_scene(2),
                        _ => {}
                    }
                    audio_output = match renderer.current_player_handle() {
                        Some(player) => match AudioOutput::from_shared(player) {
                            Ok(output) => Some(output),
                            Err(err) => {
                                eprintln!("audio disabled: {err:#}");
                                None
                            }
                        },
                        None => None,
                    };
                    window.set_title(&format!("CHARTPLAYER - {}", renderer.scene_name()));
                }
            }
            WindowEvent::RedrawRequested => match renderer.render() {
                RenderResult::Rendered | RenderResult::SkipFrame => {}
                RenderResult::Reconfigure => renderer.reconfigure(),
                RenderResult::SurfaceLost => target.exit(),
            },
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    })?;

    Ok(())
}
