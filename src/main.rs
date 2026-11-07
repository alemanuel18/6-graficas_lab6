use raylib::prelude::*;
use std::{
    env, fs,
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
};
static LIGHT_X: AtomicU32 = AtomicU32::new((-0.52_f32).to_bits());
static LIGHT_Y: AtomicU32 = AtomicU32::new((0.35_f32).to_bits());
static LIGHT_Z: AtomicU32 = AtomicU32::new((0.78_f32).to_bits());
fn set_light_angle(a: f32) {
    let d = V::new(
        -0.52 * a.cos() + 0.78 * a.sin(),
        0.35,
        0.52 * a.sin() + 0.78 * a.cos(),
    )
    .norm();
    LIGHT_X.store(d.x.to_bits(), Ordering::Relaxed);
    LIGHT_Y.store(d.y.to_bits(), Ordering::Relaxed);
    LIGHT_Z.store(d.z.to_bits(), Ordering::Relaxed);
}
fn light_dir() -> V {
    V::new(
        f32::from_bits(LIGHT_X.load(Ordering::Relaxed)),
        f32::from_bits(LIGHT_Y.load(Ordering::Relaxed)),
        f32::from_bits(LIGHT_Z.load(Ordering::Relaxed)),
    )
}

const W: usize = 600;
const H: usize = 600;
#[derive(Clone, Copy, Debug, Default)]
struct V {
    x: f32,
    y: f32,
    z: f32,
}
impl V {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    fn norm(self) -> Self {
        let l = self.len().max(1e-8);
        self / l
    }
}
impl std::ops::Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl std::ops::Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl std::ops::Mul<f32> for V {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}
impl std::ops::Div<f32> for V {
    type Output = Self;
    fn div(self, s: f32) -> Self {
        self * (1. / s)
    }
}
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0., 1.)
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b1)
        ^ (y as u32).wrapping_mul(0x85ebca77)
        ^ (z as u32).wrapping_mul(0xc2b2ae3d);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    n as f32 / u32::MAX as f32
}
fn noise(p: V) -> f32 {
    let x = p.x.floor() as i32;
    let y = p.y.floor() as i32;
    let z = p.z.floor() as i32;
    let u = smooth(0., 1., p.x - x as f32);
    let v = smooth(0., 1., p.y - y as f32);
    let w = smooth(0., 1., p.z - z as f32);
    let a = mix(hash(x, y, z), hash(x + 1, y, z), u);
    let b = mix(hash(x, y + 1, z), hash(x + 1, y + 1, z), u);
    let c = mix(hash(x, y, z + 1), hash(x + 1, y, z + 1), u);
    let d = mix(hash(x, y + 1, z + 1), hash(x + 1, y + 1, z + 1), u);
    mix(mix(a, b, v), mix(c, d, v), w)
}
fn fbm(p: V) -> f32 {
    let mut sum = 0.;
    let mut amp = 0.5;
    let mut q = p;
    for _ in 0..5 {
        sum += amp * noise(q);
        q = q * 2.03 + V::new(11.3, 4.7, 8.1);
        amp *= 0.5;
    }
    sum / 0.96875
}
fn palette(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [mix(a[0], b[0], t), mix(a[1], b[1], t), mix(a[2], b[2], t)]
}
fn light(n: V) -> f32 {
    n.dot(light_dir()).max(0.)
}

mod gas;
mod rocky;
mod star;

fn shade(kind: usize, p: V, t: f32) -> [u8; 4] {
    let c = match kind {
        0 => star::shade(p, t),
        1 => rocky::shade(p, t),
        _ => gas::shade(p, t),
    };
    [
        (c[0].clamp(0., 1.) * 255.) as u8,
        (c[1].clamp(0., 1.) * 255.) as u8,
        (c[2].clamp(0., 1.) * 255.) as u8,
        255,
    ]
}
struct Mesh {
    v: Vec<V>,
    f: Vec<[usize; 3]>,
}
fn load_mesh() -> Mesh {
    let src = include_str!("../sphere.obj");
    let mut v = Vec::new();
    let mut f = Vec::new();
    for line in src.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.first() == Some(&"v") {
            v.push(V::new(
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ));
        } else if parts.first() == Some(&"f") {
            let a: Vec<_> = parts[1..]
                .iter()
                .map(|x| x.split('/').next().unwrap().parse::<usize>().unwrap() - 1)
                .collect();
            for j in 1..a.len() - 1 {
                f.push([a[0], a[j], a[j + 1]]);
            }
        }
    }
    Mesh { v, f }
}
fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0)
}
fn render(mesh: &Mesh, kind: usize, t: f32) -> Vec<u8> {
    let mut pixels = vec![0u8; W * H * 4];
    let mut depth = vec![f32::NEG_INFINITY; W * H];
    for y in 0..H {
        for x in 0..W {
            let i = (y * W + x) * 4;
            let r = ((x as f32 - W as f32 * 0.5).powi(2) + (y as f32 - H as f32 * 0.5).powi(2))
                .sqrt()
                / (W as f32 * 0.7);
            let glow = if kind == 0 {
                (0.07 * (1. - r).max(0.).powf(5.)).min(0.08)
            } else {
                0.
            };
            pixels[i] = (4.0 + glow * 140.) as u8;
            pixels[i + 1] = (7.0 + glow * 55.) as u8;
            pixels[i + 2] = (18.0 + glow * 20.) as u8;
            pixels[i + 3] = 255;
        }
    }
    let scale = 260.;
    let project = |p: V| (W as f32 * 0.5 + p.x * scale, H as f32 * 0.5 - p.y * scale);
    for face in &mesh.f {
        let a = mesh.v[face[0]];
        let b = mesh.v[face[1]];
        let c = mesh.v[face[2]];
        if a.z + b.z + c.z < 0. {
            continue;
        }
        let pa = project(a);
        let pb = project(b);
        let pc = project(c);
        let area = edge(pa, pb, pc);
        if area.abs() < 1e-5 {
            continue;
        }
        let x0 = pa.0.min(pb.0).min(pc.0).floor().max(0.) as usize;
        let x1 = pa.0.max(pb.0).max(pc.0).ceil().min((W - 1) as f32) as usize;
        let y0 = pa.1.min(pb.1).min(pc.1).floor().max(0.) as usize;
        let y1 = pa.1.max(pb.1).max(pc.1).ceil().min((H - 1) as f32) as usize;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let u = edge(pb, pc, p) / area;
                let v = edge(pc, pa, p) / area;
                let w = 1. - u - v;
                if u < 0. || v < 0. || w < 0. {
                    continue;
                }
                let pos = a * u + b * v + c * w;
                let idx = y * W + x;
                if pos.z <= depth[idx] {
                    continue;
                }
                depth[idx] = pos.z;
                pixels[idx * 4..idx * 4 + 4].copy_from_slice(&shade(kind, pos.norm(), t));
            }
        }
    }
    pixels
}
// PNG encoder with uncompressed DEFLATE blocks; no image dependency or display required.
fn crc(bytes: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in bytes {
        c ^= b as u32;
        for _ in 0..8 {
            c = (c >> 1) ^ if c & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    !c
}
fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc(&out[start..]).to_be_bytes());
}
fn png(path: &Path, pixels: &[u8]) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(H * (W * 4 + 1));
    for row in pixels.chunks_exact(W * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let mut pos = 0;
    while pos < raw.len() {
        let n = (raw.len() - pos).min(65535);
        z.push(if pos + n == raw.len() { 1 } else { 0 });
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice((!(n as u16)).to_le_bytes().as_slice());
        z.extend_from_slice(&raw[pos..pos + n]);
        pos += n;
    }
    let mut a = 1u32;
    let mut b = 0u32;
    for &x in &raw {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&(W as u32).to_be_bytes());
    header.extend_from_slice(&(H as u32).to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    fs::write(path, out)
}
fn select_body(current: usize, one: bool, two: bool, three: bool) -> usize {
    if three {
        2
    } else if two {
        1
    } else if one {
        0
    } else {
        current
    }
}
fn main() {
    let mesh = load_mesh();
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--capture") {
        fs::create_dir_all("docs").unwrap();
        for (i, name) in ["estrella", "rocoso", "gaseoso"].iter().enumerate() {
            png(
                Path::new(&format!("docs/{name}.png")),
                &render(&mesh, i, 0.),
            )
            .unwrap();
        }
        println!("Capturas creadas en docs/");
        return;
    }
    if args.iter().any(|a| a == "--frames") {
        for (i, name) in ["estrella", "rocoso", "gaseoso", "luz"].iter().enumerate() {
            let dir = format!("docs/frames/{name}");
            fs::create_dir_all(&dir).unwrap();
            for frame in 0..24 {
                set_light_angle(if i == 3 {
                    frame as f32 * std::f32::consts::TAU / 24.
                } else {
                    0.
                });
                let kind = if i == 3 { 1 } else { i };
                let time = if i == 3 { 0. } else { frame as f32 * 0.35 };
                png(
                    Path::new(&format!("{dir}/{frame:03}.png")),
                    &render(&mesh, kind, time),
                )
                .unwrap();
            }
        }
        println!("Fotogramas creados en docs/frames/");
        return;
    }
    let (mut rl, thread) = raylib::init()
        .size(W as i32, H as i32)
        .title("Laboratorio: mundos procedurales")
        .build();
    rl.set_target_fps(30);
    let image = Image::gen_image_color(W as i32, H as i32, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image).unwrap();
    let mut kind = 0;
    let mut t = 0.;
    let mut paused = false;
    let mut light_angle = 0.;
    while !rl.window_should_close() {
        kind = select_body(
            kind,
            rl.is_key_down(KeyboardKey::KEY_ONE) || rl.is_key_down(KeyboardKey::KEY_KP_1),
            rl.is_key_down(KeyboardKey::KEY_TWO) || rl.is_key_down(KeyboardKey::KEY_KP_2),
            rl.is_key_down(KeyboardKey::KEY_THREE) || rl.is_key_down(KeyboardKey::KEY_KP_3),
        );
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
        }
        if !paused {
            t += rl.get_frame_time();
        }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            light_angle -= 0.24 + rl.get_frame_time() * 2.;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            light_angle += 0.24 + rl.get_frame_time() * 2.;
        }
        set_light_angle(light_angle);
        let pixels = render(&mesh, kind, t);
        texture.update_texture(&pixels).unwrap();
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            let name = ["estrella", "rocoso", "gaseoso"][kind];
            png(Path::new(&format!("{name}.png")), &pixels).unwrap();
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_text(
            &format!(
                "{}  |  1/2/3 seleccionar  ESPACIO pausa  FLECHAS luz  S captura",
                ["ESTRELLA", "ROCOSO", "GASEOSO"][kind]
            ),
            12,
            12,
            16,
            Color::WHITE,
        );
        d.draw_text(
            &format!(
                "Luz: {:.0} grados{}",
                light_angle.to_degrees().rem_euclid(360.),
                if kind == 0 { " (estrella emisiva)" } else { "" }
            ),
            12,
            34,
            16,
            Color::WHITE,
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mesh_is_shared_and_valid() {
        let m = load_mesh();
        assert!(!m.v.is_empty());
        assert!(m.f.len() > 100);
        assert!(m.f.iter().all(|f| f.iter().all(|&i| i < m.v.len())));
    }
    #[test]
    fn shaders_produce_distinct_pixels() {
        let p = V::new(0.3, 0.2, 0.93).norm();
        let colors = [shade(0, p, 0.), shade(1, p, 0.), shade(2, p, 0.)];
        assert_ne!(colors[0], colors[1]);
        assert_ne!(colors[1], colors[2]);
    }
    #[test]
    fn animation_changes_material() {
        let p = V::new(0.3, 0.2, 0.93).norm();
        assert_ne!(shade(0, p, 0.), shade(0, p, 7.));
    }
    #[test]
    fn third_key_selects_gas() {
        assert_eq!(select_body(0, false, false, true), 2);
        assert_eq!(select_body(2, false, false, false), 2);
    }
    #[test]
    fn arrows_change_planet_lighting() {
        let p = V::new(0.3, 0.2, 0.93).norm();
        set_light_angle(0.);
        let rocky_before = shade(1, p, 0.);
        let gas_before = shade(2, p, 0.);
        set_light_angle(1.5);
        assert_ne!(rocky_before, shade(1, p, 0.));
        assert_ne!(gas_before, shade(2, p, 0.));
        set_light_angle(0.);
    }
    #[test]
    fn renderer_fills_sphere() {
        let img = render(&load_mesh(), 0, 0.);
        assert_ne!(
            &img[(H / 2 * W + W / 2) * 4..(H / 2 * W + W / 2) * 4 + 3],
            &img[0..3]
        );
    }
}
