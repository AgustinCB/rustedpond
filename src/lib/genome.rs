use super::{GENOME_SIZE, RandomGenerator};

pub(crate) struct Genome(pub(crate) [u8; GENOME_SIZE]);

impl Genome {
    #[inline]
    pub fn new() -> Genome {
        Genome([!0; GENOME_SIZE])
    }
    #[inline]
    pub fn random(generator: &mut RandomGenerator) -> Genome {
        let mut genome = [0; GENOME_SIZE];
        for i in 0..GENOME_SIZE {
            let mut n = generator.generate_integer() as u8;
            genome[i] = n;
        }
        Genome(genome)
    }

    #[inline]
    pub(crate) fn get(&self, pointer: &GenomePointer) -> u8 {
        if pointer.is_lower_byte {
            (self.0[pointer.array_pointer] & 0xf) as u8
        } else {
            ((self.0[pointer.array_pointer] >> 4) & 0xf) as u8
        }
    }

    #[inline]
    pub(crate) fn set(&mut self, pointer: &GenomePointer, value: u8) {
        self.0[pointer.array_pointer] = if pointer.is_lower_byte {
            let high_bits = self.0[pointer.array_pointer] & 0xf0;
            high_bits | (value & 0x0f)
        } else {
            let low_bits = self.0[pointer.array_pointer] & 0x0f;
            ((value & 0x0f) << 4) | low_bits
        }
    }
}

#[derive(Clone)]
pub(crate) struct GenomePointer {
    pub(crate) array_pointer: usize,
    pub(crate) is_lower_byte: bool,
}

impl GenomePointer {
    #[inline]
    pub(crate) fn new(array_pointer: usize, is_lower_byte: bool) -> GenomePointer {
        GenomePointer { array_pointer, is_lower_byte }
    }

    #[inline]
    pub(crate) fn next(&mut self) {
        if !self.is_lower_byte {
            self.array_pointer = (self.array_pointer + 1) % GENOME_SIZE;
        }
        self.is_lower_byte = !self.is_lower_byte;
    }

    #[inline]
    pub(crate) fn prev(&mut self) {
        if self.is_lower_byte {
            self.array_pointer = if self.array_pointer == 0 {
                GENOME_SIZE-1
            } else {
                self.array_pointer-1
            }
        }
        self.is_lower_byte = !self.is_lower_byte;
    }
}