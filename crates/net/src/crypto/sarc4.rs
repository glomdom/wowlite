pub struct SARC4 {
    s: [u8; 256],
    i: u8,
    j: u8,
}

impl SARC4 {
    pub fn new(key: &[u8]) -> Self {
        let mut s = [0u8; 256];
        for (i, x) in s.iter_mut().enumerate() {
            *x = i as u8;
        }

        let mut j: usize = 0;
        for i in 0..256 {
            let key_val = key[i % key.len()] as usize;
            j = (j + key_val + s[i] as usize) & 0xFF;
            s.swap(i, j);
        }

        Self { s, i: 0, j: 0 }
    }

    pub fn process(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            self.i = self.i.wrapping_add(1);
            let i_idx = self.i as usize;
            let s_i = self.s[i_idx];

            self.j = self.j.wrapping_add(s_i);
            let j_idx = self.j as usize;

            self.s.swap(i_idx, j_idx);

            let t = self.s[i_idx].wrapping_add(self.s[j_idx]) as usize;
            *b ^= self.s[t];
        }
    }
}
