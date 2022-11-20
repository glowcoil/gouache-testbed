use crate::geom::*;

pub struct Component {
    start: usize,
    end: usize,
}

pub struct PathBuilder {
    components: Vec<Component>,
    points: Vec<Vec2>,
}

impl PathBuilder {
    pub fn new() -> PathBuilder {
        PathBuilder {
            components: Vec::new(),
            points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, point: Vec2) {
        if let Some(component) = self.components.last_mut() {
            component.end += 1;
        }
        self.points.push(point);
    }

    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        self.components.push(Component {
            start: self.points.len(),
            end: self.points.len(),
        });
        self.add_point(point);
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.add_point(point);
        self.add_point(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        self.add_point(control);
        self.add_point(point);
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        let last = self.points.last().cloned().unwrap_or(Vec2::new(0.0, 0.0));

        let width = last.x.max(control1.x).max(control2.x).max(point.x)
            - last.x.min(control1.x).min(control2.x).min(point.x);
        let height = last.y.max(control1.y).max(control2.y).max(point.y)
            - last.y.min(control1.y).min(control2.y).min(point.y);
        let factor = 0.001 * width.max(height) * 18.0 / 3.0f32.sqrt();

        let mut p1 = self.points.last().cloned().unwrap_or(Vec2::new(0.0, 0.0));
        let mut p2 = control1;
        let mut p3 = control2;
        let p4 = point;
        loop {
            let error = (3.0 * p2 - 3.0 * p3 - p1 + p4).length();
            let split = (factor / error).cbrt();

            if error == 0.0 || split > 1.0 {
                break;
            }

            let p12 = Vec2::lerp(split, p1, p2);
            let p23 = Vec2::lerp(split, p2, p3);
            let p34 = Vec2::lerp(split, p3, p4);
            let p123 = Vec2::lerp(split, p12, p23);
            let p234 = Vec2::lerp(split, p23, p34);
            let p = Vec2::lerp(split, p123, p234);

            self.quadratic_to(0.25 * (3.0 * p12 + 3.0 * p123 - p1 - p), p);

            p1 = p;
            p2 = p234;
            p3 = p34;
        }

        self.quadratic_to(0.25 * (3.0 * p2 + 3.0 * p3 - p1 - p4), p4);

        self
    }

    pub fn close(&mut self) {
        if let Some(component) = self.components.last_mut() {
            let first = self.points[component.start];
            let last = self.points[component.end - 1];
            if first != last {
                self.add_point(first);
                self.add_point(first);
            }
        }
    }

    pub fn build(&self) -> Path {
        let mut min = self.points.get(0).copied().unwrap_or(Vec2::new(0.0, 0.0));
        let mut max = self.points.get(0).copied().unwrap_or(Vec2::new(0.0, 0.0));

        for &point in self.points.iter() {
            min = min.min(point);
            max = max.max(point);
        }

        let mut components = Vec::with_capacity(self.components.len() * 2);
        for component in self.components.iter() {
            components.extend_from_slice(&[
                component.start.try_into().unwrap(),
                component.end.try_into().unwrap(),
            ]);
        }

        fn to_u16_unorm(value: f32, min: f32, max: f32) -> u16 {
            (std::u16::MAX as f32 * ((value - min) / (max - min))).round() as u16
        }

        let mut points = Vec::with_capacity(self.points.len() * 2);
        for point in self.points.iter() {
            points.extend_from_slice(&[
                to_u16_unorm(point.x, min.x, max.x),
                to_u16_unorm(point.y, min.y, max.y),
            ]);
        }

        Path {
            min,
            max,
            components,
            points,
        }
    }
}

pub struct Path {
    pub min: Vec2,
    pub max: Vec2,
    pub components: Vec<u16>,
    pub points: Vec<u16>,
}
