use std::str::{FromStr, Split};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    marker::PhantomData,
};

pub trait FromCsv {
    const HEADER: &'static str;
    const FILENAME: &'static str;
    fn parse(row: CsvIter) -> Self;
}

pub struct FileIter<T> {
    reader: BufReader<File>,
    line_buffer: String,
    line_number: usize,
    row_type: PhantomData<T>,
}

impl<T: FromCsv> FileIter<T> {
    pub fn find(dir: &str) -> Self {
        let mut full = std::path::PathBuf::from(dir);
        full.push(T::FILENAME);
        Self::new(full.as_path().as_os_str().to_str().unwrap())
    }
    pub fn new(path: &str) -> Self {
        let file = File::open(path).unwrap_or_else(|e| panic!("Failed to open file {path}: {e}"));
        let mut reader = BufReader::new(file);
        let mut line_buffer = String::new();
        reader
            .read_line(&mut line_buffer)
            .unwrap_or_else(|e| panic!("Failed to read file header for {path}: {e}"));
        assert_eq!(
            line_buffer.trim(),
            T::HEADER,
            "Unexpected file header (left=actual)"
        );
        line_buffer.clear();
        FileIter {
            reader,
            line_buffer,
            line_number: 1,
            row_type: PhantomData,
        }
    }
}

impl<T: FromCsv> Iterator for FileIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let err = |e| panic!("Failed to read line {}: {e}", self.line_number);
        if 0 == self
            .reader
            .read_line(&mut self.line_buffer)
            .unwrap_or_else(err)
        {
            return None;
        }
        let buf = &self.line_buffer.trim();
        tracing::debug!("Parsing line {} buffer \"{buf}\"", self.line_number);
        let elem = T::parse(CsvIter::new(self.line_number, buf));
        self.line_buffer.clear();
        self.line_number += 1;
        Some(elem)
    }
}

pub struct CsvIter<'a> {
    data: &'a str,
    parts: Split<'a, char>,
    line: usize,
}

impl<'a> CsvIter<'a> {
    pub fn new(line: usize, data: &'a str) -> Self {
        let parts = data.split(',');
        CsvIter { data, line, parts }
    }
    pub fn try_next(&mut self) -> Result<&'a str, String> {
        self.parts.next().ok_or_else(|| {
            format!(
                "Failed to parse line {} of CSV: Insufficient cells\n{}",
                self.line, self.data
            )
        })
    }
    pub fn next(&mut self) -> &'a str {
        self.try_next().unwrap_or_else(|s| panic!("{s}"))
    }
    pub fn next_n<const N: usize>(&mut self) -> [&'a str; N] {
        [(); N].map(|()| self.next())
    }
    pub fn try_next_n<const N: usize>(&mut self) -> Result<[&'a str; N], String> {
        let mut arr = [""; N];
        for i in 0..N {
            arr[i] = self.try_next()?;
        }
        Ok(arr)
    }
    pub fn next_as<T: FromStr>(&mut self) -> T
    where
        T::Err: std::fmt::Debug,
    {
        self.try_next_as().unwrap()
    }
    pub fn try_next_as<T: FromStr>(&mut self) -> Result<T, String>
    where
        T::Err: std::fmt::Debug,
    {
        self.next().parse().map_err(|e| {
            format!(
                "Failed to parse line {} of CSV: Wrong data type ({}): {:?}\n{}",
                self.line,
                std::any::type_name::<T>(),
                e,
                self.data
            )
        })
    }
    fn next_time(&mut self) -> (u8, u8, u8) {
        todo!()
    }
    pub fn try_next_time(&mut self) -> Result<(u8, u8, u8), String> {
        let text = self.next();
        let mut iter = text.split(':');
        let mut pop = || iter.next().map(str::parse::<u8>);
        match (pop(), pop(), pop(), pop()) {
            (Some(Ok(h)), Some(Ok(m)), Some(Ok(s)), None) => Ok((h, m, s)),
            (_, _, None, None) | (_, _, _, Some(_)) => {
                Err(format!("Malformed time '{text}' not hh:mm:ss"))
            }
            bad => Err(format!("Unknown time: {bad:?}")),
        }
    }
    pub fn finish(self) {
        self.try_finish().unwrap();
    }
    pub fn try_finish(self) -> Result<(), String> {
        let remain = self.parts.count();
        if remain == 0 {
            Ok(())
        } else {
            Err(format!(
                "Found {} unexpected fields in CSV line\n{}",
                remain, self.data
            ))
        }
    }
}
