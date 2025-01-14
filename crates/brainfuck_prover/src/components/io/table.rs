use brainfuck_vm::{instruction::InstructionType, registers::Registers};
use stwo_prover::core::fields::m31::BaseField;

/// Represents a single row in the I/O Table.
///
/// The I/O Table stores:
/// - The memory value (`mv`),
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct IOTableRow {
    /// Memory value: value of the cell pointer by `mp` - values in [0..2^31 - 1)
    pub mv: BaseField,
}

impl IOTableRow {
    pub const fn new(mv: BaseField) -> Self {
        Self { mv }
    }
}

/// Represents the I/O Table, which holds the required register
/// for the Input and Output components.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct IOTable<const N: u32> {
    /// A vector of [`IOTableRow`] representing the table rows.
    pub table: Vec<IOTableRow>,
}

impl<const N: u32> IOTable<N> {
    /// Creates a new, empty [`IOTable`].
    ///
    /// # Returns
    /// A new instance of [`IOTable`] with an empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the I/O Table from the provided register.
    ///
    /// # Arguments
    /// * `mv` - The memory value for the new row.
    ///
    /// This method pushes a new [`IOTableRow`] onto the `table` vector.
    pub fn add_row_from_register(&mut self, mv: BaseField) {
        self.table.push(IOTableRow { mv });
    }

    /// Adds a new row to the I/O Table.
    ///
    /// # Arguments
    /// * `row` - The [`IOTableRow`] to add to the table.
    ///
    /// This method pushes a new [`IOTableRow`] onto the `table` vector.
    pub fn add_row(&mut self, row: IOTableRow) {
        self.table.push(row);
    }

    /// Adds multiple rows to the I/O Table.
    ///
    /// # Arguments
    /// * `rows` - A vector of [`IOTableRow`] to add to the table.
    ///
    /// This method extends the `table` vector with the provided rows.
    pub fn add_rows(&mut self, rows: Vec<IOTableRow>) {
        self.table.extend(rows);
    }

    /// Pads the I/O table with dummy rows up to the next power of two length.
    ///
    /// Each dummy row sets the memory value register `mv` to zero.
    ///
    /// Does nothing if the table is empty.
    fn pad(&mut self) {
        let trace_len = self.table.len();
        let padding_offset = (trace_len.next_power_of_two() - trace_len) as u32;
        for _ in 0..padding_offset {
            let dummy_row = IOTableRow::default();
            self.add_row(dummy_row);
        }
    }
}

impl<const N: u32> From<Vec<Registers>> for IOTable<N> {
    fn from(registers: Vec<Registers>) -> Self {
        let mut io_table = Self::new();
        let rows = registers
            .into_iter()
            .filter(|register| register.ci == BaseField::from_u32_unchecked(N))
            .map(|x| IOTableRow { mv: x.mv })
            .collect();
        io_table.add_rows(rows);

        io_table.pad();

        io_table
    }
}

/// Input table (trace) for the Input component.
///
/// This table is made of the memory values (`mv` register) corresponding to
/// inputs (when the current instruction `ci` equals ',').
pub type InputTable = IOTable<{ InstructionType::ReadChar.to_u32() }>;

/// Output table (trace) for the Output component.
///
/// This table is made of the memory values (`mv` register) corresponding to
/// outputs (when the current instruction `ci` equals '.').
pub type OutputTable = IOTable<{ InstructionType::PutChar.to_u32() }>;

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::One;

    type TestIOTable = IOTable<10>;

    #[test]
    fn test_io_row_new() {
        let row = IOTableRow::new(BaseField::from(91));
        let expected_row = IOTableRow { mv: BaseField::from(91) };
        assert_eq!(row, expected_row);
    }

    #[test]
    fn test_table_new() {
        let io_table = TestIOTable::new();
        assert!(io_table.table.is_empty(), "I/O Table should be empty upon initialization.");
    }

    #[test]
    fn test_table_add_row_from_register() {
        let mut io_table = TestIOTable::new();
        // Create a row to add to the table
        let row = IOTableRow::new(BaseField::from(91));
        // Add the row to the table
        io_table.add_row_from_register(BaseField::from(91));
        // Check that the table contains the added row
        assert_eq!(io_table.table, vec![row], "Added row should match the expected row.");
    }

    #[test]
    fn test_table_add_row() {
        let mut io_table = TestIOTable::new();
        // Create a row to add to the table
        let row = IOTableRow::new(BaseField::from(91));
        // Add the row to the table
        io_table.add_row(row.clone());
        // Check that the table contains the added row
        assert_eq!(io_table.table, vec![row], "Added row should match the expected row.");
    }

    #[test]
    fn test_table_add_multiple_rows() {
        let mut io_table = TestIOTable::new();
        // Create a vector of rows to add to the table
        let rows = vec![
            IOTableRow::new(BaseField::from(91)),
            IOTableRow::new(BaseField::from(9)),
            IOTableRow::new(BaseField::from(43)),
        ];
        // Add the rows to the table
        io_table.add_rows(rows.clone());
        // Check that the table contains the added rows
        assert_eq!(io_table, IOTable { table: rows });
    }

    #[test]
    fn test_input_table_from_registers() {
        let reg1 = Registers::default();
        let reg2 = Registers {
            mv: BaseField::one(),
            ci: BaseField::from(InstructionType::ReadChar.to_base_field()),
            ..Default::default()
        };
        let reg3 = Registers {
            mv: BaseField::from(5),
            ci: BaseField::from(InstructionType::PutChar.to_base_field()),
            ..Default::default()
        };
        let registers: Vec<Registers> = vec![reg3, reg1, reg2];

        let row = IOTableRow::new(BaseField::one());
        // let row = IOTableRow::new(BaseField::from(5));

        let mut expected_io_table: InputTable = IOTable::new();
        expected_io_table.add_row(row);

        assert_eq!(IOTable::from(registers), expected_io_table);
    }

    #[test]
    fn test_output_table_from_registers() {
        let reg1 = Registers::default();
        let reg2 = Registers {
            mv: BaseField::one(),
            ci: BaseField::from(InstructionType::ReadChar.to_base_field()),
            ..Default::default()
        };
        let reg3 = Registers {
            mv: BaseField::from(5),
            ci: BaseField::from(InstructionType::PutChar.to_base_field()),
            ..Default::default()
        };
        let registers: Vec<Registers> = vec![reg3, reg1, reg2];

        let row = IOTableRow::new(BaseField::from(5));

        let mut expected_io_table: OutputTable = IOTable::new();
        expected_io_table.add_row(row);

        assert_eq!(IOTable::from(registers), expected_io_table);
    }
}
