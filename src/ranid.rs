use std::default::Default;

use rand::distributions::{Distribution, Uniform};

use crate::types::WindmillError;

#[derive(Clone)]
pub struct RanIDs {
    radix: u32,
    pub ids: Vec<u32>,
    pub dist: Uniform<u32>,
}

impl RanIDs {
    fn new(radix: u32) -> Self {
        RanIDs {
            radix,
            ids: Vec::new(),
            dist: Uniform::from(1..radix.pow(5)-1),
        }
    }

    pub fn get(&mut self) -> Result<String,WindmillError> {
        let mut rng =  rand::thread_rng();
        let mut x: u32 = self.dist.sample(&mut rng);
        while self.ids.contains(&x) {
            x = self.dist.sample(&mut rng);
        }
        self.ids.push(x);
        let mut result: [char;5] = [std::char::from_digit(0,self.radix).unwrap(); 5];

        let mut i = 5;
        while i > 0 {
            let m = x % self.radix;
            x = x / self.radix;

            result[i-1] = std::char::from_digit(m, self.radix).unwrap();
            if x == 0 {
                return Ok(result.iter().collect());
            }
            i -= 1;
        }
        Err(WindmillError::Incomplete)
    }
}

impl Default for RanIDs {
    fn default() -> RanIDs {
        RanIDs::new(36)
    }
}
