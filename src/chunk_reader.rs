use std::num::from_str_radix;

pub struct ChunkReader {
    data: Vec<u8>,
}

impl ChunkReader {
    pub fn new(v : &Vec<u8>) -> ChunkReader {
        ChunkReader{data: v.clone()}
    }

    pub fn read_from_chunk(&self) -> Result<(uint, uint), String> {
        let mut line = String::new();
        static MAXNUM_SIZE : uint = 16;
        static HEX_CHARS : &'static [u8] = b"0123456789abcdefABCDEF";
        let mut is_in_chunk_extension = false;
        let mut pos = 0;

        if self.data.len() > 1 && self.data[0] == 0u8 && self.data[1] == 0u8 {
            return Ok((0, self.data.len() - 1));
        }
        while pos < self.data.len() {
            match self.data[pos] as char {
                '\r' => {
                    pos += 1;
                    if pos >= self.data.len() || self.data[pos] as char != '\n' {
                        return Err("Error with '\r'".to_string());
                    }
                    break;
                }
                '\n' => {
                    break;
                }
                _ if is_in_chunk_extension => {
                }
                ';' => {
                    is_in_chunk_extension = true;
                }
                c if HEX_CHARS.contains(&(c as u8)) => {
                    line.push(c);
                }
                _ => {
                    println!("{}", self.data);
                    return Err("Chunk format error".to_string())
                }
            }
            pos += 1;
        }

        if line.len() > MAXNUM_SIZE {
            Err("http chunk transfer encoding format: size line too long".to_string())
        } else {
            match from_str_radix(line.as_slice(), 16) {
                Some(v) => Ok((v, pos + 1)),
                None => Ok((0, pos + 1)),
            }
        }
    }

    pub fn read_next(&mut self) -> Result<(Vec<u8>, uint), String> {
        let mut out = Vec::new();

        if self.data.len() > 0 {
            match self.read_from_chunk() {
                Ok((to_write, to_skip)) => {
                    if to_write == 0 {
                        self.data = self.data.clone().into_iter().skip(to_skip + to_write).collect::<Vec<u8>>();
                        if self.data.len() > 0 {
                            self.read_next()
                        } else {
                            Ok((out.clone(), self.data.len()))
                        }
                    } else {
                        let mut tmp_v = self.data.clone().into_iter().skip(to_skip).collect::<Vec<u8>>();

                        tmp_v.truncate(to_write);
                        out.extend(tmp_v.into_iter());
                        self.data = self.data.clone().into_iter().skip(to_skip + to_write).collect::<Vec<u8>>();
                        Ok((out.clone(), self.data.len()))
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            Ok((out.clone(), 0))
        }
    }
}