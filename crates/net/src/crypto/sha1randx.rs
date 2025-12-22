use sha1::{Digest, Sha1};

pub struct Sha1Randx {
    taken: usize,
    o0: [u8; 20],
    o1: [u8; 20],
    o2: [u8; 20],
}

impl Sha1Randx {
    pub fn new(buff: &[u8]) -> Self {
        let half_size = buff.len() / 2;
        let (first_half, second_half) = buff.split_at(half_size);

        let o1: [u8; 20] = Sha1::digest(first_half).into();
        let o2: [u8; 20] = Sha1::digest(second_half).into();
        let o0 = [0u8; 20];

        let mut rng = Sha1Randx {
            taken: 0,
            o0,
            o1,
            o2,
        };

        rng.fill_up();
        rng
    }

    pub fn generate(&mut self, buf: &mut [u8]) {
        for byte in buf.iter_mut() {
            if self.taken == 20 {
                self.fill_up();
            }

            *byte = self.o0[self.taken];
            self.taken += 1;
        }
    }

    fn fill_up(&mut self) {
        let mut hasher = Sha1::new();

        hasher.update(self.o1);
        hasher.update(self.o0);
        hasher.update(self.o2);

        self.o0 = hasher.finalize().into();
        self.taken = 0;
    }
}
