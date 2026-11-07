use super::*;

pub(super) fn shade(p: V, t: f32) -> [f32; 3] {
    let terrain = fbm(p * 3.8) + 0.17 * fbm(p * 13.);
    let land = smooth(0.48, 0.57, terrain);
    let ridges = (fbm(p * 19.) - 0.5).abs();
    let desert = smooth(0.57, 0.72, terrain + p.y * 0.12);
    let ocean = palette([0.015, 0.08, 0.19], [0.02, 0.35, 0.48], fbm(p * 12.));
    let ground = palette([0.13, 0.29, 0.12], [0.64, 0.42, 0.20], desert);
    let snow = smooth(0.58, 0.82, p.y.abs() + 0.24 * fbm(p * 8.));
    let mut c = palette(ocean, ground, land);
    c = palette(c, [0.84, 0.89, 0.76], snow * (0.45 + 0.55 * land));
    let n = p.norm();
    let l = light(n);
    let bump = (fbm(p * 18. + V::new(0.04, 0., 0.)) - fbm(p * 18. - V::new(0.04, 0., 0.))) * land;
    let diffuse = 0.24 + 0.76 * (l + bump * 0.8 - ridges * 0.12 * land).clamp(0., 1.);
    for v in &mut c {
        *v *= diffuse;
    }
    let spec =
        (n.dot((light_dir() + V::new(0., 0., 1.)).norm()).max(0.)).powf(45.) * (1. - land) * 0.75;
    c = palette(c, [0.65, 0.9, 1.], spec);
    let night = (1. - smooth(0.02, 0.26, l)) * land * (1. - snow);
    let city = smooth(0.67, 0.76, fbm(p * 52.)) * night * 0.85;
    c = palette(c, [1., 0.66, 0.19], city);
    // The cloud field advances along longitude while terrain remains fixed.
    let cp = V::new(
        p.x * (t * 0.14).cos() - p.z * (t * 0.14).sin(),
        p.y,
        p.x * (t * 0.14).sin() + p.z * (t * 0.14).cos(),
    );
    let cloud = smooth(0.59, 0.70, fbm(cp * 9.) + 0.16 * fbm(cp * 25.));
    // Offset sampling places the shadow down-light from the corresponding cloud.
    let shadow_p = (cp + V::new(0.055, -0.037, -0.08)).norm();
    let shadow = smooth(0.59, 0.70, fbm(shadow_p * 9.) + 0.16 * fbm(shadow_p * 25.));
    for v in &mut c {
        *v *= 1. - shadow * 0.26 * land;
    }
    palette(c, [0.82, 0.91, 0.95], cloud * (0.40 + 0.48 * l))
}
