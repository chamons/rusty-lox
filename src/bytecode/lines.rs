#[derive(Debug, Default)]
pub struct Lines {
    data: Vec<(u32, u32)>,
}

impl Lines {
    pub fn new(data: &[u32]) -> eyre::Result<Self> {
        if data.len() % 2 != 0 {
            return Err(eyre::eyre!("Lines input was not even"));
        }

        let data = data.chunks_exact(2).map(|c| (c[0], c[1])).collect();

        Ok(Self { data })
    }

    pub fn get(&self, index: u32) -> Option<u32> {
        let mut offset = 0;
        for (line, count) in &self.data {
            offset += count;
            if index < offset {
                return Some(*line);
            }
        }
        None
    }

    pub fn push(&mut self, line: u32) {
        let should_append = match self.data.last() {
            Some(last) => last.0 == line,
            None => false,
        };

        if should_append {
            self.data.last_mut().unwrap().1 += 1;
        } else {
            self.data.push((line, 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lines;

    #[test]
    fn run_encoded_lines() {
        let lines = Lines::new(&[1, 4, 2, 2, 3, 1]).unwrap();
        assert_eq!([1, 1, 1, 1, 2, 2, 3], *(0..7).map(|i| lines.get(i).unwrap()).collect::<Vec<_>>());

        assert!(lines.get(7).is_none());

        let lines = Lines::new(&[123, 2]).unwrap();
        assert_eq!([Some(123), Some(123), None], *(0..3).map(|i| lines.get(i)).collect::<Vec<_>>());
    }

    #[test]
    fn push_lines() {
        let mut lines = Lines::default();
        lines.push(123);
        lines.push(123);
        lines.push(124);
        lines.push(125);
        assert_eq!(
            [Some(123), Some(123), Some(124), Some(125), None],
            *(0..5).map(|i| lines.get(i)).collect::<Vec<_>>()
        );
    }
}
