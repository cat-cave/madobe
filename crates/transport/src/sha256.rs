//! Internal SHA-256 helper shared by smoke harnesses.

const INITIAL_STATE: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

pub(crate) fn digest_hex(message: &[u8]) -> String {
    hex(&digest(message))
}

fn digest(message: &[u8]) -> [u8; 32] {
    let mut state = INITIAL_STATE;
    let mut padded = padded_message(message);

    for chunk in padded.chunks_exact_mut(64) {
        compress(&mut state, chunk);
    }

    state_to_bytes(state)
}

fn padded_message(message: &[u8]) -> Vec<u8> {
    let mut padded = message.to_vec();
    let bit_len = u64::try_from(message.len()).map_or(u64::MAX, |len| len.saturating_mul(8));

    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    padded
}

fn compress(state: &mut [u32; 8], chunk: &[u8]) {
    let mut schedule = [0_u32; 64];
    for (index, word) in chunk.chunks_exact(4).take(16).enumerate() {
        schedule[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
    }
    for index in 16..64 {
        let small_sigma0 = schedule[index - 15].rotate_right(7)
            ^ schedule[index - 15].rotate_right(18)
            ^ (schedule[index - 15] >> 3);
        let small_sigma1 = schedule[index - 2].rotate_right(17)
            ^ schedule[index - 2].rotate_right(19)
            ^ (schedule[index - 2] >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(small_sigma0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(small_sigma1);
    }

    let mut working = WorkingState::from_state(*state);
    for index in 0..64 {
        working.round(schedule[index], ROUND_CONSTANTS[index]);
    }
    working.add_to(state);
}

struct WorkingState {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    g: u32,
    h: u32,
}

impl WorkingState {
    const fn from_state(state: [u32; 8]) -> Self {
        Self {
            a: state[0],
            b: state[1],
            c: state[2],
            d: state[3],
            e: state[4],
            f: state[5],
            g: state[6],
            h: state[7],
        }
    }

    const fn round(&mut self, schedule_word: u32, round_constant: u32) {
        let sum1 = self.e.rotate_right(6) ^ self.e.rotate_right(11) ^ self.e.rotate_right(25);
        let choice = (self.e & self.f) ^ ((!self.e) & self.g);
        let temp1 = self
            .h
            .wrapping_add(sum1)
            .wrapping_add(choice)
            .wrapping_add(round_constant)
            .wrapping_add(schedule_word);
        let sum0 = self.a.rotate_right(2) ^ self.a.rotate_right(13) ^ self.a.rotate_right(22);
        let majority = (self.a & self.b) ^ (self.a & self.c) ^ (self.b & self.c);
        let temp2 = sum0.wrapping_add(majority);

        self.h = self.g;
        self.g = self.f;
        self.f = self.e;
        self.e = self.d.wrapping_add(temp1);
        self.d = self.c;
        self.c = self.b;
        self.b = self.a;
        self.a = temp1.wrapping_add(temp2);
    }

    const fn add_to(self, state: &mut [u32; 8]) {
        state[0] = state[0].wrapping_add(self.a);
        state[1] = state[1].wrapping_add(self.b);
        state[2] = state[2].wrapping_add(self.c);
        state[3] = state[3].wrapping_add(self.d);
        state[4] = state[4].wrapping_add(self.e);
        state[5] = state[5].wrapping_add(self.f);
        state[6] = state[6].wrapping_add(self.g);
        state[7] = state[7].wrapping_add(self.h);
    }
}

fn state_to_bytes(state: [u32; 8]) -> [u8; 32] {
    let mut output = [0_u8; 32];
    for (index, value) in state.into_iter().enumerate() {
        output[index * 4..index * 4 + 4].copy_from_slice(&value.to_be_bytes());
    }

    output
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(hex_digit(byte >> 4));
        output.push(hex_digit(byte & 0x0f));
    }

    output
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::digest_hex;

    #[test]
    fn sha256_matches_standard_vectors() {
        assert_eq!(
            digest_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            digest_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
