use std::io::Read;

pub fn iter_2d<X, Y>(xs: X, ys: Y) -> impl Iterator<Item = (X::Item, Y::Item)>
where
    X: Iterator + Clone,
    Y: Iterator,
    Y::Item: Clone,
{
    ys.flat_map(move |y| xs.clone().zip(std::iter::repeat(y)))
}

#[cfg(test)]
#[test]
fn test_iter_2d() {
    let pairs = iter_2d(2..4, 4..7).collect::<Vec<_>>();
    #[rustfmt::skip]
    let expected = [
        (2, 4), (3, 4),
        (2, 5), (3, 5),
        (2, 6), (3, 6),
    ];
    assert_eq!(pairs, expected);
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    #[default]
    Right,
    Down,
    Left,
}

impl Direction {
    pub const NESW: [Self; 4] = [Self::Up, Self::Right, Self::Down, Self::Left];

    pub fn go(&self, (x, y): (usize, usize), (w, h): (usize, usize)) -> Option<(usize, usize)> {
        let result = match self {
            Self::Right => (x + 1, y),
            Self::Down => (x, y + 1),
            Self::Left => (x.checked_sub(1)?, y),
            Self::Up => (x, y.checked_sub(1)?),
        };
        if result.0 >= w || result.1 >= h {
            None
        } else {
            Some(result)
        }
    }

    #[must_use]
    pub fn turn(&self, cc: Rotation) -> Self {
        let mut all = Self::NESW;
        match cc {
            Rotation::Counter => all.rotate_right(1),
            Rotation::Clock => all.rotate_left(1),
        }
        all[*self as usize]
    }

    pub fn rotate(&mut self, rotation: Rotation) {
        *self = self.turn(rotation);
    }

    pub fn find_max(&self, iter: impl Iterator<Item = (usize, usize)>) -> (usize, usize) {
        let primary_axis = |&(x, y): &(usize, usize)| match self {
            Self::Left | Self::Right => x,
            Self::Up | Self::Down => y,
        };
        match self {
            Self::Left | Self::Up => iter.min_by_key(primary_axis),
            Self::Right | Self::Down => iter.max_by_key(primary_axis),
        }
        .unwrap()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Rotation {
    #[default]
    Counter,
    Clock,
}

impl Rotation {
    #[must_use]
    pub fn flip(&self) -> Self {
        match self {
            Self::Counter => Self::Clock,
            Self::Clock => Self::Counter,
        }
    }

    pub fn toggle(&mut self) {
        *self = self.flip();
    }
}

pub fn read_utf8_char<R: Read>(mut reader: R) -> std::io::Result<char> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf[0..1])?;
    let remainder = match buf[0] {
        n @ 0b0_0000000..=0b0_1111111 => return Ok(n as char),
        0b10_000000..=0b10_111111 => 1..2,
        0b110_00000..=0b110_11111 => 1..3,
        0b1110_0000..=0b1110_0000 => 1..4,
        0b1111.. => todo!(),
    };
    reader.read_exact(&mut buf[remainder])?;
    println!("{buf:?}");
    let s = std::str::from_utf8(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(s.chars().next().unwrap())
}
