use std::mem;
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
        0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
        0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
        0x2098d9d6, 0x91dbd320,
    ];

    let output: Vec<u32> = vec![
        0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
        0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79,
        0x2098d9d6, 0x91dbd320,
    ];

    let actual = apply_quarter_round(2, 7, 8, 13, input);

    assert_eq!(output, actual);
}

fn setup_key(key: Vec<u8>, counter: u32, nonce: Vec<u8>) -> Vec<u32> {
    // The ChaCha20 state is initialized as follows:

    let mut state: Vec<u32> = vec![0; 16];

    // o  The first four words (0-3) are constants: 0x61707865, 0x3320646e,
    //    0x79622d32, 0x6b206574.
    state[0] = 0x6170_7865;
    state[1] = 0x3320_646e;
    state[2] = 0x7962_2d32;
    state[3] = 0x6b20_6574;

    // o  The next eight words (4-11) are taken from the 256-bit key by
    //    reading the bytes in little-endian order, in 4-byte chunks.
    for i in 0..8 {
        let idx_state = i + 4;
        let idx_key = 4 * i;

        /*
        let mut x = 0;
        x |= u32::from(key[idx_key + 3]);
        x = x.wrapping_shl(8);
        x |= u32::from(key[idx_key + 2]);
        x = x.wrapping_shl(8);
        x |= u32::from(key[idx_key + 1]);
        x = x.wrapping_shl(8);
        x |= u32::from(key[idx_key]);
        x = x.wrapping_shl(8);
         */
        unsafe {
            state[idx_state] = mem::transmute::<[u8; 4], u32>([
                key[idx_key],
                key[idx_key + 1],
                key[idx_key + 2],
                key[idx_key + 3],
            ]);
        }
    }

    // o  Word 12 is a block counter.  Since each block is 64-byte, a 32-bit
    //    word is enough for 256 gigabytes of data.
    state[12] = counter;

    // o  Words 13-15 are a nonce, which should not be repeated for the same
    //    key.  The 13th word is the first 32 bits of the input nonce taken
    //    as a little-endian integer, while the 15th word is the last 32
    //    bits.
    for i in 0..3 {
        let idx_state = 13 + i;
        let idx_nonce = 4 * i;
        /*
        let mut x = 0;
        x |= u32::from(key[idx_str_nonce + 3]);
        x = x.wrapping_shl(8);
        x |= u32::from(key[idx_str_nonce + 2]);
        x = x.wrapping_shl(8);
        x |= u32::from(key[idx_str_nonce + 1]);
        x = x.wrapping_shl(8);
        x |= u32::from(key[idx_str_nonce]);
        x = x.wrapping_shl(8);
         */
        unsafe {
            state[idx_state] = mem::transmute::<[u8; 4], u32>([
                nonce[idx_nonce],
                nonce[idx_nonce + 1],
                nonce[idx_nonce + 2],
                nonce[idx_nonce + 3],
            ]);
        }
    }

    state
}

#[test]
fn test_setup_key() {
    // o  Key = 00:01:02:03:04:05:06:07:08:09:0a:0b:0c:0d:0e:0f:10:11:12:13:
    //    14:15:16:17:18:19:1a:1b:1c:1d:1e:1f.  The key is a sequence of
    //    octets with no particular structure before we copy it into the
    //    ChaCha state.
    // o  Nonce = (00:00:00:09:00:00:00:4a:00:00:00:00)
    // o  Block Count = 1.
    let key: Vec<u8> = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];
    let nonce: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
    ];
    let block_count = 1;

    let actual = setup_key(key, block_count, nonce);

    // ChaCha state with the key setup.

    // 61707865  3320646e  79622d32  6b206574
    // 03020100  07060504  0b0a0908  0f0e0d0c
    // 13121110  17161514  1b1a1918  1f1e1d1c
    // 00000001  09000000  4a000000  00000000
    let expected = vec![
        0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
        0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
        0x4a000000, 0x00000000,
    ];

    assert_eq!(actual, expected);
}

// 2.3.1.  The ChaCha20 Block Function in Pseudocode

//    Note: This section and a few others contain pseudocode for the
//    algorithm explained in a previous section.  Every effort was made for
//    the pseudocode to accurately reflect the algorithm as described in
//    the preceding section.  If a conflict is still present, the textual
//    explanation and the test vectors are normative.

//       inner_block (state):
//          Qround(state, 0, 4, 8,12)
//          Qround(state, 1, 5, 9,13)
//          Qround(state, 2, 6,10,14)
//          Qround(state, 3, 7,11,15)
//          Qround(state, 0, 5,10,15)
//          Qround(state, 1, 6,11,12)
//          Qround(state, 2, 7, 8,13)
//          Qround(state, 3, 4, 9,14)
//          end

//       chacha20_block(key, counter, nonce):
//          state = constants | key | counter | nonce
//          working_state = state
//          for i=1 upto 10
//             inner_block(working_state)
//             end
//          state += working_state
//          return serialize(state)
//          end

fn block_function(key: Vec<u8>, counter: u32, nonce: Vec<u8>) -> Vec<u32> {
    // The ChaCha20 state is initialized as follows:

    let mut state = setup_key(key, counter, nonce);

    let mut working_state = state.clone();
    for _ in 1..=10 {
        working_state = apply_quarter_round(0, 4, 8, 12, working_state);
        working_state = apply_quarter_round(1, 5, 9, 13, working_state);
        working_state = apply_quarter_round(2, 6, 10, 14, working_state);
        working_state = apply_quarter_round(3, 7, 11, 15, working_state);
        working_state = apply_quarter_round(0, 5, 10, 15, working_state);
        working_state = apply_quarter_round(1, 6, 11, 12, working_state);
        working_state = apply_quarter_round(2, 7, 8, 13, working_state);
        working_state = apply_quarter_round(3, 4, 9, 14, working_state);
    }

    for i in 0..16 {
        state[i] = state[i].overflowing_add(working_state[i]).0;
    }

    state
}

fn serialized(arr32: Vec<u32>) -> Vec<u8> {
    let mut serialized: Vec<u8> = vec![0; arr32.len() * 4];
    for i in 0..16 {
        unsafe {
            let arr8 = mem::transmute::<u32, [u8; 4]>(arr32[i]);
            serialized[i * 4] = arr8[0];
            serialized[i * 4 + 1] = arr8[1];
            serialized[i * 4 + 2] = arr8[2];
            serialized[i * 4 + 3] = arr8[3];
        }
    }

    serialized
}

#[test]
fn test_block_function() {
    // o  Key = 00:01:02:03:04:05:06:07:08:09:0a:0b:0c:0d:0e:0f:10:11:12:13:
    //    14:15:16:17:18:19:1a:1b:1c:1d:1e:1f.  The key is a sequence of
    //    octets with no particular structure before we copy it into the
    //    ChaCha state.
    // o  Nonce = (00:00:00:09:00:00:00:4a:00:00:00:00)
    // o  Block Count = 1.
    let key: Vec<u8> = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];
    let nonce: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
    ];
    let block_count = 1;

    let actual = block_function(key, block_count, nonce);

    // ChaCha state after 20 rounds

    // 837778ab  e238d763  a67ae21e  5950bb2f
    // c4f2d0c7  fc62bb2f  8fa018fc  3f5ec7b7
    // 335271c2  f29489f3  eabda8fc  82e46ebd
    // d19c12b4  b04e16de  9e83d0cb  4e3c50a2

    // ChaCha state at the end of the ChaCha20 operation

    //     e4e7f110  15593bd1  1fdd0f50  c47120a3
    //     c7f4d1c7  0368c033  9aaa2204  4e6cd4c3
    //     466482d2  09aa9f07  05d7c214  a2028bd9
    //     d19c12b5  b94e16de  e883d0cb  4e3c50a2
    let expected_at_the_end = vec![
        0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204,
        0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de,
        0xe883d0cb, 0x4e3c50a2,
    ];

    // Serialized Block:
    // 000  10 f1 e7 e4 d1 3b 59 15 50 0f dd 1f a3 20 71 c4  .....;Y.P.... q.
    // 016  c7 d1 f4 c7 33 c0 68 03 04 22 aa 9a c3 d4 6c 4e  ....3.h.."....lN
    // 032  d2 82 64 46 07 9f aa 09 14 c2 d7 05 d9 8b 02 a2  ..dF............
    // 048  b5 12 9c d1 de 16 4e b9 cb d0 83 e8 a2 50 3c 4e  ......N......P<N
    let expected = vec![
        0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20, 0x71,
        0xc4, // end row1
        0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a, 0xc3, 0xd4, 0x6c,
        0x4e, // end row2
        0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2, 0xd7, 0x05, 0xd9, 0x8b, 0x02,
        0xa2, // end row3
        0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9, 0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c,
        0x4e, // end row4
    ];

    assert_eq!(actual, expected_at_the_end);
    assert_eq!(serialized(actual), expected);
}

fn chacha20_encrypt(key: Vec<u8>, counter: u32, nonce: Vec<u8>, plaintext: Vec<u8>) -> Vec<u8> {
    let mut encrypted_message = vec![0; plaintext.len()];

    for j in 0..(plaintext.len() / 64) {
        let key_stream = serialized(block_function(
            key.clone(),
            counter + j as u32,
            nonce.clone(),
        ));
        let block = &plaintext[j * 64..=(j * 64 + 63)];

        for k in 0..64 {
            encrypted_message[j * 64 + k] = block[k] ^ key_stream[k];
        }
    }
    if plaintext.len() % 64 != 1 {
        let j = plaintext.len() / 64;
        let key_stream = serialized(block_function(
            key.clone(),
            counter + j as u32,
            nonce.clone(),
        ));
        let block = &plaintext[j * 64..plaintext.len()];

        for k in 0..plaintext.len() % 64 {
            encrypted_message[j * 64 + k] = block[k] ^ key_stream[k];
        }
    }

    encrypted_message
}

fn main() {
    println!("Hello, world!");
}
