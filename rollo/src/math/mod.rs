use glam::Vec3;

/// move to a position.
///
/// Inspired by https://docs.unity3d.com/ScriptReference/Vector3.MoveTowards.html
pub fn move_towards(current: &Vec3, target: &Vec3, max_distance_delta: f32) -> Vec3 {
    let a = *target - *current;
    let magn = a.length();

    if magn <= max_distance_delta || magn == 0.0 {
        return *target;
    }

    *current + a / magn * max_distance_delta
}
