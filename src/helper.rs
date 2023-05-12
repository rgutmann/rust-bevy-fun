#![allow(unused)]
use bevy::prelude::Vec3;

#[inline(always)]
pub fn format_vec3f(vec: Vec3) -> String {
    format!("({:>8.3},{:>8.3},{:>8.3})", vec[0], vec[1], vec[2])
}

#[inline(always)]
pub fn println_vec3f(vec: Vec3) {
    println!("({:>8.3},{:>8.3},{:>8.3})", vec[0], vec[1], vec[2]);
}

#[derive(Debug)]
pub struct SimpleTween {
    cur:f32,
    min:f32,
    max:f32,
    inc:f32,
}
impl SimpleTween {
    pub fn new(cur: f32, min: f32, max: f32, inc: f32) -> Self {
        Self { cur, min, max, inc }
    }
    pub fn current_value(&self) -> &f32 {
        &self.cur
    }
    fn apply_times(&mut self, times: isize) {
        self.cur = (self.cur + (times as f32 * self.inc)).max(self.min).min(self.max);
    }
    pub fn increase_once(&mut self) {
        self.apply_times(1);
    }
    pub fn decrease_once(&mut self) {
        self.apply_times(-1);
    }
}

#[derive(Debug)]
pub struct VelocityTween {
    cur:Vec3,
    min:f32,
    max:f32,
    inc:f32,
}
impl VelocityTween {
    pub fn new(cur: Vec3, min: f32, max: f32, inc: f32) -> Self {
        Self { cur, min, max, inc }
    }
    /// Returns the current velocity.
    pub fn current_velocity(&self) -> &Vec3 {
        &self.cur
    }
    /// Adds the given direction vector * delta_seconds * increase_step_length to the current velocity vector,
    /// but also respoects the minimum and maximum velocity in any direction.
    pub fn add_velocity(&mut self, delta_seconds: f32, direction:Vec3) {
        let new_velocity = self.cur + (delta_seconds as f32 * self.inc * direction.normalize());
        let new_speed = new_velocity.length();
        let valid_speed = new_speed.max(self.min).min(self.max);
        let valid_velocity = match new_speed {
            s if s > 0.0 => (new_velocity / new_speed) * valid_speed,
            _ => Vec3::ZERO,
        };
        self.cur = valid_velocity;
    }
    /// Slows the current velocity vector down by the given delta_seconds * increase_step_length,
    /// but also respoects the minimum and maximum velocity in any direction.
    pub fn slowdown(&mut self, delta_seconds: f32) {
        self.add_velocity(delta_seconds, self.cur * -1.0);
    }
}

