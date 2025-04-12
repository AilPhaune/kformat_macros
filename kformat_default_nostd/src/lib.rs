#![no_std]

pub trait Writeable {
    fn write(&mut self, data: char) -> Result<(), usize>;
}

pub trait Formattable {
    fn write_format(
        &self,
        writer: &mut dyn Writeable,
        hint_pretty: Option<bool>,
        hint_radix: Option<usize>,
        hint_width: Option<usize>,
        hint_precision: Option<usize>,
        hint_case: Option<bool>,
    ) -> Result<usize, usize>;
}

pub trait Debuggable {
    fn write_debug(
        &self,
        writer: &mut dyn Writeable,
        hint_pretty: Option<bool>,
        hint_radix: Option<usize>,
        hint_width: Option<usize>,
        hint_precision: Option<usize>,
        hint_case: Option<bool>,
    ) -> Result<usize, usize>;
}

impl<T> Writeable for &mut T
where
    T: Writeable,
{
    fn write(&mut self, data: char) -> Result<(), usize> {
        (**self).write(data)
    }
}

impl Formattable for char {
    fn write_format(
        &self,
        writer: &mut dyn Writeable,
        _hint_pretty: Option<bool>,
        _hint_radix: Option<usize>,
        _hint_width: Option<usize>,
        _hint_precision: Option<usize>,
        _hint_case: Option<bool>,
    ) -> Result<usize, usize> {
        writer.write(*self)?;
        Ok(1)
    }
}

impl Formattable for str {
    fn write_format(
        &self,
        writer: &mut dyn Writeable,
        _hint_pretty: Option<bool>,
        _hint_radix: Option<usize>,
        _hint_width: Option<usize>,
        _hint_precision: Option<usize>,
        _hint_case: Option<bool>,
    ) -> Result<usize, usize> {
        let mut count = 0;
        for c in self.chars() {
            writer.write(c)?;
            count += 1;
        }
        Ok(count)
    }
}

impl Formattable for &str {
    fn write_format(
        &self,
        writer: &mut dyn Writeable,
        _hint_pretty: Option<bool>,
        _hint_radix: Option<usize>,
        _hint_width: Option<usize>,
        _hint_precision: Option<usize>,
        _hint_case: Option<bool>,
    ) -> Result<usize, usize> {
        let mut count = 0;
        for c in self.chars() {
            writer.write(c)?;
            count += 1;
        }
        Ok(count)
    }
}

impl<T: Formattable> Formattable for &mut T {
    fn write_format(
        &self,
        writer: &mut dyn Writeable,
        hint_pretty: Option<bool>,
        hint_radix: Option<usize>,
        hint_width: Option<usize>,
        hint_precision: Option<usize>,
        hint_case: Option<bool>,
    ) -> Result<usize, usize> {
        (**self).write_format(
            writer,
            hint_pretty,
            hint_radix,
            hint_width,
            hint_precision,
            hint_case,
        )
    }
}

impl<T: Formattable> Formattable for &T {
    fn write_format(
        &self,
        writer: &mut dyn Writeable,
        hint_pretty: Option<bool>,
        hint_radix: Option<usize>,
        hint_width: Option<usize>,
        hint_precision: Option<usize>,
        hint_case: Option<bool>,
    ) -> Result<usize, usize> {
        (**self).write_format(
            writer,
            hint_pretty,
            hint_radix,
            hint_width,
            hint_precision,
            hint_case,
        )
    }
}

#[macro_export]
macro_rules! kwrite {
    ($writer: ident, $fmt: literal, $($args:expr),*) => {{
        use kformat_macros::kwrite_to_raw;
        #[allow(unused_imports)]
        use $crate::{Formattable, Debuggable, Writeable};
        kwrite_to_raw!($writer, Formattable, write_format, Debuggable, write_debug, Writeable, usize, $fmt, $($args),*)
    }};
}

#[macro_export]
macro_rules! impl_formattable_int_type {
    ($int_type_u: ident, $int_type_i: ident) => {
        impl Formattable for $int_type_i {
            fn write_format(
                &self,
                writer: &mut dyn Writeable,
                hint_pretty: Option<bool>,
                hint_radix: Option<usize>,
                hint_width: Option<usize>,
                hint_precision: Option<usize>,
                hint_case: Option<bool>,
            ) -> Result<usize, usize> {
                let mut count = 0;
                let mut buffer = ['\0'; 256];
                let mut idx = 0;
                let radix = match hint_radix {
                    Some(hint_radix) => hint_radix,
                    None => 10,
                } as $int_type_u;
                let alphabet = match hint_case {
                    Some(true) => "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                    _ => "0123456789abcdefghijklmnopqrstuvwxyz",
                };
                let negative = *self < 0;

                // Handle zero case
                if *self == 0 {
                    if let Some(precision) = hint_precision {
                        while idx < precision {
                            buffer[idx] = '0';
                            idx += 1;
                        }
                    } else {
                        buffer[idx] = '0';
                        idx += 1;
                    }
                } else {
                    // Convert the integer to the desired radix
                    let mut value = self.unsigned_abs();
                    while value > 0 {
                        let remainder = (value % radix) as usize;
                        buffer[idx] = alphabet.chars().nth(remainder).unwrap();
                        value /= radix;
                        idx += 1;
                    }
                }

                let precision = hint_precision.unwrap_or(idx);
                while idx < precision {
                    buffer[idx] = '0';
                    idx += 1;
                }

                if hint_pretty.unwrap_or(false) {
                    match radix {
                        2 => {
                            buffer[idx] = 'b';
                            idx += 1;
                            buffer[idx] = '0';
                            idx += 1;
                        }
                        8 => {
                            buffer[idx] = 'o';
                            idx += 1;
                            buffer[idx] = '0';
                            idx += 1;
                        }
                        16 => {
                            buffer[idx] = 'x';
                            idx += 1;
                            buffer[idx] = '0';
                            idx += 1;
                        }
                        _ => {}
                    }
                }

                if negative {
                    buffer[idx] = '-';
                    idx += 1;
                }

                // Reverse the buffer for the correct order
                let formatted_len = idx;
                let mut formatted_buffer = ['\0'; 256]; // Use a second buffer for the reversed result

                for i in 0..formatted_len {
                    formatted_buffer[i] = buffer[formatted_len - i - 1];
                }

                // Apply width and precision formatting
                let width = hint_width.unwrap_or(0);
                let padding = if idx.max(precision) < width {
                    width - idx.max(precision)
                } else {
                    0
                };

                // Add padding for width
                for _ in 0..padding {
                    writer.write(' ')?;
                    count += 1;
                }

                // Write the formatted string to the writer
                for &ch in formatted_buffer.iter().take(formatted_len) {
                    writer.write(ch)?;
                    count += 1;
                }

                Ok(count)
            }
        }

        impl Debuggable for $int_type_i {
            fn write_debug(
                &self,
                writer: &mut dyn Writeable,
                hint_pretty: Option<bool>,
                hint_radix: Option<usize>,
                hint_width: Option<usize>,
                hint_precision: Option<usize>,
                hint_case: Option<bool>,
            ) -> Result<usize, usize> {
                self.write_format(
                    writer,
                    hint_pretty,
                    hint_radix,
                    hint_width,
                    hint_precision,
                    hint_case,
                )
            }
        }

        impl Formattable for $int_type_u {
            fn write_format(
                &self,
                writer: &mut dyn Writeable,
                hint_pretty: Option<bool>,
                hint_radix: Option<usize>,
                hint_width: Option<usize>,
                hint_precision: Option<usize>,
                hint_case: Option<bool>,
            ) -> Result<usize, usize> {
                let mut count = 0;
                let mut buffer = ['\0'; 256];
                let mut idx = 0;
                let radix = match hint_radix {
                    Some(hint_radix) => hint_radix,
                    None => 10,
                } as $int_type_u;
                let alphabet = match hint_case {
                    Some(true) => "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                    _ => "0123456789abcdefghijklmnopqrstuvwxyz",
                };

                // Handle zero case
                if *self == 0 {
                    if let Some(precision) = hint_precision {
                        while idx < precision {
                            buffer[idx] = '0';
                            idx += 1;
                        }
                    } else {
                        buffer[idx] = '0';
                        idx += 1;
                    }
                } else {
                    // Convert the integer to the desired radix
                    let mut value = *self;
                    while value > 0 {
                        let remainder = (value % radix) as usize;
                        buffer[idx] = alphabet.chars().nth(remainder).unwrap();
                        value /= radix;
                        idx += 1;
                    }
                }

                let precision = hint_precision.unwrap_or(idx);
                while idx < precision {
                    buffer[idx] = '0';
                    idx += 1;
                }

                if hint_pretty.unwrap_or(false) {
                    match radix {
                        2 => {
                            buffer[idx] = 'b';
                            idx += 1;
                            buffer[idx] = '0';
                            idx += 1;
                        }
                        8 => {
                            buffer[idx] = 'o';
                            idx += 1;
                            buffer[idx] = '0';
                            idx += 1;
                        }
                        16 => {
                            buffer[idx] = 'x';
                            idx += 1;
                            buffer[idx] = '0';
                            idx += 1;
                        }
                        _ => {}
                    }
                }

                // Reverse the buffer for the correct order
                let formatted_len = idx;
                let mut formatted_buffer = ['\0'; 256]; // Use a second buffer for the reversed result

                for i in 0..formatted_len {
                    formatted_buffer[i] = buffer[formatted_len - i - 1];
                }

                // Apply width and precision formatting
                let width = hint_width.unwrap_or(0);
                let padding = if idx.max(precision) < width {
                    width - idx.max(precision)
                } else {
                    0
                };

                // Add padding for width
                for _ in 0..padding {
                    writer.write(' ')?;
                    count += 1;
                }

                // Write the formatted string to the writer
                for &ch in formatted_buffer.iter().take(formatted_len) {
                    writer.write(ch)?;
                    count += 1;
                }

                Ok(count)
            }
        }

        impl Debuggable for $int_type_u {
            fn write_debug(
                &self,
                writer: &mut dyn Writeable,
                hint_pretty: Option<bool>,
                hint_radix: Option<usize>,
                hint_width: Option<usize>,
                hint_precision: Option<usize>,
                hint_case: Option<bool>,
            ) -> Result<usize, usize> {
                self.write_format(
                    writer,
                    hint_pretty,
                    hint_radix,
                    hint_width,
                    hint_precision,
                    hint_case,
                )
            }
        }
    };
}

impl_formattable_int_type!(u8, i8);
impl_formattable_int_type!(u16, i16);
impl_formattable_int_type!(u32, i32);
impl_formattable_int_type!(u64, i64);
impl_formattable_int_type!(u128, i128);
impl_formattable_int_type!(usize, isize);

#[cfg(test)]
mod nostd_tests {
    use crate::Writeable;

    struct Buffer {
        data: [char; 1024],
        len: usize,
    }

    impl Buffer {
        fn new() -> Buffer {
            Buffer {
                data: ['\0'; 1024],
                len: 0,
            }
        }

        fn get(&self) -> &[char] {
            &self.data[0..self.len]
        }

        fn clear(&mut self) {
            self.len = 0;
        }
    }

    impl Writeable for Buffer {
        fn write(&mut self, data: char) -> Result<(), usize> {
            self.data[self.len] = data;
            self.len += 1;
            Ok(())
        }
    }

    #[test]
    fn it_works() {
        let mut buffer = Buffer::new();
        buffer.write('a').unwrap();
        buffer.write('b').unwrap();
        buffer.write('c').unwrap();

        assert_eq!(buffer.get(), &['a', 'b', 'c']);
        buffer.clear();

        assert_eq!(buffer.get(), &[]);
    }

    #[test]
    fn it_works_with_format() {
        let mut buffer = Buffer::new();

        kwrite!(buffer, "{} | {} | {}", 'a', 'b', "cd").unwrap();

        assert_eq!(
            buffer.get(),
            &['a', ' ', '|', ' ', 'b', ' ', '|', ' ', 'c', 'd']
        );
    }

    #[test]
    fn test_format_ints() {
        let mut buffer = Buffer::new();

        kwrite!(buffer, "{}", -123).unwrap();
        assert_eq!(buffer.get(), &['-', '1', '2', '3']);
        buffer.clear();

        kwrite!(buffer, "{}", -4771isize).unwrap();
        assert_eq!(buffer.get(), &['-', '4', '7', '7', '1']);
        buffer.clear();

        kwrite!(buffer, "{x}", 123).unwrap();
        assert_eq!(buffer.get(), &['7', 'b']);
        buffer.clear();

        kwrite!(buffer, "{w5}", 10).unwrap();
        assert_eq!(buffer.get(), &[' ', ' ', ' ', '1', '0']);
        buffer.clear();

        kwrite!(buffer, "{w5:x}", 10).unwrap();
        assert_eq!(buffer.get(), &[' ', ' ', ' ', ' ', 'a']);
        buffer.clear();

        kwrite!(buffer, "{w5:X}", 10usize).unwrap();
        assert_eq!(buffer.get(), &[' ', ' ', ' ', ' ', 'A']);
        buffer.clear();

        kwrite!(buffer, "{w10:p5}", 107).unwrap();
        assert_eq!(
            buffer.get(),
            &[' ', ' ', ' ', ' ', ' ', '0', '0', '1', '0', '7']
        );
        buffer.clear();

        kwrite!(buffer, "{#:w8:p4:X}", &107usize).unwrap();
        assert_eq!(buffer.get(), &[' ', ' ', '0', 'x', '0', '0', '6', 'B']);
        buffer.clear();

        kwrite!(buffer, "{w6:p4:b}", -5).unwrap();
        assert_eq!(buffer.get(), &[' ', '-', '0', '1', '0', '1']);
        buffer.clear();

        kwrite!(buffer, "{w6:p4:B}", 4).unwrap();
        assert_eq!(buffer.get(), &[' ', ' ', '0', '1', '0', '0']);
        buffer.clear();

        kwrite!(buffer, "{#:w6:p4:B}", 4).unwrap();
        assert_eq!(buffer.get(), &['0', 'b', '0', '1', '0', '0']);
        buffer.clear();

        kwrite!(
            buffer,
            "{#:w44:p40}",
            72724961945561898922996321321127181560u128
        )
        .unwrap();
        assert_eq!(
            buffer.get(),
            &[
                ' ', ' ', ' ', ' ', '0', '0', '7', '2', '7', '2', '4', '9', '6', '1', '9', '4',
                '5', '5', '6', '1', '8', '9', '8', '9', '2', '2', '9', '9', '6', '3', '2', '1',
                '3', '2', '1', '1', '2', '7', '1', '8', '1', '5', '6', '0'
            ]
        );
        buffer.clear();
    }
}
