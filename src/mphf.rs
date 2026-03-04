use pyo3::exceptions::PyValueError;
use pyo3::PyErr;
use std::collections::HashSet;
use std::ptr;

const MAX_BUILD_ATTEMPTS: usize = 32;

pub struct BuildArtifacts<M> {
    pub mphf: M,
    pub tags: Vec<u64>,
    pub permutation: Vec<usize>,
}

pub trait Mphf: Clone {
    fn build(keys: &[Vec<u8>]) -> Result<BuildArtifacts<Self>, PyErr>
    where
        Self: Sized;

    fn tag(&self, key: &[u8]) -> u64;

    fn index_from_tag(&self, tag: u64) -> usize;
}

pub type ActiveMphf = PthashMphf;

#[derive(Clone)]
pub struct PthashMphf {
    tag_seed: u64,
    bucket_seed: u64,
    bucket_count: usize,
    table_len: usize,
    pilots: Vec<u64>,
}

impl PthashMphf {
    pub fn dump(&self) -> (u64, u64, usize, usize, Vec<u64>) {
        (
            self.tag_seed,
            self.bucket_seed,
            self.bucket_count,
            self.table_len,
            self.pilots.clone(),
        )
    }

    pub fn from_dump(
        tag_seed: u64,
        bucket_seed: u64,
        bucket_count: usize,
        table_len: usize,
        pilots: Vec<u64>,
    ) -> Result<Self, PyErr> {
        if bucket_count == 0 {
            return Err(PyValueError::new_err("bucket_count must be at least 1"));
        }
        if pilots.len() != bucket_count {
            return Err(PyValueError::new_err(
                "pilots must have the same length as bucket_count",
            ));
        }
        if table_len == 0 && pilots.len() != 1 {
            return Err(PyValueError::new_err(
                "empty hash tables must use a single empty bucket",
            ));
        }

        Ok(Self {
            tag_seed,
            bucket_seed,
            bucket_count,
            table_len,
            pilots,
        })
    }

    #[inline(always)]
    fn bucket_for(&self, tag: u64) -> usize {
        if self.bucket_count <= 1 {
            0
        } else {
            (mix64(tag ^ self.bucket_seed) % self.bucket_count as u64) as usize
        }
    }

    #[inline(always)]
    fn slot_for(&self, tag: u64, pilot: u64) -> usize {
        if self.table_len == 0 {
            0
        } else {
            (mix64(tag ^ mix64(pilot ^ 0x517cc1b727220a95)) % self.table_len as u64) as usize
        }
    }

    fn bucket_count_for(nkeys: usize) -> usize {
        if nkeys <= 1 {
            return 1;
        }

        let lg = (usize::BITS - nkeys.leading_zeros() - 1).max(1) as usize;
        ((6 * nkeys).saturating_add(lg - 1) / lg).max(1)
    }
}

impl Mphf for PthashMphf {
    fn build(keys: &[Vec<u8>]) -> Result<BuildArtifacts<Self>, PyErr> {
        if keys.is_empty() {
            return Ok(BuildArtifacts {
                mphf: Self {
                    tag_seed: rand::random(),
                    bucket_seed: rand::random(),
                    bucket_count: 1,
                    table_len: 0,
                    pilots: vec![0],
                },
                tags: Vec::new(),
                permutation: Vec::new(),
            });
        }

        let bucket_count = Self::bucket_count_for(keys.len());
        let max_pilot_attempts = (keys.len() as u64).saturating_mul(16).max(1024);

        for _ in 0..MAX_BUILD_ATTEMPTS {
            let tag_seed = rand::random();
            let bucket_seed = rand::random();
            let mut seen = HashSet::with_capacity(keys.len());
            let mut tags = Vec::with_capacity(keys.len());
            let mut buckets = vec![Vec::<(usize, u64)>::new(); bucket_count];
            let mut duplicate_tag = false;

            for (index, key) in keys.iter().enumerate() {
                let tag = hash_key(key, tag_seed);
                if !seen.insert(tag) {
                    duplicate_tag = true;
                    break;
                }
                let bucket = if bucket_count == 1 {
                    0
                } else {
                    (mix64(tag ^ bucket_seed) % bucket_count as u64) as usize
                };
                tags.push(tag);
                buckets[bucket].push((index, tag));
            }

            if duplicate_tag {
                continue;
            }

            let mut order: Vec<_> = (0..bucket_count).collect();
            order.sort_unstable_by(|left, right| buckets[*right].len().cmp(&buckets[*left].len()));

            let mut occupied = vec![false; keys.len()];
            let mut permutation = vec![usize::MAX; keys.len()];
            let mut pilots = vec![0u64; bucket_count];
            let mut placements = Vec::<(usize, usize)>::new();
            let mut failed = false;

            for bucket_index in order {
                let entries = &buckets[bucket_index];
                if entries.is_empty() {
                    continue;
                }

                let mut assigned = false;
                for pilot in 0..max_pilot_attempts {
                    placements.clear();
                    let mut collided = false;

                    for &(original_index, tag) in entries {
                        let slot = if keys.len() == 1 {
                            0
                        } else {
                            (mix64(tag ^ mix64(pilot ^ 0x517cc1b727220a95)) % keys.len() as u64)
                                as usize
                        };

                        if occupied[slot] {
                            collided = true;
                            break;
                        }

                        placements.push((original_index, slot));
                    }

                    if collided {
                        continue;
                    }

                    placements.sort_unstable_by_key(|&(_, slot)| slot);
                    if placements.windows(2).any(|pair| pair[0].1 == pair[1].1) {
                        continue;
                    }

                    for &(original_index, slot) in &placements {
                        occupied[slot] = true;
                        permutation[slot] = original_index;
                    }
                    pilots[bucket_index] = pilot;
                    assigned = true;
                    break;
                }

                if !assigned {
                    failed = true;
                    break;
                }
            }

            if failed || permutation.iter().any(|&index| index == usize::MAX) {
                continue;
            }

            return Ok(BuildArtifacts {
                mphf: Self {
                    tag_seed,
                    bucket_seed,
                    bucket_count,
                    table_len: keys.len(),
                    pilots,
                },
                tags,
                permutation,
            });
        }

        Err(PyValueError::new_err(
            "failed to build a PTHash MPHF after several retries",
        ))
    }

    #[inline(always)]
    fn tag(&self, key: &[u8]) -> u64 {
        hash_key(key, self.tag_seed)
    }

    #[inline(always)]
    fn index_from_tag(&self, tag: u64) -> usize {
        let bucket = self.bucket_for(tag);
        self.slot_for(tag, self.pilots[bucket])
    }
}

#[inline(always)]
fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[inline(always)]
fn hash_key(key: &[u8], seed: u64) -> u64 {
    let mut state = seed ^ ((key.len() as u64).wrapping_mul(0x9e3779b185ebca87));

    let mut index = 0usize;
    while index + 8 <= key.len() {
        let word = unsafe { ptr::read_unaligned(key.as_ptr().add(index) as *const u64) };
        state = mix64(state ^ word);
        index += 8;
    }

    if index < key.len() {
        let mut tail = 0u64;
        for (shift, byte) in key[index..].iter().enumerate() {
            tail |= (*byte as u64) << (shift * 8);
        }
        state = mix64(state ^ tail ^ 0x6c8e9cf570932bd5);
    }

    mix64(state ^ (key.len() as u64).rotate_left(17))
}
