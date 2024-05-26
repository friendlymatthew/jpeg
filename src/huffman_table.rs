pub enum TableType {
    DC = 0,
    AC = 1,
}

pub struct HuffmanTable {
    table_type: TableType,
    table_number: usize,
}
