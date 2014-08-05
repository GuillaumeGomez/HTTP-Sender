#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]

/* automatically generated by rust-bindgen */

use libc::{c_void, c_uint, c_char, c_ulong, c_int, c_uchar, off_t, c_long};

pub type alloc_func =
    ::std::option::Option<extern "C" fn(arg1: *mut c_void, arg2: c_uint, arg3: c_uint)
                              -> *mut c_void>;
pub type free_func =
    ::std::option::Option<extern "C" fn(arg1: *mut c_void, arg2: *mut c_void)>;
pub struct Struct_z_stream_s {
    pub next_in: *mut c_char,
    pub avail_in: c_uint,
    pub total_in: c_ulong,
    pub next_out: *mut c_char,
    pub avail_out: c_uint,
    pub total_out: c_ulong,
    pub msg: *mut c_char,
    pub state: *mut Struct_internal_state,
    pub zalloc: alloc_func,
    pub zfree: free_func,
    pub opaque: *mut c_void,
    pub data_type: c_int,
    pub adler: c_ulong,
    pub reserved: c_ulong,
}
pub type z_stream = Struct_z_stream_s;
pub type z_streamp = *mut z_stream;
pub struct Struct_gz_header_s {
    pub text: c_int,
    pub time: c_ulong,
    pub xflags: c_int,
    pub os: c_int,
    pub extra: *mut c_char,
    pub extra_len: c_uint,
    pub extra_max: c_uint,
    pub name: *mut c_char,
    pub name_max: c_uint,
    pub comment: *mut c_char,
    pub comm_max: c_uint,
    pub hcrc: c_int,
    pub done: c_int,
}
pub type gz_header = Struct_gz_header_s;
pub type gz_headerp = *mut gz_header;
pub type in_func =
    ::std::option::Option<extern "C" fn
                              (arg1: *mut c_void, arg2: *mut *mut c_uchar)
                              -> c_uint>;
pub type out_func =
    ::std::option::Option<extern "C" fn
                              (arg1: *mut c_void, arg2: *mut c_uchar,
                               arg3: c_uint) -> c_int>;
pub type gzFile = *mut Struct_gzFile_s;
pub struct Struct_gzFile_s {
    pub have: c_uint,
    pub next: *mut c_uchar,
    pub pos: off_t,
}
pub struct Struct_internal_state {
    pub dummy: c_int,
}
#[link(name = "z")]
extern "C" {
    pub fn zlibVersion() -> *mut c_char;
    pub fn deflate(strm: z_streamp, flush: c_int) -> c_int;
    pub fn deflateEnd(strm: z_streamp) -> c_int;
    pub fn inflate(strm: z_streamp, flush: c_int) -> c_int;
    pub fn inflateEnd(strm: z_streamp) -> c_int;
    pub fn deflateSetDictionary(strm: z_streamp, dictionary: *mut c_char,
                                dictLength: c_uint) -> c_int;
    pub fn deflateCopy(dest: z_streamp, source: z_streamp) -> c_int;
    pub fn deflateReset(strm: z_streamp) -> c_int;
    pub fn deflateParams(strm: z_streamp, level: c_int, strategy: c_int) ->
     c_int;
    pub fn deflateTune(strm: z_streamp, good_length: c_int, max_lazy: c_int,
                       nice_length: c_int, max_chain: c_int) -> c_int;
    pub fn deflateBound(strm: z_streamp, sourceLen: c_ulong) -> c_ulong;
    pub fn deflatePending(strm: z_streamp, pending: *mut c_uint,
                          bits: *mut c_int) -> c_int;
    pub fn deflatePrime(strm: z_streamp, bits: c_int, value: c_int) -> c_int;
    pub fn deflateSetHeader(strm: z_streamp, head: gz_headerp) -> c_int;
    pub fn inflateSetDictionary(strm: z_streamp, dictionary: *mut c_char,
                                dictLength: c_uint) -> c_int;
    pub fn inflateGetDictionary(strm: z_streamp, dictionary: *mut c_char,
                                dictLength: *mut c_uint) -> c_int;
    pub fn inflateSync(strm: z_streamp) -> c_int;
    pub fn inflateCopy(dest: z_streamp, source: z_streamp) -> c_int;
    pub fn inflateReset(strm: z_streamp) -> c_int;
    pub fn inflateReset2(strm: z_streamp, windowBits: c_int) -> c_int;
    pub fn inflatePrime(strm: z_streamp, bits: c_int, value: c_int) -> c_int;
    pub fn inflateMark(strm: z_streamp) -> c_long;
    pub fn inflateGetHeader(strm: z_streamp, head: gz_headerp) -> c_int;
    pub fn inflateBack(strm: z_streamp, _in: in_func, in_desc: *mut c_void,
                       out: out_func, out_desc: *mut c_void) -> c_int;
    pub fn inflateBackEnd(strm: z_streamp) -> c_int;
    pub fn zlibCompileFlags() -> c_ulong;
    pub fn compress(dest: *mut c_char, destLen: *mut c_ulong, source: *mut c_char,
                    sourceLen: c_ulong) -> c_int;
    pub fn compress2(dest: *mut c_char, destLen: *mut c_ulong, source: *mut c_char,
                     sourceLen: c_ulong, level: c_int) -> c_int;
    pub fn compressBound(sourceLen: c_ulong) -> c_ulong;
    pub fn uncompress(dest: *mut c_char, destLen: *mut c_ulong, source: *mut c_char,
                      sourceLen: c_ulong) -> c_int;
    pub fn gzdopen(fd: c_int, mode: *mut c_char) -> gzFile;
    pub fn gzbuffer(file: gzFile, size: c_uint) -> c_int;
    pub fn gzsetparams(file: gzFile, level: c_int, strategy: c_int) -> c_int;
    pub fn gzread(file: gzFile, buf: *mut c_void, len: c_uint) -> c_int;
    pub fn gzwrite(file: gzFile, buf: *mut c_void, len: c_uint) -> c_int;
    pub fn gzprintf(file: gzFile, format: *mut c_char, ...) -> c_int;
    pub fn gzputs(file: gzFile, s: *mut c_char) -> c_int;
    pub fn gzgets(file: gzFile, buf: *mut c_char, len: c_int) ->
     *mut c_char;
    pub fn gzputc(file: gzFile, c: c_int) -> c_int;
    pub fn gzgetc(file: gzFile) -> c_int;
    pub fn gzungetc(c: c_int, file: gzFile) -> c_int;
    pub fn gzflush(file: gzFile, flush: c_int) -> c_int;
    pub fn gzrewind(file: gzFile) -> c_int;
    pub fn gzeof(file: gzFile) -> c_int;
    pub fn gzdirect(file: gzFile) -> c_int;
    pub fn gzclose(file: gzFile) -> c_int;
    pub fn gzclose_r(file: gzFile) -> c_int;
    pub fn gzclose_w(file: gzFile) -> c_int;
    pub fn gzerror(file: gzFile, errnum: *mut c_int) -> *mut c_char;
    pub fn gzclearerr(file: gzFile);
    pub fn adler32(adler: c_ulong, buf: *mut c_char, len: c_uint) -> c_ulong;
    pub fn crc32(crc: c_ulong, buf: *mut c_char, len: c_uint) -> c_ulong;
    pub fn deflateInit_(strm: z_streamp, level: c_int, version: *mut c_char,
                        stream_size: c_int) -> c_int;
    pub fn inflateInit_(strm: z_streamp, version: *mut c_char,
                        stream_size: c_int) -> c_int;
    pub fn deflateInit2_(strm: z_streamp, level: c_int, method: c_int,
                         windowBits: c_int, memLevel: c_int, strategy: c_int,
                         version: *mut c_char, stream_size: c_int) -> c_int;
    pub fn inflateInit2_(strm: z_streamp, windowBits: c_int,
                         version: *mut c_char, stream_size: c_int) -> c_int;
    pub fn inflateBackInit_(strm: z_streamp, windowBits: c_int,
                            window: *mut c_uchar, version: *mut c_char,
                            stream_size: c_int) -> c_int;
    pub fn gzgetc_(file: gzFile) -> c_int;
    pub fn gzopen(arg1: *mut c_char, arg2: *mut c_char) -> gzFile;
    pub fn gzseek(arg1: gzFile, arg2: off_t, arg3: c_int) -> off_t;
    pub fn gztell(arg1: gzFile) -> off_t;
    pub fn gzoffset(arg1: gzFile) -> off_t;
    pub fn adler32_combine(arg1: c_ulong, arg2: c_ulong, arg3: off_t) -> c_ulong;
    pub fn crc32_combine(arg1: c_ulong, arg2: c_ulong, arg3: off_t) -> c_ulong;
    pub fn zError(arg1: c_int) -> *mut c_char;
    pub fn inflateSyncPoint(arg1: z_streamp) -> c_int;
    pub fn inflateUndermine(arg1: z_streamp, arg2: c_int) -> c_int;
    pub fn inflateResetKeep(arg1: z_streamp) -> c_int;
    pub fn deflateResetKeep(arg1: z_streamp) -> c_int;
}