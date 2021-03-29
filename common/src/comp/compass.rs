use vek::Vec2;

/// Cardinal directions
pub enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl Direction {
    /// Convert a direction vector to a cardinal direction
    /// Direction vector can be trivially calculated by doing (target - source)
    pub fn from_dir(dir: Vec2<f32>) -> Self {
        if let Some(dir) = dir.try_normalized() {
            let mut angle = dir.angle_between(Vec2::unit_y()).to_degrees();
            if dir.x < 0.0 {
                angle = -angle;
            }
            match angle as i32 {
                -360..=-157 => Direction::South,
                -156..=-112 => Direction::Southwest,
                -111..=-67 => Direction::West,
                -66..=-22 => Direction::Northwest,
                -21..=22 => Direction::North,
                23..=67 => Direction::Northeast,
                68..=112 => Direction::East,
                113..=157 => Direction::Southeast,
                158..=360 => Direction::South,
                _ => Direction::North, // should never happen
            }
        } else {
            Direction::North // default value, should never happen
        }
    }

    // TODO: localization
    pub fn name(&self) -> &'static str {
        match self {
            Direction::North => "North",
            Direction::Northeast => "Northeast",
            Direction::East => "East",
            Direction::Southeast => "Southeast",
            Direction::South => "South",
            Direction::Southwest => "Southwest",
            Direction::West => "West",
            Direction::Northwest => "Northwest",
        }
    }
}

/// Arbitrarily named Distances
pub enum Distance {
    VeryFar,
    Far,
    Ahead,
    Near,
    NextTo,
}

impl Distance {
    /// Convert a length to a Distance
    pub fn from_length(length: i32) -> Self {
        match length {
            0..=100 => Distance::NextTo,
            101..=500 => Distance::Near,
            501..=3000 => Distance::Ahead,
            3001..=10000 => Distance::Far,
            _ => Distance::VeryFar,
        }
    }

    /// Convert a vector to a Distance
    pub fn from_dir(dir: Vec2<f32>) -> Self { Self::from_length(dir.magnitude() as i32) }

    // TODO: localization
    pub fn name(&self) -> &'static str {
        match self {
            Distance::VeryFar => "very far",
            Distance::Far => "far",
            Distance::Ahead => "ahead",
            Distance::Near => "near",
            Distance::NextTo => "just around",
        }
    }
}
