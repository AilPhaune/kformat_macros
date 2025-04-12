use kformat_macros::kwrite_to_raw;

#[derive(Debug)]
enum FormatError {
    Write(WriteError),
}
#[derive(Debug)]
enum WriteError {}

trait Formattable {
    fn write(
        &self,
        writer: &mut dyn Writeable,
        hint_pretty: Option<bool>,
        hint_radix: Option<usize>,
        hint_width: Option<usize>,
        hint_precision: Option<usize>,
        hint_case: Option<usize>,
    ) -> Result<usize, FormatError>;
}

trait Writeable {
    fn write(&mut self, data: char) -> Result<(), WriteError>;
}

struct Console {}

impl Writeable for Console {
    fn write(&mut self, data: char) -> Result<(), WriteError> {
        print!("{}", data);
        Ok(())
    }
}

impl Formattable for &[char] {
    fn write(
        &self,
        writer: &mut dyn Writeable,
        _hint_pretty: Option<bool>,
        _hint_radix: Option<usize>,
        _hint_width: Option<usize>,
        _hint_precision: Option<usize>,
        _hint_case: Option<usize>,
    ) -> Result<usize, FormatError> {
        for c in self.iter() {
            match writer.write(*c) {
                Ok(_) => {}
                Err(e) => return Err(FormatError::Write(e)),
            }
        }
        Ok(self.len())
    }
}

impl Formattable for &str {
    fn write(
        &self,
        writer: &mut dyn Writeable,
        _hint_pretty: Option<bool>,
        _hint_radix: Option<usize>,
        _hint_width: Option<usize>,
        _hint_precision: Option<usize>,
        _hint_case: Option<usize>,
    ) -> Result<usize, FormatError> {
        let mut count = 0;
        for c in self.chars() {
            match writer.write(c) {
                Ok(_) => {
                    count += 1;
                }
                Err(e) => return Err(FormatError::Write(e)),
            }
        }
        Ok(count)
    }
}

pub fn main() {
    let mut console = Console {};

    let data = vec!['a', 'b', 'c'];
    let actual_data = &data as &[char];

    let data2 = vec!['d', 'e', 'f'];
    let actual_data2 = &data2 as &[char];

    let amount = kwrite_to_raw!(
        console,
        Formattable,
        write,
        Formattable,
        write,
        Writeable,
        FormatError,
        "Test {?} format {#}\r\n",
        actual_data,
        actual_data2
    )
    .unwrap();

    println!("Formatted into {} chars.", amount);
}
