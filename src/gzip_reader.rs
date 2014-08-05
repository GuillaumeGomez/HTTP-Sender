use libc::{c_int};
use std::mem;
use std::vec::Vec;
use std::mem::{size_of, zeroed};
use zlib;

static Z_OK            : c_int = 0;
static Z_STREAM_END    : c_int = 1;
static Z_NEED_DICT     : c_int = 2;
static Z_ERRNO         : c_int = -1;
static Z_STREAM_ERROR  : c_int = -2;
static Z_DATA_ERROR    : c_int = -3;
static Z_MEM_ERROR     : c_int = -4;
static Z_BUF_ERROR     : c_int = -5;
static Z_VERSION_ERROR : c_int = -6;

static MAX_WBITS : c_int = 15;
static Z_NULL    : c_int = 0;

static Z_NO_FLUSH   : c_int = 0;
static Z_SYNC_FLUSH : c_int = 2;

pub struct GzipReader {
    pub inner: Vec<u8>,
}

impl GzipReader {
    pub fn decode(&mut self) -> Result<String, String> {
        let mut strm = unsafe { zeroed::<zlib::z_stream>() };
        let mut tmp_ret = Vec::from_elem(self.inner.len(), 0u8);

        strm.next_in = self.inner.as_mut_ptr() as *mut i8;
        strm.avail_in = self.inner.len() as u32;
        strm.total_out = 0;

        if unsafe {zlib::inflateInit2_(&mut strm, 16 + MAX_WBITS, zlib::zlibVersion(), size_of::<zlib::z_stream>() as i32)} != Z_OK {
            Err("inflateInit2 failed".to_string())
        } else {
            loop {
                if strm.total_out as uint >= tmp_ret.len() {
                    tmp_ret = tmp_ret.append(Vec::from_elem(self.inner.len(), 0u8).as_slice());
                }
                strm.next_out = unsafe {mem::transmute(&tmp_ret.as_mut_slice()[strm.total_out as uint])};
                strm.avail_out = (tmp_ret.len() - strm.total_out as uint) as u32;

                match unsafe {zlib::inflate(&mut strm, Z_SYNC_FLUSH)} {
                    Z_STREAM_END => {
                        if unsafe {zlib::inflateEnd(&mut strm)} != Z_OK {
                            return Err("inflateEnd failed".to_string());
                        } else {
                            tmp_ret.push(0u8);
                            return Ok(unsafe {::std::string::raw::from_buf(tmp_ret.as_ptr())});
                        }
                    },
                    Z_OK => {},
                    _ => return Err("inflate failed".to_string()),
                }
            }
        }
    }
}
