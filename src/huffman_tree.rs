use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// https://www.youtube.com/watch?v=wLoWd2KyUro
pub enum TableType {
    AC = 1,
    DC = 0,
}

impl TableType {
    fn from(ht_type: u8) -> Self {
        match ht_type {
            1 => TableType::AC,
            0 => TableType::DC,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CodeFreq {
    pub(crate) code: u8,
    pub(crate) freq: usize,
}

#[derive(Debug, Eq)]
pub(crate) struct HeapItem {
    freq: usize,
    node: NPtr,
}

impl HeapItem {
    fn from(freq: usize, node: NPtr) -> Self {
        HeapItem { freq, node }
    }
}

impl From<(usize, NPtr)> for HeapItem {
    fn from(tuple: (usize, NPtr)) -> Self {
        HeapItem {
            freq: tuple.0,
            node: tuple.1,
        }
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

struct HuffmanNode {
    internal: usize,
    leaf: CodeFreq,
    left: NPtr,
    right: NPtr,
}

impl HuffmanNode {
    fn new_leaf(code_freq: CodeFreq) -> Self {
        HuffmanNode {
            internal: u8::MAX as usize,
            leaf: code_freq,
            left: None,
            right: None,
        }
    }

    fn is_internal(&self) -> bool {
        self.internal == u8::MAX as usize
    }
}

type NPtr = Option<NonNull<HuffmanNode>>;

pub struct HuffmanTree {
    h_type: TableType,
    h_id: usize,
    root: NPtr,
    _woof: PhantomData<HuffmanNode>,
}

impl HuffmanTree {
    pub fn from(ht_type: u8, ht_id: usize, code_freqs: Vec<CodeFreq>) -> Self {
        let mut min_heap = BinaryHeap::new();

        for code_freq in code_freqs {
            let freq = code_freq.freq;
            let new_node = unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(HuffmanNode::new_leaf(code_freq))))
            };

            min_heap.push(HeapItem::from(freq, Some(new_node)))
        }

        while min_heap.len() > 1 {
            let left = min_heap.pop();

            if min_heap.len() == 1 {
                break;
            }

            let right = min_heap.pop();

            match (left, right) {
                (Some(left_item), Some(right_item)) => {
                    let sum_freq = left_item.freq + right_item.freq;

                    let new_node = unsafe {
                        NonNull::new_unchecked(Box::into_raw(Box::new(HuffmanNode {
                            internal: sum_freq,
                            leaf: CodeFreq {
                                code: u8::MAX,
                                freq: 0,
                            },
                            left: left_item.node,
                            right: right_item.node,
                        })))
                    };

                    min_heap.push(HeapItem::from(sum_freq, Some(new_node)))
                }
                _ => break,
            }
        }

        let root = min_heap.pop();
        debug_assert!(root.is_some());
        let HeapItem { node: root, .. } = root.unwrap();

        let mut tree = HuffmanTree {
            root,
            h_id: ht_id,
            h_type: TableType::from(ht_type),
            _woof: PhantomData,
        };

        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_min_heap() -> Result<()> {
        let mut min_heap = BinaryHeap::new();

        for i in 36..0 {
            min_heap.push(HeapItem {
                freq: i,
                node: Some(unsafe {
                    NonNull::new_unchecked(Box::into_raw(Box::new(HuffmanNode::new_leaf(
                        CodeFreq {
                            code: i as u8,
                            freq: i,
                        },
                    ))))
                }),
            })
        }

        let mut expected = 36;
        while !min_heap.is_empty() {
            let res = min_heap.pop();
            assert!(res.is_some());
            let HeapItem { freq, .. } = res.unwrap();

            assert_eq!(expected, freq);

            expected -= 1;
        }

        Ok(())
    }
}
