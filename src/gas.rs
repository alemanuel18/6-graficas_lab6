use super::*;

pub(super) fn shade(p: V, t: f32) -> [f32; 3] {
    let storm_center = V::new(0.35, -0.25, 0.90).norm();
    let d = (p - storm_center).len();
    let influence = 1. - smooth(0.16, 0.58, d);
    let angle = influence * 2.3;
    let q = V::new(
        p.x * angle.cos() - p.y * angle.sin(),
        p.x * angle.sin() + p.y * angle.cos(),
        p.z,
    );
    let turbulence = fbm(q * 7. + V::new(t * 0.1, 0., 0.));
    let latitude = q.y
        + (0.045 + 0.085 * turbulence) * (q.x * 9. + t * 0.23).sin()
        + 0.09 * (turbulence - 0.5);
    let bands = (latitude * 19. + 2.3 * fbm(q * 13.)).sin() * 0.5 + 0.5;
    let broad = (latitude * 5. + fbm(q * 4.) * 2.).sin() * 0.5 + 0.5;
    let c = palette(
        [0.27, 0.12, 0.19],
        [0.95, 0.56, 0.30],
        smooth(0.17, 0.82, bands),
    );
    let c = palette(c, [0.91, 0.79, 0.58], broad * 0.44);
    let vortex = smooth(0.1, 0.4, influence) * (0.5 + 0.5 * (d * 50. - t * 0.7).sin());
    let c = palette(c, [0.93, 0.68, 0.48], vortex * 0.55);
    let shade = 0.34 + 0.66 * light(p);
    [c[0] * shade, c[1] * shade, c[2] * shade]
}
