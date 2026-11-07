use super::*;

pub(super) fn shade(p: V, t: f32) -> [f32; 3] {
    let n = fbm(p * 6. + V::new(t * 0.14, -t * 0.08, t * 0.05));
    let fine = fbm(p * 19. - V::new(t * 0.34, 0., t * 0.15));
    let cells = n * 0.75 + fine * 0.25;
    let spot = 1. - smooth(0.31, 0.43, cells);
    let filament = ((p.x * 17. + p.y * 12. + n * 10. + t * 0.5).sin() * 0.5 + 0.5) * fine;
    let heat = (0.48 + cells * 0.48 + filament * 0.16 - spot * 0.52).clamp(0., 1.);
    let c = if heat < 0.5 {
        palette([0.24, 0.018, 0.035], [1., 0.19, 0.025], heat * 2.)
    } else {
        palette([1., 0.19, 0.025], [1., 0.93, 0.53], (heat - 0.5) * 2.)
    };
    palette(c, [0.12, 0.018, 0.035], spot * 0.72)
}
