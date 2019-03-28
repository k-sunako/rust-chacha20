use std::u32;

// 2.1.  The ChaCha Quarter Round

//    The basic operation of the ChaCha algorithm is the quarter round.  It
//    operates on four 32-bit unsigned integers, denoted a, b, c, and d.
//    The operation is as follows (in C-like notation):

//    1.  a += b; d ^= a; d <<<= 16;
//    2.  c += d; b ^= c; b <<<= 12;
//    3.  a += b; d ^= a; d <<<= 8;
//    4.  c += d; b ^= c; b <<<= 7;
fn quarter_round(mut a: u32, mut b: u32, mut c: u32, mut d:u32) -> (u32, u32, u32, u32) {
    a = a.overflowing_add(b).0; d ^= a; d = d.rotate_left(16);
    c = c.overflowing_add(d).0; b ^= c; b = b.rotate_left(12);
    a = a.overflowing_add(b).0; d ^= a; d = d.rotate_left(8);
    c = c.overflowing_add(d).0; b ^= c; b = b.rotate_left(7);
    (a, b, c, d)
}

// 2.1.1.  Test Vector for the ChaCha Quarter Round

//    For a test vector, we will use the same numbers as in the example,
//    adding something random for c.

//    o  a = 0x11111111
//    o  b = 0x01020304
//    o  c = 0x9b8d6f43
//    o  d = 0x01234567

//    After running a Quarter Round on these four numbers, we get these:

//    o  a = 0xea2a92f4
//    o  b = 0xcb1cf8ce
//    o  c = 0x4581472e
//    o  d = 0x5881c4bb

#[test]
fn test_quarter_round() {
    assert_eq!(quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567),
               (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb));
}

fn main() {
    println!("Hello, world!");
}
