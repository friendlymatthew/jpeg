use crate::entropy::huffman_table::HuffmanTree;

pub(crate) mod huffman_table;

pub(crate) enum EntropyCoding {
    Huffman,
    Arithmetic
}