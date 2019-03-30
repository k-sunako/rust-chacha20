use std::u32;

// 2.1.  The ChaCha Quarter Round

//    The basic operation of the ChaCha algorithm is the quarter round.  It
//    operates on four 32-bit unsigned integers, denoted a, b, c, and d.
//    The operation is as follows (in C-like notation):

//    1.  a += b; d ^= a; d <<<= 16;
//    2.  c += d; b ^= c; b <<<= 12;
//    3.  a += b; d ^= a; d <<<= 8;
//    4.  c += d; b ^= c; b <<<= 7;
fn quarter_round(mut a: u32, mut b: u32, mut c: u32, mut d: u32) -> (u32, u32, u32, u32) {
    a = a.overflowing_add(b).0;
    d ^= a;
    d = d.rotate_left(16);
    c = c.overflowing_add(d).0;
    b ^= c;
    b = b.rotate_left(12);
    a = a.overflowing_add(b).0;
    d ^= a;
    d = d.rotate_left(8);
    c = c.overflowing_add(d).0;
    b ^= c;
    b = b.rotate_left(7);
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
    assert_eq!(
        quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567),
        (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
    );
}

// 2.2.  A Quarter Round on the ChaCha State

//    The ChaCha state does not have four integer numbers: it has 16.  So
//    the quarter-round operation works on only four of them -- hence the
//    name.  Each quarter round operates on four predetermined numbers in
//    the ChaCha state.  We will denote by QUARTERROUND(x,y,z,w) a quarter-
//    round operation on the numbers at indices x, y, z, and w of the
//    ChaCha state when viewed as a vector.  For example, if we apply
//    QUARTERROUND(1,5,9,13) to a state, this means running the quarter-
//    round operation on the elements marked with an asterisk, while
//    leaving the others alone:

//       0  *a   2   3
//       4  *b   6   7
//       8  *c  10  11
//      12  *d  14  15

fn apply_quarter_round(x: usize, y: usize, z: usize, w: usize, words_16: Vec<u32>) -> Vec<u32> {
    let old_a = words_16[x];
    let old_b = words_16[y];
    let old_c = words_16[z];
    let old_d = words_16[w];

    let (new_a, new_b, new_c, new_d) = quarter_round(old_a, old_b, old_c, old_d);
    
    let mut ret = vec![0; 16];
    ret[..16].clone_from_slice(&words_16[..16]);
    ret[x] = new_a;
    ret[y] = new_b;
    ret[z] = new_c;
    ret[w] = new_d;
    
    ret
}

// 2.2.1.  Test Vector for the Quarter Round on the ChaCha State

//    For a test vector, we will use a ChaCha state that was generated
//    randomly:

//    Sample ChaCha State

//        879531e0  c5ecf37d  516461b1  c9a62f8a
//        44c20ef3  3390af7f  d9fc690b  2a5f714c
//        53372767  b00a5631  974c541a  359e9963
//        5c971061  3d631689  2098d9d6  91dbd320

//    We will apply the QUARTERROUND(2,7,8,13) operation to this state.
//    For obvious reasons, this one is part of what is called a "diagonal
//    round":

//    After applying QUARTERROUND(2,7,8,13)

//        879531e0  c5ecf37d *bdb886dc  c9a62f8a
//        44c20ef3  3390af7f  d9fc690b *cfacafd2
//       *e46bea80  b00a5631  974c541a  359e9963
//        5c971061 *ccc07c79  2098d9d6  91dbd320

//    Note that only the numbers in positions 2, 7, 8, and 13 changed.

#[test]
fn test_apply_quarter_round() {
    let input: Vec<u32> = vec![
        0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a,
        0x44c20ef3, 0x3390af7f, 0xd9fc690b, 0x2a5f714c,
        0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963,
        0x5c971061, 0x3d631689, 0x2098d9d6, 0x91dbd320
    ];

    let output: Vec<u32> = vec![
        0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a,
        0x44c20ef3, 0x3390af7f, 0xd9fc690b, 0xcfacafd2,
        0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963,
        0x5c971061, 0xccc07c79, 0x2098d9d6, 0x91dbd320
    ];

    let actual = apply_quarter_round(2, 7, 8, 13, input);

    assert_eq!(output, actual);
}

fn main() {
    println!("Hello, world!");
}
