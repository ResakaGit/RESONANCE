//! Dispatch compute mínimo: índice de paleta WGSL vs `equations::quantized_palette_index`.
//! Solo se compila con `--features gpu_cell_field_snapshot` (ver `Cargo.toml` `[[test]]`).

use std::mem::size_of;
use std::path::Path;
use std::sync::Arc;

use resonance::blueprint::equations::quantized_palette_index;
use resonance::worldgen::CELL_FIELD_SNAPSHOT_WGSL_PATH;
use wgpu::util::DeviceExt;

// --- Constantes del harness (buffers / workgroup), no duplicar en producción ---

const PALETTE_DISPATCH_ENTRY: &str = "palette_dispatch";
const WORKGROUP_SIZE: u32 = 1;
const PALETTE_DISPATCH_WORDS_BYTES: usize = 16;
const READBACK_STAGING_BYTES: u64 = 16;
const OUTPUT_STORAGE_BYTES: usize = 16;

fn palette_dispatch_words_bytes(
    enorm: f32,
    rho: f32,
    n_max: u32,
) -> [u8; PALETTE_DISPATCH_WORDS_BYTES] {
    let mut b = [0u8; PALETTE_DISPATCH_WORDS_BYTES];
    b[0..4].copy_from_slice(&enorm.to_bits().to_ne_bytes());
    b[4..8].copy_from_slice(&rho.to_bits().to_ne_bytes());
    b[8..12].copy_from_slice(&n_max.to_ne_bytes());
    b[12..16].copy_from_slice(&0u32.to_ne_bytes());
    b
}

/// wgpu pesado creado una vez; cada `dispatch` arma buffers pequeños (evita god-function async).
struct PaletteDispatchHarness {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: Arc<wgpu::ComputePipeline>,
}

impl PaletteDispatchHarness {
    async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter gráfico");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("device");
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CELL_FIELD_SNAPSHOT_WGSL_PATH);
        let shader_src =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("leer WGSL {:?}: {e}", path));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cell_field_snapshot"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });
        let pipeline = Arc::new(
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("palette_dispatch"),
                layout: None,
                module: &shader,
                entry_point: Some(PALETTE_DISPATCH_ENTRY),
                compilation_options: Default::default(),
                cache: None,
            }),
        );

        Self {
            device,
            queue,
            pipeline,
        }
    }

    fn dispatch(&self, enorm: f32, rho: f32, n_max: u32) -> u32 {
        pollster::block_on(self.dispatch_async(enorm, rho, n_max))
    }

    async fn dispatch_async(&self, enorm: f32, rho: f32, n_max: u32) -> u32 {
        let in_bytes = palette_dispatch_words_bytes(enorm, rho, n_max);
        let in_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("palette_in"),
                contents: &in_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let out_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("palette_out"),
                contents: &[0u8; OUTPUT_STORAGE_BYTES],
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: in_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: out_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("palette_dispatch_enc"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("palette_dispatch_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(WORKGROUP_SIZE, WORKGROUP_SIZE, WORKGROUP_SIZE);
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: READBACK_STAGING_BYTES,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("copy_out"),
            });
        encoder.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, READBACK_STAGING_BYTES);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            tx.send(r).expect("map callback");
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().expect("recv").expect("map_async");
        let idx = {
            let data = slice.get_mapped_range();
            let bytes: [u8; 4] = data[..size_of::<u32>()].try_into().expect("u32");
            u32::from_ne_bytes(bytes)
        };
        staging.unmap();
        idx
    }
}

#[test]
fn gpu_quantized_palette_index_matches_equations_golden() {
    let harness = pollster::block_on(PaletteDispatchHarness::new());
    let enorm = 0.52_f32;
    let rho = 0.1_f32;
    let n_max = 100_u32;
    let expected = quantized_palette_index(enorm, rho, n_max);
    let got = harness.dispatch(enorm, rho, n_max);
    assert_eq!(got, expected, "WGSL vs CPU (golden equations test)");
}

#[test]
fn gpu_quantized_palette_index_sweep_matches_cpu() {
    let harness = pollster::block_on(PaletteDispatchHarness::new());
    let cases = [
        (0.0_f32, 1.0_f32, 64_u32),
        (1.0_f32, 0.12_f32, 32_u32),
        (0.7_f32, 0.5_f32, 2_u32),
        (f32::NAN, 0.5_f32, 50_u32),
        (f32::INFINITY, 1.0_f32, 80_u32),
        (f32::NEG_INFINITY, 0.8_f32, 60_u32),
        (0.5_f32, f32::NAN, 40_u32),
        (0.5_f32, f32::INFINITY, 40_u32),
        (0.5_f32, f32::NEG_INFINITY, 40_u32),
        (f32::MAX, 1.0_f32, 100_u32),
        (f32::MIN_POSITIVE, 1.0_f32, 16_u32),
    ];
    for &(e, r, n) in &cases {
        let want = quantized_palette_index(e, r, n);
        let got = harness.dispatch(e, r, n);
        assert_eq!(got, want, "mismatch enorm={e} rho={r} n_max={n}");
    }
}
