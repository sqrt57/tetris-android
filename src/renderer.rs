//! wgpu surface bound to the current ANativeWindow. Owns everything that must be
//! torn down and rebuilt across Android's surface-destroyed/recreated lifecycle.

use crate::game::{Game, Kind, BOARD_HEIGHT, BOARD_WIDTH};
use ndk::native_window::NativeWindow;
use raw_window_handle::{
    AndroidDisplayHandle, DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
    RawDisplayHandle, WindowHandle,
};
use wgpu::util::DeviceExt;

/// `ndk::NativeWindow` only implements `HasWindowHandle`. wgpu's surface target
/// also needs `HasDisplayHandle`; on Android that handle carries no data, so this
/// just supplies the empty `AndroidDisplayHandle` marker alongside the real window handle.
struct SurfaceWindow(NativeWindow);

impl HasWindowHandle for SurfaceWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for SurfaceWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Android(AndroidDisplayHandle::new()))
        })
    }
}

/// A unit-square corner; the vertex shader scales/offsets it per instance.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    unit: [f32; 2],
}

const QUAD_VERTICES: [QuadVertex; 6] = [
    QuadVertex { unit: [0.0, 0.0] },
    QuadVertex { unit: [1.0, 0.0] },
    QuadVertex { unit: [0.0, 1.0] },
    QuadVertex { unit: [1.0, 0.0] },
    QuadVertex { unit: [1.0, 1.0] },
    QuadVertex { unit: [0.0, 1.0] },
];

/// One rectangle in clip space (bottom-left `offset` + `size`), fully resolved
/// on the CPU each frame so the shader needs no projection uniform.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    offset: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
}

fn kind_color(kind: Kind) -> [f32; 4] {
    match kind {
        Kind::I => [0.0, 0.9, 0.9, 1.0],
        Kind::O => [0.9, 0.9, 0.0, 1.0],
        Kind::T => [0.6, 0.0, 0.8, 1.0],
        Kind::S => [0.0, 0.8, 0.0, 1.0],
        Kind::Z => [0.9, 0.0, 0.0, 1.0],
        Kind::J => [0.0, 0.3, 0.9, 1.0],
        Kind::L => [0.9, 0.5, 0.0, 1.0],
    }
}

const PANEL_COLOR: [f32; 4] = [0.1, 0.1, 0.16, 1.0];
/// Panel tint while the high-score name prompt has the soft keyboard up.
const ENTRY_PANEL_COLOR: [f32; 4] = [0.18, 0.12, 0.05, 1.0];
const PIP_COLOR: [f32; 4] = [0.95, 0.85, 0.3, 1.0];
/// Fraction of a cell's size left as a gap on each side, so locked/falling
/// blocks read as a grid rather than a solid mass.
const CELL_INSET: f32 = 0.06;
/// Name entry is capped at this many visualized characters (see MAX_LEN in
/// text_entry.rs); extra typed characters just don't grow the pip row further.
const MAX_NAME_PIPS: usize = 12;

/// Maps board layout (in pixels, top-left origin, y-down) to clip space
/// (bottom-left origin, y-up) for a screen of `screen_w` x `screen_h` pixels.
struct BoardLayout {
    board_left_px: f32,
    board_top_px: f32,
    cell_px: f32,
    screen_w: f32,
    screen_h: f32,
}

impl BoardLayout {
    fn new(screen_w: u32, screen_h: u32) -> Self {
        let screen_w = screen_w.max(1) as f32;
        let screen_h = screen_h.max(1) as f32;
        let cell_px = (screen_w / BOARD_WIDTH as f32).min(screen_h / BOARD_HEIGHT as f32);
        let board_w_px = cell_px * BOARD_WIDTH as f32;
        let board_h_px = cell_px * BOARD_HEIGHT as f32;
        BoardLayout {
            board_left_px: (screen_w - board_w_px) / 2.0,
            board_top_px: (screen_h - board_h_px) / 2.0,
            cell_px,
            screen_w,
            screen_h,
        }
    }

    fn px_to_clip(&self, x_px: f32, y_px: f32) -> [f32; 2] {
        [x_px / self.screen_w * 2.0 - 1.0, 1.0 - y_px / self.screen_h * 2.0]
    }

    /// Instance rect for board cell (col, row), inset by `inset_frac` of a cell on each side.
    fn cell_instance(&self, col: i32, row: i32, inset_frac: f32, color: [f32; 4]) -> Instance {
        let inset = self.cell_px * inset_frac;
        let x0_px = self.board_left_px + col as f32 * self.cell_px + inset;
        let x1_px = self.board_left_px + (col + 1) as f32 * self.cell_px - inset;
        let y0_px = self.board_top_px + row as f32 * self.cell_px + inset;
        let y1_px = self.board_top_px + (row + 1) as f32 * self.cell_px - inset;

        // Pixel y grows downward, clip-space y grows upward, so the pixel
        // *bottom* (y1_px) becomes the clip-space rect's bottom-left origin.
        let bottom_left = self.px_to_clip(x0_px, y1_px);
        let top_right = self.px_to_clip(x1_px, y0_px);
        Instance {
            offset: bottom_left,
            size: [top_right[0] - bottom_left[0], top_right[1] - bottom_left[1]],
            color,
        }
    }

    fn panel_instance(&self, color: [f32; 4]) -> Instance {
        let top_left = self.px_to_clip(self.board_left_px, self.board_top_px);
        let bottom_right = self.px_to_clip(
            self.board_left_px + self.cell_px * BOARD_WIDTH as f32,
            self.board_top_px + self.cell_px * BOARD_HEIGHT as f32,
        );
        Instance {
            offset: [top_left[0], bottom_right[1]],
            size: [bottom_right[0] - top_left[0], top_left[1] - bottom_right[1]],
            color,
        }
    }

    /// One indicator square per typed character, in a row in the top margin
    /// above the board (stands in for rendering actual glyphs, which this
    /// renderer has no font pipeline for).
    fn pip_instance(&self, index: usize, color: [f32; 4]) -> Instance {
        let pip_size = self.cell_px * 0.3;
        let gap = pip_size * 0.3;
        let x0_px = self.board_left_px + index as f32 * (pip_size + gap);
        let top_px = (self.board_top_px - gap - pip_size).max(0.0);
        let bottom_px = top_px + pip_size;

        let bottom_left = self.px_to_clip(x0_px, bottom_px);
        let top_right = self.px_to_clip(x0_px + pip_size, top_px);
        Instance {
            offset: bottom_left,
            size: [top_right[0] - bottom_left[0], top_right[1] - bottom_left[1]],
            color,
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(window: NativeWindow, width: u32, height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::default();

        let surface = instance
            .create_surface(SurfaceWindow(window))
            .map_err(|e| format!("create_surface failed: {e}"))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or("no suitable GPU adapter found")?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .map_err(|e| format!("request_device failed: {e}"))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("board_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("board_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x4],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("board_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout, instance_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Renderer { surface, device, queue, config, pipeline, quad_vertex_buffer })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Draws the board panel, locked cells, and the current falling piece.
    /// `name_entry_chars`, when `Some`, means the high-score name prompt is
    /// active: the panel is tinted and one pip is drawn per typed character
    /// instead of the (frozen) falling piece.
    pub fn render(&mut self, game: &Game, name_entry_chars: Option<usize>) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(err) => {
                log::error!("get_current_texture failed: {err}");
                return;
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let layout = BoardLayout::new(self.config.width, self.config.height);
        let mut instances = Vec::with_capacity(1 + BOARD_WIDTH * BOARD_HEIGHT / 2);
        let panel_color = if name_entry_chars.is_some() { ENTRY_PANEL_COLOR } else { PANEL_COLOR };
        instances.push(layout.panel_instance(panel_color));
        for (row, cells) in game.board.iter().enumerate() {
            for (col, cell) in cells.iter().enumerate() {
                if let Some(kind) = cell {
                    instances.push(layout.cell_instance(
                        col as i32,
                        row as i32,
                        CELL_INSET,
                        kind_color(*kind),
                    ));
                }
            }
        }
        if !game.game_over {
            for (x, y) in game.current.cells() {
                if y >= 0 {
                    instances.push(layout.cell_instance(
                        x,
                        y,
                        CELL_INSET,
                        kind_color(game.current.kind),
                    ));
                }
            }
        }
        if let Some(count) = name_entry_chars {
            for i in 0..count.min(MAX_NAME_PIPS) {
                instances.push(layout.pip_instance(i, PIP_COLOR));
            }
        }

        let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("board"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.04, g: 0.04, b: 0.08, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.draw(0..QUAD_VERTICES.len() as u32, 0..instances.len() as u32);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
