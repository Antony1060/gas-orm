pub(crate) const fn join_static_str<const SIZE_GUARANTEE: usize>(
    values: &[&'static str],
) -> &'static str {
    let size = {
        let mut size = 0usize;
        let mut i = 0;
        while i < values.len() {
            size += values[i].len();
            i += 1;
        }

        size
    };

    assert!(size <= SIZE_GUARANTEE);

    let mut out: [u8; SIZE_GUARANTEE] = [0; SIZE_GUARANTEE];
    let mut cursor = 0;
    let mut i = 0;
    while i < values.len() {
        let value = values[i].as_bytes();
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_ptr(),
                out.as_mut_ptr().add(cursor),
                value.len(),
            );
        }

        cursor += value.len();
        i += 1;
    }

    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(out.as_ptr(), size)) }
}

#[cfg(test)]
mod test {
    use crate::const_util::join_static_str;

    #[test]
    pub fn test_join_static_str() {
        assert_eq!(
            join_static_str::<256>(&["test", "foo", "bar", "baz"]),
            "testfoobarbaz"
        );

        assert_eq!(
            join_static_str::<256>(&[
                "TIMESTAMP WITH TIME ZONE",
                " REFERENCES ",
                "foo_table",
                "(",
                "bar_field",
                ")",
            ]),
            "TIMESTAMP WITH TIME ZONE REFERENCES foo_table(bar_field)"
        );
    }

    #[test]
    pub fn test_join_static_str_empty() {
        assert_eq!(join_static_str::<256>(&[]), "");
    }

    #[test]
    #[should_panic]
    pub fn test_join_static_str_panic() {
        join_static_str::<4>(&["larger"]);
    }
}
