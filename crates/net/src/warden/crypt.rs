use crate::crypto::{SARC4, Sha1Randx};

pub struct WardenCrypt {
    input: SARC4,
    output: SARC4,
}

impl WardenCrypt {
    pub fn new(session_key: &[u8]) -> Self {
        let mut rng = Sha1Randx::new(session_key);

        let mut key_a = [0u8; 16];
        let mut key_b = [0u8; 16];
        rng.generate(&mut key_a);
        rng.generate(&mut key_b);

        let input_key = key_b;
        let output_key = key_a;

        Self {
            input: SARC4::new(&input_key),
            output: SARC4::new(&output_key),
        }
    }

    pub fn decrypt(&mut self, data: &mut [u8]) {
        self.input.process(data);
    }

    pub fn encrypt(&mut self, data: &mut [u8]) {
        self.output.process(data);
    }
}
